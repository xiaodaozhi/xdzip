# scripts/test-release-protocol.py

import copy
from pathlib import Path
import tempfile
import unittest
from release_protocol import TARGETS, create, select, validate


class ReleaseProtocolTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        for suffix, _ in TARGETS.values():
            (self.directory/('xdzip-0.1.0-alpha.1-'+suffix)).write_bytes(b'owned package fixture')
        self.manifest = create(self.directory, dict(display_version='0.1.0-alpha.1', channel='alpha', build=14), 'owner/repo', 'a'*40)

    def test_channel_and_semantic_version_selection(self):
        args = ('windows', 'x86_64', 'zip')
        self.assertIsNotNone(select(self.manifest, '0.0.9', ['alpha'], *args))
        self.assertIsNone(select(self.manifest, '0.1.0-alpha.1', ['alpha'], *args))
        self.assertIsNone(select(self.manifest, '0.1.0', ['alpha'], *args))
        self.assertIsNone(select(self.manifest, '0.0.9', ['stable'], *args))
        self.assertIsNone(select(self.manifest, '0.0.9', ['alpha'], 'windows', 'arm64', 'zip'))

    def test_missing_package_and_mutated_hash_fail_closed(self):
        invalid = copy.deepcopy(self.manifest)
        invalid['assets'].pop()
        with self.assertRaises(ValueError): validate(invalid)
        invalid = copy.deepcopy(self.manifest)
        invalid['assets'][0]['url'] = invalid['assets'][0]['url'].replace('/v0.1.0-alpha.1/', '/latest/')
        with self.assertRaises(ValueError): validate(invalid)
        asset = self.directory/self.manifest['assets'][0]['filename']
        asset.write_bytes(b'tampered')
        with self.assertRaises(ValueError): validate(self.manifest, self.directory)


if __name__ == '__main__': unittest.main()
