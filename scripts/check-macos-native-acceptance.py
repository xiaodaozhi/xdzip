#!/usr/bin/env python3
"""Check native probe receipts and the data rows covering the release failures."""

import argparse
import json
from pathlib import Path
import re

from importlib.util import module_from_spec, spec_from_file_location


spec = spec_from_file_location('native_sessions', Path(__file__).with_name('check-native-sessions.py'))
native = module_from_spec(spec)
spec.loader.exec_module(native)


def check(path, architecture):
    receipt = json.loads(path.read_text(encoding='utf-8'))
    if receipt.get('status') != 'passed' or receipt.get('shell_status') != 'passed':
        raise ValueError('Native session or shell checks failed')
    if receipt.get('native_plugin') != 'cocoa' or receipt.get('timed_out'):
        raise ValueError('Expected completed native Cocoa checks')
    browser = receipt['browser_runs']
    expected_groups = 4 if architecture == 'x86_64' else 1
    if len(browser) != expected_groups or sum((run['cases'] for run in browser), []) != native.CASES:
        raise ValueError('Native session groups do not cover the complete case list')
    runs = browser + [receipt['shell_run']] + receipt['cocoa_runs']
    if any(run['exit_code'] != 0 or run['timed_out'] for run in runs):
        raise ValueError('A native group failed or timed out')
    cases = {
        'cocoaMinimizeAtFullscreenEntry': {'minimize', 'restore-during-retry', 'destroy-during-retry'},
        'cocoaMinimizeDuringFullscreenRestore': {'minimize', 'restore', 'normal', 'maximize'},
        'directoryContextExpansionReasons': {f'{language}-{kind}' for language in ('zh-CN', 'en-US')
                                             for kind in ('mouse', 'menu', 'shift-f10')},
        'globalQuitReturnsToPasswordTaskThenCancelsAndJoinsEverySession': {
            'ready-password', 'notify-send-before-password-present', 'post-after-password-present-control',
            'other-session-notify-send-before-password-present',
            'other-window-notify-send-before-password-present', 'notify-send-wait-resumes-password'},
    }
    for name, expected in cases.items():
        matches = [run for run in runs if name in run['cases']]
        if len(matches) != 1:
            raise ValueError(f'Expected one group for {name}')
        log = (path.parent / matches[0]['log']).read_text(encoding='utf-8')
        rows = re.findall(r'^PASS\s+: BrowserTest::' + re.escape(name) + r'\(([^)]+)\)', log, re.M)
        if len(rows) != len(expected) or set(rows) != expected:
            raise ValueError(f'Incomplete native data rows for {name}: {rows}')
        if name.startswith('cocoaMinimize'):
            for row in expected - {'destroy-during-retry'}:
                prefix = f'BrowserTest::{name}({row})'
                notifications = re.findall(
                    r'NSWindow(?:Will|Did)(?:Enter|Exit)FullScreenNotification',
                    '\n'.join(line for line in log.splitlines() if prefix in line))
                if (not notifications or
                        notifications[-1] != 'NSWindowDidExitFullScreenNotification'):
                    raise ValueError(f'Native fullscreen exit did not finish for {name}({row})')
    return {run['log']: run['elapsed_ms'] for run in runs}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--architecture', choices=['arm64', 'x86_64'], required=True)
    parser.add_argument('receipts', nargs='+', type=Path)
    args = parser.parse_args()
    for receipt in args.receipts:
        print(json.dumps({'receipt': str(receipt), 'elapsed_ms': check(receipt, args.architecture)}))


if __name__ == '__main__':
    main()
