# scripts/test-native-runtime-materials.py

"""Reject corrupted notice sources and unsafe members before material delivery."""
import hashlib
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

from native_runtime_materials import ROOT, archive_notices, fetch, installer_notices, prepare


class NativeMaterialsTests(unittest.TestCase):
    def fixture(self, root, member='owned-1/LICENSE'):
        path = root/'owned-1.tar.xz'
        with tarfile.open(path, 'w:xz') as archive:
            data = b'/* Copyright owned fixture. Permission granted. */\n'
            item = tarfile.TarInfo(member); item.size = len(data)
            archive.addfile(item, io.BytesIO(data))
        return {'file': path.name, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                'url': 'https://invalid.test/not-used', 'notices': ['LICENSE']}

    def test_offline_verified_cache_and_missing_notice(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); item = self.fixture(root)
            self.assertEqual(fetch(item, root), root/item['file'])
            result = archive_notices(root/item['file'], item)
            self.assertIn(b'Copyright owned fixture', result['LICENSE'])
            item['notices'] = ['MISSING']
            with self.assertRaisesRegex(ValueError, 'Missing upstream'):
                archive_notices(root/item['file'], item)
            (root/item['file']).write_bytes(b'corrupted')
            with self.assertRaisesRegex(ValueError, 'checksum'):
                fetch(item, root)

    def test_archive_escape_is_not_extracted_or_trusted(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for name in ['../escape', 'owned-1/../escape', '/absolute', 'other/LICENSE']:
                item = self.fixture(root, name)
                with self.assertRaisesRegex(ValueError, 'Unbounded'):
                    archive_notices(root/item['file'], item)
            self.assertFalse((root/'escape').exists())

    def test_icu_notice_copy_is_bound_to_hash_and_recorded(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); cache = root/'cache'; cache.mkdir(); output = root/'notices'
            data = b'Owned ICU license fixture'
            (cache/'ICU.txt').write_bytes(data)
            config = {'icu': {'file': 'ICU.txt', 'sha256': hashlib.sha256(data).hexdigest()}}
            with patch('native_runtime_materials.configuration', return_value=config):
                report = prepare(output, 'linux', cache)
            self.assertEqual(report['files'][0]['sha256'], hashlib.sha256(data).hexdigest())
            self.assertEqual((output/'ICU-73.2/ICU.txt').read_bytes(), data)

    def test_installer_acknowledgement_is_delivered_with_exact_original_licenses(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); tools = root/'tools'; tools.mkdir(); output = root/'notices'
            import json
            config = json.loads((ROOT/'scripts/installer-toolchain.json').read_text())['inno_setup']
            entries = [i for i in config['materials'] if i['file'] != 'ChineseSimplified.isl']
            for entry in entries:
                (tools/entry['file']).write_bytes(b'tampered')
            with self.assertRaisesRegex(ValueError, 'hash mismatch'):
                installer_notices(tools, output)
            self.assertFalse(output.exists())
            # Owned fixtures exercise successful delivery without network I/O.
            for entry in entries:
                entry['sha256'] = hashlib.sha256((tools/entry['file']).read_bytes()).hexdigest()
            with patch('native_runtime_materials.json.loads', return_value={'inno_setup': {'materials': entries}}):
                installer_notices(tools, output)
            text = (output/'Installer-CREDITS.txt').read_text(encoding='utf-8')
            self.assertIn('RemObjects Pascal Script (https://www.remobjects.com/ps)', text)
            self.assertTrue((output/'InnoSetup-PascalScript-license.md').is_file())


if __name__ == '__main__':
    unittest.main()
