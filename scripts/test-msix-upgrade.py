#!/usr/bin/env python3
"""Check version-only MSIX upgrade fixtures without installing any package."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch
import xml.etree.ElementTree as ET
import zipfile

from deployment_audit import digest

ROOT = Path(__file__).resolve().parents[1]


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / filename)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


upgrade = module('msix_upgrade', 'prepare-msix-upgrade.py')
builder = module('msix_upgrade_source', 'build-windows-msix.py')


class UpgradeTests(unittest.TestCase):
    def test_only_manifest_version_changes(self):
        original = builder.manifest('xdzip.MsixCI', 'CN=xdzip MSIX CI', 27, 'xdzip')
        content, old, new = upgrade.advance_manifest(original)
        self.assertEqual((old, new), ('1.0.27.0', '1.0.28.0'))
        before, after = ET.fromstring(original), ET.fromstring(content)
        after.find(builder.tag(builder.NS, 'Identity')).set('Version', old)
        self.assertEqual(ET.tostring(before), ET.tostring(after))

    def test_non_fixture_identity_and_version_limit_are_rejected(self):
        for name, publisher, build in (
                ('xdzip.Real', 'CN=real', 27),
                ('xdzip.MsixCI', 'CN=other', 27),
                ('xdzip.MsixCI', 'CN=xdzip MSIX CI', 65535)):
            with self.subTest(name=name, build=build), self.assertRaises(ValueError):
                upgrade.advance_manifest(builder.manifest(name, publisher, build, 'xdzip'))

    def test_unsafe_archive_fails_before_staging(self):
        (ROOT / 'build').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='msix-upgrade-test-', dir=ROOT / 'build') as temporary:
            folder = Path(temporary)
            package = folder / 'input.msix'
            with zipfile.ZipFile(package, 'w') as archive:
                archive.writestr('AppxManifest.xml',
                    builder.manifest('xdzip.MsixCI', 'CN=xdzip MSIX CI', 27, 'xdzip'))
                archive.writestr('../escape', b'owned malicious-path fixture')
            receipt = {'name': 'xdzip.MsixCI', 'publisher': 'CN=xdzip MSIX CI',
                       'publisher_display_name': 'xdzip', 'version': '1.0.27.0',
                       'signed': False, 'modern_context_menu': False, 'sha256': digest(package)}
            report = folder / 'input.json'
            report.write_text(json.dumps(receipt), encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'Unsafe'):
                upgrade.prepare(package, report, folder / 'upgrade.msix', Path('unused'))
            self.assertFalse((folder / 'upgrade-stage').exists())

    @unittest.skipUnless(os.name == 'nt', 'requires native Windows SDK')
    def test_owned_makeappx_roundtrip_preserves_product_bytes(self):
        (ROOT / 'build').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='msix-upgrade-test-', dir=ROOT / 'build') as temporary:
            folder = Path(temporary)
            runtime = folder / 'runtime'
            runtime.mkdir()
            app = runtime / 'xdzip.exe'
            app.write_bytes(b'owned non-executable fixture')
            identity = {'version': '0.2.0', 'build': 27, 'revision': 'a' * 12, 'dirty': False}
            audit = {'status': 'passed', 'platform': 'windows', 'mode': 'deployed',
                     'identity': identity, 'files': [{'path': 'xdzip.exe', 'sha256': digest(app)}]}
            audit_path = folder / 'audit.json'
            audit_path.write_text(json.dumps(audit), encoding='utf-8')
            real_run = builder.subprocess.run

            def run(command, **kwargs):
                if command[0] == str(app):
                    return Mock(stdout=json.dumps({**identity, 'packaged_update_policy': 1}))
                return real_run(command, **kwargs)

            with patch.object(builder.subprocess, 'run', side_effect=run):
                builder.build(runtime, audit_path, folder / 'input.msix', 'xdzip.MsixCI',
                              'CN=xdzip MSIX CI', folder / 'input.json', 'xdzip')
            output = folder / 'upgrade.msix'
            result = upgrade.prepare(folder / 'input.msix', folder / 'input.json', output, builder.sdk_tool())
            self.assertEqual(result['new_version'], '1.0.28.0')
            self.assertEqual(result['role'], 'same-payload-version-fixture')
            with zipfile.ZipFile(folder / 'input.msix') as old, zipfile.ZipFile(output) as new:
                for name in old.namelist():
                    if name not in ('AppxManifest.xml', 'AppxBlockMap.xml', '[Content_Types].xml'):
                        self.assertEqual(old.read(name), new.read(name))


if __name__ == '__main__':
    unittest.main()
