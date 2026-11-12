#!/usr/bin/env python3
"""Build an unsigned experimental MSIX from a clean, audited Windows runtime."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import xml.etree.ElementTree as ET
import zipfile
import hashlib

from association_types import WINDOWS_ICONS
from deployment_audit import digest
from installer_payload import verify_payload

ROOT = Path(__file__).resolve().parents[1]
NS = 'http://schemas.microsoft.com/appx/manifest/foundation/windows10'
UAP = 'http://schemas.microsoft.com/appx/manifest/uap/windows10'
RESCAP = 'http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities'
ET.register_namespace('', NS)
ET.register_namespace('uap', UAP)
ET.register_namespace('rescap', RESCAP)


def tag(namespace, name):
    return '{' + namespace + '}' + name


def manifest(name, publisher, build):
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9.-]{2,49}', name) or name.endswith('.'):
        raise ValueError('Invalid package identity name')
    if not publisher.startswith('CN=') or any(ord(c) < 32 for c in publisher) or len(publisher) > 512:
        raise ValueError('Publisher must be an explicit certificate subject')
    if type(build) is not int or not 1 <= build <= 65535:
        raise ValueError('Product build must fit the MSIX version')
    package = ET.Element(tag(NS, 'Package'), {'IgnorableNamespaces': 'uap rescap'})
    ET.SubElement(package, tag(NS, 'Identity'), Name=name, Publisher=publisher,
                  Version=f'1.0.{build}.0', ProcessorArchitecture='x64')
    props = ET.SubElement(package, tag(NS, 'Properties'))
    ET.SubElement(props, tag(NS, 'DisplayName')).text = 'xdzip'
    ET.SubElement(props, tag(NS, 'PublisherDisplayName')).text = 'xdzip'
    ET.SubElement(props, tag(NS, 'Logo')).text = 'Assets\\logo50.png'
    deps = ET.SubElement(package, tag(NS, 'Dependencies'))
    ET.SubElement(deps, tag(NS, 'TargetDeviceFamily'), Name='Windows.Desktop',
                  MinVersion='10.0.19045.0', MaxVersionTested='10.0.22000.0')
    resources = ET.SubElement(package, tag(NS, 'Resources'))
    ET.SubElement(resources, tag(NS, 'Resource'), Language='en-us')
    apps = ET.SubElement(package, tag(NS, 'Applications'))
    app = ET.SubElement(apps, tag(NS, 'Application'), Id='App', Executable='xdzip.exe',
                        EntryPoint='Windows.FullTrustApplication')
    ET.SubElement(app, tag(UAP, 'VisualElements'), DisplayName='xdzip',
                  Description='xdzip archive manager', BackgroundColor='transparent',
                  Square150x150Logo='Assets\\logo150.png', Square44x44Logo='Assets\\logo44.png')
    extensions = ET.SubElement(app, tag(NS, 'Extensions'))
    for ext in WINDOWS_ICONS:
        extension = ET.SubElement(extensions, tag(UAP, 'Extension'), Category='windows.fileTypeAssociation')
        association = ET.SubElement(extension, tag(UAP, 'FileTypeAssociation'), Name='xdzip-' + ext)
        types = ET.SubElement(association, tag(UAP, 'SupportedFileTypes'))
        ET.SubElement(types, tag(UAP, 'FileType')).text = '.' + ext
    capabilities = ET.SubElement(package, tag(NS, 'Capabilities'))
    ET.SubElement(capabilities, tag(RESCAP, 'Capability'), Name='runFullTrust')
    return ET.tostring(package, encoding='utf-8', xml_declaration=True)


def sdk_tool():
    root = Path(os.environ.get('ProgramFiles(x86)', 'C:/Program Files (x86)')) / 'Windows Kits/10/bin'
    candidates = sorted((p for p in root.glob('10.*/x64/makeappx.exe') if p.is_file()),
                        key=lambda p: tuple(map(int, p.parent.parent.name.split('.'))), reverse=True)
    if not candidates:
        raise ValueError('Windows SDK x64 MakeAppx.exe is required')
    return candidates[0]


def build(root, audit_path, output, name, publisher, report):
    root, audit_path, output, report = (p.resolve() for p in (root, audit_path, output, report))
    build_root = (ROOT / 'build').resolve()
    if any(not p.is_relative_to(build_root) or p == build_root for p in (root, output, report)):
        raise ValueError('Use dedicated paths under build/')
    if output.is_relative_to(root) or report.is_relative_to(root) or output == report:
        raise ValueError('Output and report must be outside the audited runtime')
    if output.exists() or report.exists():
        raise ValueError('Output or report already exists')
    audit = json.loads(audit_path.read_text(encoding='utf-8'))
    if (audit.get('status') != 'passed' or audit.get('platform') != 'windows' or
            audit.get('mode') != 'deployed' or audit.get('identity', {}).get('dirty') is not False):
        raise ValueError('MSIX requires a clean deployed Windows audit')
    verify_payload(root, audit)
    identity = audit['identity']
    product = subprocess.run([str(root / 'xdzip.exe'), '--version-json'], check=True,
                             capture_output=True, text=True, encoding='utf-8')
    product_identity = json.loads(product.stdout)
    if (product_identity.get('packaged_update_policy') != 1 or
            any(product_identity.get(key) != identity.get(key)
                for key in ('version', 'build', 'revision', 'dirty'))):
        raise ValueError('Audited product does not provide the MSIX update policy')
    content = manifest(name, publisher, identity.get('build'))
    stage = output.parent / (output.stem + '-stage')
    if stage.exists() or stage == root or stage.is_relative_to(root):
        raise ValueError('MSIX staging directory exists or overlaps the audited runtime')
    if any((root / entry).exists() for entry in ('AppxManifest.xml', 'Assets')):
        raise ValueError('Runtime contains reserved MSIX paths')
    tool = sdk_tool()
    shutil.copytree(root, stage)
    verify_payload(stage, audit)
    (stage / 'Assets').mkdir()
    subprocess.run(['pwsh', '-NoProfile', '-NonInteractive', '-File',
                    str(ROOT / 'scripts/prepare-msix-assets.ps1'), '-Source',
                    str(ROOT / 'design/assets/brand/xdzip-256.png'), '-OutputDirectory',
                    str(stage / 'Assets')], check=True, capture_output=True)
    (stage / 'AppxManifest.xml').write_bytes(content)
    subprocess.run([str(tool), 'pack', '/d', str(stage), '/p', str(output), '/o'],
                   check=True, capture_output=True)
    if not output.is_file() or not output.stat().st_size:
        raise ValueError('MakeAppx did not produce a package')
    expected = {p.relative_to(stage).as_posix().casefold(): digest(p)
                for p in stage.rglob('*') if p.is_file()}
    with zipfile.ZipFile(output) as archive:
        files = [item for item in archive.infolist() if not item.is_dir()]
        actual = {item.filename.replace('\\', '/').casefold(): item for item in files}
        if len(actual) != len(files) or set(actual) != set(expected) | {'appxblockmap.xml', '[content_types].xml'}:
            raise ValueError('MSIX contents do not match the staging directory')
        for member_name, expected_hash in expected.items():
            value = hashlib.sha256()
            with archive.open(actual[member_name]) as source:
                for block in iter(lambda: source.read(1024 * 1024), b''):
                    value.update(block)
            if value.hexdigest() != expected_hash:
                raise ValueError('MSIX payload differs from audited staging')
    report.write_text(json.dumps({'schema_version': 1, 'status': 'passed', 'package': output.name,
        'sha256': digest(output), 'audit_sha256': digest(audit_path), 'name': name,
        'publisher': publisher, 'version': f"1.0.{identity['build']}.0",
        'signed': False, 'store_submission': False, 'modern_context_menu': False,
        'minimum_windows_build': 19045}, indent=2) + '\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ('root', 'audit', 'output', 'report'):
        parser.add_argument('--' + arg, type=Path, required=True)
    parser.add_argument('--name', required=True)
    parser.add_argument('--publisher', required=True)
    args = parser.parse_args()
    build(args.root, args.audit, args.output, args.name, args.publisher, args.report)


if __name__ == '__main__':
    main()
