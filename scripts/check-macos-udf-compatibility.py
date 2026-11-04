#!/usr/bin/env python3
"""Check UDF association compatibility on an isolated GitHub-hosted Mac."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import subprocess
import sys
import time

from macos_launch_services import BUNDLE_ID, LSREGISTER, ROOT, registered_apps


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--candidate-probe', type=Path, required=True)
    parser.add_argument('--settings-probe', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if (sys.platform != 'darwin' or os.environ.get('GITHUB_ACTIONS') != 'true' or
            os.environ.get('RUNNER_ENVIRONMENT') != 'github-hosted'):
        parser.error('This registration test requires a disposable GitHub-hosted macOS runner')
    build = (ROOT/'build').resolve()
    app, fixture = args.app.resolve(), args.fixture.resolve()
    output = args.output.resolve()
    scratch = output.parent/(output.stem+'-fixtures')
    if (not app.is_relative_to(build) or not fixture.is_relative_to(build) or
            not output.is_relative_to(build) or output.exists() or scratch.exists() or
            not fixture.is_file() or not args.candidate_probe.is_file() or
            not args.settings_probe.is_file()):
        parser.error('Use existing owned build inputs and new build-only output paths')
    info = plistlib.loads((app/'Contents/Info.plist').read_bytes())
    docs = [x for x in info['CFBundleDocumentTypes']
            if x.get('CFBundleTypeIconFile') == 'xdzip-udf.icns']
    require(info.get('CFBundleIdentifier') == BUNDLE_ID and len(docs) == 1 and
            docs[0].get('CFBundleTypeExtensions') == ['udf'] and
            'LSItemContentTypes' not in docs[0], 'Unexpected product UDF declaration')
    require(not registered_apps(), 'The disposable runner already has xdzip registrations')
    spec = importlib.util.spec_from_file_location('registration', ROOT/'scripts/check-macos-registration.py')
    registration = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(registration)
    preferences = registration.handler_preferences()
    scratch.mkdir(parents=True)
    active = []

    def make_bundle(name, identifier, document, declarations=None):
        bundle = scratch/(name+'.app')
        contents = bundle/'Contents'
        binary = contents/'MacOS'/name
        binary.parent.mkdir(parents=True)
        binary.write_text('#!/bin/sh\nexit 0\n')
        binary.chmod(0o755)
        data = {'CFBundleIdentifier': identifier, 'CFBundleExecutable': name,
                'CFBundlePackageType': 'APPL', 'CFBundleName': name,
                'CFBundleVersion': '21', 'CFBundleDocumentTypes': [document]}
        if declarations:
            data.update(declarations)
        (contents/'Info.plist').write_bytes(plistlib.dumps(data, sort_keys=False))
        active.append(bundle)
        return bundle

    def register(bundle):
        result = subprocess.run([str(LSREGISTER), '-f', str(bundle)], capture_output=True, timeout=15)
        require(result.returncode == 0, 'Could not register a temporary UDF fixture')

    def retire(bundle):
        result = subprocess.run([str(LSREGISTER), '-u', str(bundle)], capture_output=True, timeout=15)
        require(result.returncode == 0, 'Could not unregister a temporary UDF fixture')
        bundle.rename(bundle.with_suffix('.retired'))
        active.remove(bundle)

    def candidate_state(candidate):
        result = subprocess.run([str(args.candidate_probe.resolve()), str(fixture), str(candidate)],
                                capture_output=True, text=True, timeout=15)
        require(result.returncode == 0, 'Could not query native UDF candidates')
        return json.loads(result.stdout)

    def settings_state():
        result = subprocess.run([str(args.settings_probe.resolve()), str(fixture)],
                                capture_output=True, text=True, timeout=15)
        state = json.loads(result.stdout)
        require(result.returncode == 0, 'UDF settings disagree with the actual file default: '+
                json.dumps(state, sort_keys=True))
        return state

    def wait_state(candidate, predicate, message):
        deadline = time.monotonic()+8
        while True:
            result = candidate_state(candidate)
            if predicate(result):
                return result
            if time.monotonic() >= deadline:
                raise RuntimeError(message)
            time.sleep(0.2)

    udf_doc = dict(docs[0])
    imported = [x for x in info['UTImportedTypeDeclarations']
                if x.get('UTTypeIdentifier') == 'org.xdzip.udf-image']
    require(len(imported) == 1, 'Missing product UDF type declaration')
    old_type = dict(imported[0], UTTypeConformsTo=['public.disk-image'])
    old_doc = dict(udf_doc, LSHandlerRank='Alternate',
                   LSItemContentTypes=['org.xdzip.udf-image'])
    foreign_id = 'org.xdzip.qa-foreign-udf'
    foreign_type = 'org.xdzip.qa-foreign-udf-image'
    foreign_doc = {'CFBundleTypeName': 'UDF fixture', 'CFBundleTypeRole': 'Viewer',
                   'LSHandlerRank': 'Owner', 'LSItemContentTypes': [foreign_type],
                   'CFBundleTypeExtensions': ['udf']}
    foreign_declaration = {'UTTypeIdentifier': foreign_type,
                           'UTTypeConformsTo': ['public.data', 'public.disk-image'],
                           'UTTypeTagSpecification': {'public.filename-extension': ['udf']}}
    report = {}
    try:
        absent = scratch/'not-registered.app'
        before = candidate_state(absent)
        require(not before['has_default_handler'], 'The runner already has a UDF default handler')
        legacy = make_bundle('legacy', BUNDLE_ID, old_doc,
                             {'UTImportedTypeDeclarations': [old_type]})
        register(legacy)
        wait_state(absent, lambda x: not x['has_default_handler'], 'Unexpected legacy UDF default')
        settings = settings_state()
        require(not settings['file_default_is_xdzip'] and settings['legacy_type_default_is_xdzip'],
                'Legacy UDF fixture did not reproduce the conflicting type lookup')
        report['legacy_settings_match_file'] = True

        none = make_bundle('none', BUNDLE_ID, dict(udf_doc, LSHandlerRank='None'),
                           {'UTImportedTypeDeclarations': imported})
        register(none)
        wait_state(none, lambda x: not x['selected_in_candidates'] and
                   not x['has_default_handler'], 'None UDF candidate appeared')
        settings_state()
        retire(none)
        report['none_not_in_candidates'] = True

        alternate = make_bundle('alternate', BUNDLE_ID, dict(udf_doc, LSHandlerRank='Alternate'),
                                {'UTImportedTypeDeclarations': imported})
        register(alternate)
        wait_state(alternate, lambda x: x['selected_in_candidates'], 'Alternate UDF candidate missing')
        settings_state()
        retire(alternate)
        report['alternate_in_candidates'] = True

        foreign = make_bundle('foreign', foreign_id, foreign_doc,
                              {'UTExportedTypeDeclarations': [foreign_declaration]})
        register(foreign)
        wait_state(absent, lambda x: x['default_bundle_id'] == foreign_id,
                   'Could not establish the foreign UDF handler')
        alternate2 = make_bundle('alternate-foreign', BUNDLE_ID,
                                 dict(udf_doc, LSHandlerRank='Alternate'),
                                 {'UTImportedTypeDeclarations': imported})
        register(alternate2)
        wait_state(alternate2, lambda x: x['selected_in_candidates'] and
                   x['default_bundle_id'] == foreign_id,
                   'UDF candidate missing or foreign default changed')
        settings_state()
        require(registration.handler_preferences() == preferences, 'User handler preferences changed')
        report['foreign_default_preserved'] = True
    finally:
        for bundle in reversed(active[:]):
            retire(bundle)
    require(registration.handler_preferences() == preferences, 'User handler preferences changed')
    final = candidate_state(scratch/'not-registered.app')
    require(final['has_default_handler'] == before['has_default_handler'] and
            final['default_bundle_id'] == before['default_bundle_id'] and not registered_apps(),
            'Temporary UDF registration did not restore the original state')
    report.update(status='passed', user_handler_preferences_unchanged=True,
                  original_handler_restored=True)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2)+'\n')
    print('macOS legacy UDF, handler ranks, foreign default and cleanup: passed')


if __name__ == '__main__':
    try:
        main()
    except RuntimeError as error:
        # RuntimeError messages here contain only fixed labels or native booleans.
        raise SystemExit('macOS UDF compatibility check failed: '+str(error)) from None
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        # Some native exceptions include private paths; retain only the exception class.
        raise SystemExit('macOS UDF compatibility check failed: '+type(error).__name__) from None
