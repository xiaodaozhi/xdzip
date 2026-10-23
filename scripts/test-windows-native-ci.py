# scripts/test-windows-native-ci.py

"""Exercise hosted diagnostic admission and owned-child cleanup without GUI."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('windows_native_ci', ROOT / 'scripts/check-windows-native-ci.py')
driver = importlib.util.module_from_spec(spec)
spec.loader.exec_module(driver)
display_spec = importlib.util.spec_from_file_location('display_tool', ROOT / 'scripts/windows_native_ci_display.py')
display = importlib.util.module_from_spec(display_spec)
display_spec.loader.exec_module(display)
SOURCE = '1' * 40
HOSTED = dict(GITHUB_ACTIONS='true', RUNNER_ENVIRONMENT='github-hosted', RUNNER_OS='Windows')


def fake_mode(width, height):
    mode = display.DeviceMode()
    mode.size = 220
    mode.width, mode.height, mode.bits_per_pixel, mode.frequency = width, height, 32, 60
    return bytes(mode)


class FakeDisplay:
    def __init__(self):
        self.before, self.target = fake_mode(1024, 768), fake_mode(1920, 1080)
        self.raw = self.before
        self.applied = []
        self.test_code = 0
        self.fail_read_after_apply = False
        self.pending_failed_read = False

    def conditions(self):
        return dict(session_id=1, input_desktop_accessible=True, monitor_count=1)

    def current_raw(self):
        if self.pending_failed_read:
            self.pending_failed_read = False
            raise OSError('synthetic first readback failure')
        return self.raw

    def enumerated_raw(self):
        return [self.before, fake_mode(1280, 1024), self.target]

    def test(self, raw):
        return self.test_code

    def apply(self, raw):
        self.applied.append(raw)
        self.raw = raw
        self.pending_failed_read = self.fail_read_after_apply and len(self.applied) == 1
        return 0


class DisplayReceiptTests(unittest.TestCase):
    def setUp(self):
        (ROOT / 'build').mkdir(exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix='display-mock-', dir=ROOT / 'build')
        self.addCleanup(temporary.cleanup)
        self.folder = Path(temporary.name) / 'receipt'
        self.api = FakeDisplay()
        self.binding = dict(full_source_sha=SOURCE, github=dict(run_id='123', run_attempt='1', job='native'),
                            helper_sha256=display.digest(display.HELPER.read_bytes()), session_id=1)
        self.binding_mock = mock.patch.object(display, '_binding', return_value=self.binding)
        self.binding_mock.start()
        self.addCleanup(self.binding_mock.stop)

    def test_enumerated_mode_apply_readback_restore_and_immutable_active_receipt(self):
        record = display.prepare(self.folder, SOURCE, api=self.api)
        self.assertEqual(record['status'], 'active')
        self.assertEqual(record['active']['width'], 1920)
        file = self.folder / 'receipt.json'
        original = file.read_bytes()
        first = display.validate_active_receipt(file, SOURCE, api=self.api)
        self.assertEqual(first, display.validate_active_receipt(file, SOURCE, api=self.api))
        restored = display.restore(self.folder, SOURCE, api=self.api)
        self.assertTrue(restored['restored'])
        self.assertEqual(self.api.raw, self.api.before)
        self.assertEqual(file.read_bytes(), original)
        with self.assertRaises(display.DisplayBlocked):
            display.validate_active_receipt(file, SOURCE, api=self.api)

    def test_wrong_run_source_or_live_readback_is_rejected(self):
        display.prepare(self.folder, SOURCE, api=self.api)
        file = self.folder / 'receipt.json'
        original = file.read_text()
        for key, value in [('full_source_sha', '2' * 40), ('github', dict(run_id='124', run_attempt='1', job='native'))]:
            with self.subTest(key=key):
                record = json.loads(original)
                record[key] = value
                file.write_text(json.dumps(record), encoding='utf-8')
                with self.assertRaises(display.DisplayBlocked):
                    display.validate_active_receipt(file, SOURCE, api=self.api)
        file.write_text(original, encoding='utf-8')
        self.api.raw = fake_mode(1600, 1200)
        with self.assertRaises(display.DisplayBlocked):
            display.validate_active_receipt(file, SOURCE, api=self.api)
        with self.assertRaises(display.DisplayBlocked):
            display.restore(self.folder, SOURCE, api=self.api)
        self.assertEqual(len(self.api.applied), 1)

    def test_failed_cds_test_has_no_apply_and_remains_restorable(self):
        self.api.test_code = -2
        record = display.prepare(self.folder, SOURCE, api=self.api)
        self.assertEqual(record['status'], 'blocked')
        self.assertEqual(self.api.applied, [])
        self.assertTrue(display.restore(self.folder, SOURCE, api=self.api)['restored'])
        with self.assertRaises(display.DisplayBlocked):
            display.select_mode([self.api.before], self.api.before)

    def test_successful_apply_then_failed_readback_still_restores_only_its_requested_mode(self):
        self.api.fail_read_after_apply = True
        record = display.prepare(self.folder, SOURCE, api=self.api)
        self.assertEqual(record['status'], 'blocked')
        self.assertNotIn('active', record)
        self.assertEqual(self.api.raw, self.api.target)
        self.assertTrue(display.restore(self.folder, SOURCE, api=self.api)['restored'])
        self.assertEqual(self.api.raw, self.api.before)

    def test_binding_requires_actual_hosted_run_clean_source_and_committed_helper(self):
        self.binding_mock.stop()
        env = dict(HOSTED, GITHUB_RUN_ID='123', GITHUB_RUN_ATTEMPT='1', GITHUB_JOB='native')
        with mock.patch.object(display.sys, 'platform', 'win32'):
            with mock.patch.object(display.subprocess, 'check_output', side_effect=[SOURCE.encode(), b'', b'blob', b'blob']):
                result = display._binding(SOURCE, env, self.api)
                self.assertEqual(result['github'], dict(run_id='123', run_attempt='1', job='native'))
            for values in ([b'2' * 40], [SOURCE.encode(), b' M source.py'], [SOURCE.encode(), b'', b'blob', b'changed']):
                with self.subTest(values=values), mock.patch.object(display.subprocess, 'check_output', side_effect=values):
                    with self.assertRaises(display.DisplayBlocked):
                        display._binding(SOURCE, env, self.api)
            with mock.patch.object(display.subprocess, 'check_output') as git:
                with self.assertRaises(display.DisplayBlocked):
                    display._binding(SOURCE, dict(env, GITHUB_RUN_ID='unbound'), self.api)
                git.assert_not_called()

    def test_unusable_enumerated_depth_is_skipped_without_weakening_selected_mode(self):
        bad = display.DeviceMode.from_buffer_copy(self.api.before)
        bad.bits_per_pixel = 16
        selected = display.select_mode([bytes(bad), self.api.target], self.api.before)
        self.assertEqual(selected, self.api.target)

    def test_display_helper_self_test_is_synthetic_and_never_calls_native_display(self):
        result = display.self_test()
        self.assertEqual(result['status'], 'self_test_passed')
        self.assertEqual(result['native_display_calls'], 0)
        self.assertGreaterEqual(result['checks'], 23)


class HostedMenuTests(unittest.TestCase):
    def setUp(self):
        (ROOT / 'build').mkdir(exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix='hosted-menu-unit-', dir=ROOT / 'build')
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.tests = self.directory / 'owned-not-executable.exe'
        self.tests.write_bytes(b'not an executable, no process is launched by these tests')
        self.binary = driver.digest(self.tests)
        self.snapshot = dict(full_source_sha=SOURCE, clean=True, binary_sha256=self.binary,
                             build_version_sha256='3' * 64)
        self.contract = driver.menus.load_contract()

    def sample(self, directory, case='toolbar-more', **changes):
        directory.mkdir(parents=True, exist_ok=True)
        record = dict(menu=case, revision=SOURCE[:12], trigger_to_complete_frame_ms=25,
                      trigger_to_capture_complete_ms=35, shadow_same_frame=True,
                      first_visible_menu_sync_generation=2, **self.contract['windows'],
                      **{field: 2 for field in self.contract['required_generation_fields']})
        record.update(changes)
        diagnostic = dict(captured_owned_pixels_accepted=True, native_capture_failed=False,
                          native_capture_error=0, occlusion_rejections=0, first_frame_superseded=False,
                          first_menu_face_visible=True, **record)
        (directory / 'capture-diagnostic.json').write_text(json.dumps(diagnostic), encoding='utf-8')
        (directory / (case + '.json')).write_text(json.dumps(record), encoding='utf-8')
        return record, diagnostic

    def guard(self, directory, **kwargs):
        return driver.guard_sample(directory, 'toolbar-more', subprocess.CompletedProcess([], 0),
                                   dict(timed_out=False), SOURCE, self.contract, **kwargs)

    def test_source_snapshot_binds_clean_full_source_binary_and_release_metadata(self):
        generated = self.directory / 'generated'
        generated.mkdir()
        header = generated / 'BuildVersion.h'
        header.write_text('#define XDZIP_REVISION "' + SOURCE[:12] + '"\n'
                          '#define XDZIP_DIRTY false\n#define XDZIP_CONFIGURATION "Release"\n', encoding='utf-8')
        with mock.patch.object(driver.subprocess, 'check_output', side_effect=[SOURCE.encode(), b'']):
            self.assertEqual(driver.source_snapshot(self.tests, SOURCE, self.binary)['full_source_sha'], SOURCE)
        for values in ([b'2' * 40, b''], [SOURCE.encode(), b' M changed.py']):
            with self.subTest(values=values), mock.patch.object(driver.subprocess, 'check_output', side_effect=values):
                with self.assertRaises(driver.Blocked):
                    driver.source_snapshot(self.tests, SOURCE, self.binary)
        header.write_text('#define XDZIP_CONFIGURATION "Debug"', encoding='utf-8')
        with mock.patch.object(driver.subprocess, 'check_output', side_effect=[SOURCE.encode(), b'']):
            with self.assertRaises(driver.Blocked):
                driver.source_snapshot(self.tests, SOURCE, self.binary)

    def test_environment_preserves_private_temp_and_path_without_dpi_overrides_or_tokens(self):
        inherited = dict(PATH='existing-tools', SystemRoot='Windows', TEMP='private-temp', TMP='private-temp',
                         QT_SCALE_FACTOR='2', QT_QUICK_BACKEND='software', GITHUB_TOKEN='never-in-child',
                         XDZIP_NATIVE_CI_DISPLAY_RECEIPT='controlled-receipt.json')
        with mock.patch.dict(os.environ, inherited, clear=True):
            result = driver.native_environment(self.directory)
        normalized = {key.upper(): value for key, value in result.items()}
        for key in ('PATH', 'SystemRoot', 'TEMP', 'TMP'):
            self.assertEqual(normalized[key.upper()], inherited[key])
        for key in ('QT_SCALE_FACTOR', 'QT_QUICK_BACKEND', 'GITHUB_TOKEN'):
            self.assertNotIn(key, result)
        self.assertEqual(result['QT_QPA_PLATFORM'], 'windows')
        self.assertEqual(result['QSG_RHI_BACKEND'], 'd3d11')
        self.assertEqual(result['XDZIP_NATIVE_CI_DISPLAY_RECEIPT'], 'controlled-receipt.json')

    def test_unverified_environment_boolean_cannot_claim_display_mode_change(self):
        with mock.patch.dict(os.environ, {'XDZIP_VIRTUAL_DISPLAY_MODE_CHANGED': 'true'}, clear=True):
            state = driver.display_state(SOURCE)
        self.assertFalse(state['virtual_display_mode_changed'])
        self.assertFalse(state['os_display_settings_changed'])
        self.assertFalse(state['managed_display_receipt'])

    def test_display_claim_uses_controlled_receipt_validator_and_live_mode_comparison(self):
        mode = dict(width=1024, height=768, bits_per_pixel=32, frequency=60,
                    orientation=0, position_x=0, position_y=0, display_flags=0)
        receipt = dict(before=mode, active=dict(mode, width=1920, height=1080),
                       full_source_sha=SOURCE, receipt_path='build/controlled/receipt.json', receipt_sha256='a' * 64)
        helper = mock.Mock()
        helper.validate_active_receipt.return_value = receipt
        specification = mock.Mock()
        with mock.patch.dict(os.environ, {'XDZIP_NATIVE_CI_DISPLAY_RECEIPT': 'build/controlled/receipt.json'}), \
                mock.patch.object(driver.importlib.util, 'spec_from_file_location', return_value=specification), \
                mock.patch.object(driver.importlib.util, 'module_from_spec', return_value=helper):
            state = driver.display_state(SOURCE)
            self.assertTrue(state['virtual_display_mode_changed'])
            self.assertTrue(state['os_display_settings_changed'])
            self.assertEqual(state['display_mode_receipt']['receipt_sha256'], 'a' * 64)
            self.assertEqual(helper.validate_active_receipt.call_args.args[1], SOURCE)
            # An already-target-sized runner has no actual mode change.
            receipt['before'] = receipt['active']
            state = driver.display_state(SOURCE)
            self.assertFalse(state['virtual_display_mode_changed'])
            helper.validate_active_receipt.side_effect = ValueError('wrong_source_run_or_current_mode')
            with self.assertRaises(driver.Blocked):
                driver.display_state(SOURCE)

    def test_display_mode_change_during_child_blocks_even_if_source_and_binary_match(self):
        first = dict(virtual_display_mode_changed=True, os_display_settings_changed=True,
                     managed_display_receipt=True, display_mode_receipt={'receipt_sha256': 'a' * 64})
        with mock.patch.object(driver, 'display_state', side_effect=[first, driver.Blocked('mode_changed')]):
            with self.assertRaises(driver.Blocked):
                self.run_child(self.process(0))
        receipt = json.loads((self.directory / 'process-receipt.json').read_text())
        self.assertFalse(receipt['identity_unchanged'])
        self.assertEqual(receipt['exit_code'], 0)

    def test_rejected_display_receipt_starts_no_qt_and_cannot_claim_os_unchanged(self):
        with mock.patch.object(driver.sys, 'platform', 'win32'), mock.patch.dict(os.environ, HOSTED), \
                mock.patch.object(driver, 'source_snapshot', return_value=self.snapshot), \
                mock.patch.object(driver, 'display_state', side_effect=driver.Blocked('display_receipt_rejected')), \
                mock.patch.object(driver.subprocess, 'Popen') as child:
            code, report = driver.execute(self.tests, self.directory, SOURCE, self.binary)
        self.assertEqual(code, 2)
        self.assertIsNone(report['virtual_display_mode_changed'])
        self.assertIsNone(report['os_display_settings_changed'])
        child.assert_not_called()

    def test_nested_home_and_repository_paths_are_redacted_in_specific_order(self):
        home = Path('/synthetic-home')
        repository = home / 'work' / 'xdzip'
        with mock.patch.object(driver, 'ROOT', repository), mock.patch.object(Path, 'home', return_value=home):
            self.assertEqual(driver.sanitized(str(repository / 'sample') + ' ' + str(home / 'other')),
                             str(Path('<repo>') / 'sample') + ' ' + str(Path('<home>') / 'other'))

    def process(self, exit_code=7, timeout=False):
        child = mock.Mock(pid=4321, returncode=exit_code)
        child.poll.return_value = exit_code
        child.communicate.side_effect = ([subprocess.TimeoutExpired('owned', 35), (b'out', b'err')]
                                         if timeout else [(str(ROOT).encode() + b' out', b'err')])
        return child

    def run_child(self, child, source_values=None, image=True):
        with mock.patch.object(driver.subprocess, 'Popen', return_value=child) as popen, \
                mock.patch.object(driver, 'source_snapshot', side_effect=source_values or [self.snapshot, self.snapshot]), \
                mock.patch.object(driver, 'owned_image_matches', return_value=image):
            result = driver.run_owned([str(self.tests), 'nativeMenuFirstFrame:toolbar-more'],
                                      {'TEMP': 'private'}, self.directory, self.tests, SOURCE, self.binary)
        return result, popen

    def test_direct_owned_pid_and_raw_exit_survive_and_output_is_sanitized(self):
        (result, receipt), popen = self.run_child(self.process())
        self.assertEqual(receipt['owned_child_pid'], 4321)
        self.assertEqual(receipt['exit_code'], 7)
        self.assertEqual(result.returncode, 7)
        self.assertTrue(receipt['identity_unchanged'])
        self.assertEqual(popen.call_args.args[0][0], str(self.tests))
        self.assertNotIn(str(ROOT).encode(), result.stdout)
        self.assertIn(b'<repo>', result.stdout)
        self.assertEqual(json.loads((self.directory / 'process-receipt.json').read_text())['exit_code'], 7)

    def test_timeout_kills_and_waits_for_exact_child_without_retry(self):
        child = self.process(exit_code=-9, timeout=True)
        (result, receipt), popen = self.run_child(child)
        self.assertTrue(receipt['timed_out'])
        self.assertEqual(result.returncode, -9)
        child.kill.assert_called_once()
        child.wait.assert_called_once_with(timeout=5)
        self.assertEqual(popen.call_count, 1)

    def test_failed_image_identity_kills_joins_and_preserves_failed_receipt(self):
        child = self.process(exit_code=-9)
        child.poll.return_value = None
        with self.assertRaises(driver.Blocked):
            self.run_child(child, image=False)
        child.kill.assert_called_once()
        child.wait.assert_called_once_with(timeout=5)
        receipt = json.loads((self.directory / 'process-receipt.json').read_text())
        self.assertFalse(receipt['binary_path_verified'])
        self.assertEqual(receipt['owned_child_pid'], 4321)
        self.assertTrue((self.directory / 'console.log').is_file())

    def test_binary_change_after_child_is_blocked_with_original_exit_preserved(self):
        with self.assertRaises(driver.Blocked):
            self.run_child(self.process(7), source_values=[self.snapshot, driver.Blocked('changed')])
        receipt = json.loads((self.directory / 'process-receipt.json').read_text())
        self.assertEqual(receipt['exit_code'], 7)
        self.assertFalse(receipt['identity_unchanged'])

    def test_capture_generation_stage_source_or_occlusion_fail_closed(self):
        invalid = dict(capture_end_sync_generation=3, first_visible_menu_sync_generation=0,
                       capture_stage='queued later capture', revision='wrong')
        for field, value in invalid.items():
            with self.subTest(field=field):
                self.sample(self.directory, **{field: value})
                with self.assertRaises(driver.Blocked):
                    self.guard(self.directory)
        self.sample(self.directory)
        path = self.directory / 'capture-diagnostic.json'
        value = json.loads(path.read_text())
        value['occlusion_rejections'] = 1
        path.write_text(json.dumps(value), encoding='utf-8')
        with self.assertRaises(driver.Blocked):
            self.guard(self.directory)

    def test_record_generation_cannot_disagree_with_accepted_diagnostic(self):
        self.sample(self.directory)
        path = self.directory / 'toolbar-more.json'
        record = json.loads(path.read_text())
        record.update({field: 3 for field in [self.contract['reference_generation_field'], *self.contract['required_generation_fields']]})
        path.write_text(json.dumps(record), encoding='utf-8')
        with self.assertRaises(driver.Blocked):
            self.guard(self.directory)

    def test_pilot_invalid_guard_blocks_matrix_and_keeps_raw_failure(self):
        with mock.patch.object(driver.sys, 'platform', 'win32'), mock.patch.dict(os.environ, HOSTED), \
                mock.patch.object(driver, 'source_snapshot', return_value=self.snapshot), \
                mock.patch.object(driver, 'desktop_diagnostics', return_value={}), \
                mock.patch.object(driver, 'automatic_stage', side_effect=driver.Blocked('guard_rejected')) as stage:
            code, report = driver.execute(self.tests, self.directory, SOURCE, self.binary)
        self.assertEqual(code, 2)
        self.assertEqual(stage.call_count, 1)
        self.assertEqual(report['status'], 'blocked')
        self.assertFalse(report['capture_complete'])
        self.assertFalse(report['strict_menu_contract_passed'])

    def test_real_automatic_runner_collects_after_timing_failure_without_global_patch(self):
        calls = []
        global_run = subprocess.run
        original_dependency = driver.menus.subprocess
        def owned(command, env, directory, tests, source, binary, timeout):
            calls.append(command)
            self.assertEqual(env['PATH'], 'inherited-tools')
            self.assertEqual(env['TEMP'], 'protected-temp')
            self.assertNotIn('QT_SCALE_FACTOR', env)
            self.sample(directory, command[1].split(':')[1], trigger_to_complete_frame_ms=201)
            return subprocess.CompletedProcess(command, 1, b'original qtest timing failure', b''), \
                dict(timed_out=False, owned_child_pid=4000 + len(calls), exit_code=1)
        with mock.patch.dict(os.environ, {**HOSTED, 'PATH': 'inherited-tools', 'TEMP': 'protected-temp', 'QT_SCALE_FACTOR': '2'}), \
                mock.patch.object(driver.sys, 'platform', 'win32'), \
                mock.patch.object(driver.menus.platform, 'platform', return_value='hosted Windows'), \
                mock.patch.object(driver.menus.platform, 'processor', return_value='test'), \
                mock.patch.object(driver, 'run_owned', side_effect=owned):
            before = dict(os.environ)
            receipts = []
            report = driver.automatic_stage('pilot', ['toolbar-more'], 1, self.directory / 'pilot',
                                            self.tests, SOURCE, self.binary, self.contract, receipts)
            self.assertEqual(report['status'], 'failed')
            self.assertEqual(report['max_ms'], 201)
            self.assertEqual(dict(os.environ), before)
            self.assertIs(subprocess.run, global_run)
        self.assertIs(driver.menus.subprocess, original_dependency)
        self.assertEqual(len(calls), 1)

    def test_complete_diagnostics_keep_strict_failure_and_never_claim_manual_or_reference_pass(self):
        def stage(name, cases, repetitions, output, tests, source, binary, contract, receipts):
            receipts.extend(dict(stage=name) for _ in range(len(cases) * repetitions))
            return dict(status='failed' if name == 'pilot' else 'passed', records=[])
        with mock.patch.object(driver.sys, 'platform', 'win32'), mock.patch.dict(os.environ, HOSTED), \
                mock.patch.object(driver, 'source_snapshot', return_value=self.snapshot), \
                mock.patch.object(driver, 'desktop_diagnostics', return_value={}), \
                mock.patch.object(driver, 'automatic_stage', side_effect=stage) as run:
            code, report = driver.execute(self.tests, self.directory, SOURCE, self.binary)
        self.assertEqual(run.call_count, 2)
        self.assertEqual(code, 1)
        self.assertTrue(report['capture_complete'])
        self.assertFalse(report['strict_menu_contract_passed'])
        for key in ('real_user_click', 'manual_native_acceptance', 'reference_performance_approved', 'physical_dual_monitor_tested'):
            self.assertIs(report[key], False)

    def test_real_runner_collects_separate_pilot_and_complete_45_after_only_timing_failure(self):
        calls = []
        def owned(command, env, directory, tests, source, binary, timeout):
            calls.append(command)
            case = command[1].split(':')[1]
            self.sample(directory, case, trigger_to_complete_frame_ms=201 if len(calls) == 1 else 25)
            return subprocess.CompletedProcess(command, 1 if len(calls) == 1 else 0, b'', b''), \
                dict(timed_out=False, owned_child_pid=5000 + len(calls), exit_code=1 if len(calls) == 1 else 0)
        with mock.patch.object(driver.sys, 'platform', 'win32'), mock.patch.dict(os.environ, HOSTED), \
                mock.patch.object(driver, 'source_snapshot', return_value=self.snapshot), \
                mock.patch.object(driver, 'desktop_diagnostics', return_value={}), \
                mock.patch.object(driver.menus.platform, 'platform', return_value='hosted Windows'), \
                mock.patch.object(driver.menus.platform, 'processor', return_value='test'), \
                mock.patch.object(driver, 'run_owned', side_effect=owned):
            code, report = driver.execute(self.tests, self.directory, SOURCE, self.binary)
        self.assertEqual(len(calls), 46)
        self.assertEqual(code, 1)
        self.assertEqual(report['pilot']['status'], 'failed')
        self.assertEqual(report['matrix']['status'], 'passed')
        self.assertEqual(len(report['matrix']['records']), 45)
        self.assertTrue(report['capture_complete'])
        self.assertFalse(report['strict_menu_contract_passed'])

    def test_real_adapter_stops_on_first_invalid_guard_and_restores_dependencies(self):
        original = driver.menus.subprocess
        def owned(command, env, directory, *unused):
            self.sample(directory, command[1].split(':')[1], capture_end_sync_generation=3)
            return subprocess.CompletedProcess(command, 0, b'', b''), dict(timed_out=False, owned_child_pid=4321)
        with mock.patch.object(driver, 'run_owned', side_effect=owned) as child:
            with self.assertRaises(driver.Blocked):
                driver.automatic_stage('pilot', ['toolbar-more', 'toolbar-extract'], 1, self.directory / 'pilot',
                                       self.tests, SOURCE, self.binary, self.contract, [])
        self.assertEqual(child.call_count, 1)
        self.assertIs(driver.menus.subprocess, original)

    def test_non_hosted_environment_is_blocked_without_any_child(self):
        with mock.patch.dict(os.environ, {}, clear=True), mock.patch.object(driver.subprocess, 'Popen') as child:
            code, report = driver.execute(self.tests, self.directory, SOURCE, self.binary)
        self.assertEqual(code, 2)
        self.assertEqual(report['reason'], 'windows_github_hosted_runner_required')
        child.assert_not_called()

    def test_unchanged_interactive_validator_rejects_automatic_diagnostics(self):
        launcher = dict(status='diagnostic_passed', full_source_sha=SOURCE, binary_sha256=self.binary,
                        started_by_real_foreground_mouse_click=False, fresh_process_per_sample=True,
                        menu_warmup=False, valid_samples=0, records=[], failures=[], owned_child_launches=[])
        file = self.directory / 'review.json'
        file.write_text(json.dumps(launcher), encoding='utf-8')
        before = file.read_bytes()
        result = driver.menus.validate_existing(self.directory, self.tests, SOURCE, self.binary, self.contract)
        self.assertEqual(result['status'], 'failed')
        self.assertTrue(any('required input/process gates' in item['failure'] for item in result['failures']))
        self.assertEqual(file.read_bytes(), before)

    def test_visual_driver_pure_contract_checks(self):
        spec = importlib.util.spec_from_file_location('windows_visual_ci', ROOT / 'scripts/check-windows-native-visual-ci.py')
        visual = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(visual)
        self.assertEqual(visual.self_test(), 0)

    def test_layout_driver_pure_contract_checks(self):
        spec = importlib.util.spec_from_file_location('windows_layout_ci', ROOT / 'scripts/check-windows-native-layout-ci.py')
        layout = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(layout)
        result = layout.self_test()
        self.assertEqual(result['status'], 'self_test_passed')
        self.assertGreaterEqual(result['checks'], 9)
        self.assertTrue(result['synthetic_data_only'])
        self.assertEqual(result['native_processes_started'], 0)


if __name__ == '__main__':
    unittest.main()
