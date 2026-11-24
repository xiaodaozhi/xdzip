#!/usr/bin/env python3
"""Test the publisher against local receipts and mocked Store channels."""

import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import snap_store


class SnapStoreTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.directory = Path(directory.name)
        self.package = self.directory / 'xdzip_0.3.1_amd64.snap'
        self.package.write_bytes(b'snap fixture')
        self.sha256 = hashlib.sha256(self.package.read_bytes()).hexdigest()
        self.sha3 = hashlib.sha3_384(self.package.read_bytes()).hexdigest()
        self.source = 'a' * 40
        self.receipt = self.package.with_suffix('.json')
        self.receipt.write_text(json.dumps({
            'filename': self.package.name, 'version_json': {'version': '0.3.1', 'dirty': False},
            'source_revision': self.source, 'base': 'core24', 'confinement': 'strict', 'grade': 'stable',
            'sha256': self.sha256, 'bytes': self.package.stat().st_size,
        }), encoding='utf-8')
        self.entry = {'revision': 9, 'version': '0.3.1', 'download': {'sha3-384': self.sha3}}
        self.output = self.directory / 'output'
        self.summary = self.directory / 'summary'
        environment = patch.dict(snap_store.os.environ, {
            'GITHUB_OUTPUT': str(self.output), 'GITHUB_STEP_SUMMARY': str(self.summary),
        })
        environment.start()
        self.addCleanup(environment.stop)

    def test_rejects_wrong_commit_and_tampered_candidate(self):
        with self.assertRaisesRegex(ValueError, 'selected source'):
            snap_store.read_candidate(self.directory, 'b' * 40)
        self.package.write_bytes(b'changed snap')
        with self.assertRaisesRegex(ValueError, 'bytes'):
            snap_store.read_candidate(self.directory, self.source)

    def test_beta_upload_records_only_hash_matched_store_revision(self):
        previous = {**self.entry, 'revision': 8, 'download': {'sha3-384': 'b' * 96}}
        with (patch.object(snap_store, 'channel_entry', side_effect=[previous, previous, self.entry]),
              patch.object(snap_store.time, 'sleep'), patch.object(snap_store.subprocess, 'run') as run):
            snap_store.publish_beta(self.directory, self.source)
        run.assert_called_once_with(['snapcraft', 'upload', str(self.package), '--release=latest/beta'], check=True)
        self.assertEqual(self.output.read_text(), f'revision=9\nsha3={self.sha3}\n')

    def test_beta_retry_reuses_published_bytes(self):
        with (patch.object(snap_store, 'channel_entry', return_value=self.entry),
              patch.object(snap_store.subprocess, 'run') as run):
            snap_store.publish_beta(self.directory, self.source)
        run.assert_not_called()
        self.assertIn('revision=9', self.output.read_text())

    def test_stable_promotes_the_same_revision_and_verifies_it(self):
        with (patch.object(snap_store, 'channel_entry', side_effect=[self.entry, None, self.entry]),
              patch.object(snap_store.subprocess, 'run') as run):
            snap_store.publish_stable(9, self.sha3)
        run.assert_called_once_with(['snapcraft', 'release', 'xdzip', '9', 'latest/stable'], check=True)
        self.assertIn('revision 9 verified', self.summary.read_text())

    def test_beta_moved_during_approval_blocks_stable_promotion(self):
        with (patch.object(snap_store, 'channel_entry', return_value={**self.entry, 'revision': 10}),
              patch.object(snap_store.subprocess, 'run') as run):
            with self.assertRaisesRegex(ValueError, 'beta channel changed'):
                snap_store.publish_stable(9, self.sha3)
        run.assert_not_called()


if __name__ == '__main__':
    unittest.main()
