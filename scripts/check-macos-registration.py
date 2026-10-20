#!/usr/bin/env python3
"""Verify that actual macOS handler candidates respect the generated bundle rank."""
import argparse
import json
from pathlib import Path
import plistlib
import subprocess
import sys
from macos_launch_services import BUNDLE_ID, LSREGISTER, ROOT, build_copy, transient_registrations


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, required=True)
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
    output.parent.mkdir(parents=True, exist_ok=True)
    executable = output.parent/'xdzip-launch-services-probe'
    compiled = subprocess.run(['/usr/bin/clang++', '-std=c++17', '-fobjc-arc', '-framework', 'AppKit',
        str(ROOT/'tools/launch-services-probe/main.mm'), '-o', str(executable)],
        capture_output=True, text=True, timeout=60)
    if compiled.returncode:
        (output.parent/'launch-services-compile.log').write_text(
            compiled.stderr.replace(str(ROOT), '<checkout>'))
        raise RuntimeError('Native registration probe compilation failed')

    def query():
        result = subprocess.run([str(executable), str(fixture), str(app)],
            capture_output=True, text=True, timeout=15)
        if result.returncode:
            raise RuntimeError('Native handler candidate query failed')
        return json.loads(result.stdout)

    before = query()
    with transient_registrations(app):
        registered = subprocess.run([str(LSREGISTER), '-f', str(app)], capture_output=True, timeout=15)
        if registered.returncode:
            raise RuntimeError('Owned test application registration failed')
        after = query()
        if after['selected_in_candidates'] != (rank == 'Alternate'):
            raise RuntimeError('Native handler candidates disagree with the bundle rank')
        if before['default_bundle_id'] != after['default_bundle_id']:
            raise RuntimeError('Default document handler unexpectedly changed')
    cleaned = query()
    output.write_text(json.dumps({'status': 'passed', 'rank': rank, 'before': before,
        'after_forced_registration': after, 'after_cleanup': cleaned,
        'limits': 'Native document handler query, not visual acceptance of a client application menu.'},
        ensure_ascii=False, indent=2)+'\n')
    print('Native macOS handler rank and unchanged default: passed')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        # Native exceptions can embed private paths. Reports contain no paths.
        raise SystemExit('macOS registration check failed: '+type(error).__name__) from None
