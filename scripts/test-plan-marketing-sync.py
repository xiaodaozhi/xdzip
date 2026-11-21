#!/usr/bin/env python3
"""Ensure latest-only plans preserve unrelated Drive files and reject ambiguity."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('marketing_sync', ROOT / 'scripts/plan-marketing-sync.py')
sync = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sync)


class SyncTests(unittest.TestCase):
    def setUp(self):
        (ROOT / 'build').mkdir(exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix='test-marketing-sync-', dir=ROOT / 'build')
        self.addCleanup(self.temp.cleanup)
        self.source = Path(self.temp.name) / 'source'
        (self.source / 'ja-JP/backgrounds').mkdir(parents=True)
        image = b'\x89PNG\r\n\x1a\n\0\0\0\rIHDR' + b'\0' * 16
        self.path = 'ja-JP/backgrounds/01.png'
        (self.source / self.path).write_bytes(image)
        self.manifest = {'schema_version': 1, 'platform': 'macos', 'status': 'backgrounds-only',
                         'captured_languages': [], 'assets': [{'file': self.path, 'bytes': len(image),
                                                              'sha256': sync.package.digest(image)}]}
        manifest_data = (json.dumps(self.manifest) + '\n').encode()
        (self.source / 'manifest.json').write_bytes(manifest_data)
        (self.source / 'SHA256SUMS.txt').write_text(
            f'{sync.package.digest(image)}  {self.path}\n{sync.package.digest(manifest_data)}  manifest.json\n')

    def test_updates_and_owned_removal_preserve_unrelated(self):
        previous = dict(self.manifest)
        previous['assets'] = self.manifest['assets'] + [{'file': 'ja-JP/old.png', 'sha256': 'a' * 64}]
        inventory = {'files': [{'path': self.path, 'id': 'existing'}, {'path': 'ja-JP/old.png', 'id': 'old'},
                               {'path': 'manifest.json', 'id': 'manifest'}, {'path': 'SHA256SUMS.txt', 'id': 'sum'},
                               {'path': 'personal.png', 'id': 'personal'}]}
        plan = sync.build_plan('macos', self.source, inventory, previous)
        self.assertEqual(plan['operations'][0]['action'], 'update')
        self.assertEqual(plan['operations'][0]['id'], 'existing')
        self.assertEqual([x['id'] for x in plan['operations'] if x['action'] == 'trash'], ['old'])
        self.assertNotIn('personal', [x.get('id') for x in plan['operations']])
        self.assertEqual([x['path'] for x in plan['operations'][-2:]], ['manifest.json', 'SHA256SUMS.txt'])

    def test_collision_duplicate_and_wrong_manifest_rejected(self):
        inventory = {'files': [{'path': self.path, 'id': 'a'}]}
        with self.assertRaisesRegex(ValueError, 'not recorded'):
            sync.build_plan('macos', self.source, inventory)
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            sync.build_plan('macos', self.source, {'files': inventory['files'] * 2}, self.manifest)
        with self.assertRaisesRegex(ValueError, 'platform'):
            sync.build_plan('macos', self.source, inventory, {**self.manifest, 'platform': 'linux'})
        with self.assertRaisesRegex(ValueError, 'invalid or duplicate'):
            sync.build_plan('macos', self.source, {'files': [{'path': '../bad', 'id': 'a'}]})


if __name__ == '__main__':
    unittest.main()
