#!/usr/bin/env python3
"""Prepare fixed inputs and execute the complete real-worker runtime test inventory."""
import argparse
import csv
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / 'specs/engineering/runtime-worker-tests.json'
SUMMARY = re.compile(r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', re.MULTILINE)
OUTCOME = re.compile(r'^test ([\w:]+) \.\.\. (ok|FAILED|ignored)(?:\s|$)', re.MULTILINE)


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def load_module(filename):
    spec = importlib.util.spec_from_file_location(filename.replace('-', '_'), ROOT/'scripts'/filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def inventory():
    spec = json.loads(SPEC.read_text(encoding='utf-8'))
    cases = spec['cases']
    names = [case['name'] for case in cases]
    if spec['schema_version'] != 1 or len(names) != 31 or len(set(names)) != len(names):
        raise ValueError('Runtime inventory must contain the 31 distinct tracked cases')
    for case in cases:
        if (case['profile'] not in spec['profiles'] or not 1 <= case['timeout_seconds'] <= 900
                or not re.fullmatch(r'[a-z_][\w]*(?:::[a-z_][\w]*)+', case['name'])
                or not set(case['prerequisites']) <= {'rar-fixed', 'rar-comments', 'rar7-genuine', 'million-index'}
                or not (ROOT/case['source']).resolve().is_relative_to(ROOT/'crates/xdzip-runtime/src')):
            raise ValueError('Invalid runtime case declaration')
    return cases


def select(cases, profile, names):
    if names:
        if profile or len(set(names)) != len(names) or not set(names) <= {case['name'] for case in cases}:
            raise ValueError('Explicit cases must be unique inventory names, without a profile')
        return [case for case in cases if case['name'] in names]
    return [case for case in cases if profile in (None, 'all', case['profile'])]


def sanitized(text):
    for path, replacement in [(str(ROOT), '<workspace>'), (str(Path.home()), '<home>')]:
        for variant in sorted({path, path.replace('\\', '/'), path.replace('\\', '\\\\')}, key=len, reverse=True):
            text = text.replace(variant, replacement)
    return text


def write_json(path, data):
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(data, indent=2) + '\n', encoding='utf-8')
    temporary.replace(path)


def private_directory(path):
    path.mkdir(mode=0o700)
    if os.name == 'nt':
        result = subprocess.run(['whoami', '/user', '/fo', 'csv', '/nh'], capture_output=True, text=True, check=True)
        sid = next(csv.reader(result.stdout.splitlines()))[1]
        subprocess.run(['icacls', str(path), '/inheritance:r', '/grant:r', '*' + sid + ':(OI)(CI)F',
                        '/grant:r', 'SYSTEM:(OI)(CI)F'], capture_output=True, check=True)


def prepare(selected, output, args):
    required = {value for case in selected for value in case['prerequisites']}
    environment, sources = {}, {}
    if 'rar-fixed' in required:
        module = load_module('prepare-rar-rust-fixtures.py')
        environment, sources['rar'] = module.prepare(output/'fixtures/rar', args.rar_cache, args.offline)
        environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'  # Existing public testdata password.
    if 'rar-comments' in required:
        module = load_module('prepare-rar-rust-fixtures.py')
        destination = output/'fixtures/rar-comments'
        items = [
            ('FCOMM.RAR', 75, 'db470414d97b8d7a3213eaf5957b139e01621012ede09eda1565342709aa1191',
             'bitplane/rar-research/c87266e2e47c210c60b3cc0acb81f2633ec1fcb2/fixtures/1.402/FCOMM.RAR'),
            ('rar202-comment-nopsw.rar', 204, '16715eaf4163092733579f8a616e83a78e3feae2b8bae15b1538bc88bf0dbc86',
             'markokr/rarfile/d2f7df6fc843dae356fd6b0a85971dc36fd6e757/test/files/rar202-comment-nopsw.rar'),
            ('rar3-comment-plain.rar', 300, '224a3f0aacab5cbb140b72dde616280a871589f251b9147bdb55b49651661813',
             'markokr/rarfile/d2f7df6fc843dae356fd6b0a85971dc36fd6e757/test/files/rar3-comment-plain.rar'),
            ('rar3-comment-hpsw.rar', 484, '29c61403d1e7cec0b15eb7e33cfa217cebd56bf1a4a78a2642715eca474350b2',
             'markokr/rarfile/d2f7df6fc843dae356fd6b0a85971dc36fd6e757/test/files/rar3-comment-hpsw.rar'),
            ('rar3-comment-psw.rar', 332, 'ffb9b41571ac4d56ebf9a5ce5323740ea6150d66823b66abf7566f8dd2639803',
             'markokr/rarfile/d2f7df6fc843dae356fd6b0a85971dc36fd6e757/test/files/rar3-comment-psw.rar'),
        ]
        records = []
        for name, size, sha256, path in items:
            item = dict(file=name, bytes=size, sha256=sha256,
                        url='https://raw.githubusercontent.com/' + path, encoding='raw',
                        cache_subdirectory='public')
            _, downloaded = module.public_data(item, destination, args.rar_cache, args.offline)
            records.append({**item, 'downloaded_bytes': downloaded})
        sources['rar_comments'] = records
        environment['XDZIP_RAR_COMMENT_FIXTURES'] = str(destination)
    if 'rar7-genuine' in required:
        module = load_module('check-rar7-genuine-fixture.py')
        path, sources['rar7'] = module.prepare(output/'fixtures/rar7', args.rar7_cache, args.offline)
        environment['XDZIP_RAR7_GENUINE_FIXTURE'] = str(path)
    if 'million-index' in required:
        module = load_module('probe-large-index.py')
        path = output/'fixtures/owned-million.zip'
        path.parent.mkdir(parents=True, exist_ok=True)
        module.generate(path, 1_000_000)
        sources['index'] = {'records': 1_000_000, 'bytes': path.stat().st_size,
                            'sha256': digest(path), 'recipe_sha256': digest(ROOT/'scripts/probe-large-index.py')}
        environment['XDZIP_LARGE_INDEX_FIXTURE'] = str(path)
    write_json(output/'sources.json', sources)
    return environment


def compile_binary(output, environment):
    result = subprocess.run(['cargo', 'test', '--locked', '--release', '-p', 'xdzip-runtime', '--lib',
                             '--no-run', '--message-format=json'], cwd=ROOT, env=environment,
                            capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=600)
    (output/'build.log').write_text(sanitized(result.stdout + result.stderr), encoding='utf-8')
    executables = []
    for line in result.stdout.splitlines():
        record = json.loads(line)
        if (record.get('reason') == 'compiler-artifact' and record.get('target', {}).get('name') == 'xdzip_runtime'
                and record.get('profile', {}).get('test') and record.get('executable')):
            executables.append(Path(record['executable']).resolve(strict=True))
    if result.returncode or len(executables) != 1:
        raise ValueError('Exactly one successfully built runtime library test binary is required')
    return executables[0]


def list_tests(binary, environment, ignored):
    result = subprocess.run([str(binary), '--list', '--format=terse', *(['--ignored'] if ignored else [])],
                            cwd=ROOT, env=environment, capture_output=True, text=True, timeout=30, check=True)
    return {line.removesuffix(': test') for line in result.stdout.splitlines() if line.endswith(': test')}


def validate_result(log, expected):
    summaries = SUMMARY.findall(log)
    outcomes = OUTCOME.findall(log)
    if (len(summaries) != 1 or summaries[0] != ('ok', str(len(expected)), '0', '0')
            or len(outcomes) != len(expected) or {name for name, _ in outcomes} != set(expected)
            or any(status != 'ok' for _, status in outcomes)):
        raise ValueError('Exact selected tests must all pass, with no ignored, missing or extra outcomes')


def execute(binary, arguments, names, environment, log, timeout):
    # A fresh process group lets the timeout kill only this test and its worker descendants.
    options = {'creationflags': subprocess.CREATE_NEW_PROCESS_GROUP} if os.name == 'nt' else {'start_new_session': True}
    started = time.monotonic()
    with log.open('w', encoding='utf-8') as stream:
        process = subprocess.Popen([str(binary), *arguments, '--test-threads=1'], cwd=ROOT, env=environment,
                                   stdout=stream, stderr=subprocess.STDOUT, **options)
        try:
            code = process.wait(timeout=timeout)
        except BaseException:
            if process.poll() is None:
                if os.name == 'nt':
                    subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'], capture_output=True, timeout=15)
                else:
                    os.killpg(process.pid, signal.SIGKILL)
                process.kill()
                process.wait(timeout=15)
            raise
    text = sanitized(log.read_text(encoding='utf-8', errors='replace'))
    log.write_text(text, encoding='utf-8')
    if code:
        raise ValueError('Runtime test process failed; see the individual log')
    validate_result(text, names)
    return time.monotonic() - started


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--worker', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--profile', choices=['all', 'worker', 'large'])
    parser.add_argument('--case', action='append', default=[])
    parser.add_argument('--include-regular', action='store_true', help='also run the ordinary runtime suite, without ignores')
    parser.add_argument('--rar-cache', type=Path)
    parser.add_argument('--rar7-cache', type=Path)
    parser.add_argument('--offline', action='store_true')
    args = parser.parse_args()
    cases = inventory()
    selected = select(cases, args.profile, args.case)
    if args.include_regular and (args.case or len(selected) != len(cases)):
        parser.error('--include-regular requires the complete inventory')
    output = args.output.resolve()
    if output == ROOT/'build' or not output.is_relative_to(ROOT/'build'):
        parser.error('Use a dedicated output below build/')
    output.mkdir(parents=True, exist_ok=True)
    run = output/('run-' + uuid.uuid4().hex[:12])
    run.mkdir()
    report = {'schema_version': 1, 'status': 'running', 'selected': len(selected), 'passed': 0,
              'cases': [{'name':case['name'], 'status':'not_run'} for case in selected],
              'excluded_by_profile': [case['name'] for case in cases if case not in selected],
              'run': run.relative_to(ROOT).as_posix()}
    def save():
        write_json(run/'report.json', report)
        write_json(output/'report.json', report)
    save()
    try:
        worker = args.worker.resolve(strict=True)
        if not worker.is_file() or args.worker.is_symlink():
            raise ValueError('A built regular worker is required')
        report['worker_sha256'] = digest(worker)
        report['inventory_sha256'] = digest(SPEC)
        environment = dict(os.environ, CARGO_INCREMENTAL='0', XDZIP_NATIVE_CODEC=str(worker),
                           XDZIP_NATIVE_CODEC_WORKER=str(worker), XDZIP_CAB_BACKEND='rust')
        temporary = run/'tmp'
        private_directory(temporary)
        environment.update(TEMP=str(temporary), TMP=str(temporary), TMPDIR=str(temporary))
        environment.update(prepare(selected, run, args))
        print('Fixed fixtures prepared; building the actual runtime tests', flush=True)
        binary = compile_binary(run, environment)
        report['test_binary_sha256'] = digest(binary)
        observed = list_tests(binary, environment, True)
        if observed != {case['name'] for case in cases}:
            raise ValueError('Compiled ignored inventory differs from the declared cases')
        if args.include_regular:
            all_names = list_tests(binary, environment, False)
            print('Running the complete runtime suite including all integration cases', flush=True)
            elapsed = execute(binary, ['--include-ignored'], all_names, environment, run/'all-runtime.log', 3600)
            report['ordinary_and_integration'] = {'passed': len(all_names), 'failed': 0, 'ignored': 0, 'seconds': elapsed}
            report['cases'] = [{'name': case['name'], 'status': 'passed'} for case in selected]
            report['passed'] = len(selected)
        else:
            for case, outcome in zip(selected, report['cases']):
                label = case['name'].replace('::', '-')
                print('Running ' + case['name'], flush=True)
                outcome['status'] = 'running'
                save()
                elapsed = execute(binary, ['--ignored', '--exact', case['name']], {case['name']}, environment,
                                  run/(label+'.log'), case['timeout_seconds'])
                outcome.update(status='passed', seconds=elapsed)
                report['passed'] += 1
                save()
        report['status'] = 'passed'
        save()
    except Exception as error:
        if args.include_regular and (run/'all-runtime.log').is_file():
            observed = dict(OUTCOME.findall((run/'all-runtime.log').read_text(encoding='utf-8', errors='replace')))
            for outcome in report['cases']:
                if outcome['name'] in observed:
                    outcome['status'] = 'passed' if observed[outcome['name']] == 'ok' else 'failed'
            report['passed'] = sum(outcome['status'] == 'passed' for outcome in report['cases'])
        for outcome in report['cases']:
            if outcome['status'] == 'running':
                outcome['status'] = 'failed'
        report.update(status='failed', error=sanitized(type(error).__name__ + ': ' + str(error)))
        save()
        raise
    print('Runtime integration inventory: ' + str(report['passed']) + ' passed, 0 failed, 0 ignored', flush=True)


if __name__ == '__main__':
    main()
