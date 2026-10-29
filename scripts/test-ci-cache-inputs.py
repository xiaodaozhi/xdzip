#!/usr/bin/env python3
"""Cache provenance, byte integrity and bounded-input regressions, without network."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('ci_cache', ROOT / 'scripts/ci-cache-inputs.py')
cache = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cache)


class CacheTests(unittest.TestCase):
    def setUp(self):
        parent = ROOT / 'build/test-ci-cache-inputs'
        parent.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=parent)
        self.assertTrue(Path(self.temp.name).resolve().is_relative_to(parent.resolve()))
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.paths = {'pip': self.root / 'pip'}
        self.paths['pip'].mkdir()
        (self.paths['pip'] / 'owned-wheel').write_bytes(b'owned dependency input')
        self.env = {'CI': 'true', 'GITHUB_EVENT_NAME': 'push', 'GITHUB_REF': 'refs/heads/main',
                    'GITHUB_REPOSITORY': 'owned/repo', 'GITHUB_RUN_ID': '123'}
        self.plan = {'key': 'owned-key', 'scope': 'html', 'host': {'os': 'fixture'}, 'root_labels': ['pip']}
        self.record = cache.seal(self.plan, self.paths, self.env)

    def test_exact_restore_accepts_only_frozen_bytes(self):
        self.assertEqual(cache.verify(self.plan, self.paths, self.record, self.env), 1)
        (self.paths['pip'] / 'owned-wheel').write_bytes(b'corrupted')
        with self.assertRaisesRegex(ValueError, 'bytes'):
            cache.verify(self.plan, self.paths, self.record, self.env)

    def test_added_and_removed_files_are_rejected(self):
        extra = self.paths['pip'] / 'extra'
        extra.write_bytes(b'unrecorded')
        with self.assertRaises(ValueError):cache.verify(self.plan, self.paths, self.record, self.env)
        extra.unlink()
        (self.paths['pip'] / 'owned-wheel').unlink()
        with self.assertRaises(ValueError):cache.verify(self.plan, self.paths, self.record, self.env)

    def test_identity_scope_host_and_repository_cannot_drift(self):
        for field, value in [('key', 'old-key'), ('scope', 'qt'), ('host', {'os': 'other'}),
                             ('repository', 'foreign/repo'), ('producer_ref', 'refs/pull/7/merge'),
                             ('producer_event', 'pull_request')]:
            with self.subTest(field=field):
                record = copy.deepcopy(self.record);record[field] = value
                with self.assertRaises(ValueError):cache.verify(self.plan, self.paths, record, self.env)

    def test_untrusted_or_non_ci_runs_cannot_seal(self):
        for edits in [{'CI': ''}, {'GITHUB_EVENT_NAME': 'pull_request'},
                      {'GITHUB_EVENT_NAME': 'workflow_dispatch'}, {'GITHUB_REF': 'refs/heads/coding/owned'}]:
            with self.subTest(edits=edits), self.assertRaises(ValueError):
                cache.seal(self.plan, self.paths, dict(self.env, **edits))

    def test_escape_duplicate_and_credential_roots_are_rejected(self):
        for edits in [{'file': '../credentials'}, {'file': '/outside'}, {'file': 'C:/private'},
                      {'file': 'dir\\escape'}, {'root': 'credentials'}, {'bytes': -1}, {'sha256': 'invalid'}]:
            with self.subTest(edits=edits):
                record = copy.deepcopy(self.record);record['files'][0].update(edits)
                with self.assertRaises(ValueError):cache.verify(self.plan, self.paths, record, self.env)
        record = copy.deepcopy(self.record);record['files'] *= 2
        with self.assertRaises(ValueError):cache.verify(self.plan, self.paths, record, self.env)

    def test_inventory_budget_is_enforced(self):
        with patch.object(cache, 'MAX_BYTES', 1), self.assertRaisesRegex(ValueError, 'budget'):
            cache.inventory(['pip'], self.paths)

    def test_link_escape_is_rejected_when_host_supports_links(self):
        outside = self.root / 'outside';outside.write_bytes(b'owned outside')
        link = self.paths['pip'] / 'link'
        try:link.symlink_to(outside)
        except OSError:self.skipTest('Host cannot create symlinks; other byte/path guards still execute')
        with self.assertRaisesRegex(ValueError, 'link'):
            cache.inventory(['pip'], self.paths)

    def test_dependency_or_host_change_invalidates_key(self):
        env = {'RUNNER_OS': 'owned', 'RUNNER_ARCH': 'x64', 'ImageOS': 'image', 'ImageVersion': '1'}
        first = cache.identity('html', env=env)
        changed = cache.identity('html', env=dict(env, RUNNER_ARCH='arm64'))
        self.assertNotEqual(first['key'], changed['key'])
        changed = cache.identity('html', env=dict(env, ImageVersion='2'))
        self.assertNotEqual(first['key'], changed['key'])
        policy = json.loads((ROOT / 'scripts/ci-cache-policy.json').read_text(encoding='utf-8'))
        for name in policy['common_inputs'] + policy['scopes']['html']['inputs']:
            target = self.root / name;target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((ROOT / name).read_bytes())
        before = cache.identity('html', root=self.root, env=env)
        (self.root / 'scripts/html-design-requirements.txt').write_bytes(b'owned changed dependency\n')
        after = cache.identity('html', root=self.root, env=env)
        self.assertNotEqual(before['key'], after['key'])
        for label in policy['scopes']['qt']['roots']:
            self.assertNotIn(label, ['target', 'product', 'credentials', 'sdk', 'verdict'])
        self.assertNotIn('cargo-src', policy['scopes']['foundation']['roots'])


if __name__ == '__main__':unittest.main()
