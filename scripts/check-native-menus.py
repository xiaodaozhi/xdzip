# scripts/check-native-menus.py

"""Measure cold menu presentation in fresh native Release test processes."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
CONTRACT_PATH = ROOT / 'tests/fixtures/native/menu-qa-contract.json'


def load_contract(path=CONTRACT_PATH):
    contract = json.loads(path.read_text(encoding='utf-8'))
    if (contract['schema_version'] != 1 or not contract['cases']
            or len(set(contract['cases'])) != len(contract['cases'])
            or any(not re.fullmatch('[a-z-]+', case) for case in contract['cases'])
            or contract['repetitions'] < 1
            or not 0 < contract['limits_ms']['p95'] <= 100
            or not contract['limits_ms']['p95'] <= contract['limits_ms']['max'] <= 200):
        raise ValueError('Invalid native menu contract')
    return contract


def number(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def validate_record(record, contract, source=None, strict_windows=False):
    """Validate the same frame, timing and identity fields consumed by the launcher."""
    errors = []
    for field in ('trigger_to_complete_frame_ms', 'trigger_to_capture_complete_ms'):
        if not number(record.get(field)) or record[field] < 0:
            errors.append('invalid ' + field)
    if number(record.get('trigger_to_complete_frame_ms')) and record['trigger_to_complete_frame_ms'] > contract['limits_ms']['max']:
        errors.append('sample timing budget exceeded')
    first = record.get(contract['reference_generation_field'])
    if not number(first) or first < 1 or first != int(first):
        errors.append('invalid first visible generation')
    for field in contract['required_generation_fields']:
        if not number(record.get(field)) or record[field] != first:
            errors.append('first frame mismatch: ' + field)
    if record.get('shadow_same_frame') is not True:
        errors.append('shadow missing from first frame')
    if source is not None and record.get('revision') != source[:12]:
        errors.append('record source mismatch')
    if strict_windows:
        for field, expected in contract['windows'].items():
            if record.get(field) != expected:
                errors.append('Windows record mismatch: ' + field)
    return errors


def summarize(records, failures, contract, *, cases=None, repetitions=None, source=None, strict_windows=False):
    cases = contract['cases'] if cases is None else cases
    repetitions = contract['repetitions'] if repetitions is None else repetitions
    failures = list(failures)
    expected = {(case, repetition) for case in cases for repetition in range(1, repetitions + 1)}
    seen = set()
    for record in records:
        key = (record.get('menu'), record.get('repetition'))
        if key not in expected or key in seen:
            failures.append({'menu': key[0], 'repetition': key[1], 'failure': 'unexpected or duplicate sample'})
        seen.add(key)
        for reason in validate_record(record, contract, source, strict_windows):
            failures.append({'menu': key[0], 'repetition': key[1], 'failure': reason})
    if seen != expected or len(records) != len(expected):
        failures.append({'failure': 'incomplete sample matrix', 'expected': len(expected), 'actual': len(records)})
    values = sorted(record['trigger_to_complete_frame_ms'] for record in records if number(record.get('trigger_to_complete_frame_ms')))
    capture = sorted(record['trigger_to_capture_complete_ms'] for record in records if number(record.get('trigger_to_capture_complete_ms')))
    p95 = values[math.ceil(len(values) * .95) - 1] if values else None
    maximum = max(values) if values else None
    if p95 is not None and p95 > contract['limits_ms']['p95']:
        failures.append({'failure': 'p95 timing budget exceeded'})
    return {'status': 'failed' if failures else 'passed', 'p95_ms': p95, 'max_ms': maximum,
            'limits_ms': contract['limits_ms'], 'records': records, 'failures': failures,
            'capture_p95_ms': capture[math.ceil(len(capture) * .95) - 1] if capture else None}


def validate_existing(directory, tests, source, binary_hash, contract):
    """Recheck a complete interactive run without starting any process or window."""
    launcher = json.loads((directory / 'review.json').read_text(encoding='utf-8-sig'))
    failures = []
    if hashlib.sha256(tests.read_bytes()).hexdigest() != binary_hash or launcher.get('binary_sha256') != binary_hash:
        failures.append({'failure': 'binary SHA256 mismatch'})
    if launcher.get('full_source_sha') != source:
        failures.append({'failure': 'launcher source mismatch'})
    if (launcher.get('status') != 'passed' or launcher.get('started_by_real_foreground_mouse_click') is not True
            or launcher.get('fresh_process_per_sample') is not True or launcher.get('menu_warmup') is not False):
        failures.append({'failure': 'interactive run not completed under required input/process gates'})
    failures.extend(launcher.get('failures', []))
    records, expected_launches = [], []
    for repetition in range(1, contract['repetitions'] + 1):
        for case in contract['cases']:
            sample = directory / f'{repetition:02}-{case}'
            expected_launches.append((case, repetition))
            if (sample / 'occlusion-rejected.json').exists():
                failures.append({'menu': case, 'repetition': repetition, 'failure': 'occluded sample'})
            path = sample / (case + '.json')
            if not path.is_file():
                failures.append({'menu': case, 'repetition': repetition, 'failure': 'missing native record'})
                continue
            record = json.loads(path.read_text(encoding='utf-8-sig'))
            if record.get('menu') != case:
                failures.append({'menu': case, 'repetition': repetition, 'failure': 'native record menu mismatch'})
            record['repetition'] = repetition
            records.append(record)
    if records != launcher.get('records') or launcher.get('valid_samples') != len(records):
        failures.append({'failure': 'launcher records differ from native records'})
    launches = launcher.get('owned_child_launches', [])
    if len(launches) != len(expected_launches):
        failures.append({'failure': 'incomplete owned-child launch matrix'})
    for index, launch in enumerate(launches):
        # Windows may recycle a PID after the prior owned handle is closed.
        # Runtime identity comes from each direct Process.Start and live handle.
        pid = launch.get('owned_child_pid')
        if (index >= len(expected_launches) or launch.get('sample') != index + 1
                or type(pid) is not int or not 0 < pid <= 0xffffffff
                or launch.get('menu') != expected_launches[index][0]
                or launch.get('binary_sha256') != binary_hash or launch.get('binary_path_verified') is not True
                or launch.get('child_os_foreground_seen') is not True or launch.get('exit_code') != 0):
            failures.append({'failure': 'owned-child identity/foreground/exit mismatch', 'sample': index + 1})
    return {**summarize(records, failures, contract, source=source, strict_windows=True),
            'full_source_sha': source, 'binary_sha256': binary_hash,
            'existing_evidence_only': True, 'processes_started': 0}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tests', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--contract', type=Path, default=CONTRACT_PATH)
    parser.add_argument('--repetitions', type=int)
    parser.add_argument('--cases', nargs='+')
    parser.add_argument('--backend', choices=['native', 'software'], default='native')
    parser.add_argument('--validate-existing', type=Path)
    parser.add_argument('--source')
    parser.add_argument('--binary-sha256', '--hash', dest='binary_sha256')
    args = parser.parse_args()
    contract = load_contract(args.contract)
    args.repetitions = contract['repetitions'] if args.repetitions is None else args.repetitions
    args.cases = contract['cases'] if args.cases is None else args.cases
    output = args.output.resolve()
    if (not output.is_relative_to(ROOT / 'build') or args.repetitions < 1
            or not args.cases or any(case not in contract['cases'] for case in args.cases)):
        parser.error('Evidence belongs under build/; repetitions must be positive')
    output.mkdir(parents=True, exist_ok=True)
    if args.validate_existing:
        if (not args.validate_existing.resolve().is_relative_to(ROOT / 'build')
                or not re.fullmatch('[0-9a-f]{40}', args.source or '')
                or not re.fullmatch('[0-9a-f]{64}', args.binary_sha256 or '')
                or args.cases != contract['cases'] or args.repetitions != contract['repetitions']):
            parser.error('Existing interactive evidence requires full source/binary hashes and the complete contract matrix')
        report = validate_existing(args.validate_existing.resolve(), args.tests, args.source, args.binary_sha256, contract)
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
        print(json.dumps({'status': report['status'], 'samples': len(report['records']), 'failures': len(report['failures'])}))
        raise SystemExit(0 if report['status'] == 'passed' else 1)
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
            rejection = directory / 'occlusion-rejected.json'
            if rejection.is_file():
                failures.append({'menu': case, 'repetition': repetition + 1,
                                 'failure': 'owned client occluded; capture rejected',
                                 'occlusion_rejections': json.loads(rejection.read_text(encoding='utf-8'))['occlusion_rejections']})
                continue
            if result.returncode or not evidence.is_file():
                failures.append({'menu': case, 'repetition': repetition + 1,
                                 'exit_code': result.returncode, 'missing_record': not evidence.is_file()})
            if evidence.is_file():
                record = json.loads(evidence.read_text(encoding='utf-8'))
                record['repetition'] = repetition + 1
                records.append(record)
    report = {**summarize(records, failures, contract, cases=args.cases, repetitions=args.repetitions, source=args.source,
                         strict_windows=sys.platform == 'win32' and args.backend == 'native'),
              'reference_machine': platform.platform(),
              'processor': platform.processor(),
              'binary_sha256': hashlib.sha256(args.tests.read_bytes()).hexdigest(),
              'backend': args.backend, 'fresh_process_per_sample': True, 'menu_warmup': False,
              'measurement': 'Input to first complete face frame confirmed by same-frame shadow pixels. Windows records DwmFlush return before desktop bitmap readback. Capture completion and pixel-validation completion remain separate metrics; the 100/200ms gate excludes those probe costs. Other platforms retain screen-capture completion as their conservative presentation timestamp. frameSwapped observation is retained separately.'}
    (output / 'review.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': report['status'], 'samples': len(records), 'p95_ms': report['p95_ms'],
                      'max_ms': report['max_ms'], 'failures': len(report['failures'])}))
    raise SystemExit(0 if report['status'] == 'passed' else 1)


if __name__ == '__main__':
    main()
