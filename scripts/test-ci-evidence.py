#!/usr/bin/env python3
"""Prove failure/cancellation and missing platform, phase or raw logs block gates."""
import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
import uuid

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('ci_evidence', ROOT / 'scripts/ci-evidence.py')
ci = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ci)
SOURCE = 'a' * 40
QT_PHASES = ['prepare', 'source-contracts', 'sdk-correspondence', 'settings-compat', 'native-build',
             'native-integration', 'component-tests', 'browser-repeat', 'resource-metrics', 'visual-default',
             'codec-interop', 'visual-density', 'runtime-package', 'deployed-runtime', 'manual-template',
             'historical-fixture', 'installer-package', 'installer-lifecycle']


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        parent = ROOT / 'build/test-ci-evidence';parent.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=parent);self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.assertTrue(self.root.resolve().is_relative_to(parent.resolve()))
        self.context = {'revision': SOURCE, 'run_id': '123', 'run_attempt': '1', 'platform': 'Windows', 'architecture': 'X64'}
        self.config = ci.planner.policy()
        self.plan = dict(schema_version=1, scope='qt', mode='shadow', classification='full', reason='owned-fixture',
                         run_heavy=True, changed_files=1, base='b' * 40,
                         **{k: self.context[k] for k in ('revision', 'run_id', 'run_attempt')})
        self.needs = {job: {'result': 'success'} for job in ci.JOBS['qt']}
        self.write('scripts/check-native-sessions.py', "CASES = ['ownedNative']\n")
        self.write_json('build/review/ci-timings.json', dict(schema_version=1, scope='qt',
            **{k: self.context[k] for k in ('revision', 'run_id', 'run_attempt')},
            commands=[dict(phase=phase, command='owned.py', status='passed', exit_code=0, elapsed_ms=1) for phase in QT_PHASES]))
        self.write_json('build/review/build.json', dict(commit=SOURCE, platform='windows', architecture='X64', packaged=True, dependencyAuditMode='deployed'))
        self.write_json('build/review/runtime-resources.json', dict(status='source-runtime-resource-bytes-matched',
                        identity=dict(revision=SOURCE[:12], dirty=False), files=[{'owned': 'resource'}]))
        self.write_json('build/review/native-sessions.json', dict(status='passed', timed_out=False, cases=['ownedNative'], native_plugin='windows'))
        log = 'PASS : Owned::owned()\nTotals: 1 passed, 0 failed, 0 skipped, 0 blacklisted, 1ms\nFinished testing\n'
        for name in ['native-sessions.log', 'diagnostics/qml-initial.log',
                     'diagnostics/qml-requested-scale-1.25.log', 'diagnostics/qml-requested-scale-2.log']:
            self.write('build/review/' + name, log)
        self.write('build/review/native-sessions.log', log.replace('Owned::owned()', 'BrowserTest::ownedNative()'))
        for _ in range(11):
            suite = str(uuid.uuid4());stem = 'build/review/browser-timings/browser-suite-' + suite
            shard = 'browser-suite-' + suite + '-shard-0'
            timing = stem + '-shard-0/browser-owned.json'
            self.write_json(timing, dict(status='passed', exit_code=0, revision=SOURCE[:12], dirty=False, log='browser-owned.log'))
            xml = '<TestCase name="BrowserTest">' + ''.join('<TestFunction name="' + n + '"><Incident type="pass"/></TestFunction>'
                                                          for n in ['initTestCase', 'ownedAlpha', 'ownedBeta', 'cleanupTestCase']) + '</TestCase>'
            self.write(stem + '-shard-0/browser-owned.log', xml)
            self.write_json(stem + '.json', dict(schema_version=1, suite_id=suite, status='passed', functions=2,
                            shards=[dict(index=0, functions=['ownedAlpha', 'ownedBeta'], timing_directory=shard,
                                         status='passed', exit_code=0, missing_or_duplicate=[], extra=[])]))

    def write(self, name, text):
        path = self.root / name;path.parent.mkdir(parents=True, exist_ok=True);path.write_text(text, encoding='utf-8')

    def write_json(self, name, value):
        self.write(name, json.dumps(value))

    def qt(self):
        return ci.collect('qt', self.root, self.context)

    def records(self):
        record = self.qt()
        return [dict(copy.deepcopy(record), platform=host, architecture=arch) for host, arch in ci.HOSTS]

    def gate(self, needs=None, plan=None, records=None, config=None):
        return ci.gate('qt', needs or self.needs, plan or self.plan, self.records() if records is None else records,
                       self.context, config or self.config)

    def test_complete_native_evidence_and_all_platforms_pass(self):
        record = self.qt()
        self.assertEqual(record['facts']['browser']['suites'], 11)
        self.assertEqual(record['facts']['browser']['functions_per_suite'], 2)
        self.assertEqual(self.gate()['heavy_validation'], 'executed')

    def test_partial_rerun_preserves_successful_same_source_platforms_and_actual_attempts(self):
        records = self.records()
        records.append(dict(copy.deepcopy(records[1]), run_attempt='2'))
        context = dict(self.context, run_attempt='2')
        result = ci.gate('qt', self.needs, self.plan, records, context, self.config)
        self.assertEqual(result['platform_records'], 3)
        self.assertEqual(result['evidence_attempts'], {'qt/Linux/X64': '1', 'qt/Windows/X64': '2', 'qt/macOS/ARM64': '1'})
        needs = copy.deepcopy(self.needs);needs['native-qt']['result'] = 'failure'
        with self.assertRaises(ValueError):ci.gate('qt', needs, self.plan, records, context, self.config)

    def test_failure_cancellation_timeout_missing_and_unexpected_skip_never_pass(self):
        records = self.records()
        for job in self.needs:
            for outcome in ['failure', 'cancelled', 'timed_out', 'skipped', None]:
                needs = copy.deepcopy(self.needs);needs[job]['result'] = outcome
                with self.subTest(job=job, outcome=outcome), self.assertRaises(ValueError):self.gate(needs=needs, records=records)
        with self.assertRaises(ValueError):self.gate(needs={'qt-plan': {'result': 'success'}}, records=records)

    def test_missing_duplicate_or_stale_platform_evidence_cannot_be_green(self):
        records = self.records()
        for changed in [records[:-1], records + [records[0]], []]:
            with self.assertRaises(ValueError):self.gate(records=changed)
        for key, value in [('revision', 'b' * 40), ('run_id', '122'), ('run_attempt', '2'), ('status', 'running')]:
            changed = copy.deepcopy(records);changed[0][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):self.gate(records=changed)

    def test_raw_log_absence_is_rejected_by_producer_and_summary(self):
        records = self.records()
        for required in ['build/review/native-sessions.log', 'build/review/diagnostics/qml-initial.log']:
            changed = copy.deepcopy(records)
            changed[0]['files'] = [f for f in changed[0]['files'] if f['file'] != required]
            with self.assertRaisesRegex(ValueError, 'missing-platform-logs'):self.gate(records=changed)
        log = next((self.root / 'build/review/browser-timings').rglob('*.log'));log.unlink()
        with self.assertRaisesRegex(ValueError, 'missing-raw-evidence'):self.qt()

    def test_shadow_never_allows_a_plain_document_skip(self):
        plan = dict(self.plan, classification='plain-docs', reason='explicit-navigation-document', run_heavy=False)
        needs = copy.deepcopy(self.needs);needs['native-qt']['result'] = 'skipped'
        with self.assertRaises(ValueError):self.gate(needs=needs, plan=plan, records=[])
        config = dict(self.config, mode='enabled');plan['mode'] = 'enabled'
        self.assertEqual(self.gate(needs=needs, plan=plan, records=[], config=config)['heavy_validation'], 'not-applicable')
        for edits in [dict(classification='full'), dict(base=None), dict(reason='unknown-path'), dict(changed_files=2)]:
            with self.subTest(edits=edits), self.assertRaises(ValueError):
                self.gate(needs=needs, plan=dict(plan, **edits), records=[], config=config)

    def test_unfinished_or_missing_repeat_and_dirty_binary_are_rejected(self):
        manifests = sorted((self.root / 'build/review/browser-timings').glob('browser-suite-*.json'))
        path = manifests[0];original = path.read_text(encoding='utf-8');data = json.loads(original)
        data['status'] = 'running';path.write_text(json.dumps(data), encoding='utf-8')
        with self.assertRaisesRegex(ValueError, 'incomplete-browser-suite'):self.qt()
        path.write_text(original, encoding='utf-8')
        record = next(path.parent.rglob('browser-owned.json'));data = json.loads(record.read_text(encoding='utf-8'))
        data['dirty'] = True;record.write_text(json.dumps(data), encoding='utf-8')
        with self.assertRaisesRegex(ValueError, 'stale-or-incomplete-browser-record'):self.qt()
        path.unlink()
        with self.assertRaisesRegex(ValueError, 'missing-initial-or-repeated-suite'):self.qt()

    def test_raw_xml_omission_failure_or_wrong_shard_cannot_hide_behind_success_metadata(self):
        log = next((self.root / 'build/review/browser-timings').rglob('*.log'))
        original = log.read_text(encoding='utf-8')
        for changed in [original.replace('ownedBeta', 'unrelated'), original.replace('type="pass"', 'type="fail"')]:
            log.write_text(changed, encoding='utf-8')
            with self.assertRaises(ValueError):self.qt()
        log.write_text(original, encoding='utf-8')
        manifest = next((self.root / 'build/review/browser-timings').glob('*.json'))
        data = json.loads(manifest.read_text(encoding='utf-8'));data['shards'][0]['timing_directory'] = '../outside'
        manifest.write_text(json.dumps(data), encoding='utf-8')
        with self.assertRaisesRegex(ValueError, 'unsafe-evidence-name'):self.qt()

    def test_failed_postcondition_missing_phase_and_wrong_run_timing_block_collection(self):
        path = self.root / 'build/review/ci-timings.json';original = json.loads(path.read_text(encoding='utf-8'))
        for changed in [dict(original, commands=original['commands'][:-1]), dict(original, run_attempt='2'),
                        dict(original, commands=[dict(original['commands'][0], status='failed')] + original['commands'][1:])]:
            path.write_text(json.dumps(changed), encoding='utf-8')
            with self.assertRaises(ValueError):self.qt()

    def test_failed_native_session_or_qml_raw_result_is_rejected(self):
        self.write_json('build/review/native-sessions.json', dict(status='passed', timed_out=True, cases=['ownedNative'], native_plugin='windows'))
        with self.assertRaisesRegex(ValueError, 'missing-or-failed-native-sessions'):self.qt()
        self.write('build/review/diagnostics/qml-initial.log', 'Totals: 4 passed, 1 failed, 0 skipped\nFinished testing\n')
        with self.assertRaisesRegex(ValueError, 'incomplete-or-failed-qt-log'):self.qt()

    def test_isolated_build_tooling_is_clean_but_root_environment_and_source_edits_are_rejected(self):
        self.write('.gitignore', '/build/\n/dist/\n')
        def git(*args):
            subprocess.run(['git', '-c', 'user.name=CI fixture', '-c', 'user.email=ci@example.invalid', *args],
                           cwd=self.root, check=True, capture_output=True)
        git('init', '-q', '-b', 'main');git('add', '.');git('commit', '-q', '-m', 'owned fixture')
        self.write('build/ci-html-tools/owned.py', 'owned CI interpreter fixture')
        ci.require_clean_source(self.root)
        self.write('.venv-design/owned.py', 'owned unexpected root environment')
        with self.assertRaisesRegex(ValueError, 'validation-requires-clean-source'):ci.require_clean_source(self.root)
        (self.root / '.venv-design/owned.py').unlink()
        self.write('scripts/check-native-sessions.py', 'owned changed source')
        with self.assertRaisesRegex(ValueError, 'validation-requires-clean-source'):ci.require_clean_source(self.root)


if __name__ == '__main__':unittest.main()
