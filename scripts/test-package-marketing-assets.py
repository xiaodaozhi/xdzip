#!/usr/bin/env python3
"""Verify platform identity, tamper protection and recoverable ZIP content."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('marketing_package', ROOT / 'scripts/package-marketing-assets.py')
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)


class PackageTests(unittest.TestCase):
    def setUp(self):
        (ROOT / 'build').mkdir(exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix='test-marketing-package-', dir=ROOT / 'build')
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.source = self.base / 'export'
        self.source.mkdir()
        self.files = {'en-US/backgrounds/01.png': b'\x89PNG\r\n\x1a\n\0\0\0\rIHDR' + b'\0' * 16,
                      'en-US/text/description.txt': b'fixture'}
        self.write_source()

    def write_source(self, platform='linux', windows_legacy=False):
        items = []
        for name, data in self.files.items():
            target = self.source / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            items.append({'file': name, 'bytes': len(data), 'sha256': package.digest(data)})
        manifest = {'schema_version': 1, 'captured_languages': []}
        if windows_legacy:
            manifest.update(msix_version='1.0.1.0', baseline={}, files=items)
        else:
            manifest.update(platform=platform, status='backgrounds-only', assets=items)
        data = (json.dumps(manifest) + '\n').encode()
        (self.source / 'manifest.json').write_bytes(data)
        all_items = {**self.files, 'manifest.json': data}
        (self.source / 'SHA256SUMS.txt').write_text(''.join(
            f'{package.digest(data)}  {name}\n' for name, data in sorted(all_items.items())))

    def test_valid_archives_and_windows_legacy(self):
        for platform, legacy in [('linux', False), ('windows', True)]:
            self.write_source(platform, legacy)
            result = package.package(self.source, platform, self.base / 'output')
            self.assertRegex(result.name, rf'^xdzip-{platform}-marketing-backgrounds-[a-f0-9]{{16}}\.zip$')
            with zipfile.ZipFile(result) as archive:
                self.assertEqual(set(archive.namelist()), set(self.files) | {'manifest.json', 'SHA256SUMS.txt'})
                self.assertEqual(archive.read('en-US/text/description.txt'), b'fixture')
                self.assertIsNone(archive.testzip())
            with self.assertRaises(FileExistsError):
                package.package(self.source, platform, self.base / 'output')

    def test_wrong_platform_missing_extra_and_tampered_rejected(self):
        with self.assertRaisesRegex(ValueError, 'platform'):
            package.package(self.source, 'macos', self.base / 'wrong')
        target = self.source / 'en-US/text/description.txt'
        target.write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'digest'):
            package.package(self.source, 'linux', self.base / 'changed')
        target.write_bytes(b'fixture')
        target.unlink()
        with self.assertRaisesRegex(ValueError, 'do not match'):
            package.package(self.source, 'linux', self.base / 'missing')
        target.write_bytes(b'fixture')
        (self.source / 'extra').write_bytes(b'extra')
        with self.assertRaisesRegex(ValueError, 'do not match'):
            package.package(self.source, 'linux', self.base / 'extra')

    def test_symlinks_and_path_traversal_rejected(self):
        (self.source / 'link').symlink_to(self.source / 'manifest.json')
        with self.assertRaisesRegex(ValueError, 'symlinks'):
            package.export_files(self.source, 'linux')
        (self.source / 'link').unlink()
        with (self.source / 'SHA256SUMS.txt').open('a') as sums:
            sums.write('0' * 64 + '  ../outside\n')
        with self.assertRaisesRegex(ValueError, 'path'):
            package.export_files(self.source, 'linux')


if __name__ == '__main__':
    unittest.main()
