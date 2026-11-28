#!/usr/bin/env python3
"""Check native session batching without starting a window manager."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('native_sessions', ROOT / 'scripts/check-native-sessions.py')
sessions = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sessions)
probe_spec = importlib.util.spec_from_file_location(
    'macos_native_acceptance', ROOT / 'scripts/check-macos-native-acceptance.py')
probe = importlib.util.module_from_spec(probe_spec)
probe_spec.loader.exec_module(probe)
diagnostic_spec = importlib.util.spec_from_file_location(
    'macos_fullscreen_diagnostic', ROOT / 'scripts/diagnose-macos-fullscreen.py')
diagnostic = importlib.util.module_from_spec(diagnostic_spec)
diagnostic_spec.loader.exec_module(diagnostic)


class NativeSessionGroupsTest(unittest.TestCase):
    def test_fullscreen_diagnostic_retains_first_failed_attempt(self):
        with tempfile.TemporaryDirectory(dir=ROOT / 'build') as directory, \
             patch.object(diagnostic.sys, 'platform', 'darwin'), \
             patch.object(diagnostic.sys, 'argv', ['diagnose', '--tests', 'browser',
                                                   '--output', directory, '--attempts', '3']), \
             patch.object(diagnostic.native, 'run_cases', side_effect=[
                 {'exit_code': 0, 'elapsed_ms': 1}, {'exit_code': 1, 'elapsed_ms': 2}]) as run:
            with self.assertRaises(SystemExit):
                diagnostic.main()
            self.assertEqual(run.call_count, 2)
            self.assertEqual(run.call_args_list[0].args[1], sessions.COCOA_RESTORE_CASES)
            self.assertEqual(run.call_args_list[1].args[1], sessions.COCOA_ENTRY_CASES)
            report = json.loads((Path(directory) / 'fullscreen-diagnostic.json').read_text(encoding='utf-8'))
            self.assertEqual([result['exit_code'] for result in report['runs']], [0, 1])

    def test_intel_groups_keep_every_case_and_independent_deadlines(self):
        calls = []

        def run_cases(executable, cases, log, env, timeout):
            calls.append((list(cases), timeout))
            log.write_text('group log\n', encoding='utf-8')
            log.with_suffix('.console.log').write_text('console log\n', encoding='utf-8')
            return {'cases': cases, 'log': log.name, 'exit_code': 0, 'timed_out': False}

        with tempfile.TemporaryDirectory() as directory, \
             patch.object(sessions.sys, 'platform', 'darwin'), \
             patch.object(sessions.platform, 'machine', return_value='x86_64'), \
             patch.object(sessions, 'run_cases', side_effect=run_cases):
            log = Path(directory) / 'native-sessions.log'
            runs = sessions.browser_runs(Path('browser'), sessions.CASES, log, {})
            self.assertEqual(len(runs), 4)
            self.assertEqual([case for group, _ in calls for case in group], sessions.CASES)
            self.assertEqual([timeout for _, timeout in calls], [150, 150, 150, 150])
            self.assertEqual(calls[2][0], ['globalQuitReturnsToPasswordTaskThenCancelsAndJoinsEverySession'])
            self.assertEqual(log.read_text(encoding='utf-8').count('group log'), 4)
            self.assertEqual(log.with_suffix('.console.log').read_text(encoding='utf-8').count('console log'), 4)

    def test_probe_rejects_skipped_or_missing_data_rows(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            browser = {'cases': sessions.CASES, 'log': 'browser.log', 'exit_code': 0,
                       'timed_out': False, 'elapsed_ms': 1}
            shell = {'cases': [], 'log': 'shell.log', 'exit_code': 0, 'timed_out': False, 'elapsed_ms': 1}
            cocoa = [{'cases': cases, 'log': name + '.log', 'exit_code': 0,
                      'timed_out': False, 'elapsed_ms': 1}
                     for name, cases in (('conflict', sessions.COCOA_CONFLICT_CASES),
                                         ('restore', sessions.COCOA_RESTORE_CASES),
                                         ('entry', sessions.COCOA_ENTRY_CASES))]
            rows = {
                'browser.log': {
                    'directoryContextExpansionReasons': [f'{language}-{kind}' for language in ('zh-CN', 'en-US')
                                                          for kind in ('mouse', 'menu', 'shift-f10')],
                    'globalQuitReturnsToPasswordTaskThenCancelsAndJoinsEverySession': [
                        'ready-password', 'notify-send-before-password-present',
                        'post-after-password-present-control', 'other-session-notify-send-before-password-present',
                        'other-window-notify-send-before-password-present', 'notify-send-wait-resumes-password'],
                },
                'restore.log': {'cocoaMinimizeDuringFullscreenRestore': ['minimize', 'restore', 'normal', 'maximize']},
                'entry.log': {'cocoaMinimizeAtFullscreenEntry': ['minimize', 'restore-during-retry', 'destroy-during-retry']},
            }
            for name, tests in rows.items():
                (root / name).write_text(''.join(
                    (f'QINFO  : BrowserTest::{test}({row}) NSWindowWillExitFullScreenNotification\n'
                     f'QINFO  : BrowserTest::{test}({row}) NSWindowDidExitFullScreenNotification\n'
                     if name in ('restore.log', 'entry.log') and row != 'destroy-during-retry' else '') +
                    f'PASS   : BrowserTest::{test}({row})\n'
                    for test, values in tests.items() for row in values), encoding='utf-8')
            receipt = root / 'native-sessions.json'
            receipt.write_text(json.dumps({'status': 'passed', 'shell_status': 'passed',
                                           'native_plugin': 'cocoa', 'timed_out': False,
                                           'browser_runs': [browser], 'shell_run': shell, 'cocoa_runs': cocoa}),
                               encoding='utf-8')
            self.assertTrue(probe.check(receipt, 'arm64'))
            restore = root / 'restore.log'
            original_restore = restore.read_text(encoding='utf-8')
            restore.write_text(original_restore.replace(
                'QINFO  : BrowserTest::cocoaMinimizeDuringFullscreenRestore(maximize) '
                'NSWindowDidExitFullScreenNotification\n', ''), encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'Native fullscreen exit did not finish'):
                probe.check(receipt, 'arm64')
            restore.write_text(original_restore, encoding='utf-8')
            entry = root / 'entry.log'
            entry.write_text(entry.read_text(encoding='utf-8').replace('PASS   :', 'SKIP   :', 1), encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'Incomplete native data rows'):
                probe.check(receipt, 'arm64')


if __name__ == '__main__':
    unittest.main()
