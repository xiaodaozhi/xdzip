#!/usr/bin/env python3
"""Reject empty, skipped and incomplete runtime integration evidence."""
import importlib.util
import json
import subprocess
import re
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('runtime_worker_tests', ROOT/'scripts/check-runtime-worker-tests.py')
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


class RuntimeInventoryTests(unittest.TestCase):
    def test_private_paths_are_removed_from_json_escaped_diagnostics_too(self):
        private = Path(r'C:\Users\private-test')
        with patch.object(CHECK.Path, 'home', return_value=private):
            text = CHECK.sanitized(str(private) + '\n' + json.dumps(str(private)))
        self.assertNotIn('private-test', text)
        self.assertEqual(text.count('<home>'), 2)

    def test_timeout_fails_and_stops_the_owned_test_process(self):
        parent = ROOT/'build/test-tmp'
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=parent, prefix='runtime-worker-timeout-') as directory:
            with self.assertRaises(subprocess.TimeoutExpired):
                CHECK.execute(Path(sys.executable), ['-c', 'import time; time.sleep(30)'],
                              {'owned::case'}, None, Path(directory)/'timeout.log', .2)

    def workflow(self, skip_rust=False):
        workflow_type = CHECK.load_module('desktop.py').Workflow
        workflow = workflow_type.__new__(workflow_type)
        workflow.system = 'windows'
        workflow.args = SimpleNamespace(skip_rust=skip_rust)
        workflow.env = {}
        workflow.app = ROOT/'build/owned/app/xdzip.exe'
        workflow.output = ROOT/'build/owned/evidence'
        workflow.build = ROOT/'build/owned'
        workflow.qt = ROOT/'build/toolchain/Qt/owned'
        workflow.doctor = Mock()
        workflow.build_receipt = Mock(return_value={'source':'owned-source', 'product_sha256':'owned-sha'})
        workflow.run = Mock()
        workflow.script = Mock()
        return workflow

    def test_desktop_uses_deployed_worker_and_reports_skipped_rust_honestly(self):
        workflow = self.workflow()
        self.assertEqual(workflow.test()['runtime_worker_inventory'], 'passed')
        call = next(call for call in workflow.script.call_args_list if call.args[0] == 'check-runtime-worker-tests.py')
        self.assertEqual(call.args[1:3], ('--worker', str(workflow.app.parent/'xdzip-native-codec.exe')))
        self.assertEqual(call.kwargs['env']['XDZIP_NATIVE_CODEC'], call.args[2])
        self.assertNotIn('--case', call.args)
        self.assertNotIn('--profile', call.args)
        skipped = self.workflow(skip_rust=True)
        self.assertEqual(skipped.test()['runtime_worker_inventory'], 'not-run')
        self.assertFalse(any(call.args[0] == 'check-runtime-worker-tests.py' for call in skipped.script.call_args_list))

    def test_worker_inventory_failure_stops_the_desktop_test_stage(self):
        workflow = self.workflow()
        def script(name, *_args, **_kwargs):
            if name == 'check-runtime-worker-tests.py':
                raise ValueError('integration failure')
        workflow.script.side_effect = script
        with self.assertRaises(ValueError):
            workflow.test()
        self.assertFalse(any(call.args[0] == 'cli.py' for call in workflow.script.call_args_list))

    def test_prepared_flat_rar_cache_is_revalidated_before_offline_reuse(self):
        module = CHECK.load_module('prepare-rar-rust-fixtures.py')
        parent = ROOT/'build/test-tmp'
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=parent, prefix='runtime-rar-cache-') as directory:
            root = Path(directory)
            cache = root/'cache'
            cache.mkdir()
            data = b'fixed test archive'
            item = {'file':'owned.rar', 'bytes':len(data), 'sha256':module.digest(data), 'cache_subdirectory':'upstream'}
            (cache/'owned.rar').write_bytes(data)
            prepared, downloaded = module.public_data(item, root/'prepared', cache, offline=True)
            self.assertEqual((prepared, downloaded), (data, 0))
            (cache/'owned.rar').write_bytes(b'X' + data[1:])
            with self.assertRaises(ValueError):
                module.public_data(item, root/'other', cache, offline=True)
            self.assertFalse((root/'other/owned.rar').exists())

    def test_inventory_matches_every_current_ignored_function(self):
        declared = CHECK.inventory()
        actual = set()
        for path in (ROOT/'crates/xdzip-runtime/src').rglob('*.rs'):
            module = '::'.join(path.relative_to(ROOT/'crates/xdzip-runtime/src').with_suffix('').parts)
            for name in re.findall(r'#\[ignore[^\n]*\]\s*fn\s+(\w+)', path.read_text(encoding='utf-8')):
                actual.add(module+'::'+name)
        self.assertEqual(actual, {case['name'] for case in declared})
        self.assertEqual(len(CHECK.select(declared, 'worker', [])), 27)
        self.assertEqual(len(CHECK.select(declared, 'large', [])), 4)
        self.assertEqual(len(CHECK.select(declared, None, [])), 31)

    def test_explicit_retry_cannot_select_nothing_or_duplicate_a_case(self):
        cases = CHECK.inventory()
        name = cases[0]['name']
        self.assertEqual(CHECK.select(cases, None, [name]), [cases[0]])
        for profile, names in [(None, ['missing::case']), (None, [name, name]), ('worker', [name])]:
            with self.assertRaises(ValueError):
                CHECK.select(cases, profile, names)

    def test_result_requires_the_exact_unique_passed_set_and_no_ignored(self):
        valid = 'test owned::case ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out;\n'
        CHECK.validate_result(valid, {'owned::case'})
        for invalid in [
            'test result: ok. 0 passed; 0 failed; 0 ignored;\n',
            valid.replace('1 passed', '0 passed').replace('0 ignored', '1 ignored'),
            valid.replace('owned::case', 'wrong::case'),
            valid.replace('... ok', '... ignored'),
            valid.replace('test result: ok.', 'test result: FAILED.'),
            'test owned::case ... ok\n' + valid,
            valid + valid,
        ]:
            with self.subTest(log=invalid), self.assertRaises(ValueError):
                CHECK.validate_result(invalid, {'owned::case'})

    def test_successful_subprocess_with_zero_tests_is_not_evidence(self):
        parent = ROOT/'build/test-tmp'
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=parent, prefix='runtime-worker-runner-') as directory:
            log = Path(directory)/'zero.log'
            with self.assertRaises(ValueError):
                CHECK.execute(Path(sys.executable), ['-c', 'print("test result: ok. 0 passed; 0 failed; 0 ignored;")'],
                              {'owned::case'}, None, log, 10)


if __name__ == '__main__':
    unittest.main()
