#!/usr/bin/env python3
"""Exercise complete Git diffs, PR merge identity and conservative fallbacks."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('ci_plan', ROOT / 'scripts/ci-plan.py')
ci = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ci)


class PlanTests(unittest.TestCase):
    def setUp(self):
        parent = ROOT / 'build/test-ci-plan';parent.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=parent);self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.assertTrue(self.root.resolve().is_relative_to(parent.resolve()))
        self.git('init', '-q', '-b', 'main')
        self.paths = ['docs/README.md', 'README.md', 'docs/engineering/ci-policy.md', 'docs/qa/release-review.md',
                      'docs/visual/tokens.json', 'docs/visual/layout-scale.json', 'docs/ux/main-window-components.md',
                      'apps/desktop/qml/Main.qml', 'design/assets/fonts/owned.ttf', 'resources/i18n/zh-CN.json',
                      'Cargo.lock', 'CMakeLists.txt', 'crates/owned/src/lib.rs', '.github/workflows/owned.yml']
        for name in self.paths:
            path = self.root / name;path.parent.mkdir(parents=True, exist_ok=True);path.write_text('owned input\n', encoding='utf-8')
        self.config = ci.policy()
        self.write_policy()
        self.base = self.commit()

    def git(self, *args):
        return subprocess.check_output(['git', '-c', 'user.name=CI fixture', '-c', 'user.email=ci@example.invalid',
                                        *args], cwd=self.root, stderr=subprocess.PIPE).decode('utf-8').strip()

    def write_policy(self):
        path = self.root / 'scripts/ci-plan-policy.json';path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(self.config), encoding='utf-8')

    def commit(self):
        self.git('add', '-A');self.git('commit', '-q', '-m', 'owned fixture')
        return self.git('rev-parse', 'HEAD')

    def edit(self, name):
        path = self.root / name;path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('owned changed input\n', encoding='utf-8')
        return self.commit()

    def plan(self, head, event='push', payload=None, **edits):
        env = {'GITHUB_SHA': head, 'GITHUB_REF': 'refs/heads/main', 'GITHUB_EVENT_NAME': event,
               'GITHUB_REPOSITORY': 'owned/repo', 'GITHUB_RUN_ID': '123', 'GITHUB_RUN_ATTEMPT': '1'}
        env.update(edits)
        payload = payload if payload is not None else {'before': self.base, 'after': head}
        return ci.plan(self.root, env, payload)

    def test_navigation_document_is_shadowed_then_explicitly_enabled(self):
        head = self.edit('docs/README.md')
        result = self.plan(head)
        self.assertEqual((result['classification'], result['run_heavy'], result['changed_files']), ('plain-docs', True, 1))
        self.config['mode'] = 'enabled';self.write_policy()
        result = self.plan(head)
        self.assertEqual((result['classification'], result['run_heavy']), ('plain-docs', False))

    def test_fonts_qml_parameters_contracts_and_shared_inputs_are_never_plain_docs(self):
        self.config['mode'] = 'enabled';self.write_policy();self.base = self.commit()
        for name in self.paths[1:]:
            with self.subTest(path=name):
                head = self.edit(name)
                result = self.plan(head)
                self.assertEqual((result['classification'], result['run_heavy']), ('full', True))
                self.base = head

    def test_unknown_added_deleted_and_renamed_paths_fall_back_to_full(self):
        head = self.edit('unknown/new.md')
        self.assertEqual(self.plan(head)['classification'], 'full')
        self.base = head
        self.git('mv', 'docs/README.md', 'docs/renamed.md');head = self.commit()
        self.assertEqual(self.plan(head)['classification'], 'full')
        self.base = head
        self.git('rm', 'docs/renamed.md');head = self.commit()
        self.assertEqual(self.plan(head)['classification'], 'full')

    def test_missing_base_unrelated_base_and_wrong_checkout_are_full(self):
        head = self.edit('docs/README.md')
        for before in ['', '0' * 40, 'f' * 40]:
            with self.subTest(before=before):
                self.assertTrue(self.plan(head, payload={'before': before, 'after': head})['run_heavy'])
        self.assertEqual(self.plan(self.base)['classification'], 'full')
        self.assertEqual(self.plan(head, payload={'before': self.base, 'after': self.base})['classification'], 'full')
        self.git('checkout', '--orphan', 'unrelated');self.edit('docs/README.md')
        self.assertEqual(self.plan(self.git('rev-parse', 'HEAD'))['classification'], 'full')

    def test_real_same_repository_merge_tree_is_verified_and_forks_are_full(self):
        self.git('checkout', '-b', 'feature');head = self.edit('docs/README.md')
        self.git('checkout', 'main');self.git('merge', '--no-ff', '-m', 'owned merge', 'feature')
        merge = self.git('rev-parse', 'HEAD')
        pr = {'pull_request': {'head': {'repo': {'full_name': 'owned/repo'}, 'sha': head}, 'base': {'sha': self.base}}}
        self.assertEqual(self.plan(merge, 'pull_request', pr)['classification'], 'plain-docs')
        pr['pull_request']['head']['repo']['full_name'] = 'foreign/fork'
        self.assertEqual(self.plan(merge, 'pull_request', pr)['reason'], 'fork-pr')
        pr['pull_request']['head']['repo']['full_name'] = 'owned/repo'
        pr['pull_request']['head']['sha'] = self.base
        self.assertEqual(self.plan(merge, 'pull_request', pr)['reason'], 'unverified-pr-merge-tree')

    def test_tags_dispatch_and_unknown_events_keep_full_validation(self):
        head = self.edit('docs/README.md')
        for event, ref in [('workflow_dispatch', 'refs/heads/main'), ('push', 'refs/tags/owned'),
                           ('unknown', 'refs/heads/main')]:
            self.assertTrue(self.plan(head, event, GITHUB_REF=ref)['run_heavy'])

    def test_large_binary_or_malformed_documents_cannot_suppress_jobs(self):
        head = self.edit('docs/README.md')
        self.config['max_document_bytes'] = 1;self.write_policy()
        self.assertEqual(self.plan(head)['classification'], 'full')
        self.config['max_document_bytes'] = 131072;self.write_policy()
        (self.root / 'docs/README.md').write_bytes(b'owned\0document')
        head = self.commit();self.assertEqual(self.plan(head)['classification'], 'full')
        (self.root / 'docs/README.md').write_bytes(b'\xff\xfe')
        head = self.commit();self.assertEqual(self.plan(head)['classification'], 'full')

    def test_complete_diff_limits_missing_terminator_duplicates_and_modes_are_rejected(self):
        row = b':100644 100644 ' + b'a' * 40 + b' ' + b'b' * 40 + b' M\0docs/README.md\0'
        self.assertEqual(ci.changed_records(row, self.config)[0][0], 'docs/README.md')
        for raw in [row[:-1], row * 2, row.replace(b'100644', b'100755', 1), b'x' * 1048577, b'']:
            with self.subTest(bytes=len(raw)), self.assertRaises(ValueError):ci.changed_records(raw, self.config)
        small = dict(self.config, max_files=1)
        with self.assertRaises(ValueError):ci.changed_records(row + row.replace(b'docs/README.md', b'other.md'), small)

    def test_git_read_or_policy_failure_returns_full_without_private_error(self):
        head = self.edit('docs/README.md')
        with patch.object(ci, 'git', side_effect=OSError('owned private host path')):
            result = self.plan(head)
        self.assertEqual((result['classification'], result['reason']), ('full', 'classification-read-failed'))
        self.assertNotIn('private', json.dumps(result))
        (self.root / 'scripts/ci-plan-policy.json').write_text('{broken', encoding='utf-8')
        self.assertTrue(self.plan(head)['run_heavy'])

    def test_html_shared_json_and_font_dependencies_are_covered_without_generated_paths(self):
        import fnmatch
        for name in ['docs/visual/qt-main-reference.json', 'docs/visual/preflight-reference.json',
                     'docs/visual/about-reference.json', 'docs/visual/tokens.json', 'design/assets/fonts/owned.ttf',
                     '.github/actions/cache-inputs/action.yml', 'scripts/ci-timing.sh']:
            self.assertTrue(any(fnmatch.fnmatchcase(name, pattern) for pattern in self.config['html_inputs']), name)
        self.assertFalse(any(pattern.startswith(('build/', 'target/', 'dist/')) for pattern in self.config['html_inputs']))


if __name__ == '__main__':unittest.main()
