#!/usr/bin/env python3
"""Test Snap input integrity and publication without writing to the Store."""

import hashlib
import importlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import snap_store


class SnapInputTests(unittest.TestCase):
    def test_mirror_uses_the_original_package_hash(self):
        pack = importlib.import_module('build-linux-snap')
        url = 'https://archive.ubuntu.com/ubuntu/pool/main/package.deb'
        mirror = 'https://mirror.example/ubuntu/'
        data = b'locked Ubuntu package'
        pin = {'package': 'package', 'url': url, 'sha256': hashlib.sha256(data).hexdigest()}
        with tempfile.TemporaryDirectory() as temporary:
            cache = Path(temporary)
            with patch.object(pack.urllib.request, 'urlopen', side_effect=[OSError('HTTP 403'), io.BytesIO(b'bad')]):
                with self.assertRaisesRegex(ValueError, 'differs from its lock'):
                    pack.fetch_deb(cache, pin, [mirror])
            self.assertEqual(list(cache.iterdir()), [])
            with patch.object(pack.urllib.request, 'urlopen', side_effect=[OSError('HTTP 403'), io.BytesIO(data)]) as fetch:
                self.assertEqual(pack.fetch_deb(cache, pin, [mirror]).read_bytes(), data)
                fetch.assert_called_with(mirror + 'pool/main/package.deb', timeout=20)


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
        self.summary = self.directory / 'summary'
        environment = patch.dict(snap_store.os.environ, {'GITHUB_STEP_SUMMARY': str(self.summary)})
        environment.start()
        self.addCleanup(environment.stop)

    def test_rejects_wrong_commit_and_tampered_candidate(self):
        with self.assertRaisesRegex(ValueError, 'selected source'):
            snap_store.read_candidate(self.directory, 'b' * 40)
        self.package.write_bytes(b'changed snap')
        with self.assertRaisesRegex(ValueError, 'bytes'):
            snap_store.read_candidate(self.directory, self.source)

    def test_direct_stable_upload_records_only_hash_matched_store_revision(self):
        previous = {**self.entry, 'revision': 8, 'download': {'sha3-384': 'b' * 96}}
        with (patch.object(snap_store, 'stable_entry', side_effect=[previous, previous, self.entry]),
              patch.object(snap_store.time, 'sleep'), patch.object(snap_store.subprocess, 'run') as run):
            snap_store.publish_stable(self.directory, self.source, 'v0.3.1')
        run.assert_called_once_with(['snapcraft', 'upload', str(self.package), '--release=latest/stable'], check=True)
        self.assertIn('Store revision: **9**', self.summary.read_text())

    def test_stable_retry_reuses_published_bytes(self):
        with (patch.object(snap_store, 'stable_entry', return_value=self.entry),
              patch.object(snap_store.subprocess, 'run') as run):
            snap_store.publish_stable(self.directory, self.source, 'v0.3.1')
        run.assert_not_called()
        self.assertIn('Store revision: **9**', self.summary.read_text())

    def test_wrong_and_prerelease_tags_reject_upload(self):
        with patch.object(snap_store, 'stable_entry') as store:
            for tag in ('v0.3.2', 'v0.3.1-beta.1'):
                with self.subTest(tag=tag), self.assertRaisesRegex(ValueError, 'stable release tag'):
                    snap_store.publish_stable(self.directory, self.source, tag)
        store.assert_not_called()


if __name__ == '__main__':
    unittest.main()
