# scripts/test-sparkle-bundle.py

"""Exercise Mac packaging boundaries without claiming native updater execution."""
import copy
import hashlib
import json
import os
from pathlib import Path
import plistlib
import runpy
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import sparkle_bundle as sparkle

image = runpy.run_path(str(Path(__file__).with_name('test-macos-attribution.py')))['image']


class ProbeClock:
    """Exercise real orchestration without consuming the CI retry interval."""
    def __init__(self, extra_wait=0, interrupt=False):
        self.elapsed = 0.0
        self.waits = []
        self.extra_wait = extra_wait
        self.interrupt = interrupt

    def time(self):
        return 1000 + self.elapsed

    def monotonic(self):
        return self.elapsed

    def sleep(self, seconds):
        self.waits.append(seconds)
        self.elapsed += seconds + self.extra_wait
        if self.interrupt:
            raise InterruptedError('fixture interruption')


class MacHttpsRetryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.module = runpy.run_path(str(Path(__file__).with_name('check-macos-update.py')))

    def setUp(self):
        temporary_root = sparkle.ROOT/'build/sparkle-tests'
        temporary_root.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(dir=temporary_root)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.app = self.root/'deployed/fixture.app'
        (self.app/'Contents/MacOS').mkdir(parents=True)
        (self.app/'Contents/MacOS/xdzip-update-probe').write_bytes(b'owned fixture executable')
        self.output = self.root/'records'
        self.output.mkdir()

    def limited(self, status=429, **headers):
        return dict(success=False, error='rate_limited', timeout=False, http_diagnostics_schema=1,
                    rate_limit_confirmed=True, confirmed_from_controller=True,
                    authorization_present=False, request_count=1, response_records_truncated=False,
                    http_responses=[dict(status=status, host='api.github.com', route='release_list',
                                         server_date_epoch=1000, rate_headers=headers)])

    def plan(self, record, attempt=1, now=1000):
        return self.module['https_retry_plan'](record, attempt, now)

    def execute(self, records, clock=None):
        clock = clock or ProbeClock()
        calls = []
        probe = self.module['https_probe']
        def native(arguments, **options):
            index = len(calls)
            calls.append((arguments, options))
            # A previous success must never survive the next actual launch.
            self.assertFalse((self.app.parent/'https.json').exists())
            record = records[index]
            if isinstance(record, Exception):
                raise record
            if isinstance(record, bytes):
                (self.app.parent/'https.json').write_bytes(record)
                code = 69
            else:
                (self.app.parent/'https.json').write_text(json.dumps(record))
                code = 0 if record.get('success') is True else 69
            return SimpleNamespace(returncode=code, stdout=f'attempt {index+1}\n'.encode(), stderr=b'')
        changes = {'audit': lambda *args: {'status': 'passed'}, 'time': clock,
                   'clean_environment': lambda: {}}
        with patch.dict(probe.__globals__, changes), patch('subprocess.run', side_effect=native):
            try:
                result = probe(self.app, self.output)
            except Exception as error:
                return error, clock, calls
        return result, clock, calls

    def read(self, name):
        return json.loads((self.output/name).read_bytes())

    def test_only_confirmed_anonymous_api_limits_retry(self):
        for status in (403, 429):
            for attempt, delay in ((1, 60), (2, 120)):
                with self.subTest(status=status, attempt=attempt):
                    self.assertEqual(self.plan(self.limited(status), attempt)['delay_seconds'], delay)
        changes = ({'http_diagnostics_schema': True}, {'http_diagnostics_schema': 0},
                   {'rate_limit_confirmed': False}, {'confirmed_from_controller': False},
                   {'authorization_present': True}, {'error': 'network'}, {'error': 'tls'},
                   {'timeout': True}, {'response_records_truncated': True}, {'request_count': True})
        for change in changes:
            with self.subTest(change=change):
                record = self.limited()
                record.update(change)
                self.assertFalse(self.plan(record)['retry'])
        for field, value in (('status', 500), ('status', True), ('host', 'other_allowed_host'),
                             ('route', 'other')):
            with self.subTest(field=field, value=value):
                record = self.limited()
                record['http_responses'][-1][field] = value
                self.assertFalse(self.plan(record)['retry'])

    def test_primary_reset_uses_server_clock_and_later_retry_after(self):
        record = self.limited(403, remaining='0', reset='1120')
        self.assertEqual(self.plan(record, now=9000)['delay_seconds'], 121)
        record['http_responses'][-1]['rate_headers']['retry_after'] = '180'
        self.assertEqual(self.plan(record)['delay_seconds'], 180)
        record['http_responses'][-1]['rate_headers']['retry_after'] = '30'
        self.assertEqual(self.plan(record)['delay_seconds'], 121)
        record['http_responses'][-1]['server_date_epoch'] = None
        self.assertEqual(self.plan(record, now=1100)['delay_seconds'], 30)

    def test_missing_invalid_or_expired_primary_deadline_never_falls_back(self):
        for reset in (None, 'invalid', '-1', 'nan', '1000', '999', '9'*21):
            with self.subTest(reset=reset):
                self.assertFalse(self.plan(self.limited(403, remaining='0', reset=reset,
                                                         retry_after='120'))['retry'])
        record = self.limited(remaining='0', reset='1120')
        record['http_responses'][-1]['server_date_epoch'] = None
        self.assertFalse(self.plan(record, now=float('nan'))['retry'])

    def test_malformed_headers_do_not_become_short_backoff(self):
        for header in ('remaining', 'retry_after'):
            for value in ('', '-1', '1.5', 'nan', 'Infinity', 'Wed, 21 Oct 2015 07:28:00 GMT',
                          '９', '9'*21, 60, True):
                with self.subTest(header=header, value=value):
                    self.assertFalse(self.plan(self.limited(**{header: value}))['retry'])
        self.assertEqual(self.plan(self.limited(retry_after='0'))['delay_seconds'], 1)

    def test_success_after_reset_preserves_both_raw_attempts_and_wait(self):
        success = dict(success=True, error='', timeout=False, ssl_available=True,
                       encrypted_connections=1, https_responses=1, state='up_to_date')
        result, clock, calls = self.execute([self.limited(403, remaining='0', reset='1120'), success])
        self.assertTrue(result['success'])
        self.assertEqual(clock.waits, [121])
        self.assertEqual(len(calls), 2)
        self.assertEqual([call[1]['timeout'] for call in calls], [200, 200])
        self.assertEqual(self.read('https-attempt-1.json')['error'], 'rate_limited')
        self.assertTrue(self.read('https-attempt-2.json')['success'])
        self.assertEqual(self.read('https.json'), self.read('https-attempt-2.json'))
        self.assertEqual(self.read('https-attempt-1.json')['probe_binary_sha256'],
                         self.read('https-attempt-2.json')['probe_binary_after_sha256'])
        self.assertTrue(self.read('https-attempt-1.json')['probe_binary_unchanged'])
        wait = self.read('https-waits.json')[0]
        self.assertEqual(wait['wait_status'], 'finished')
        self.assertEqual(wait['actual_wait_seconds'], 121)
        self.assertEqual(wait['total_actual_wait_seconds'], 121)
        self.assertEqual((self.output/'https-attempt-1.log').read_bytes(), b'attempt 1\n')
        self.assertEqual((self.output/'https.log').read_bytes(), b'attempt 2\n')

    def test_three_total_launches_with_no_sleep_after_terminal_failure(self):
        result, clock, calls = self.execute([self.limited()] * 3)
        self.assertIsInstance(result, RuntimeError)
        self.assertIn('attempt_limit_reached', str(result))
        self.assertEqual(len(calls), 3)
        self.assertEqual(clock.waits, [60, 120])
        self.assertFalse(self.read('https.json')['success'])
        waits = self.read('https-waits.json')
        self.assertEqual(len(waits), 3)
        self.assertFalse(waits[-1]['retry'])
        self.assertNotIn('wait_status', waits[-1])

    def test_deadline_beyond_total_budget_stops_without_clamping(self):
        result, clock, calls = self.execute([self.limited(remaining='0', reset='4700')])
        self.assertIn('wait_budget_exceeded', str(result))
        self.assertEqual(len(calls), 1)
        self.assertEqual(clock.waits, [])
        self.assertEqual(self.read('https-waits.json')[0]['delay_seconds'], 3701)

    def test_cumulative_wait_budget_is_not_renewed_for_second_limit(self):
        result, clock, calls = self.execute([self.limited(retry_after='3600'),
                                             self.limited(retry_after='101')])
        self.assertIn('wait_budget_exceeded', str(result))
        self.assertEqual(clock.waits, [3600])
        self.assertEqual(len(calls), 2)
        self.assertEqual(self.read('https-waits.json')[-1]['remaining_wait_budget_seconds'], 100)

    def test_monotonic_oversleep_exhausts_budget_before_next_launch(self):
        result, clock, calls = self.execute([self.limited()], ProbeClock(extra_wait=3700))
        self.assertIn('wait budget exhausted', str(result))
        self.assertEqual(len(calls), 1)
        self.assertEqual(self.read('https-waits.json')[0]['actual_wait_seconds'], 3760)

    def test_interrupted_sleep_records_elapsed_without_another_launch(self):
        result, clock, calls = self.execute([self.limited()], ProbeClock(interrupt=True))
        self.assertIsInstance(result, InterruptedError)
        self.assertEqual(len(calls), 1)
        self.assertEqual(self.read('https-waits.json')[0]['wait_status'], 'interrupted_or_short')

    def test_early_sleep_return_never_launches_before_server_deadline(self):
        result, clock, calls = self.execute([self.limited()], ProbeClock(extra_wait=-59))
        self.assertIn('before the required deadline', str(result))
        self.assertEqual(len(calls), 1)
        self.assertEqual(self.read('https-waits.json')[0]['wait_status'], 'interrupted_or_short')
        self.assertEqual(self.read('https-waits.json')[0]['actual_wait_seconds'], 1)

    def test_generic_http_tls_and_invalid_evidence_fail_without_retry(self):
        records = [dict(success=False, error='network', timeout=False),
                   dict(success=False, error='tls', timeout=False), b'not JSON', b'[]']
        for record in records:
            with self.subTest(record=record):
                result, clock, calls = self.execute([record])
                self.assertIsInstance(result, RuntimeError)
                self.assertEqual(len(calls), 1)
                self.assertEqual(clock.waits, [])
                self.assertFalse(self.read('https.json')['success'])

    def test_subprocess_timeout_preserves_failure_output(self):
        result, clock, calls = self.execute([subprocess.TimeoutExpired('fixture', 200,
                                            output=b'partial output\n', stderr=b'partial error\n')])
        self.assertIsInstance(result, RuntimeError)
        self.assertEqual(len(calls), 1)
        self.assertEqual(clock.waits, [])
        self.assertTrue(self.read('https.json')['timeout'])
        self.assertIsNone(self.read('https.json')['process_returncode'])
        self.assertEqual((self.output/'https-attempt-1.log').read_bytes(),
                         b'partial output\npartial error\n')

    def test_stale_success_is_removed_before_each_launch(self):
        (self.app.parent/'https.json').write_text('{"success":true}')
        result, clock, calls = self.execute([b'not JSON'])
        self.assertIsInstance(result, RuntimeError)
        self.assertFalse(self.read('https.json')['success'])

    def test_native_launch_error_records_only_safe_kind_and_number(self):
        result, clock, calls = self.execute([FileNotFoundError(2, 'fixture missing binary',
                                                              '/private/fixture/binary')])
        self.assertIsInstance(result, RuntimeError)
        self.assertEqual(len(calls), 1)
        self.assertEqual(clock.waits, [])
        record = self.read('https-attempt-1.json')
        self.assertEqual(record['process_error_kind'], 'FileNotFoundError')
        self.assertEqual(record['process_error_number'], 2)
        self.assertNotIn('/private/fixture', json.dumps(record))

    def test_changed_binary_after_wait_is_rejected_before_launch(self):
        clock = ProbeClock()
        sleep = clock.sleep
        def change_binary(seconds):
            sleep(seconds)
            (self.app/'Contents/MacOS/xdzip-update-probe').write_bytes(b'changed fixture executable')
        clock.sleep = change_binary
        result, clock, calls = self.execute([self.limited()], clock)
        self.assertIsInstance(result, RuntimeError)
        self.assertEqual(len(calls), 1)
        self.assertEqual(clock.waits, [60])
        record = self.read('https-attempt-2.json')
        self.assertEqual(record['error'], 'probe_binary_changed')
        self.assertFalse(record['probe_binary_unchanged'])
        self.assertNotEqual(record['probe_binary_sha256'], self.read('https-attempt-1.json')['probe_binary_sha256'])


class SparkleBundleTests(unittest.TestCase):
    def setUp(self):
        temporary_root = sparkle.ROOT/'build/sparkle-tests'
        temporary_root.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(dir=temporary_root)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.sdk = self.root/'sdk'
        self.app = self.root/'xdzip.app'
        self.spec = copy.deepcopy(sparkle.configuration())
        self.spec['license_sha256'] = hashlib.sha256(b'fixture legal notice').hexdigest()
        self.addCleanup(patch.stopall)
        patch('sparkle_bundle.configuration', return_value=self.spec).start()
        self.write(self.sdk/'LICENSE', b'fixture legal notice')
        framework = self.sdk/'Sparkle.framework/Versions/B'
        self.write(framework/'Resources/Info.plist', plistlib.dumps({'CFBundleShortVersionString': '2.10.0'}))
        self.write(framework/'Headers/SPUUpdater.h', b'fixture SDK header')
        for relative in ('Sparkle', 'Autoupdate', 'Updater.app/Contents/MacOS/Updater',
                         'XPCServices/Downloader.xpc/Contents/MacOS/Downloader'):
            self.write(framework/relative, image())
        self.write(self.app/'Contents/Info.plist', plistlib.dumps({'SUPublicEDKey': 'fixture-only-public-key'}))

    def write(self, path, data):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    def test_deployment_identity_and_unused_service_removal(self):
        framework = sparkle.embed(self.sdk, self.app, installable=True)
        info = plistlib.loads((self.app/'Contents/Info.plist').read_bytes())
        self.assertTrue(info['XdzipUpdateInstallable'])
        self.assertEqual(info['XdzipUpdateInstallId'], sparkle.INSTALL_ID)
        self.assertFalse((framework/'Versions/B/XPCServices').exists())
        self.assertTrue((framework/'Versions/B/Autoupdate').is_file())
        self.assertTrue((self.sdk/'Sparkle.framework/Versions/B/XPCServices').is_dir())
        sparkle.embed(self.sdk, self.app, installable=False)
        self.assertFalse(plistlib.loads((self.app/'Contents/Info.plist').read_bytes())['XdzipUpdateInstallable'])

    def test_missing_public_key_cannot_create_installable_marker(self):
        self.write(self.app/'Contents/Info.plist', plistlib.dumps({}))
        sparkle.embed(self.sdk, self.app, installable=True)
        self.assertFalse(plistlib.loads((self.app/'Contents/Info.plist').read_bytes())['XdzipUpdateInstallable'])

    def test_notice_identity_is_pinned(self):
        files = sparkle.write_notice(self.sdk, self.root/'notices')
        self.assertEqual(files[0].read_bytes(), b'fixture legal notice')
        self.write(self.sdk/'LICENSE', b'changed notice')
        with self.assertRaisesRegex(ValueError, 'identity differs'):
            sparkle.write_notice(self.sdk, self.root/'other')

    def test_helper_runtime_flags_and_entitlements_are_separate(self):
        framework = sparkle.embed(self.sdk, self.app, installable=False)
        entitlements = {'com.apple.security.cs.disable-library-validation': True}
        metadata = sparkle.signing_metadata(framework, lambda command: plistlib.dumps(entitlements).decode())
        calls = []
        sparkle.sign_helpers(framework, metadata, lambda command: calls.append(command), self.root)
        self.assertEqual(len(calls), 4)
        for command in calls[:3]:
            self.assertIn('runtime', command)
            self.assertNotIn('--deep', command)
        self.assertEqual(plistlib.loads((self.root/'sparkle-helper-0.entitlements').read_bytes()), entitlements)
        with self.assertRaisesRegex(ValueError, 'debugger'):
            sparkle.signing_metadata(framework, lambda command: plistlib.dumps({'com.apple.security.get-task-allow': True}).decode())

    def test_native_attribution_rejects_changed_code(self):
        sparkle.embed(self.sdk, self.app, installable=False)
        relative = sparkle.PREFIX+'Versions/B/Sparkle'
        path = self.app/relative
        report = {'files': [], 'components': [], 'dependencies': [],
                  'gaps': [{'kind': 'native-outside-qt-attribution', 'file': relative}]}
        def audit():
            return {'files': [{'path': relative, 'sha256': sparkle.checksum(path)}]}
        sparkle.attribute(report, self.app, self.sdk, audit())
        self.assertEqual(report['gaps'], [])
        self.assertEqual(report['files'][0]['owners'], ['sparkle:Sparkle@2.10.0'])
        self.assertNotIn(str(self.sdk), json.dumps(report))
        path.write_bytes(image(b'changed executable'))
        with self.assertRaisesRegex(ValueError, 'differ'):
            sparkle.attribute(copy.deepcopy(report), self.app, self.sdk, audit())

    def test_signing_uses_final_asset_and_fails_on_missing_package(self):
        with patch.dict(os.environ, {'XDZIP_SPARKLE_SIGNING_KEY_HEX': ''}):
            self.assertFalse(sparkle.sign_staged_archive(self.root, '0.1.0-alpha.7', 20))
        with patch.dict(os.environ, {'XDZIP_SPARKLE_SIGNING_KEY_HEX': 'fixture-only'}):
            with self.assertRaisesRegex(ValueError, 'missing'):
                sparkle.sign_staged_archive(self.root, '0.1.0-alpha.7', 20)
            package = self.root/'xdzip-0.1.0-alpha.7-macos-arm64.app.zip'
            package.write_bytes(b'fixture package')
            calls = []
            def signer(arguments, tool):
                calls.append(arguments)
                if arguments[0] == 'sign-archive':
                    (self.root/'macos-update.json').write_text(json.dumps({'ed_signature': 'fixture-signature'}))
            with patch('release_updates.run_tool', side_effect=signer):
                self.assertTrue(sparkle.sign_staged_archive(self.root, '0.1.0-alpha.7', 20))
            self.assertEqual(calls[0][calls[0].index('--archive')+1], str(package))
            self.assertEqual(calls[1][0], 'verify-archive')
            self.assertNotIn('fixture-only', repr(calls))

    def test_https_probe_has_no_sdk_fallback_and_requires_native_success(self):
        probe = runpy.run_path(str(Path(__file__).with_name('check-macos-update.py')))['https_probe']
        self.write(self.app/'Contents/MacOS/xdzip-update-probe', b'owned fixture executable')
        sdk_variables = {'QT_PLUGIN_PATH': '/fixture/sdk/plugins', 'QML2_IMPORT_PATH': '/fixture/sdk/qml',
                         'DYLD_FRAMEWORK_PATH': '/fixture/sdk/lib', 'XDZIP_QT_PREFIX': '/fixture/sdk',
                         'CMAKE_PREFIX_PATH': '/fixture/sdk', 'QTDIR': '/fixture/sdk'}
        calls = []
        def native_https(arguments, **options):
            calls.append((arguments, options))
            self.assertTrue(all(name not in options['env'] for name in sdk_variables))
            self.assertEqual(options['env']['PATH'], '/usr/bin:/bin:/usr/sbin:/sbin')
            self.assertEqual(options['env']['RUSTUP_TOOLCHAIN'], probe.__globals__['locked_rust_version']())
            self.assertEqual(arguments[-1], '--update-https-probe')
            (self.app.parent/'https.json').write_text(json.dumps({'success': True, 'ssl_backend': 'fixture'}))
            return SimpleNamespace(returncode=0, stdout=b'', stderr=b'')
        with patch.dict(probe.__globals__, {'audit': lambda *args: {'status': 'passed'}}), \
                patch.dict(os.environ, {**sdk_variables, 'RUSTUP_TOOLCHAIN': 'fixture-other-channel'}), \
                patch('subprocess.run', side_effect=native_https):
            result = probe(self.app, self.root)
        self.assertTrue(result['sdk_environment_cleared'])
        self.assertEqual(len(calls), 1)
        with patch.dict(probe.__globals__, {'audit': lambda *args: {'status': 'failed'}}), \
                patch('subprocess.run') as launch:
            with self.assertRaisesRegex(RuntimeError, 'dependencies outside'):
                probe(self.app, self.root)
            launch.assert_not_called()

    def test_product_probe_retains_home_and_uses_the_real_default_profile(self):
        probe = runpy.run_path(str(Path(__file__).with_name('check-macos-product-update.py')))
        supplied = {'HOME': '/fixture/unchanged-home', 'XDZIP_SETTINGS_PATH': '/fixture/override.ini',
                    'XDZIP_LANGUAGE': 'zh-CN', 'QML_IMPORT_PATH': '/fixture/sdk/qml',
                    'DYLD_FRAMEWORK_PATH': '/fixture/sdk/lib', 'RUSTUP_TOOLCHAIN': 'fixture-other-channel'}
        with patch.dict(os.environ, supplied):
            environment = probe['environment']()
        self.assertEqual(environment['HOME'], supplied['HOME'])
        for name in ('XDZIP_SETTINGS_PATH', 'XDZIP_LANGUAGE', 'QML_IMPORT_PATH', 'DYLD_FRAMEWORK_PATH'):
            self.assertNotIn(name, environment)
        self.assertEqual(environment['RUSTUP_TOOLCHAIN'], probe['locked_rust_version']())
        with self.assertRaisesRegex(RuntimeError, 'escaped'):
            probe['require_owned'](self.root/'other/owned.app', self.root/'expected')


if __name__ == '__main__':
    unittest.main()
