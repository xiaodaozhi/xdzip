#!/usr/bin/env python3
"""Test hosted MSIX artifact identity and corruption checks without installing."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
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
coexistence = module('prepare_msix_coexistence', 'prepare-msix-coexistence.py')


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
            archive.writestr('AppxManifest.xml', build.manifest('xdzip.MsixCI', 'CN=xdzip MSIX CI', 27, 'xdzip'))
            archive.writestr('xdzip.exe', b'owned fixture, not executable')
        sha = hashlib.sha256(self.package.read_bytes()).hexdigest()
        self.report = {'status': 'passed', 'name': 'xdzip.MsixCI', 'publisher': 'CN=xdzip MSIX CI',
                       'publisher_display_name': 'xdzip',
                       'languages': list(build.LANGUAGES),
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
        receipt_path = self.folder / 'package-msix.json'
        receipt = json.loads(receipt_path.read_text(encoding='utf-8'))
        receipt['source']['revision'] = 'b' * 40
        receipt_path.write_text(json.dumps(receipt), encoding='utf-8')
        with self.assertRaisesRegex(ValueError, 'receipt'):
            extract.main()
        receipt['source']['revision'] = 'a' * 40
        receipt_path.write_text(json.dumps(receipt), encoding='utf-8')
        with self.package.open('ab') as target:
            target.write(b'tampered')
        with self.assertRaisesRegex(ValueError, 'bytes'):
            extract.main()
        self.assertFalse((self.folder / 'windows').exists())

    def test_recheck_allows_only_explicit_test_changes(self):
        old, current = 'a' * 40, 'b' * 40
        def run_with_change(path):
            results = [subprocess.CompletedProcess([], 0),
                       subprocess.CompletedProcess([], 0, (path + '\0').encode())]
            with patch.object(extract.subprocess, 'run', side_effect=results):
                extract.validate_source_revision(old, current)
        run_with_change('scripts/check-windows-msix-install.ps1')
        run_with_change('scripts/msix-origin-probe.py')
        run_with_change('scripts/prepare-msix-upgrade.py')
        with self.assertRaisesRegex(ValueError, 'inputs changed'):
            run_with_change('scripts/build-windows-msix.py')
        with self.assertRaisesRegex(ValueError, 'Invalid source revision'):
            extract.validate_source_revision('not-a-revision', current)

    def test_local_execution_is_rejected(self):
        with patch.dict(os.environ, {'GITHUB_ACTIONS': 'false'}):
            with self.assertRaisesRegex(ValueError, 'hosted CI only'):
                extract.main()

    def prepare_installer_fixture(self):
        extract.main()
        executable = self.folder / 'windows/xdzip.exe'
        audit = {'status': 'passed', 'platform': 'windows', 'mode': 'deployed',
                 'identity': {'revision': 'a' * 12, 'dirty': False},
                 'files': [{'path': 'xdzip.exe', 'sha256': hashlib.sha256(executable.read_bytes()).hexdigest()}]}
        audit_path = Path('build/msix-ci-evidence/msix-ci-package/deployment-audit.json')
        audit_path.parent.mkdir(parents=True)
        audit_path.write_text(json.dumps(audit), encoding='utf-8')
        self.report['audit_sha256'] = hashlib.sha256(audit_path.read_bytes()).hexdigest()
        (self.folder / 'windows-msix.json').write_text(json.dumps(self.report), encoding='utf-8')
        return executable

    def test_installer_payload_reconstruction(self):
        self.prepare_installer_fixture()
        coexistence.main()
        self.assertEqual((self.folder / 'installer-runtime/xdzip.exe').read_bytes(), b'owned fixture, not executable')
        self.assertFalse((self.folder / 'installer-runtime/AppxManifest.xml').exists())

    def test_installer_payload_tampering_is_rejected(self):
        executable = self.prepare_installer_fixture()
        executable.write_bytes(b'tampered')
        with self.assertRaisesRegex(ValueError, 'payload changed'):
            coexistence.main()
        self.assertTrue((self.folder / 'installer-runtime').exists())


if __name__ == '__main__':
    unittest.main()
