#!/usr/bin/env python3
"""Make a same-payload version upgrade fixture for the owned MSIX CI package."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import xml.etree.ElementTree as ET
import zipfile

from desktop import verify_msix_identity
from deployment_audit import digest

ROOT = Path(__file__).resolve().parents[1]
NS = 'http://schemas.microsoft.com/appx/manifest/foundation/windows10'
ET.register_namespace('', NS)
ET.register_namespace('uap', 'http://schemas.microsoft.com/appx/manifest/uap/windows10')
ET.register_namespace('rescap', 'http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities')


def advance_manifest(content):
    root = ET.fromstring(content)
    identity = root.find('{' + NS + '}Identity')
    if (identity is None or identity.get('Name') != 'xdzip.MsixCI' or
            identity.get('Publisher') != 'CN=xdzip MSIX CI'):
        raise ValueError('Only the owned CI identity may be used for this fixture')
    try:
        version = [int(v) for v in identity.get('Version', '').split('.')]
    except ValueError as error:
        raise ValueError('Invalid fixture version') from error
    if len(version) != 4 or version[:2] != [1, 0] or not 1 <= version[2] < 65535 or version[3] != 0:
        raise ValueError('Cannot advance the fixture package version')
    old_version = identity.get('Version')
    new_version = '1.0.' + str(version[2] + 1) + '.0'
    identity.set('Version', new_version)
    return ET.tostring(root, encoding='utf-8', xml_declaration=True), old_version, new_version


def prepare(package, receipt, output, tool):
    package, receipt, output = (Path(p).resolve() for p in (package, receipt, output))
    build = (ROOT / 'build').resolve()
    stage = output.parent / (output.stem + '-stage')
    report = output.with_suffix('.json')
    if any(not p.is_relative_to(build) or p == build for p in (package, receipt, output, stage, report)):
        raise ValueError('Use dedicated fixture paths below build/')
    if any(p.exists() for p in (output, stage, report)):
        raise ValueError('Upgrade fixture output already exists')
    if not output.parent.is_dir():
        raise ValueError('Fixture output parent must exist')
    source = json.loads(receipt.read_text(encoding='utf-8'))
    if (source.get('sha256') != digest(package) or source.get('signed') is not False or
            source.get('modern_context_menu') is not False):
        raise ValueError('Unsigned source package differs from its receipt')
    verify_msix_identity(package, source)
    with zipfile.ZipFile(package) as archive:
        manifest, old_version, new_version = advance_manifest(archive.read('AppxManifest.xml'))
        entries = []
        for info in archive.infolist():
            path = PurePosixPath(info.filename.replace('\\', '/'))
            if (path.is_absolute() or '..' in path.parts or ':' in str(path) or
                    ((info.external_attr >> 16) & 0o170000) == 0o120000):
                raise ValueError('Unsafe MSIX fixture member')
            if not info.is_dir() and str(path).lower() not in ('appxblockmap.xml', '[content_types].xml'):
                entries.append((info, path))
        stage.mkdir()
        expected = {}
        for info, path in entries:
            target = stage / str(path)
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.open(info) as source_file, target.open('xb') as destination:
                shutil.copyfileobj(source_file, destination)
            if str(path) == 'AppxManifest.xml':
                target.write_bytes(manifest)
            expected[str(path).casefold()] = digest(target)
    subprocess.run([str(tool), 'pack', '/d', str(stage), '/p', str(output), '/o'],
                   check=True, capture_output=True)
    upgraded_receipt = {**source, 'version': new_version}
    verify_msix_identity(output, upgraded_receipt)
    with zipfile.ZipFile(output) as archive:
        actual = {i.filename.replace('\\', '/').casefold(): i for i in archive.infolist() if not i.is_dir()}
        if set(actual) != set(expected) | {'appxblockmap.xml', '[content_types].xml'}:
            raise ValueError('Upgrade fixture has unexpected payload')
        for name, expected_hash in expected.items():
            with archive.open(actual[name]) as stream:
                if hashlib.file_digest(stream, 'sha256').hexdigest() != expected_hash:
                    raise ValueError('Upgrade fixture payload changed during packaging')
    result = {
        'schema_version': 1, 'status': 'passed', 'role': 'same-payload-version-fixture',
        'source_sha256': digest(package), 'sha256': digest(output),
        'old_version': old_version, 'new_version': new_version,
        'limits': 'Only the MSIX manifest version advances; this is not a new product release.',
    }
    report.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ('package', 'receipt', 'output', 'makeappx'):
        parser.add_argument('--' + arg, type=Path, required=True)
    args = parser.parse_args()
    if (os.name != 'nt' or os.environ.get('GITHUB_ACTIONS') != 'true' or
            os.environ.get('RUNNER_ENVIRONMENT') != 'github-hosted'):
        raise ValueError('This fixture entry point requires disposable hosted Windows CI')
    prepare(args.package, args.receipt, args.output, args.makeappx)
    print('Prepared owned same-payload MSIX version fixture')


if __name__ == '__main__':
    main()
