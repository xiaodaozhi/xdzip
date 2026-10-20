#!/usr/bin/env python3
"""Mount read-only, relocate and run a local developer DMG without SDK search paths."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import re
import shutil
import subprocess
import zipfile
from macos_launch_services import temporary_bundle_path, transient_registrations
from macos_bundle import mach_o_files, validate_optimized_bundle
from deployment_audit import audit, digest, within

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('macdmg', ROOT/'scripts/build-macos-dmg.py')
macdmg = importlib.util.module_from_spec(spec)
spec.loader.exec_module(macdmg)


def clean_environment(output):
    # An inherited style, plugin or diagnostic override must not silently turn
    # this into an SDK-backed test or bypass the requested native product flow.
    env = {key: value for key, value in os.environ.items()
           if not key.startswith(('QT_', 'QML_', 'QML2_', 'QSG_', 'DYLD_', 'XDZIP_'))}
    env.update(PATH='/usr/bin:/bin:/usr/sbin:/sbin', QT_QPA_PLATFORM='cocoa',
               QSG_RHI_BACKEND='metal', XDZIP_SETTINGS_PATH=str(output/'isolated/settings.ini'))
    return env


def run_product(binary, arguments, env, output, label, *, timeout=30):
    result = subprocess.run([str(binary), *arguments], env=env, capture_output=True,
                            text=True, timeout=timeout)
    # Native diagnostics can mention the SDK or home directory. Keep useful
    # local evidence without exposing private absolute paths in copied logs.
    log = (result.stdout+result.stderr).replace(str(ROOT), '<workspace>')
    log = log.replace(str(Path.home()), '<home>')
    (output/(label+'.log')).write_text(log, encoding='utf-8')
    if result.returncode:
        raise ValueError(label+' failed; see local verification log')
    return result.stdout


def verify_signatures(app):
    images = mach_o_files(app)
    if not images:
        raise ValueError('No native code found for signature verification')
    for path in images:
        macdmg.checked(['codesign', '--verify', '--strict', '--all-architectures', path])
    macdmg.checked(['codesign', '--verify', '--deep', '--strict', '--all-architectures', app])
    return {'status': 'passed', 'native_binary_count': len(images),
            'bundle_deep_strict': True, 'all_architectures': True}


def product_identity(binary, env):
    text = macdmg.checked([binary, '--version'], env=env).strip()
    match = re.fullmatch(r'xdzip (\S+) \(build (\d+), ([0-9a-f]{12})\)', text)
    if not match:
        raise ValueError('Deployed application needs a clean verifiable build identity')
    return text, {'version': match[1], 'build': int(match[2]), 'revision': match[3], 'dirty': False}


def extraction_fixture(output):
    fixtures = output/'fixtures'
    fixtures.mkdir()
    path = fixtures/'打开 中文.zip'
    payloads = {'item-000.txt': '部署后的真实解压验证\n'.encode('utf-8'),
                '目录/item-000.bin': bytes(range(256))*1024, 'empty.txt': b''}
    directories = ['目录', 'empty-directory']
    with zipfile.ZipFile(path, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name in directories:
            archive.writestr(name+'/', b'')
        for name, payload in payloads.items():
            archive.writestr(name, payload)
    expected = {name: {'bytes': len(payload), 'sha256': hashlib.sha256(payload).hexdigest()}
                for name, payload in payloads.items()}
    return path, expected, directories


def verify_extraction(binary, env, output, identity):
    fixture, expected, directories = extraction_fixture(output)
    target = output/'解压 中文 space'
    target.mkdir()
    probe_env = dict(env, XDZIP_PROBE_OPERATION='extract',
                     XDZIP_PROBE_EXTRACT_TARGET=str(target), XDZIP_PROBE_IDLE_MS='0')
    text = run_product(binary, ['--desktop-probe', str(fixture)], probe_env, output,
                       'relocated-product-extraction', timeout=45)
    events = [json.loads(line.removeprefix('XDZIP_PROBE ')) for line in text.splitlines()
              if line.startswith('XDZIP_PROBE ')]
    by = {event['event']: event for event in events}
    required = {'first_frame', 'open_ready', 'search_frame', 'sort_frame',
                'browse_finished', 'extract_requested', 'extract_finished', 'quit_requested'}
    if not required.issubset(by):
        raise ValueError('Deployed product did not complete its real open/query/extract/quit flow')
    frame = by['first_frame']
    if (frame.get('backend') != 'metal' or frame.get('architecture') != 'arm64' or
            frame.get('runtime_available') is not True or frame.get('dirty') is not False or
            any(frame.get(key) != identity[key] for key in ['version', 'build', 'revision'])):
        raise ValueError('Deployed product frame identity or native Metal backend mismatch')
    expected_bytes = sum(item['bytes'] for item in expected.values())
    if (by['open_ready'].get('records') != len(expected)+len(directories) or
            by['search_frame'].get('rows', 0) < 1 or
            by['extract_finished'].get('status') != 'succeeded' or
            by['extract_finished'].get('committed_bytes') != str(expected_bytes)):
        raise ValueError('Deployed product did not open or extract the expected fixture records')
    paths = list(target.rglob('*'))
    if any(path.is_symlink() for path in paths):
        raise ValueError('Unexpected link in plain-file extraction fixture')
    actual = {path.relative_to(target).as_posix(): {'bytes': path.stat().st_size, 'sha256': digest(path)}
              for path in paths if path.is_file()}
    actual_directories = sorted(path.relative_to(target).as_posix() for path in paths if path.is_dir())
    if actual != expected or actual_directories != sorted(directories):
        raise ValueError('Deployed extraction failed independent payload or directory verification')
    return {'status': 'passed', 'fixture_sha256': digest(fixture),
            'records': len(expected)+len(directories), 'committed_bytes': expected_bytes,
            'files': actual, 'directories': actual_directories, 'events': events,
            'independent_byte_check': True, 'backend': 'metal'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dmg', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if not within(output, ROOT/'build') or output.exists():
        parser.error('Use a new verification directory under build/')
    app = output/'中文 space/xdzip.app'
    with temporary_bundle_path(app), transient_registrations(output):
        output.mkdir(parents=True)
        mounted = plistlib.loads(subprocess.check_output(
            ['hdiutil', 'attach', '-readonly', '-nobrowse', '-noautoopen', '-plist', str(args.dmg.resolve())]))
        mount = next(Path(e['mount-point']) for e in mounted['system-entities'] if 'mount-point' in e)
        try:
            if not (mount/'Applications').is_symlink() or os.readlink(mount/'Applications') != '/Applications':
                raise ValueError('Drag-to-Applications entry missing')
            shutil.copytree(mount/'xdzip.app', app, symlinks=True)
        finally:
            subprocess.run(['hdiutil', 'detach', str(mount)], check=True, capture_output=True)
        binary = macdmg.validate_bundle(app)
        optimization = validate_optimized_bundle(app)
        signatures = verify_signatures(app)
        report = audit(app, binary, None, 'macos', 'deployed')
        if report['status'] != 'passed':
            raise ValueError('Relocated DMG still depends on external SDK libraries')
        env = clean_environment(output)
        identity_text, identity = product_identity(binary, env)
        run_product(binary, ['--smoke-test'], env, output, 'relocated-cocoa-metal')
        extraction = verify_extraction(binary, env, output, identity)
        core = app/'Contents/Frameworks/QtCore.framework/Versions/A/QtCore'
        before = digest(core)
        # Change a library load identity in an isolated copy, then ad-hoc re-sign.
        # This verifies replaceability of a modified compatible image, not a rebuild
        # of Qt from source or arbitrary ABI compatibility.
        macdmg.checked(['install_name_tool', '-id', '@rpath/QtCore-replacement-proof', core])
        macdmg.sign_local(app)
        modified = digest(core)
        if modified == before:
            raise ValueError('Replacement proof did not modify the framework')
        validate_optimized_bundle(app)
        replacement_signatures = verify_signatures(app)
        run_product(binary, ['--smoke-test'], env, output, 'modified-qtcore-cocoa-metal')
    result = {'schema_version': 2, 'status': 'passed', 'dmg_sha256': digest(args.dmg),
              'dmg_bytes': args.dmg.stat().st_size, 'platform': 'macos-arm64',
              'identity_text': identity_text, 'identity': identity,
              'optimization': optimization, 'signatures': signatures, 'extraction': extraction,
              'cases': ['read-only-mount', 'applications-link', 'unicode-space-relocation',
                        'strict-arm64-native-images', 'basic-only-quick-controls',
                        'strict-nested-code-signatures', 'sdk-independent-native-imports',
                        'cocoa-metal-smoke', 'real-zip-open-query-extract-byte-check',
                        'modified-qtcore-ad-hoc-smoke'],
              'replacement': {'original_sha256': before, 'modified_sha256': modified,
                              'kind': 'Mach-O load-identity modification', 'signatures': replacement_signatures},
              'developer_id_signed': False, 'notarized': False,
              'limits': ['Current host has SDK installed; clean search paths and recursive import audit, not a separate clean machine.',
                         'No Finder drag/drop, default selection, Gatekeeper trust or source-rebuilt Qt validation claimed.']}
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    print('DMG arm64/Basic, strict signatures, native startup, real ZIP extraction and QtCore replacement passed')


if __name__ == '__main__':
    main()
