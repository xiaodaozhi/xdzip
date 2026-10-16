#!/usr/bin/env python3
"""Source cache integrity and portable bundle boundary tests, using owned fixtures."""
import importlib.util
import io
from pathlib import Path
import plistlib
import tarfile
import tempfile
import unittest
from unittest.mock import patch
from deployment_audit import digest

ROOT = Path(__file__).resolve().parents[1]


def module(name):
    spec = importlib.util.spec_from_file_location(name, ROOT/'scripts'/(name+'.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


source = module('prepare-source-delivery')
mac = module('build-macos-dmg')


class DeliveryTests(unittest.TestCase):
    def archive(self, root, member='owned-src-1/file.txt'):
        path = root/'owned-src-1.tar.xz'
        with tarfile.open(path, 'w:xz') as archive:
            info = tarfile.TarInfo(member)
            info.size = 4
            archive.addfile(info, io.BytesIO(b'own!'))
        return {'file': path.name, 'sha256': digest(path), 'url': 'https://invalid.test/no-network'}

    def test_verified_cache_is_offline_and_tampered_cache_fails(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            item = self.archive(root)
            self.assertEqual(source.archive_source(item, root).name, item['file'])
            (root/item['file']).write_bytes(b'corrupted')
            with self.assertRaisesRegex(ValueError, 'checksum'):
                source.archive_source(item, root)

    def test_source_paths_are_inspected_without_extracting(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for member in ['../escape', 'owned-src-1/../escape', '/absolute', 'wrong-src/file']:
                item = self.archive(root, member)
                with self.assertRaisesRegex(ValueError, 'layout'):
                    source.archive_source(item, root)
            self.assertFalse((root/'escape').exists())

    def test_bundle_links_cannot_escape_the_application(self):
        with tempfile.TemporaryDirectory() as temp:
            app = Path(temp)/'xdzip.app'
            (app/'Contents/MacOS').mkdir(parents=True)
            (app/'Contents/MacOS/xdzip').write_bytes(b'owned')
            (app/'Contents/Info.plist').write_bytes(plistlib.dumps(
                {'CFBundleIdentifier': 'org.xdzip.desktop', 'CFBundleExecutable': 'xdzip'}))
            (app/'Contents/escape').symlink_to(Path(temp))
            with patch.object(mac, 'checked', return_value='arm64'):
                with self.assertRaisesRegex(ValueError, 'escapes'):
                    mac.validate_bundle(app)


if __name__ == '__main__':
    unittest.main()
