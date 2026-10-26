"""Ensure production acceptance collection cannot export signed update payloads."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('production_evidence', Path(__file__).with_name('collect-production-update-evidence.py'))
collector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(collector)


class CollectionTests(unittest.TestCase):
    def test_signed_candidates_and_apps_are_excluded(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / 'input'
            source.mkdir()
            for name in ('release.json', 'release.json.sig', 'session-release.json', 'session-release.json.sig',
                         'session-releases.json', 'appcast.xml', 'macos-update.json', 'setup.exe'):
                (source / name).write_bytes(b'candidate must stay on runner')
            (source / 'owned.app').mkdir()
            (source / 'owned.app' / 'private.log').write_bytes(b'app content')
            (source / 'result.json').write_text('{"status":"passed"}')
            (source / 'session-201-started.json').write_text('{"build":201}')
            (source / 'product.log').write_bytes(b'public diagnostic')
            result = collector.collect(source, root / 'output')
            self.assertEqual({r['source'] for r in result['records']},
                             {'result.json', 'session-201-started.json', 'product.log'})
            self.assertFalse(result['acceptance_passed'])
            self.assertFalse(result['signed_candidates_included'])

    def test_signed_data_in_result_slot_is_rejected(self):
        for value in ({'assets': [], 'tag': 'v1', 'repository': 'owner/repo'},
                      {'algorithm': 'ed25519', 'signature': 'public signature'},
                      {'ed_signature': 'public signature', 'filename': 'archive.app.zip'}):
            with self.subTest(value=value), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                source = root / 'input'
                source.mkdir()
                (source / 'result.json').write_text(json.dumps(value))
                with self.assertRaisesRegex(ValueError, 'Signed candidate'):
                    collector.collect(source, root / 'output')

    def test_output_and_size_guards(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / 'input'
            source.mkdir()
            with self.assertRaisesRegex(ValueError, 'separate'):
                collector.collect(source, source / 'output')
            (source / 'product.log').write_bytes(b'bounded')
            original = collector.PER_FILE_LIMIT
            try:
                collector.PER_FILE_LIMIT = 2
                with self.assertRaisesRegex(ValueError, 'bounded'):
                    collector.collect(source, Path(temporary) / 'output')
            finally:
                collector.PER_FILE_LIMIT = original


if __name__ == '__main__':
    unittest.main()
