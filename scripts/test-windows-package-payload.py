#!/usr/bin/env python3
"""Exercise the installer and portable payload comparison with owned fixtures."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

SCRIPT = Path(__file__).with_name('check-windows-package-payload.py')
FILES = {
    'xdzip.exe': b'owned executable',
    'Qt6Core.dll': b'owned qt library',
    'xdzip-update-helper.exe': b'owned updater',
    'plugins/platforms/qwindows.dll': b'owned platform plugin',
}


class PayloadComparisonTests(unittest.TestCase):
    def fixture(self, root, installed=None, archive=None, manifest=None):
        installed = dict(FILES) if installed is None else installed
        archive = dict(FILES) if archive is None else archive
        app = root/'app'
        app.mkdir()
        for name, value in installed.items():
            path = app/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(value)
        if manifest is None:
            manifest = 'xdzip-install-files-v1\n' + ''.join(
                hashlib.sha256(value).hexdigest() + '\t' + name.replace('/', '\\') + '\n'
                for name, value in archive.items())
        (app/'install-manifest.txt').write_text(manifest, encoding='utf-8')
        portable = root/'portable.zip'
        with zipfile.ZipFile(portable, 'w') as zipped:
            for name, value in archive.items():
                zipped.writestr(name, value)
        result = subprocess.run(
            [sys.executable, str(SCRIPT), '--installer-app', str(app), '--portable', str(portable),
             '--output', str(root/'result.json')], capture_output=True, text=True)
        return result

    def test_matching_payload_and_manifest(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            result = self.fixture(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads((root/'result.json').read_text())
            self.assertEqual(report['portable_files'], len(FILES))
            self.assertEqual(report['different'], [])
            self.assertEqual(report['extra'], ['install-manifest.txt'])

    def test_modified_or_missing_installed_file_is_rejected(self):
        for case in ('modified', 'missing'):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as temp:
                files = dict(FILES)
                if case == 'modified':
                    files['xdzip.exe'] = b'changed executable'
                else:
                    del files['xdzip.exe']
                self.assertNotEqual(self.fixture(Path(temp), installed=files).returncode, 0)

    def test_changed_manifest_is_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            self.assertNotEqual(self.fixture(Path(temp), manifest='xdzip-install-files-v1\n').returncode, 0)

    def test_unsafe_or_excluded_archive_member_is_rejected(self):
        for name in ('../escape', 'd3dcompiler_47.dll'):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp:
                files = dict(FILES)
                files[name] = b'owned invalid member'
                self.assertNotEqual(self.fixture(Path(temp), archive=files).returncode, 0)


if __name__ == '__main__':
    unittest.main()
