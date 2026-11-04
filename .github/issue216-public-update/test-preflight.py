#!/usr/bin/env python3
"""Exercise preflight admission and refusal against owned ZIP fixtures."""
import hashlib
import io
import os
from pathlib import Path
import re
import tempfile
import unittest
from unittest.mock import patch
import zipfile

SOURCE = Path(__file__).with_name('preflight.py').read_text(encoding='utf-8')
LICENSE = b'owned license fixture'
APP = b'owned app fixture'
OLD_LICENSE = '8099dc3cf9502c335da829e5c755948a12e3e6de490eb492a99deb673d883d8b'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def bundle(extra=(), license_text=LICENSE):
    out = io.BytesIO()
    with zipfile.ZipFile(out, 'w') as archive:
        archive.writestr('xdzip.exe', APP)
        archive.writestr('share/notices/Microsoft-VC-runtime/license.rtf', license_text)
        for name, data in extra:
            archive.writestr(name, data)
    return out.getvalue()


class PreflightTests(unittest.TestCase):
    def run_case(self, data, expected_license=LICENSE, expected_hash=None, source=SOURCE):
        source, archive_count = re.subn(
            r"(archive_hash != ')[^']+(')", rf"\g<1>{expected_hash or sha(data)}\2", source)
        source, app_count = re.subn(
            r"(app_hash != ')[^']+(')", rf"\g<1>{sha(APP)}\2", source)
        if archive_count != 1 or app_count != 1 or source.count(OLD_LICENSE) != 1:
            raise AssertionError('Unexpected preflight identity checks')
        source = source.replace(OLD_LICENSE, sha(expected_license))
        environment = {'GITHUB_ACTIONS': 'true', 'RUNNER_OS': 'Windows',
                       'RUNNER_ENVIRONMENT': 'github-hosted'}
        with tempfile.TemporaryDirectory(dir=Path('build').resolve()) as directory:
            previous = Path.cwd()
            try:
                os.chdir(directory)
                with patch.dict(os.environ, environment), patch('urllib.request.urlopen', return_value=io.BytesIO(data)):
                    exec(compile(source, 'preflight.py', 'exec'), {})
            finally:
                os.chdir(previous)

    def test_owned_package(self):
        self.run_case(bundle())

    def test_owned_package_after_public_hashes_are_bound(self):
        source = (SOURCE.replace('__PUBLIC_PORTABLE_SHA256__', 'a' * 64)
                  .replace('__PUBLIC_APP_SHA256__', 'b' * 64))
        self.run_case(bundle(), source=source)

    def test_rejects_sdk_file_at_any_depth_and_case(self):
        with self.assertRaisesRegex(RuntimeError, 'SDK graphics'):
            self.run_case(bundle([('nested/DxIl.DLL', b'fixture')]))

    def test_rejects_changed_license(self):
        with self.assertRaisesRegex(RuntimeError, 'license differs'):
            self.run_case(bundle(license_text=b'changed'))

    def test_rejects_archive_hash_mismatch(self):
        with self.assertRaisesRegex(RuntimeError, 'identity differs'):
            self.run_case(bundle(), expected_hash='0' * 64)

    def test_rejects_case_insensitive_duplicate(self):
        with self.assertRaisesRegex(RuntimeError, 'Duplicate'):
            self.run_case(bundle([('XDZIP.EXE', b'other')]))


if __name__ == '__main__':
    unittest.main()
