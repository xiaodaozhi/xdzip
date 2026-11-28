#!/usr/bin/env python3
"""Repeat the native fullscreen entry cases and retain the first failure."""

import argparse
import importlib.util
import json
import os
from pathlib import Path
import sys

spec = importlib.util.spec_from_file_location('native_sessions', Path(__file__).with_name('check-native-sessions.py'))
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tests', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--attempts', type=int, default=12)
    args = parser.parse_args()
    output = args.output.resolve()
    if sys.platform != 'darwin' or not output.is_relative_to(native.ROOT / 'build') or args.attempts < 1:
        parser.error('Requires macOS, an output directory under build, and at least one attempt')
    output.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, QT_QPA_PLATFORM='cocoa')
    runs = []
    for attempt in range(1, args.attempts + 1):
        result = native.run_cases(args.tests, native.COCOA_ENTRY_CASES,
                                  output / f'fullscreen-entry-{attempt}.log', env, 90)
        runs.append(result)
        print(f"Attempt {attempt}: exit={result['exit_code']}, elapsed_ms={result['elapsed_ms']}", flush=True)
        if result['exit_code']:
            break
    (output / 'fullscreen-diagnostic.json').write_text(json.dumps({'runs': runs}, indent=2) + '\n', encoding='utf-8')
    if any(run['exit_code'] for run in runs):
        raise SystemExit(1)


if __name__ == '__main__':
    main()
