# scripts/prepare-release.py

"""Assemble six verified native package assets and immutable updater metadata."""
import argparse
import json
import os
from pathlib import Path, PurePosixPath
import runpy
import shutil
import subprocess
import tarfile
import zipfile
from release_protocol import TARGETS, create, sha256

ROOT = Path(__file__).resolve().parents[1]


def materials_archive(source, output):
    notices = source/'notices'
    if not notices.is_dir():
        raise ValueError('Required notices are missing')
    roots = {notices}
    attribution = source/'runtime-attribution.json'
    if attribution.is_file():
        report = json.loads(attribution.read_text(encoding='utf-8'))
        relative = report.get('license_materials_base', '')
        if not relative or PurePosixPath(relative).is_absolute() or '..' in PurePosixPath(relative).parts or '\\' in relative or ':' in relative:
            raise ValueError('Unbounded runtime license material base')
        base = source/relative
        if not base.resolve().is_relative_to(source.resolve()):
            raise ValueError('Runtime license material escapes package')
        for item in report['license_materials'].values():
            path = base/item['file']
            if not path.resolve().is_relative_to(base.resolve()) or not path.is_file() or sha256(path) != item['sha256']:
                raise ValueError('Referenced runtime license text is missing or changed')
        if not base.is_relative_to(notices):
            roots.add(base)
    with tarfile.open(output, 'w:gz') as archive:
        for path in sorted(source.glob('*.json')):
            archive.add(path, arcname=path.name)
        for path in sorted(roots):
            archive.add(path, arcname=path.relative_to(source))


def stage(source, output, platform, version):
    receipt = json.loads((source/'verify.json').read_text(encoding='utf-8'))
    if receipt.get('status') != 'passed' or receipt.get('platform') != platform:
        raise ValueError('Native package verification did not pass')
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True, cwd=ROOT).strip()
    build = json.loads((source/'build-version.json').read_text(encoding='utf-8')) if platform != 'macos' else None
    if build and build['version'] != version:
        raise ValueError('Verified package version mismatch')
    package = json.loads((source/'package.json').read_text(encoding='utf-8'))
    if package.get('source', {}).get('revision') != revision or package['source']['dirty']:
        raise ValueError('Verified packages must match this clean commit')
    artifact_receipts = [package] if platform == 'macos' else [json.loads((source/'package-installer.json').read_text())]
    for artifact_receipt in artifact_receipts:
        if artifact_receipt.get('source') != package['source']:
            raise ValueError('Installer source identity mismatch')
        for artifact in artifact_receipt['artifacts']:
            path = (source/artifact['file']).resolve()
            if not path.is_relative_to(source.resolve()) or path.stat().st_size != artifact['bytes'] or sha256(path) != artifact['sha256']:
                raise ValueError('Verified package bytes changed before staging')
    if platform != 'macos':
        from installer_payload import verify_payload
        verify_payload(source/platform, json.loads((source/'deployment-audit.json').read_text()))
    output.mkdir(parents=True, exist_ok=True)
    files = {
        'windows': [('windows-x86_64-setup.exe', source/'xdzip-windows-x64-setup.exe')],
        'linux': [('linux-x86_64.tar.gz', source/'xdzip-linux-x86_64.tar.gz'), ('ubuntu24.04-x86_64.deb', source/'xdzip-ubuntu24.04-amd64.deb')],
        'macos': [('macos-arm64.dmg', source/f'xdzip-{version}-macos-arm64.dmg'), ('macos-arm64.app.zip', source/f'xdzip-{version}-macos-arm64.app.zip')],
    }[platform]
    for suffix, path in files:
        shutil.copy2(path, output/f'xdzip-{version}-{suffix}')
    if platform == 'macos':
        from sparkle_bundle import sign_staged_archive
        build_number = runpy.run_path(str(ROOT/'scripts/sync-version.py'))['version_info']()['build']
        sign_staged_archive(output, version, build_number)
    if platform == 'windows':
        # The runtime was already audited byte-for-byte. Include its complete
        # relative tree, with no SDK tools or developer filesystem locations.
        target = output/f'xdzip-{version}-windows-x86_64.zip'
        with zipfile.ZipFile(target, 'w', zipfile.ZIP_DEFLATED) as archive:
            for path in sorted((source/'windows').rglob('*')):
                if path.is_file(): archive.write(path, path.relative_to(source/'windows'))
        with zipfile.ZipFile(target) as archive:
            if archive.testzip() is not None: raise ValueError('Portable ZIP integrity failure')
    materials = output/f'xdzip-{version}-{platform}-materials.tar.gz'
    materials_archive(source, materials)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--packages', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--platform', choices=['windows', 'linux', 'macos'])
    parser.add_argument('--repository', default='xiaodaozhi/xdzip')
    parser.add_argument('--update-key-id', default=os.environ.get('XDZIP_UPDATE_KEY_ID'))
    signing = parser.add_mutually_exclusive_group()
    signing.add_argument('--require-update-signature', action='store_true')
    signing.add_argument('--validation-only', action='store_true',
                         help='Assemble an unsigned candidate without using update signing keys')
    args = parser.parse_args()
    if args.validation_only and args.platform:
        parser.error('--validation-only applies only to complete candidate assembly')
    for value in (args.packages, args.output):
        if not value.resolve().is_relative_to(ROOT/'build'):
            raise ValueError('Release staging belongs under build/')
    version = runpy.run_path(str(ROOT/'scripts/sync-version.py'))['version_info']()
    if args.platform:
        stage(args.packages, args.output, args.platform, version['display_version'])
        return
    if args.validation_only:
        for directory in (args.packages, args.output):
            for name in ('release.json.sig', 'appcast.xml', 'macos-update.json'):
                path = directory/name
                if path.exists() or path.is_symlink():
                    raise ValueError('Validation-only candidate contains update signing material')
    args.output.mkdir(parents=True, exist_ok=True)
    for path in args.packages.iterdir():
        if path.is_file(): shutil.copy2(path, args.output/path.name)
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True, cwd=ROOT).strip()
    manifest = create(args.output, version, args.repository, revision)
    if args.validation_only:
        appcast_signed = False
    else:
        from release_updates import prepare_appcast
        appcast_signed = prepare_appcast(args.output, manifest, required=args.require_update_signature)
    (args.output/'release.json').write_text(json.dumps(manifest, indent=2)+'\n', encoding='utf-8')
    if args.validation_only:
        manifest_signed = False
    else:
        from release_updates import sign_manifest
        manifest_signed = sign_manifest(args.output, key_id=args.update_key_id, required=args.require_update_signature)
    (args.output/'update-readiness.json').write_text(json.dumps(dict(schema_version=1,
        signed_manifest=manifest_signed, signed_appcast=appcast_signed,
        status='authenticated' if manifest_signed and appcast_signed else 'signing_not_configured'), indent=2)+'\n', encoding='utf-8')
    files = sorted(path for path in args.output.iterdir() if path.is_file() and path.name != 'SHA256SUMS')
    (args.output/'SHA256SUMS').write_text(''.join(f'{sha256(path)}  {path.name}\n' for path in files), encoding='utf-8')
    print('Complete immutable release manifest verified: ' + manifest['tag'])


if __name__ == '__main__':
    main()
