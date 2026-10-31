#!/usr/bin/env python3
"""Validate same-run evidence and fail closed on absent jobs, platforms or logs."""
import argparse
import ast
from collections import Counter
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import uuid
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('ci_plan', ROOT / 'scripts/ci-plan.py')
planner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(planner)
HOSTS = [('Linux', 'X64'), ('Windows', 'X64'), ('macOS', 'ARM64')]
JOBS = {'docs': ['docs-plan', 'docs'], 'foundation': ['foundation-plan', 'bridge', 'rar7-official-reference', 'cpp-format'],
        'qt': ['qt-plan', 'native-qt']}
HEAVY = {'docs': [], 'foundation': ['bridge', 'rar7-official-reference'], 'qt': ['native-qt']}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def basename(name):
    require(isinstance(name, str) and len(PurePosixPath(name).parts) == 1
            and re.fullmatch('[a-zA-Z0-9_.-]+', name) and name not in ('.', '..'), 'unsafe-evidence-name')
    return name


class Reader:
    def __init__(self, root):
        self.root = root.resolve()
        self.files = {}

    def read(self, name):
        path = self.root / name
        require(path.resolve().is_relative_to(self.root / 'build') or path.resolve().is_relative_to(self.root / 'dist'),
                'evidence-outside-generated-directories')
        require(path.is_file() and not path.is_symlink(), 'missing-raw-evidence')
        content = path.read_bytes()
        require(0 < len(content) <= 100 * 1024 ** 2, 'empty-or-oversized-evidence')
        self.files[name] = {'file': name, 'bytes': len(content), 'sha256': hashlib.sha256(content).hexdigest()}
        return content

    def json(self, name):
        return json.loads(self.read(name).decode('utf-8-sig'))


def timings(reader, scope, context):
    data = reader.json('build/review/ci-timings.json')
    require(all(data.get(k) == context[k] for k in ('revision', 'run_id', 'run_attempt'))
            and data.get('scope') == scope and data.get('schema_version') == 1, 'timing-identity-mismatch')
    rows = data.get('commands')
    require(isinstance(rows, list) and rows, 'missing-command-timings')
    require(all(row.get('status') == 'passed' and row.get('exit_code') == 0
                and isinstance(row.get('elapsed_ms'), (int, float)) and math.isfinite(row['elapsed_ms'])
                and row['elapsed_ms'] >= 0 for row in rows), 'failed-or-incomplete-command')
    phases = {row['phase'] for row in rows}
    expected = {'foundation': {'rust-contracts', 'native-worker', 'rust-tests', 'codec-interop', 'archive-transfers', 'native-bridge'},
                'qt': {'prepare', 'source-contracts', 'sdk-correspondence', 'settings-compat', 'native-build',
                       'native-integration', 'component-tests', 'browser-repeat', 'resource-metrics', 'visual-default',
                       'codec-interop', 'visual-density', 'runtime-package', 'deployed-runtime', 'manual-template'}}[scope]
    if scope == 'qt':
        expected |= {'attribution'} if context['platform'] == 'macOS' else {'historical-fixture', 'installer-package', 'installer-lifecycle'}
    require(expected <= phases, 'missing-validation-phase')
    return {'commands': len(rows), 'phases': sorted(phases)}


def text_totals(reader, name):
    content = reader.read(name).decode('utf-8', errors='replace')
    totals = re.findall(r'Totals:\s+(\d+) passed,\s+(\d+) failed,\s+(\d+) skipped', content)
    require(len(totals) == 1 and int(totals[0][0]) > 0 and int(totals[0][1]) == 0
            and 'Finished testing' in content, 'incomplete-or-failed-qt-log')
    return {'passed': int(totals[0][0]), 'skipped': int(totals[0][2])}


def browser_evidence(reader, revision):
    directory = reader.root / 'build/review/browser-timings'
    manifests = sorted(directory.glob('browser-suite-*.json'))
    require(len(manifests) == 11, 'missing-initial-or-repeated-suite')
    all_functions, skips, workers = None, 0, 0
    for path in manifests:
        manifest = reader.json(path.relative_to(reader.root).as_posix())
        suite_id = str(uuid.UUID(manifest['suite_id']))
        require(path.name == 'browser-suite-' + suite_id + '.json' and manifest.get('status') == 'passed',
                'incomplete-browser-suite')
        functions = []
        require(manifest.get('shards'), 'missing-browser-shards')
        for index, shard in enumerate(manifest['shards']):
            workers += 1
            require(shard.get('index') == index and shard.get('status') == 'passed' and shard.get('exit_code') == 0
                    and shard.get('missing_or_duplicate') == [] and shard.get('extra') == [], 'failed-browser-shard')
            name = basename(shard['timing_directory'])
            require(name == 'browser-suite-' + suite_id + '-shard-' + str(index), 'browser-shard-identity-mismatch')
            process_id = shard.get('process_id')
            require(type(process_id) is int and process_id > 0, 'missing-browser-worker-identity')
            # Restart/helper children inherit the timing directory. Their outcome
            # is checked by the parent test; only the launched worker is this shard.
            records = list((directory / name).glob('browser-' + str(process_id) + '-*.json'))
            require(len(records) == 1, 'missing-or-ambiguous-browser-record')
            timing = reader.json(records[0].relative_to(reader.root).as_posix())
            require(timing.get('status') == 'passed' and timing.get('exit_code') == 0
                    and timing.get('revision') == revision[:12] and timing.get('dirty') is False
                    and timing.get('process_id') == process_id,
                    'stale-or-incomplete-browser-record')
            log = basename(timing['log'])
            require(log == records[0].with_suffix('.log').name, 'browser-log-identity-mismatch')
            xml = ET.fromstring(reader.read((directory / name / log).relative_to(reader.root).as_posix()))
            require(xml.tag == 'TestCase' and xml.get('name') == 'BrowserTest', 'invalid-browser-raw-log')
            names = [row.get('name') for row in xml.findall('TestFunction')]
            expected = shard['functions'] + ['initTestCase', 'cleanupTestCase']
            require(Counter(names) == Counter(expected), 'missing-or-duplicate-browser-function')
            incidents = xml.findall('.//Incident')
            require(incidents and all(row.get('type') in ('pass', 'skip', 'xfail') for row in incidents),
                    'browser-raw-log-failed')
            skips += sum(row.get('type') == 'skip' for row in incidents)
            functions.extend(shard['functions'])
        require(len(functions) == manifest.get('functions') and len(set(functions)) == len(functions),
                'browser-suite-coverage-mismatch')
        if all_functions is None:
            all_functions = set(functions)
        require(set(functions) == all_functions, 'repeated-suite-coverage-mismatch')
    return {'suites': len(manifests), 'workers': workers, 'functions_per_suite': len(all_functions), 'recorded_platform_skips': skips}


def collect(scope, root, context, role=None):
    role = role or {'docs': 'docs', 'foundation': 'bridge', 'qt': 'qt'}[scope]
    require(role in {'docs': ['docs'], 'foundation': ['bridge', 'rar7'], 'qt': ['qt']}[scope],
            'invalid-evidence-role')
    reader = Reader(root)
    facts = {}
    if role in ('bridge', 'qt'):
        facts['timings'] = timings(reader, scope, context)
    if role == 'bridge':
        log = reader.read('build/review/foundation-ctest.log').decode('utf-8', errors='replace')
        require('Test Passed.' in log and 'Test Failed.' not in log, 'missing-or-failed-foundation-log')
        reader.json('build/review/legacy-name-rewrite.json')
        records = list((root / 'build/review/independent-rust-codecs').rglob('*.json'))
        require(len(records) >= 6, 'missing-codec-records')
        for path in records:
            reader.json(path.relative_to(root).as_posix())
    elif role == 'rar7':
        record = reader.json('build/review/independent-rust-codecs/rar7-official-reference/report.json')
        require(record.get('status') == 'passed' and record.get('explicit_reference_passed') == 1,
                'missing-or-failed-rar7-reference')
        facts['explicit_reference_passed'] = 1
    elif role == 'qt':
        build = reader.json('build/review/build.json')
        platform = {'Windows': 'windows', 'Linux': 'linux', 'macOS': 'mac'}[context['platform']]
        require(build.get('commit') == context['revision'] and build.get('platform') == platform
                and build.get('architecture', '').upper() == context['architecture']
                and build.get('packaged') is (context['platform'] != 'macOS')
                and build.get('dependencyAuditMode') == ('sdk' if platform == 'mac' else 'deployed'), 'build-identity-mismatch')
        resources = reader.json('build/review/runtime-resources.json')
        require(resources.get('status') == 'source-runtime-resource-bytes-matched'
                and resources['identity']['revision'] == context['revision'][:12]
                and resources['identity']['dirty'] is False and resources.get('files'), 'runtime-resource-identity-mismatch')
        facts['qml'] = [text_totals(reader, 'build/review/diagnostics/qml-' + scale + '.log')
                        for scale in ('initial', 'requested-scale-1.25', 'requested-scale-2')]
        facts['browser'] = browser_evidence(reader, context['revision'])
        session = reader.json('build/review/native-sessions.json')
        tree = ast.parse((root / 'scripts/check-native-sessions.py').read_text(encoding='utf-8'))
        cases = next(ast.literal_eval(row.value) for row in tree.body if isinstance(row, ast.Assign)
                     and any(isinstance(t, ast.Name) and t.id == 'CASES' for t in row.targets))
        require(session.get('status') == 'passed' and session.get('timed_out') is False
                and session.get('cases') == cases
                and session.get('native_plugin') == {'Windows': 'windows', 'Linux': 'xcb', 'macOS': 'cocoa'}[context['platform']],
                'missing-or-failed-native-sessions')
        native_log = reader.read('build/review/native-sessions.log').decode('utf-8', errors='replace')
        require(all('BrowserTest::' + case + '(' in native_log for case in cases), 'missing-native-session-case-log')
        facts['native_sessions'] = text_totals(reader, 'build/review/native-sessions.log')
    return dict(schema_version=1, scope=scope, role=role, **context, status='passed', facts=facts,
                files=sorted(reader.files.values(), key=lambda row: row['file']))


def expected_producers(scope):
    if scope == 'foundation':
        return [('bridge', *host) for host in HOSTS] + [('rar7', 'Linux', 'X64')]
    if scope == 'qt':
        return [('qt', *host) for host in HOSTS]
    return [(scope, 'Linux', 'X64')]


def evidence_inventory(record):
    role = record['role']
    files = record.get('files')
    require(isinstance(files, list), 'missing-platform-logs')
    names = []
    for file in files:
        name = file['file']
        require(isinstance(name, str) and not PurePosixPath(name).is_absolute() and '..' not in name.split('/')
                and '\\' not in name and ':' not in name and name.startswith(('build/', 'dist/'))
                and type(file.get('bytes')) is int and file['bytes'] > 0
                and re.fullmatch('[a-f0-9]{64}', file.get('sha256', '')), 'invalid-platform-inventory')
        names.append(name)
    require(len(set(names)) == len(names), 'duplicate-platform-inventory')
    required = {'docs': set(), 'rar7': {'build/review/independent-rust-codecs/rar7-official-reference/report.json'},
                'bridge': {'build/review/ci-timings.json', 'build/review/foundation-ctest.log',
                           'build/review/legacy-name-rewrite.json'},
                'qt': {'build/review/ci-timings.json', 'build/review/build.json', 'build/review/runtime-resources.json',
                       'build/review/native-sessions.json', 'build/review/native-sessions.log',
                       *('build/review/diagnostics/qml-' + scale + '.log'
                         for scale in ('initial', 'requested-scale-1.25', 'requested-scale-2'))}}
    require(role in required and required[role] <= set(names), 'missing-platform-logs')
    if role == 'qt':
        browser = record['facts']['browser']
        require(browser.get('suites') == 11 and browser.get('functions_per_suite', 0) > 0
                and browser.get('workers', 0) >= 11
                and sum(n.startswith('build/review/browser-timings/') and n.endswith('.log') for n in names) == browser['workers'],
                'missing-browser-raw-logs')


def attempt(value):
    require(isinstance(value, str) and value.isdigit() and 1 <= int(value) <= 100000, 'invalid-run-attempt')
    return int(value)


def same_run(record, context):
    return (all(record.get(k) == context[k] for k in ('revision', 'run_id'))
            and attempt(record.get('run_attempt')) <= attempt(context['run_attempt']))


def gate(scope, needs, plan, records, context, config):
    require(set(needs) == set(JOBS[scope]), 'missing-or-unexpected-job')
    require(plan.get('schema_version') == 1 and plan.get('scope') == scope
            and same_run(plan, context)
            and plan.get('mode') == config['mode'] and plan.get('classification') in ('plain-docs', 'full'),
            'missing-or-stale-plan')
    skip = (config['mode'] == 'enabled' and plan['classification'] == 'plain-docs' and plan.get('run_heavy') is False)
    require(plan.get('run_heavy') is (not skip), 'invalid-plan-skip')
    require(not skip or (plan.get('reason') == 'explicit-navigation-document' and plan.get('changed_files') == 1
                         and isinstance(plan.get('base'), str) and planner.SHA.fullmatch(plan['base'])), 'unverified-plain-docs')
    for job in JOBS[scope]:
        expected = 'skipped' if skip and job in HEAVY[scope] else 'success'
        require(needs[job].get('result') == expected, 'failed-cancelled-timed-out-or-missing-job')
    expected = [] if skip and scope != 'docs' else expected_producers(scope)
    selected, seen = {}, set()
    for record in records:
        require(record.get('schema_version') == 1 and record.get('scope') == scope and record.get('status') == 'passed'
                and same_run(record, context), 'stale-or-incomplete-platform-evidence')
        role = record.get('role')
        evidence_inventory(record)
        key = (role, record.get('platform'), record.get('architecture'))
        identity = (*key, attempt(record['run_attempt']))
        require(identity not in seen, 'duplicate-platform-attempt')
        seen.add(identity)
        if key not in selected or attempt(record['run_attempt']) > attempt(selected[key]['run_attempt']):
            selected[key] = record
    actual = list(selected)
    require(Counter(actual) == Counter(expected), 'missing-duplicate-or-unexpected-platform')
    return {'status': 'passed', 'scope': scope, 'mode': config['mode'], 'classification': plan['classification'],
            'heavy_validation': 'not-applicable' if skip else 'executed', 'platform_records': len(selected),
            'evidence_attempts': {'/'.join(key): record['run_attempt'] for key, record in selected.items()},
            **{k: context[k] for k in ('revision', 'run_id', 'run_attempt')}}


def require_clean_source(root):
    require(not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=normal'], cwd=root),
            'validation-requires-clean-source')


def context():
    env = os.environ
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    require(env.get('GITHUB_ACTIONS') == 'true' and env.get('CI') == 'true' and revision == env.get('GITHUB_SHA')
            and planner.SHA.fullmatch(revision) and env.get('GITHUB_RUN_ID', '').isdigit()
            and env.get('GITHUB_RUN_ATTEMPT', '').isdigit(), 'same-run-github-context-required')
    require_clean_source(ROOT)
    return {'revision': revision, 'run_id': env['GITHUB_RUN_ID'], 'run_attempt': env['GITHUB_RUN_ATTEMPT'],
            'platform': env['RUNNER_OS'], 'architecture': env['RUNNER_ARCH']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['collect', 'gate'])
    parser.add_argument('--scope', choices=planner.SCOPES, required=True)
    parser.add_argument('--role')
    parser.add_argument('--input', type=Path, default=ROOT / 'build/ci-gate-input')
    args = parser.parse_args()
    current = context()
    if args.operation == 'collect':
        record = collect(args.scope, ROOT, current, args.role)
        name = '-'.join((args.scope, record['role'], current['platform'], current['architecture'])) + '.json'
        target = ROOT / 'build/ci-evidence' / name
    else:
        directory = args.input.resolve()
        require(directory.is_relative_to(ROOT / 'build'), 'gate-input-must-be-generated')
        plans = [json.loads(p.read_text(encoding='utf-8')) for p in directory.rglob(args.scope + '-plan.json')]
        require(plans and all(same_run(p, current) for p in plans), 'missing-or-stale-plan')
        require(len({attempt(p['run_attempt']) for p in plans}) == len(plans), 'duplicate-plan-attempt')
        plan = max(plans, key=lambda p: attempt(p['run_attempt']))
        records = [json.loads(p.read_text(encoding='utf-8')) for p in directory.rglob('*.json') if not p.name.endswith('-plan.json')]
        record = gate(args.scope, json.loads(os.environ['CI_NEEDS_JSON']), plan, records, current, planner.policy())
        target = ROOT / 'build/ci-gate-results' / (args.scope + '.json')
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'scope': args.scope, 'operation': args.operation, 'status': 'passed'}))


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError, StopIteration, ET.ParseError, subprocess.SubprocessError) as error:
        reason = str(error)
        print(json.dumps({'status': 'failed', 'error_type': type(error).__name__,
                          'reason': reason if re.fullmatch('[a-z][a-z0-9-]+', reason) else 'validation-error'}), file=sys.stderr)
        raise SystemExit(1)
