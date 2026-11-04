#!/usr/bin/env python3
"""Verify optional CI evidence accepts only actual matching successful runs."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('collect_wp12_ci', ROOT / 'scripts/collect-wp12-ci.py')
collector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(collector)
SHA = 'a' * 40


class CollectCiTests(unittest.TestCase):
    def test_selected_runs_require_matching_identity_and_success(self):
        run = dict(name='Foundation probes', head_sha=SHA, status='completed', conclusion='success',
                   event='workflow_dispatch')
        collector.require_run(run, {'Foundation probes'}, SHA)
        for field, value in [('name', 'CI'), ('head_sha', 'b' * 40),
                             ('status', 'in_progress'), ('conclusion', 'failure')]:
            with self.subTest(field=field), self.assertRaises(ValueError):
                collector.require_run(dict(run, **{field: value}), {'Foundation probes'}, SHA)

    def test_product_requires_three_platforms_and_latest_success(self):
        names = ['qt-plan', 'qt-gate', 'native-qt (ubuntu-24.04)',
                 'native-qt (windows-2022)', 'native-qt (macos-15)']
        jobs = [dict(id=i + 1, name=name, status='completed', conclusion='success',
                     started_at='2026-01-01T00:00:00Z') for i, name in enumerate(names)]
        collector.require_product_jobs(jobs)
        with self.assertRaises(ValueError):
            collector.require_product_jobs(jobs[:-1])
        failed = copy.deepcopy(jobs)
        failed[2]['conclusion'] = 'failure'
        with self.assertRaises(ValueError):
            collector.require_product_jobs(failed)
        retried = copy.deepcopy(jobs)
        retried.append(dict(retried[2], id=10, started_at='2026-01-01T00:01:00Z', conclusion='failure'))
        with self.assertRaises(ValueError):
            collector.require_product_jobs(retried)
        retried.append(dict(retried[-1], id=11, started_at='2026-01-01T00:02:00Z', conclusion='success'))
        collector.require_product_jobs(retried)

    def test_collector_uses_explicit_same_commit_manual_runs(self):
        names = ['qt-plan', 'qt-gate', 'native-qt (ubuntu-24.04)',
                 'native-qt (windows-2022)', 'native-qt (macos-15)']
        jobs = [dict(id=i + 1, name=name, status='completed', conclusion='success',
                     started_at='2026-01-01T00:00:00Z') for i, name in enumerate(names)]
        def run(name):
            return dict(name=name, head_sha=SHA, status='completed', conclusion='success', event='workflow_dispatch')
        responses = {
            'actions/runs/1': run('Qt components'),
            'actions/runs/1/jobs?filter=all&per_page=100': {'jobs': jobs},
            'actions/runs/2': run('Foundation probes'),
            'actions/runs/3': run('CI'),
            'actions/runs/1/artifacts': {'artifacts': [
                {'id': 10, 'name': 'xdzip-m4-Windows-X64'},
                {'id': 11, 'name': 'xdzip-m4-Linux-X64'},
                {'id': 12, 'name': 'xdzip-checks-macOS-ARM64'},
                {'id': 13, 'name': 'ci-qt-input-qt-Windows-X64-1'},
            ]},
        }
        parent = ROOT / 'build/test-collect-wp12-ci'
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=parent) as temporary:
            root = Path(temporary)
            (root / 'build').mkdir()
            output = root / 'build/evidence'
            argv = ['collector', '--run', '1', '--foundation-run', '2', '--ci-run', '3', '--output', str(output)]
            files = {'build.json': 'a' * 64, 'deployment-audit.json': 'b' * 64}
            with patch.object(collector, 'ROOT', root), patch.object(collector, 'api', side_effect=responses.get), \
                    patch.object(collector.subprocess, 'check_output', return_value=SHA + '\n'), \
                    patch.object(collector, 'download_records', return_value=files) as download, \
                    patch('sys.argv', argv):
                collector.main()
            report = json.loads((output / 'ci-evidence.json').read_text(encoding='utf-8'))
            self.assertEqual(report['revision'], SHA)
            self.assertEqual((report['foundation_run_id'], report['ci_run_id']), (2, 3))
            self.assertEqual({r['platform'] for r in report['artifacts']}, {'windows', 'linux', 'macos'})
            self.assertEqual(download.call_count, 3)

            missing = root / 'build/missing-installer'
            argv = ['collector', '--run', '1', '--foundation-run', '2', '--ci-run', '3',
                    '--output', str(missing), '--packages']
            with patch.object(collector, 'ROOT', root), patch.object(collector, 'api', side_effect=responses.get), \
                    patch.object(collector.subprocess, 'check_output', return_value=SHA + '\n'), \
                    patch.object(collector, 'download_records', return_value=files), patch('sys.argv', argv), \
                    self.assertRaisesRegex(ValueError, 'installer is missing'):
                collector.main()
            self.assertFalse((missing / 'ci-evidence.json').exists())


if __name__ == '__main__':
    unittest.main()
