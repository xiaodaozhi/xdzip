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
from msix_shell_policy import COM, D4, D5, D9, CLASSIC_STORE, CLASSIC_SIDELOAD, MODERN, marker, verify_manifest

ROOT = Path(__file__).resolve().parents[1]
LANGUAGES = ('en-US', 'zh-CN')
NS = 'http://schemas.microsoft.com/appx/manifest/foundation/windows10'
UAP = 'http://schemas.microsoft.com/appx/manifest/uap/windows10'
RESCAP = 'http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities'
ET.register_namespace('', NS)
ET.register_namespace('uap', UAP)
ET.register_namespace('rescap', RESCAP)
for prefix, uri in [('com', COM), ('desktop4', D4), ('desktop5', D5), ('desktop9', D9)]:
    ET.register_namespace(prefix, uri)


def tag(namespace, name):
    return '{' + namespace + '}' + name


def manifest(name, publisher, build, publisher_display_name, channel='sideload', shell_library='xdzip-shell.dll'):
    if channel not in ('store', 'sideload'):
        raise ValueError('Unknown MSIX channel')
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9.-]{2,49}', name) or name.endswith('.'):
        raise ValueError('Invalid package identity name')
    if not publisher.startswith('CN=') or any(ord(c) < 32 for c in publisher) or len(publisher) > 512:
        raise ValueError('Publisher must be an explicit certificate subject')
    if (not publisher_display_name.strip() or len(publisher_display_name) > 256 or
            any(ord(c) < 32 or 0xD800 <= ord(c) <= 0xDFFF or ord(c) in (0xFFFE, 0xFFFF)
                for c in publisher_display_name)):
        raise ValueError('Invalid publisher display name')
    if type(build) is not int or not 1 <= build <= 65535:
        raise ValueError('Product build must fit the MSIX version')
    package = ET.Element(tag(NS, 'Package'), {'IgnorableNamespaces': 'uap rescap com desktop9' + (' desktop4 desktop5' if channel == 'store' else '')})
    ET.SubElement(package, tag(NS, 'Identity'), Name=name, Publisher=publisher,
                  Version=f'1.0.{build}.0', ProcessorArchitecture='x64')
    props = ET.SubElement(package, tag(NS, 'Properties'))
    ET.SubElement(props, tag(NS, 'DisplayName')).text = 'xdzip'
    ET.SubElement(props, tag(NS, 'PublisherDisplayName')).text = publisher_display_name
    ET.SubElement(props, tag(NS, 'Logo')).text = 'Assets\\logo50.png'
    deps = ET.SubElement(package, tag(NS, 'Dependencies'))
    ET.SubElement(deps, tag(NS, 'TargetDeviceFamily'), Name='Windows.Desktop',
                  MinVersion='10.0.19045.0', MaxVersionTested='10.0.22621.1555')
    resources = ET.SubElement(package, tag(NS, 'Resources'))
    for language in LANGUAGES:
        ET.SubElement(resources, tag(NS, 'Resource'), Language=language)
    apps = ET.SubElement(package, tag(NS, 'Applications'))
    app = ET.SubElement(apps, tag(NS, 'Application'), Id='App', Executable='xdzip.exe',
                        EntryPoint='Windows.FullTrustApplication')
    ET.SubElement(app, tag(UAP, 'VisualElements'), DisplayName='xdzip',
                  Description='xdzip archive manager', BackgroundColor='transparent',
                  Square150x150Logo='Assets\\logo150.png', Square44x44Logo='Assets\\logo44.png')
    extensions = ET.SubElement(app, tag(NS, 'Extensions'))
    server = ET.SubElement(extensions, tag(COM, 'Extension'), Category='windows.comServer')
    server = ET.SubElement(ET.SubElement(server, tag(COM, 'ComServer')), tag(COM, 'SurrogateServer'), DisplayName='xdzip')
    classic = CLASSIC_STORE if channel == 'store' else CLASSIC_SIDELOAD
    for clsid in ([classic, MODERN] if channel == 'store' else [classic]):
        ET.SubElement(server, tag(COM, 'Class'), Id=clsid, Path=shell_library.replace('/', '\\'), ThreadingModel='STA')
    legacy = ET.SubElement(extensions, tag(D9, 'Extension'), Category='windows.fileExplorerClassicContextMenuHandler')
    legacy = ET.SubElement(legacy, tag(D9, 'FileExplorerClassicContextMenuHandler'))
    for item in ('*', 'Directory'):
        ET.SubElement(legacy, tag(D9, 'ExtensionHandler'), Type=item, Clsid=classic)
    if channel == 'store':
        modern = ET.SubElement(extensions, tag(D4, 'Extension'), Category='windows.fileExplorerContextMenus')
        modern = ET.SubElement(modern, tag(D4, 'FileExplorerContextMenus'))
        for item in ('*', 'Directory'):
            owner = ET.SubElement(modern, tag(D5, 'ItemType'), Type=item)
            ET.SubElement(owner, tag(D5, 'Verb'), Id='xdzip', Clsid=MODERN)
    for ext in WINDOWS_ICONS:
        extension = ET.SubElement(extensions, tag(UAP, 'Extension'), Category='windows.fileTypeAssociation')
        association = ET.SubElement(extension, tag(UAP, 'FileTypeAssociation'), Name='xdzip-' + ext)
        types = ET.SubElement(association, tag(UAP, 'SupportedFileTypes'))
        ET.SubElement(types, tag(UAP, 'FileType')).text = '.' + ext
    capabilities = ET.SubElement(package, tag(NS, 'Capabilities'))
    ET.SubElement(capabilities, tag(RESCAP, 'Capability'), Name='runFullTrust')
    verify_manifest(package, channel, shell_library)
    return ET.tostring(package, encoding='utf-8', xml_declaration=True)


def sdk_tool():
    root = Path(os.environ.get('ProgramFiles(x86)', 'C:/Program Files (x86)')) / 'Windows Kits/10/bin'
    candidates = sorted((p for p in root.glob('10.*/x64/makeappx.exe') if p.is_file()),
                        key=lambda p: tuple(map(int, p.parent.parent.name.split('.'))), reverse=True)
    if not candidates:
        raise ValueError('Windows SDK x64 MakeAppx.exe is required')
    return candidates[0]


def shell_library_path(root):
    # Normal deployment names the DLL by digest to protect installed-edition
    # upgrades. Bind COM to that audited file without duplicating/renaming it.
    metadata = root / 'shell/integration.json'
    if metadata.is_file():
        data = json.loads(metadata.read_text(encoding='utf-8'))
        filename = data.get('file', '')
        if (data.get('schema_version') != 1 or
                not re.fullmatch(r'xdzip-shell-[a-f0-9]{16}\.dll', filename)):
            raise ValueError('Invalid audited shell metadata')
        path = root / 'shell' / filename
        actual = digest(path)
        if data.get('sha256') != actual or filename != 'xdzip-shell-' + actual[:16] + '.dll':
            raise ValueError('Audited shell digest differs')
        return 'shell/' + filename
    if not (root / 'xdzip-shell.dll').is_file():
        raise ValueError('Audited runtime lacks the shell extension')
    return 'xdzip-shell.dll'


def build(root, audit_path, output, name, publisher, report, publisher_display_name, channel='sideload'):
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
    if any(product_identity.get(key) != 1 for key in ('packaged_shell_policy', 'packaged_association_policy')):
        raise ValueError('Audited product does not provide the MSIX integration policies')
    shell_library = shell_library_path(root)
    content = manifest(name, publisher, identity.get('build'), publisher_display_name, channel, shell_library)
    stage = output.parent / (output.stem + '-stage')
    if stage.exists() or stage == root or stage.is_relative_to(root):
        raise ValueError('MSIX staging directory exists or overlaps the audited runtime')
    if any((root / entry).exists() for entry in ('AppxManifest.xml', 'Assets', 'xdzip-msix-channel.txt')):
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
    (stage / 'xdzip-msix-channel.txt').write_bytes(marker(channel))
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
        'publisher': publisher, 'publisher_display_name': publisher_display_name,
        'version': f"1.0.{identity['build']}.0",
        'signed': False, 'store_submission': False, 'modern_context_menu': False,
        'channel': channel, 'classic_context_menu': True, 'modern_context_menu_declaration': channel == 'store',
        'shell_library': shell_library,
        'minimum_windows_build': 19045, 'languages': list(LANGUAGES)}, indent=2) + '\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ('root', 'audit', 'output', 'report'):
        parser.add_argument('--' + arg, type=Path, required=True)
    parser.add_argument('--name', required=True)
    parser.add_argument('--publisher', required=True)
    parser.add_argument('--publisher-display-name', required=True)
    parser.add_argument('--channel', choices=['store', 'sideload'], default='sideload')
    args = parser.parse_args()
    build(args.root, args.audit, args.output, args.name, args.publisher, args.report,
          args.publisher_display_name, args.channel)


if __name__ == '__main__':
    main()
