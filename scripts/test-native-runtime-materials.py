# scripts/test-native-runtime-materials.py

"""Verify pinned notice sources and the platform-specific material set."""
import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from native_runtime_materials import ROOT, fetch, installer_notices, prepare


class NativeMaterialsTests(unittest.TestCase):
    def fixture(self, root):
        path = root/'owned-license.txt'
        path.write_bytes(b'Owned license fixture')
        return {'file': path.name, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                'url': 'https://invalid.test/not-used'}

    def test_offline_verified_cache_and_tampering(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); item = self.fixture(root)
            self.assertEqual(fetch(item, root), root/item['file'])
            (root/item['file']).write_bytes(b'corrupted')
            with self.assertRaisesRegex(ValueError, 'checksum'):
                fetch(item, root)

    def test_windows_no_longer_fetches_software_opengl_notices(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); output = root/'notices'
            with patch('native_runtime_materials.fetch') as download:
                report = prepare(output, 'windows', root/'cache')
            download.assert_not_called()
            self.assertEqual(report['files'], [])
            self.assertTrue((output/'native-runtime-materials.json').is_file())
            (output/'Mesa-11.2.2').mkdir()
            with self.assertRaisesRegex(ValueError, 'Obsolete software OpenGL notices'):
                prepare(output, 'windows', root/'cache')

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
