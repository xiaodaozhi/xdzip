#!/usr/bin/env python3
"""MSIX identity, channel and payload regression checks."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET

from deployment_audit import digest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('msix_builder', ROOT / 'scripts/build-windows-msix.py')
msix = importlib.util.module_from_spec(spec)
spec.loader.exec_module(msix)


class MsixTests(unittest.TestCase):
    def test_manifest_contains_associations_but_no_shell_menu(self):
        data = msix.manifest('xdzip.Development', 'CN=xdzip development', 27)
        package = ET.fromstring(data)
        identity = package.find(msix.tag(msix.NS, 'Identity'))
        self.assertEqual(identity.attrib['Version'], '1.0.27.0')
        self.assertEqual(identity.attrib['ProcessorArchitecture'], 'x64')
        extensions = {item.text for item in package.iter(msix.tag(msix.UAP, 'FileType'))}
        self.assertEqual(extensions, {'.' + ext for ext in msix.WINDOWS_ICONS})
        self.assertNotIn(b'fileExplorerContextMenus', data)
        self.assertNotIn(b'comServer', data)
        self.assertIn(b'runFullTrust', data)

    def test_invalid_identity_and_version_are_rejected(self):
        for name, publisher, build in [('a', 'CN=test', 1), ('xdzip.test', 'invalid', 1),
                                       ('xdzip.test', 'CN=test', 0), ('xdzip.test', 'CN=test', 65536)]:
            with self.subTest(name=name, publisher=publisher, build=build):
                with self.assertRaises(ValueError):
                    msix.manifest(name, publisher, build)

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
                               'CN=test', folder / 'report.json')
            app.write_bytes(b'tampered')
            with self.assertRaisesRegex(ValueError, 'payload changed'):
                msix.build(runtime, audit_path, folder / 'app.msix', 'xdzip.Test',
                           'CN=test', folder / 'report.json')

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
            identity = {'version': '0.2.0', 'build': 27, 'revision': 'a' * 12, 'dirty': False}
            audit = {'status': 'passed', 'platform': 'windows', 'mode': 'deployed',
                     'identity': identity, 'files': [{'path': 'xdzip.exe', 'sha256': digest(app)}]}
            audit_path = folder / 'audit.json'
            audit_path.write_text(json.dumps(audit), encoding='utf-8')
            real_run = msix.subprocess.run
            def run(command, **kwargs):
                if command[0] == str(app):
                    return unittest.mock.Mock(stdout=json.dumps({**identity, 'packaged_update_policy': 1}))
                return real_run(command, **kwargs)
            with patch.object(msix.subprocess, 'run', side_effect=run):
                msix.build(runtime, audit_path, folder / 'app.msix', 'xdzip.Test',
                           'CN=test', folder / 'report.json')
            report = json.loads((folder / 'report.json').read_text(encoding='utf-8'))
            self.assertFalse(report['signed'])
            self.assertEqual(report['sha256'], digest(folder / 'app.msix'))


if __name__ == '__main__':
    unittest.main()
