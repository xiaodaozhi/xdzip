#!/usr/bin/env python3
"""MSIX identity, channel and payload regression checks."""
import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET
import zipfile

from deployment_audit import digest
import msix_icon_assets as icons

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('msix_builder', ROOT / 'scripts/build-windows-msix.py')
msix = importlib.util.module_from_spec(spec)
spec.loader.exec_module(msix)


class MsixTests(unittest.TestCase):
    def test_associations_declare_format_icons_in_both_channels(self):
        for channel in ('store', 'sideload'):
            root = ET.fromstring(msix.manifest('xdzip.Test', 'CN=test', 28, 'xdzip', channel))
            icons.verify_manifest_icons(root)
            for association in root.iter(icons.UAP + 'FileTypeAssociation'):
                extension = association.get('Name').removeprefix('xdzip-')
                self.assertEqual(association.find(icons.UAP + 'Logo').text,
                                 icons.association_logo(msix.WINDOWS_ICONS[extension]).replace('/', '\\'))
            association = next(root.iter(icons.UAP + 'FileTypeAssociation'))
            association.remove(association.find(icons.UAP + 'Logo'))
            with self.assertRaisesRegex(ValueError, 'canonical format icon'):
                icons.verify_manifest_icons(root)

    def test_canonical_file_images_keep_all_original_size_profiles(self):
        assets = dict(icons.association_assets())
        for icon in set(msix.WINDOWS_ICONS.values()):
            frames = icons.ico_frames(ROOT / 'design/assets/file-associations' / f'xdzip-{icon}.ico')
            self.assertEqual(set(frames), {16, 24, 32, 48, 64, 128, 256})
            for size, frame in frames.items():
                self.assertEqual(assets[f'file-{icon}.targetsize-{size}.png'], frame)
            self.assertEqual(assets[f'file-{icon}.png'], frames[256])
        self.assertNotEqual(assets['file-zip.png'], assets['file-7z.png'])
        self.assertNotEqual(assets['file-zip.png'], (ROOT / 'design/assets/brand/xdzip-256.png').read_bytes())

    def test_unplated_icons_cover_shell_sizes_and_both_themes(self):
        assets = dict(icons.application_assets())
        for size in (16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 256):
            for suffix in ('', '_altform-unplated', '_altform-lightunplated'):
                self.assertEqual(assets[f'logo44.targetsize-{size}{suffix}.png'], size)
        for base, size in (('logo44', 44), ('logo150', 150)):
            self.assertEqual(assets[base + '.png'], size)
            self.assertEqual(assets[base + '.scale-200.png'], size * 2)
            self.assertEqual(assets[base + '.scale-400.png'], size * 4)

    def test_manifest_contains_associations_and_classic_only_for_sideload(self):
        data = msix.manifest('xdzip.Development', 'CN=xdzip development', 27, '测试 & Publisher')
        package = ET.fromstring(data)
        identity = package.find(msix.tag(msix.NS, 'Identity'))
        self.assertEqual(identity.attrib['Version'], '1.0.27.0')
        self.assertEqual(identity.attrib['ProcessorArchitecture'], 'x64')
        languages = [item.get('Language') for item in package.findall(
            msix.tag(msix.NS, 'Resources') + '/' + msix.tag(msix.NS, 'Resource'))]
        self.assertEqual(languages, list(msix.packaging_languages()))
        catalogs = set(msix.packaging_catalogs())
        self.assertEqual(set(languages), catalogs)
        self.assertEqual(package.find(msix.tag(msix.NS, 'Properties')).find(
            msix.tag(msix.NS, 'PublisherDisplayName')).text, '测试 & Publisher')
        extensions = {item.text for item in package.iter(msix.tag(msix.UAP, 'FileType'))}
        self.assertEqual(extensions, {'.' + ext for ext in msix.WINDOWS_ICONS})
        self.assertNotIn(b'fileExplorerContextMenus', data)
        self.assertIn(b'comServer', data)
        self.assertIn(b'windows.fileExplorerClassicContextMenuHandler', data)
        self.assertNotIn(msix.MODERN.encode(), data)
        self.assertIn(b'runFullTrust', data)

    def test_store_manifest_registers_only_modern_class(self):
        data = msix.manifest('xdzip.Store', 'CN=test', 27, 'xdzip', 'store')
        root = ET.fromstring(data)
        classes = {element.get('Id') for element in root.iter(msix.tag(msix.COM, 'Class'))}
        self.assertEqual(classes, {msix.MODERN})
        self.assertFalse(list(root.iter(msix.tag(msix.D9, 'ExtensionHandler'))))
        self.assertEqual({element.get('Clsid') for element in root.iter(msix.tag(msix.D5, 'Verb'))}, {msix.MODERN})
        self.assertNotIn(msix.CLASSIC_SIDELOAD.encode(), data)
        self.assertNotIn(msix.CLASSIC_STORE.encode(), data)
        application_extensions = next(root.iter(msix.tag(msix.NS, 'Extensions')))
        ET.SubElement(application_extensions, msix.tag(msix.D9, 'Extension'),
                      Category='windows.fileExplorerClassicContextMenuHandler')
        with self.assertRaisesRegex(ValueError, 'must not register a classic handler'):
            msix.verify_manifest(root, 'store')
        native = (ROOT / 'apps/desktop/shell/PackageMenu.h').read_text(encoding='utf-8')
        for clsid in (msix.CLASSIC_STORE, msix.CLASSIC_SIDELOAD, msix.MODERN):
            self.assertIn(clsid, native)
        with self.assertRaisesRegex(ValueError, 'channel'):
            msix.manifest('xdzip.Store', 'CN=test', 27, 'xdzip', 'unknown')

    def test_invalid_identity_and_version_are_rejected(self):
        for name, publisher, build in [('a', 'CN=test', 1), ('xdzip.test', 'invalid', 1),
                                       ('xdzip.test', 'CN=test', 0), ('xdzip.test', 'CN=test', 65536)]:
            with self.subTest(name=name, publisher=publisher, build=build):
                with self.assertRaises(ValueError):
                    msix.manifest(name, publisher, build, 'test')

    def test_invalid_publisher_display_name_is_rejected(self):
        for display in ('', '  ', 'a' * 257, 'bad\x01name', '\ud800'):
            with self.subTest(display=repr(display)), self.assertRaises(ValueError):
                msix.manifest('xdzip.Test', 'CN=test', 27, display)

    def test_old_product_and_tampered_runtime_are_rejected(self):
        build_dir = ROOT / 'build'
        build_dir.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='msix-test-', dir=build_dir) as temp:
            folder = Path(temp)
            runtime = folder / 'runtime'
            runtime.mkdir()
            app = runtime / 'xdzip.exe'
            app.write_bytes(b'fixture')
            identity = {'version': '0.2.0', 'build': 27, 'revision': 'a' * 12, 'dirty': False}
            audit = {'status': 'passed', 'platform': 'windows', 'mode': 'deployed',
                     'identity': identity, 'files': [{'path': 'xdzip.exe', 'sha256': digest(app)}]}
            audit_path = folder / 'audit.json'
            audit_path.write_text(json.dumps(audit), encoding='utf-8')
            result = unittest.mock.Mock(stdout=json.dumps(identity))
            with patch.object(msix.subprocess, 'run', return_value=result):
                with self.assertRaisesRegex(ValueError, 'update policy'):
                    msix.build(runtime, audit_path, folder / 'app.msix', 'xdzip.Test',
                               'CN=test', folder / 'report.json', 'Test Publisher')
            app.write_bytes(b'tampered')
            with self.assertRaisesRegex(ValueError, 'payload changed'):
                msix.build(runtime, audit_path, folder / 'app.msix', 'xdzip.Test',
                           'CN=test', folder / 'report.json', 'Test Publisher')

    @unittest.skipUnless(os.name == 'nt', 'requires native Windows SDK')
    def test_makeappx_roundtrip_with_owned_fixture(self):
        build_dir = ROOT / 'build'
        build_dir.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='msix-test-', dir=build_dir) as temp:
            folder = Path(temp)
            runtime = folder / 'runtime'
            runtime.mkdir()
            app = runtime / 'xdzip.exe'
            app.write_bytes(b'owned test fixture, not a runnable product')
            (runtime / 'shell').mkdir()
            library = runtime / 'shell/temporary.dll'
            library.write_bytes(b'owned shell fixture, not a runnable DLL')
            library = library.rename(library.with_name('xdzip-shell-' + digest(library)[:16] + '.dll'))
            metadata = runtime / 'shell/integration.json'
            metadata.write_text(json.dumps({'schema_version': 1, 'file': library.name, 'sha256': digest(library)}), encoding='utf-8')
            identity = {'version': '0.2.0', 'build': 27, 'revision': 'a' * 12, 'dirty': False}
            audit = {'status': 'passed', 'platform': 'windows', 'mode': 'deployed',
                     'identity': identity, 'files': [{'path': 'xdzip.exe', 'sha256': digest(app)}]}
            audit_path = folder / 'audit.json'
            audit['files'].extend([{'path': str(library.relative_to(runtime)).replace('\\', '/'), 'sha256': digest(library)},
                                   {'path': 'shell/integration.json', 'sha256': digest(metadata)}])
            audit_path.write_text(json.dumps(audit), encoding='utf-8')
            real_run = msix.subprocess.run
            def run(command, **kwargs):
                if command[0] == str(app):
                    return unittest.mock.Mock(stdout=json.dumps({**identity, 'packaged_update_policy': 1,
                        'packaged_shell_policy': 1, 'packaged_association_policy': 1}))
                return real_run(command, **kwargs)
            with patch.object(msix.subprocess, 'run', side_effect=run):
                msix.build(runtime, audit_path, folder / 'app.msix', 'xdzip.Test',
                           'CN=test', folder / 'report.json', 'Test Publisher')
                msix.build(runtime, audit_path, folder / 'store.msix', 'xdzip.StoreTest',
                           'CN=test', folder / 'store.json', 'Test Publisher', channel='store')
            store_report = json.loads((folder / 'store.json').read_text(encoding='utf-8'))
            self.assertFalse(store_report['classic_context_menu'])
            self.assertTrue(store_report['modern_context_menu_declaration'])
            import zipfile
            with zipfile.ZipFile(folder / 'store.msix') as package:
                self.assertEqual(package.read('xdzip-msix-channel.txt'), b'xdzip-msix-v1\nstore\n')
                msix.verify_manifest(ET.fromstring(package.read('AppxManifest.xml')), 'store',
                                     store_report['shell_library'])
            report = json.loads((folder / 'report.json').read_text(encoding='utf-8'))
            self.assertFalse(report['signed'])
            self.assertEqual(report['sha256'], digest(folder / 'app.msix'))
            self.assertEqual(report['name'], 'xdzip.Test')
            self.assertEqual(report['publisher'], 'CN=test')
            self.assertEqual(report['publisher_display_name'], 'Test Publisher')
            self.assertEqual(report['languages'], list(msix.packaging_languages()))
            self.assertEqual(report['shell_library'], 'shell/' + library.name)
            self.assertEqual(report['version'], '1.0.27.0')
            with zipfile.ZipFile(folder / 'app.msix') as package:
                self.assertEqual(package.read('xdzip-msix-channel.txt'), b'xdzip-msix-v1\nsideload\n')
                self.assertGreater(package.getinfo('resources.pri').file_size, 1024)
                root = ET.fromstring(package.read('AppxManifest.xml'))
                count = icons.verify_package_icons(package, root)
                self.assertEqual(report['icon_assets']['png_count'], count)
                files = {name: package.read(name) for name in package.namelist()}
            # Inspect the real SDK index, not just PNG filenames in the ZIP.
            dump = folder / 'resources.xml'
            msix.subprocess.run([str(msix.sdk_tool('makepri.exe')), 'dump', '/if',
                            str(folder / 'app-stage/resources.pri'), '/of', str(dump), '/dt', 'detailed', '/o'],
                           check=True, capture_output=True)
            index = dump.read_text(encoding='utf-8-sig')
            resources = {item.get('uri') for item in ET.fromstring(index).iter('NamedResource')}
            self.assertIn('ms-resource://xdzip.Test/Files/Assets/logo44.png', resources)
            self.assertIn('ms-resource://xdzip.Test/Resources/AppDescription', resources)
            for icon in set(msix.WINDOWS_ICONS.values()):
                self.assertIn('ms-resource://xdzip.Test/Files/' + icons.association_logo(icon), resources)
            for ext in msix.WINDOWS_ICONS:
                self.assertIn('ms-resource://xdzip.Test/Resources/Association_' + ext, resources)
            qualifiers = {(item.get('name'), item.get('value')) for item in ET.fromstring(index).iter('Qualifier')}
            self.assertIn(('AlternateForm', 'UNPLATED'), qualifiers)
            self.assertIn(('AlternateForm', 'LIGHTUNPLATED'), qualifiers)
            for name in dict(icons.application_assets()) | dict(icons.association_assets()):
                self.assertIn(name, index)
            pixels = folder / 'check-icon-pixels.ps1'
            pixels.write_text(r'''
param([string]$AssetsDirectory)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
foreach ($stem in @('logo44', 'file-zip', 'file-7z', 'file-rar', 'file-iso')) {
    foreach ($size in @(48, 128)) {
        $bitmap = [System.Drawing.Bitmap]::new((Join-Path $AssetsDirectory "$stem.targetsize-$size.png"))
        try {
            if ($bitmap.GetPixel(0, 0).A -ne 0 -or $bitmap.GetPixel($size-1, $size-1).A -ne 0) {
                throw 'Opaque icon background'
            }
            $left=$size; $top=$size; $right=-1; $bottom=-1
            for ($y=0; $y -lt $size; $y++) { for ($x=0; $x -lt $size; $x++) {
                if ($bitmap.GetPixel($x, $y).A -ge 128) {
                    $left=[Math]::Min($left,$x); $top=[Math]::Min($top,$y)
                    $right=[Math]::Max($right,$x); $bottom=[Math]::Max($bottom,$y)
                }
            }}
            if ($bottom-$top+1 -lt $size*0.8 -or
                [Math]::Abs(($left+$right+1)/2-$size/2) -gt 1.25 -or
                [Math]::Abs(($top+$bottom+1)/2-$size/2) -gt 1.25) {
                throw 'Undersized or misaligned icon artwork'
            }
        } finally { $bitmap.Dispose() }
    }
}
''', encoding='utf-8')
            msix.subprocess.run(['pwsh', '-NoProfile', '-NonInteractive', '-File', str(pixels),
                                 '-AssetsDirectory', str(folder / 'app-stage/Assets')],
                                check=True, capture_output=True)
            for missing in ('resources.pri', 'Assets/logo44.targetsize-256_altform-unplated.png'):
                with self.subTest(missing=missing):
                    stream = io.BytesIO()
                    with zipfile.ZipFile(stream, 'w') as changed:
                        for name, data in files.items():
                            if name != missing:
                                changed.writestr(name, data)
                    with zipfile.ZipFile(stream) as changed, self.assertRaises(ValueError):
                        icons.verify_package_icons(changed, root)
            stream = io.BytesIO()
            with zipfile.ZipFile(stream, 'w') as changed:
                for name, data in files.items():
                    changed.writestr(name, files['Assets/logo44.targetsize-256.png']
                                     if name == 'Assets/file-zip.png' else data)
            with zipfile.ZipFile(stream) as changed, self.assertRaisesRegex(ValueError, 'pixels differ'):
                icons.verify_package_icons(changed, root)


if __name__ == '__main__':
    unittest.main()
