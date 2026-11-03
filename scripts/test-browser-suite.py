#!/usr/bin/env python3
"""Exercise browser-suite discovery, coverage, failure and process cleanup."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('browser_suite', ROOT / 'scripts/run-browser-suite.py')
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)

STUB = r'''
import json, os, pathlib, subprocess, sys, time
names = ['testAlpha', 'testBeta', 'testGamma', 'testDelta', 'testEpsilon', 'testZeta']
def sleep_with_child():
    child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)'])
    pathlib.Path(os.environ['PROBE_PIDS'], str(os.getpid()) + '.json').write_text(
        json.dumps({'worker': os.getpid(), 'child': child.pid}))
    time.sleep(60)
if '-functions' in sys.argv:
    if os.environ.get('PROBE_DISCOVERY_SLEEP'):
        sleep_with_child()
    if os.environ.get('PROBE_DUPLICATE'):
        names.append(names[0])
    print('\n'.join(name + '()' for name in names))
    sys.exit(0)
arguments = sys.argv[1:]
selected = arguments[:arguments.index('-v1')]
if not selected:
    selected = names
if os.environ.get('PROBE_NO_LOG') in selected:
    sys.exit(0)
output = pathlib.Path(arguments[arguments.index('-o') + 1].removesuffix(',txt'))
if os.environ.get('PROBE_SLEEP'):
    sleep_with_child()
with output.open('w', encoding='utf-8') as log:
    for name in selected:
        if name != os.environ.get('PROBE_OMIT'):
            log.write('INFO   : BrowserTest::' + name + '() entering\n')
    if os.environ.get('PROBE_FAILURE') in selected:
        log.write('FAIL!  : injected failure\n')
        sys.exit(7)
'''


def alive(pid):
    if os.name == 'nt':
        import ctypes
        from ctypes import wintypes
        kernel = ctypes.WinDLL('kernel32', use_last_error=True)
        kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
        kernel.OpenProcess.restype = wintypes.HANDLE
        kernel.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
        kernel.GetExitCodeProcess.restype = wintypes.BOOL
        kernel.CloseHandle.argtypes = [wintypes.HANDLE]
        handle = kernel.OpenProcess(0x1000, False, pid)
        if not handle:
            return False
        try:
            status = wintypes.DWORD()
            return bool(kernel.GetExitCodeProcess(handle, ctypes.byref(status)) and status.value == 259)
        finally:
            kernel.CloseHandle(handle)
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    proc = Path('/proc') / str(pid) / 'stat'
    if proc.exists() and proc.read_text().split(') ', 1)[1].startswith('Z '):
        return False
    return True


def ctest_diagnostics(returncode, stdout, stderr, elapsed, discovery, evidence_count, process_timeout=False):
    """Retain bounded process facts without printing fixture paths or output."""
    def has_timeout(value):
        if isinstance(value, bytes):
            return b'Timeout' in value
        return isinstance(value, str) and 'Timeout' in value

    def output_length(value):
        if isinstance(value, str):
            return {'length': len(value), 'unit': 'characters'}
        if isinstance(value, bytes):
            return {'length': len(value), 'unit': 'bytes'}
        return {'length': 0, 'unit': 'absent'}

    return {
        'command': ['ctest', '--test-dir', '<owned-temp>', '--output-on-failure'],
        'phase': 'discovery' if discovery else 'workers',
        'exit_code': returncode,
        'exit_code_hex32': f'0x{returncode & 0xffffffff:08X}' if type(returncode) is int else None,
        'elapsed_ms': round(elapsed * 1000),
        'ctest_timeout_seconds': 6,
        'outer_timeout_seconds': 25,
        'outer_process_timeout': process_timeout,
        'stdout': output_length(stdout),
        'stderr': output_length(stderr),
        'timeout_marker': has_timeout(stdout) or has_timeout(stderr),
        'owned_process_records': evidence_count,
    }


class BrowserSuiteTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.stub = self.root / 'qt_test_stub.py'
        self.stub.write_text(STUB, encoding='utf-8')
        self.output = self.root / 'browser-tests.log'
        self.env = os.environ.copy()
        self.env['QT_QPA_PLATFORM'] = 'offscreen'
        self.env['XDZIP_BROWSER_TIMING_DIR'] = str(self.root / 'timings')
        self.env.pop('XDZIP_SETTINGS_PATH', None)
        for name in list(self.env):
            if name.startswith('XDZIP_') and '_AUDIT' in name:
                self.env.pop(name)

    def run_probe(self):
        return runner.run_suite([sys.executable, str(self.stub)], self.output, self.env, self.root)

    def test_every_discovered_function_runs_once_and_is_recorded(self):
        self.assertEqual(self.run_probe(), 0)
        manifest = json.loads(next((self.root / 'timings').glob('browser-suite-*.json')).read_text())
        self.assertEqual(manifest['status'], 'passed')
        self.assertEqual(manifest['functions'], 6)
        names = [name for shard in manifest['shards'] for name in shard['functions']]
        self.assertEqual(len(names), len(set(names)))
        self.assertEqual(len(names), 6)
        self.assertTrue(all(shard['status'] == 'passed' for shard in manifest['shards']))
        pids = [shard['process_id'] for shard in manifest['shards']]
        self.assertEqual(len(set(pids)), len(pids))
        self.assertTrue(all(type(pid) is int and pid > 0 for pid in pids))
        self.assertEqual(self.output.read_text().count('() entering'), 6)

    def test_two_worker_run_still_covers_every_function(self):
        with mock.patch.object(runner, 'worker_count', return_value=2):
            self.assertEqual(self.run_probe(), 0)
        manifest = json.loads(next((self.root / 'timings').glob('browser-suite-*.json')).read_text())
        self.assertEqual(len(manifest['shards']), 2)
        self.assertEqual(self.output.read_text().count('() entering'), 6)

    def test_four_worker_run_still_covers_every_function(self):
        with mock.patch.object(runner, 'worker_count', return_value=4):
            self.assertEqual(self.run_probe(), 0)
        manifest = json.loads(next((self.root / 'timings').glob('browser-suite-*.json')).read_text())
        self.assertEqual(len(manifest['shards']), 4)
        self.assertEqual(self.output.read_text().count('() entering'), 6)

    def test_a_missing_function_and_a_failed_worker_fail_the_suite(self):
        self.env['PROBE_OMIT'] = 'testGamma'
        self.assertEqual(self.run_probe(), 1)
        self.env.pop('PROBE_OMIT')
        self.env['PROBE_FAILURE'] = 'testBeta'
        self.assertEqual(self.run_probe(), 1)
        self.assertIn('injected failure', self.output.read_text())

    def test_duplicate_discovery_is_rejected(self):
        self.env['PROBE_DUPLICATE'] = '1'
        with self.assertRaisesRegex(RuntimeError, 'duplicate'):
            self.run_probe()

    def test_silent_worker_cannot_reuse_previous_log(self):
        self.assertEqual(self.run_probe(), 0)
        self.env['PROBE_NO_LOG'] = 'testGamma'
        self.assertEqual(self.run_probe(), 1)

    def test_native_and_shared_settings_runs_are_serial(self):
        self.env['QT_QPA_PLATFORM'] = 'windows'
        self.assertEqual(self.run_probe(), 0)
        self.assertFalse((self.root / 'timings').exists())
        self.env['QT_QPA_PLATFORM'] = 'offscreen'
        self.env['XDZIP_SETTINGS_PATH'] = str(self.root / 'shared.ini')
        self.assertEqual(self.run_probe(), 0)
        self.assertFalse((self.root / 'timings').exists())
        self.env.pop('XDZIP_SETTINGS_PATH')
        self.env['XDZIP_VISUAL_AUDIT'] = str(self.root / 'captures')
        self.assertEqual(self.run_probe(), 0)
        self.assertFalse((self.root / 'timings').exists())

    def test_ctest_diagnostics_retains_exit_and_omits_private_output(self):
        record = ctest_diagnostics(-1073741502, 'private fixture path', '', .125, False, 0)
        self.assertEqual(record['exit_code_hex32'], '0xC0000142')
        self.assertEqual(record['stdout'], {'length': len('private fixture path'), 'unit': 'characters'})
        self.assertFalse(record['timeout_marker'])
        self.assertNotIn('private fixture path', json.dumps(record))
        record = ctest_diagnostics(None, b'partial Timeout', None, 25, True, 1, True)
        self.assertTrue(record['outer_process_timeout'])
        self.assertTrue(record['timeout_marker'])
        self.assertEqual(record['phase'], 'discovery')

    def test_ctest_empty_failure_reports_status_without_exposing_paths(self):
        result = subprocess.CompletedProcess([], -1073741502, stdout='', stderr='')
        with mock.patch('shutil.which', return_value='ctest'), mock.patch('subprocess.run', return_value=result):
            with self.assertRaisesRegex(AssertionError, 'CTest timeout marker missing') as raised:
                self.assert_ctest_timeout_terminates_processes()
        message = str(raised.exception)
        self.assertIn('0xC0000142', message)
        self.assertIn('"owned_process_records": 0', message)
        self.assertNotIn(str(self.root), message)

    @unittest.skipUnless(sys.platform == 'win32', 'Windows loader behavior only')
    def test_real_windows_loader_failures_keep_empty_output_and_status(self):
        compiler = shutil.which('cl')
        if not compiler:
            if os.environ.get('CI', '').lower() == 'true':
                self.fail('MSVC is required for the Windows CI loader fixture')
            self.skipTest('MSVC compiler is unavailable')
        fixtures = ROOT / 'tests/fixtures/native'
        shutil.copy2(fixtures/'windows-loader-failure-dll.c', self.root/'owned-dll.c')
        shutil.copy2(fixtures/'windows-loader-failure-main.c', self.root/'owned-loader.c')
        builds = [
            [compiler, '/nologo', '/MT', '/LD', '/Fe:owned-dll.dll', '/Fo:owned-dll.obj',
             'owned-dll.c', '/link', '/IMPLIB:owned-dll.lib'],
            [compiler, '/nologo', '/MT', '/Fe:owned-loader.exe', '/Fo:owned-loader.obj',
             'owned-loader.c', 'owned-dll.lib'],
        ]
        for command in builds:
            built = subprocess.run(command, cwd=self.root, capture_output=True, timeout=60)
            self.assertEqual(built.returncode, 0, 'Owned loader fixture compilation failed')
        missing = self.root/'missing-dll'
        missing.mkdir()
        shutil.copy2(self.root/'owned-loader.exe', missing/'owned-loader.exe')
        env = dict(os.environ, PATH=str(Path(os.environ['SystemRoot'])/'System32'))
        import ctypes
        kernel = ctypes.WinDLL('kernel32', use_last_error=True)
        kernel.SetErrorMode.argtypes = [ctypes.c_uint]
        kernel.SetErrorMode.restype = ctypes.c_uint
        previous = kernel.SetErrorMode(0x0001 | 0x0002 | 0x8000)
        try:
            for directory, expected in ((self.root, 0xC0000142), (missing, 0xC0000135)):
                with self.subTest(expected=f'0x{expected:08X}'):
                    started = time.monotonic()
                    process = subprocess.run([str(directory/'owned-loader.exe')], cwd=directory,
                                             env=env, capture_output=True, timeout=10,
                                             creationflags=subprocess.CREATE_NO_WINDOW)
                    record = ctest_diagnostics(process.returncode, process.stdout, process.stderr,
                                               time.monotonic()-started, False, 0)
                    self.assertEqual(record['exit_code_hex32'], f'0x{expected:08X}')
                    self.assertEqual((process.stdout, process.stderr), (b'', b''))
                    self.assertEqual(record['stdout'], {'length': 0, 'unit': 'bytes'})
                    self.assertEqual(record['stderr'], {'length': 0, 'unit': 'bytes'})
                    self.assertFalse(record['timeout_marker'])
                    self.assertNotIn(json.dumps(str(self.root))[1:-1], json.dumps(record))
        finally:
            kernel.SetErrorMode(previous)

    def test_ctest_outer_timeout_reports_partial_output_without_exposing_it(self):
        timeout = subprocess.TimeoutExpired('ctest', 25, output=b'private fixture path')
        with mock.patch('shutil.which', return_value='ctest'), mock.patch('subprocess.run', side_effect=timeout):
            with self.assertRaisesRegex(AssertionError, 'CTest process exceeded outer timeout') as raised:
                self.assert_ctest_timeout_terminates_processes()
        message = str(raised.exception)
        self.assertIn('"outer_process_timeout": true', message)
        self.assertIn('"unit": "bytes"', message)
        self.assertNotIn('private fixture path', message)

    def assert_ctest_timeout_terminates_processes(self, discovery=False):
        ctest = shutil.which('ctest')
        if not ctest:
            self.skipTest('CTest is unavailable')
        pids = self.root / 'pids'
        pids.mkdir()
        self.env['PROBE_DISCOVERY_SLEEP' if discovery else 'PROBE_SLEEP'] = '1'
        self.env['PROBE_PIDS'] = str(pids)
        launcher = self.root / 'launcher.py'
        launcher.write_text(
            'import importlib.util, sys\n'
            f's=importlib.util.spec_from_file_location("runner", {str(ROOT / "scripts/run-browser-suite.py")!r})\n'
            'm=importlib.util.module_from_spec(s); s.loader.exec_module(m)\n'
            f'sys.exit(m.run_suite([sys.executable, {str(self.stub)!r}], {str(self.output)!r}))\n',
            encoding='utf-8')
        # Bracket arguments retain Windows spaces and backslashes without shell parsing.
        (self.root / 'CTestTestfile.cmake').write_text(
            f'add_test(timeout [==[{sys.executable}]==] [==[{launcher}]==])\n'
            'set_tests_properties(timeout PROPERTIES TIMEOUT 6)\n', encoding='utf-8')
        started = time.monotonic()
        try:
            result = subprocess.run([ctest, '--test-dir', str(self.root), '--output-on-failure'],
                                    env=self.env, capture_output=True, text=True, timeout=25)
        except subprocess.TimeoutExpired as error:
            record = ctest_diagnostics(None, error.stdout, error.stderr, time.monotonic() - started,
                                       discovery, len(list(pids.glob('*.json'))), True)
            self.fail('CTest process exceeded outer timeout: ' + json.dumps(record))
        record = ctest_diagnostics(result.returncode, result.stdout, result.stderr,
                                   time.monotonic() - started, discovery, len(list(pids.glob('*.json'))))
        self.assertNotEqual(result.returncode, 0, json.dumps(record))
        self.assertTrue(record['timeout_marker'], 'CTest timeout marker missing: ' + json.dumps(record))
        evidence = [json.loads(path.read_text()) for path in pids.glob('*.json')]
        self.assertTrue(evidence, 'No worker and child started before CTest timed out')
        ids = [item[key] for item in evidence for key in ('worker', 'child')]
        deadline = time.monotonic() + 5
        while any(alive(pid) for pid in ids) and time.monotonic() < deadline:
            time.sleep(.1)
        self.assertFalse([pid for pid in ids if alive(pid)], 'CTest left a test process running')

    def test_ctest_timeout_terminates_workers_and_their_children(self):
        self.assert_ctest_timeout_terminates_processes()

    def test_ctest_timeout_terminates_discovery_and_its_child(self):
        self.assert_ctest_timeout_terminates_processes(discovery=True)


if __name__ == '__main__':
    unittest.main()
