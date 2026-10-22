# scripts/check-native-menus.py

"""Measure cold menu presentation in fresh native Release test processes."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
CASES = ['toolbar-more', 'toolbar-extract', 'sort', 'view', 'entry', 'directory',
         'background', 'tasks', 'settings-density', 'settings-scale',
         'settings-language', 'settings-open-mode', 'create-level',
         'create-volume', 'create-volume-unit']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tests', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--repetitions', type=int, default=3)
    parser.add_argument('--cases', nargs='+', choices=CASES, default=CASES)
    parser.add_argument('--backend', choices=['native', 'software'], default='native')
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(ROOT / 'build') or args.repetitions < 1:
        parser.error('Evidence belongs under build/; repetitions must be positive')
    output.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, XDZIP_MENU_FIRST_FRAME='1')
    env['QT_QPA_PLATFORM'] = {'win32': 'windows', 'darwin': 'cocoa', 'linux': 'xcb'}[sys.platform]
    if args.backend == 'native':
        env.pop('QT_QUICK_BACKEND', None)
    else:
        env['QT_QUICK_BACKEND'] = 'software'
    records, failures = [], []
    for repetition in range(args.repetitions):
        for case in args.cases:
            directory = output / f'{repetition + 1:02}-{case}'
            directory.mkdir(exist_ok=True)
            env['XDZIP_MENU_FRAME_OUTPUT'] = str(directory)
            log = directory / 'qt.log'
            try:
                result = subprocess.run([str(args.tests.resolve()), 'nativeMenuFirstFrame:' + case,
                                         '-v1', '-o', str(log) + ',txt'], env=env,
                                        capture_output=True, timeout=35, check=False)
            except subprocess.TimeoutExpired:
                failures.append({'menu': case, 'repetition': repetition + 1,
                                 'failure': 'native process timed out after 35 seconds'})
                continue
            console = (result.stdout + result.stderr).decode('utf-8', errors='replace')
            (directory / 'console.log').write_text(console.replace(str(Path.home()), '<home>'), encoding='utf-8')
            evidence = directory / f'{case}.json'
            if result.returncode or not evidence.is_file():
                failures.append({'menu': case, 'repetition': repetition + 1,
                                 'exit_code': result.returncode, 'missing_record': not evidence.is_file()})
            if evidence.is_file():
                record = json.loads(evidence.read_text(encoding='utf-8'))
                record['repetition'] = repetition + 1
                records.append(record)
    values = sorted(record['trigger_to_complete_frame_ms'] for record in records)
    p95 = values[max(0, math.ceil(len(values) * .95) - 1)] if values else None
    maximum = max(values) if values else None
    passed = (not failures and len(records) == len(args.cases) * args.repetitions
              and p95 <= 100 and maximum <= 200 and all(record['shadow_same_frame'] for record in records))
    capture_values = sorted(record['trigger_to_capture_complete_ms'] for record in records)
    report = {'status': 'passed' if passed else 'failed', 'reference_machine': platform.platform(),
              'processor': platform.processor(),
              'binary_sha256': hashlib.sha256(args.tests.read_bytes()).hexdigest(),
              'backend': args.backend, 'fresh_process_per_sample': True, 'menu_warmup': False,
              'p95_ms': p95, 'max_ms': maximum, 'limits_ms': {'p95': 100, 'max': 200},
              'capture_p95_ms': capture_values[max(0, math.ceil(len(capture_values) * .95) - 1)] if capture_values else None,
              'records': records, 'failures': failures,
              'measurement': 'Input to first complete face frame confirmed by same-frame shadow pixels. Windows records DwmFlush return before desktop bitmap readback. Capture completion and pixel-validation completion remain separate metrics; the 100/200ms gate excludes those probe costs. Other platforms retain screen-capture completion as their conservative presentation timestamp. frameSwapped observation is retained separately.'}
    (output / 'review.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': report['status'], 'samples': len(records), 'p95_ms': p95,
                      'max_ms': maximum, 'failures': len(failures)}))
    raise SystemExit(0 if passed else 1)


if __name__ == '__main__':
    main()
