#!/usr/bin/env python3
"""Test hosted MSIX artifact identity and corruption checks without installing."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / filename)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


extract = module('extract_msix_ci', 'extract-msix-ci.py')
build = module('build_msix_ci_fixture', 'build-windows-msix.py')


class ArtifactTests(unittest.TestCase):
    def setUp(self):
        (ROOT / 'build').mkdir(exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix='issue272-msix-input-', dir=ROOT / 'build')
        self.previous = Path.cwd()
        os.chdir(self.temp.name)
        self.addCleanup(self.temp.cleanup)
        self.addCleanup(os.chdir, self.previous)
        self.folder = Path('build/msix-ci-package')
        self.folder.mkdir(parents=True)
        self.package = self.folder / 'xdzip-windows-x64.msix'
        with zipfile.ZipFile(self.package, 'w') as archive:
            archive.writestr('AppxManifest.xml', build.manifest('xdzip.MsixCI', 'CN=xdzip MSIX CI', 27))
            archive.writestr('xdzip.exe', b'owned fixture, not executable')
        sha = hashlib.sha256(self.package.read_bytes()).hexdigest()
        self.report = {'status': 'passed', 'name': 'xdzip.MsixCI', 'publisher': 'CN=xdzip MSIX CI',
                       'version': '1.0.27.0', 'signed': False, 'modern_context_menu': False,
                       'sha256': sha}
        receipt = {'status': 'passed', 'source': {'revision': 'a' * 40, 'dirty': False},
                   'artifacts': [{'file': self.package.name, 'sha256': sha}]}
        (self.folder / 'windows-msix.json').write_text(json.dumps(self.report), encoding='utf-8')
        (self.folder / 'package-msix.json').write_text(json.dumps(receipt), encoding='utf-8')
        self.environment = patch.dict(os.environ, {'GITHUB_ACTIONS': 'true',
                                                   'RUNNER_ENVIRONMENT': 'github-hosted',
                                                   'GITHUB_SHA': 'a' * 40})
        self.environment.start()
        self.addCleanup(self.environment.stop)

    def test_same_run_fixture_extracts(self):
        extract.main()
        self.assertEqual((self.folder / 'windows/xdzip.exe').read_bytes(), b'owned fixture, not executable')

    def test_wrong_revision_and_modified_bytes_are_rejected(self):
        with patch.dict(os.environ, {'GITHUB_SHA': 'b' * 40}):
            with self.assertRaisesRegex(ValueError, 'receipt'):
                extract.main()
        with self.package.open('ab') as target:
            target.write(b'tampered')
        with self.assertRaisesRegex(ValueError, 'bytes'):
            extract.main()
        self.assertFalse((self.folder / 'windows').exists())

    def test_local_execution_is_rejected(self):
        with patch.dict(os.environ, {'GITHUB_ACTIONS': 'false'}):
            with self.assertRaisesRegex(ValueError, 'hosted CI only'):
                extract.main()


if __name__ == '__main__':
    unittest.main()
