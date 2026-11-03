#!/usr/bin/env python3
"""Verify that actual macOS handler candidates respect the generated bundle rank."""
import argparse
import json
from pathlib import Path
import plistlib
import subprocess
import sys
from macos_launch_services import BUNDLE_ID, LSREGISTER, ROOT, build_copy, transient_registrations


def validate_transition(before, during, cleaned, rank, label='fixture'):
    if during['selected_in_candidates'] != (rank == 'Alternate'):
        raise RuntimeError('Native handler candidates disagree with the bundle rank: '+label)
    previous = before['default_bundle_id']
    if before['has_default_handler'] and (
            not during['has_default_handler'] or during['default_bundle_id'] != previous or
            during['selected_is_default'] != before['selected_is_default']):
        raise RuntimeError('Existing default handler unexpectedly changed: '+label)
    if (cleaned['has_default_handler'] != before['has_default_handler'] or
            cleaned['default_bundle_id'] != previous or
            cleaned['selected_is_default'] != before['selected_is_default'] or
            cleaned['selected_in_candidates'] != before['selected_in_candidates']):
        raise RuntimeError('Handler state did not return to its original value after cleanup: '+label)


def handler_preferences():
    domain = 'com.apple.LaunchServices/com.apple.launchservices.secure'
    result = subprocess.run(['/usr/bin/defaults', 'export', domain, '-'], capture_output=True,
                            timeout=15)
    if result.returncode:
        domains = subprocess.run(['/usr/bin/defaults', 'domains'], capture_output=True, text=True,
                                 timeout=15)
        if domains.returncode or domain in {name.strip() for name in domains.stdout.split(',')}:
            raise RuntimeError('Could not inspect user handler preferences')
        return plistlib.dumps([], fmt=plistlib.FMT_BINARY)
    handlers = plistlib.loads(result.stdout).get('LSHandlers', [])
    if not isinstance(handlers, list):
        raise RuntimeError('Invalid user handler preferences')
    # Compare in memory only; preference contents can contain private identifiers.
    return plistlib.dumps(handlers, fmt=plistlib.FMT_BINARY, sort_keys=True)


def validate_preferences(before, during, cleaned):
    if before != during or before != cleaned:
        raise RuntimeError('User handler preferences changed during registration')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--formats', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if sys.platform != 'darwin':
        parser.error('This native check requires macOS')
    app, fixture, output = args.app.resolve(), args.fixture.resolve(), args.output.resolve()
    if (not build_copy(args.app) or not fixture.is_file() or not output.is_relative_to(ROOT/'build')
            or output.is_relative_to(app) or output.suffix != '.json' or output.exists()):
        parser.error('Use an owned build application, existing fixture and new build/ JSON outside the app')
    info = plistlib.loads((app/'Contents/Info.plist').read_bytes())
    types = info.get('CFBundleDocumentTypes', [])
    if not isinstance(types, list) or not types or any(not isinstance(item, dict) for item in types):
        parser.error('Expected archive document declarations')
    ranks = {item.get('LSHandlerRank') for item in types}
    if info.get('CFBundleIdentifier') != BUNDLE_ID or len(ranks) != 1 or not ranks <= {'None', 'Alternate'}:
        parser.error('Expected an explicit None or Alternate archive handler rank')
    rank = next(iter(ranks))
    expected_icons = {'zip','tar','tar-gz','gz','7z','rar','cab','iso','udf'}
    declared = {item.get('CFBundleTypeIconFile') for item in types}
    if declared != {f'xdzip-{name}.icns' for name in expected_icons}:
        parser.error('Expected one document icon for each declared type')
    if any(not (app/'Contents/Resources'/name).is_file() for name in declared):
        parser.error('A declared document icon is missing from the application')
    format_names = ['sample.zip','sample.tar','sample.tar.gz','sample.tgz',
                    'sample.7z','sample.rar','sample.cab','sample.iso','sample.udf']
    formats = args.formats.resolve()
    if not formats.is_relative_to(ROOT/'build') or any(not (formats/name).is_file() for name in format_names):
        parser.error('Expected owned format fixtures under build/')
    output.parent.mkdir(parents=True, exist_ok=True)
    executable = output.parent/'xdzip-launch-services-probe'
    compiled = subprocess.run(['/usr/bin/clang++', '-std=c++17', '-fobjc-arc', '-framework', 'AppKit',
        str(ROOT/'tools/launch-services-probe/main.mm'), '-o', str(executable)],
        capture_output=True, text=True, timeout=60)
    if compiled.returncode:
        (output.parent/'launch-services-compile.log').write_text(
            compiled.stderr.replace(str(ROOT), '<checkout>'))
        raise RuntimeError('Native registration probe compilation failed')

    def query(target):
        result = subprocess.run([str(executable), str(target), str(app)],
            capture_output=True, text=True, timeout=15)
        if result.returncode:
            raise RuntimeError('Native handler candidate query failed')
        return json.loads(result.stdout)

    preferences_before = handler_preferences()
    before = query(fixture)
    format_before = {name:query(formats/name) for name in format_names}
    with transient_registrations(app):
        registered = subprocess.run([str(LSREGISTER), '-f', str(app)], capture_output=True, timeout=15)
        if registered.returncode:
            raise RuntimeError('Owned test application registration failed')
        after = query(fixture)
        format_after = {name:query(formats/name) for name in format_names}
        preferences_during = handler_preferences()
    cleaned = query(fixture)
    format_cleaned = {name:query(formats/name) for name in format_names}
    preferences_cleaned = handler_preferences()
    validate_preferences(preferences_before, preferences_during, preferences_cleaned)
    validate_transition(before, after, cleaned, rank)
    for name in format_names:
        validate_transition(format_before[name], format_after[name], format_cleaned[name], rank, name)
    output.write_text(json.dumps({'status': 'passed', 'rank': rank, 'before': before,
        'after_forced_registration': after, 'after_cleanup': cleaned,
        'user_handler_preferences_unchanged': True,
        'format_candidates': {name:{'before':format_before[name], 'after':format_after[name],
                                     'after_cleanup':format_cleaned[name]} for name in format_names},
        'limits': 'A previously absent default may be resolved implicitly during registration. '
                  'Only the current user LSHandlers preference is compared; visual acceptance is separate.'},
        ensure_ascii=False, indent=2)+'\n')
    print('Native macOS handler rank, existing defaults and cleanup: passed')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        # Native exceptions can embed private paths. Reports contain no paths.
        raise SystemExit('macOS registration check failed: '+type(error).__name__) from None
