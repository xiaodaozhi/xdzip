"""Exercise the opt-in isolated build console with real Windows processes."""

import ctypes
import json
import os
from pathlib import Path
import queue
import shutil
import subprocess
import sys
import threading
import unittest
import uuid
import venv

import windows_detached_console as isolated


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT/'scripts/windows_detached_console.py'
BASE = getattr(sys, '_base_executable', None) or sys.executable
WINDOWS = sys.platform == 'win32'


@unittest.skipUnless(WINDOWS, 'requires Windows process and console APIs')
class IsolatedConsoleTests(unittest.TestCase):
    def setUp(self):
        self.kernel = isolated.windows_api()
        self.kernel.OpenProcess.argtypes = [ctypes.c_uint32, ctypes.c_int, ctypes.c_uint32]
        self.kernel.OpenProcess.restype = ctypes.c_void_p

    def run_wrapper(self, code, *, timeout=20, executable=sys.executable, data=b''):
        return subprocess.run([executable, str(SCRIPT), '--timeout', str(timeout), '--',
                               BASE, '-u', '-c', code], input=data, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, timeout=timeout+15)

    def test_redirected_unicode_and_console_restore(self):
        before = (self.kernel.GetConsoleCP(), self.kernel.GetConsoleOutputCP())
        code = '''import ctypes,json,sys
k=ctypes.WinDLL("kernel32"); k.GetConsoleWindow.restype=ctypes.c_void_p
u=ctypes.WinDLL("user32"); u.IsWindowVisible.argtypes=[ctypes.c_void_p]
data=sys.stdin.buffer.read().decode("utf-8")
sys.stdout.buffer.write((json.dumps([data,k.GetConsoleCP(),k.GetConsoleOutputCP(),bool(u.IsWindowVisible(k.GetConsoleWindow()))],ensure_ascii=False)+"\\n").encode("utf-8"))
sys.stderr.buffer.write(b"owned stderr\\n")
'''
        result = self.run_wrapper(code, data='中文𠮷\n'.encode('utf-8'))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout.decode('utf-8')), ['中文𠮷\n', 65001, 65001, False])
        self.assertEqual(result.stderr, b'owned stderr\n')
        self.assertEqual(before, (self.kernel.GetConsoleCP(), self.kernel.GetConsoleOutputCP()))

    def test_child_nonzero_and_missing_completion_are_failures(self):
        before = (self.kernel.GetConsoleCP(), self.kernel.GetConsoleOutputCP())
        result = self.run_wrapper('raise SystemExit(7)')
        self.assertEqual(result.returncode, 7, result.stderr)
        self.assertNotIn(b'did not report completion', result.stderr)
        # TerminateProcess can yield exit code 0, so the completion signal is mandatory.
        code = '''import ctypes,os
k=ctypes.WinDLL("kernel32",use_last_error=True); k.OpenProcess.restype=ctypes.c_void_p
k.TerminateProcess.argtypes=[ctypes.c_void_p,ctypes.c_uint]
h=k.OpenProcess(1,False,os.getppid())
assert h and k.TerminateProcess(h,0)
'''
        result = self.run_wrapper(code)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b'did not report completion', result.stderr)
        self.assertEqual(before, (self.kernel.GetConsoleCP(), self.kernel.GetConsoleOutputCP()))

    def test_worker_exception_fails_closed_and_preserves_caller(self):
        before = (self.kernel.GetConsoleCP(), self.kernel.GetConsoleOutputCP())
        result = subprocess.run([sys.executable, str(SCRIPT), '--timeout', '20', '--',
                                 'xdzip-owned-nonexistent-command-'+uuid.uuid4().hex],
                                input=b'', stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=35)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b'did not report completion', result.stderr)
        self.assertEqual(before, (self.kernel.GetConsoleCP(), self.kernel.GetConsoleOutputCP()))

    def start_tree(self, timeout):
        code = '''import os,subprocess,sys,time
p=subprocess.Popen([sys.executable,"-c","import time; time.sleep(120)"])
print(os.getpid(),p.pid,flush=True)
time.sleep(120)
'''
        process = subprocess.Popen([sys.executable, str(SCRIPT), '--timeout', str(timeout), '--',
                                    BASE, '-u', '-c', code], stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        lines = queue.Queue()
        threading.Thread(target=lambda: lines.put(process.stdout.readline()), daemon=True).start()
        try:
            line = lines.get(timeout=15)
        except queue.Empty:
            process.kill()
            process.wait()
            self.fail('owned process tree did not become ready')
        if not line:
            _, error = process.communicate(timeout=10)
            self.fail('owned process tree exited before readiness: '+repr(error))
        pids = [int(value) for value in line.split()]
        handles = [self.kernel.OpenProcess(0x00100000, False, pid) for pid in pids]
        self.assertEqual(len(handles), 2)
        self.assertTrue(all(handles))
        return process, handles

    def assert_tree_stopped(self, handles):
        try:
            for handle in handles:
                self.assertEqual(self.kernel.WaitForSingleObject(handle, 10000), 0)
        finally:
            for handle in handles:
                self.kernel.CloseHandle(handle)

    def test_timeout_stops_owned_process_tree(self):
        before = (self.kernel.GetConsoleCP(), self.kernel.GetConsoleOutputCP())
        process, handles = self.start_tree(4)
        try:
            _, error = process.communicate(timeout=15)
            self.assertEqual(process.returncode, 124, error)
            self.assertIn(b'timed out', error)
            self.assert_tree_stopped(handles)
            self.assertEqual(before, (self.kernel.GetConsoleCP(), self.kernel.GetConsoleOutputCP()))
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()

    def test_supervisor_termination_stops_owned_process_tree(self):
        process, handles = self.start_tree(30)
        try:
            process.terminate()
            process.communicate(timeout=10)
            self.assert_tree_stopped(handles)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()

    def test_two_isolated_commands_enter_concurrently(self):
        token = uuid.uuid4().hex
        names = ['Local\\xdzip-isolation-test-'+token+'-'+str(index) for index in range(3)]
        events = [self.kernel.CreateEventW(None, True, False, name) for name in names]
        self.assertTrue(all(events))
        code = '''import ctypes,sys
k=ctypes.WinDLL("kernel32"); k.OpenEventW.restype=ctypes.c_void_p
k.SetEvent.argtypes=[ctypes.c_void_p]; k.WaitForSingleObject.argtypes=[ctypes.c_void_p,ctypes.c_uint]
ready=k.OpenEventW(2,False,sys.argv[1]); gate=k.OpenEventW(0x100000,False,sys.argv[2])
assert ready and gate and k.SetEvent(ready)
assert k.WaitForSingleObject(gate,12000)==0
'''
        processes = []
        try:
            for index in range(2):
                processes.append(subprocess.Popen([sys.executable, str(SCRIPT), '--timeout', '20', '--',
                    BASE, '-c', code, names[index], names[2]], stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE))
            for event in events[:2]:
                self.assertEqual(self.kernel.WaitForSingleObject(event, 10000), 0)
            self.assertTrue(self.kernel.SetEvent(events[2]))
            for process in processes:
                _, error = process.communicate(timeout=15)
                self.assertEqual(process.returncode, 0, error)
        finally:
            self.kernel.SetEvent(events[2])
            for process in processes:
                if process.poll() is None:
                    process.kill()
                    process.wait()
            for event in events:
                self.kernel.CloseHandle(event)

    def test_virtual_environment_launcher_is_detached(self):
        directory = ROOT/'build'/'test-detached-console'/uuid.uuid4().hex
        venv.EnvBuilder(with_pip=False).create(directory)
        result = self.run_wrapper('print("venv worker succeeded")',
                                  executable=str(directory/'Scripts/python.exe'))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(b'venv worker succeeded', result.stdout)

    def test_ci_has_native_build_tools(self):
        if os.environ.get('CI', '').lower() == 'true':
            for name in ('cl', 'cmake', 'ninja'):
                self.assertIsNotNone(shutil.which(name), name+' is required for Windows CI')

    @unittest.skipUnless(shutil.which('cl') and shutil.which('cmake') and shutil.which('ninja'),
                         'requires the MSVC, CMake and Ninja development environment')
    def test_owned_msvc_ninja_header_change_rebuilds(self):
        directory = ROOT/'build'/'test-detached-console'/uuid.uuid4().hex
        source = directory/'资料𠮷'
        source.mkdir(parents=True)
        header = source/'含𠮷.h'
        header.write_text('#define OWNED_VALUE 42\n', encoding='utf-8')
        (source/'main.cpp').write_text('#include "含𠮷.h"\nint main(){return OWNED_VALUE;}\n', encoding='utf-8')
        (source/'CMakeLists.txt').write_text('cmake_minimum_required(VERSION 3.24)\n'
            'project(owned LANGUAGES CXX)\nadd_compile_options(/utf-8)\n'
            'add_executable(owned main.cpp)\n', encoding='utf-8')
        build = directory/'out'
        guard = ROOT/'scripts/windows_ninja_safety.py'

        def execute(command):
            result = subprocess.run([sys.executable, str(SCRIPT), '--timeout', '120', '--',
                sys.executable, str(guard), '--build-dir', str(build), '--', *command],
                stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=135)
            self.assertEqual(result.returncode, 0, result.stdout+result.stderr)

        execute(['cmake', '-S', str(source), '-B', str(build), '-G', 'Ninja',
                 '-DCMAKE_BUILD_TYPE=Release', '-DCMAKE_CXX_FLAGS_DEBUG=/Z7 /Od /RTC1'])
        execute(['cmake', '--build', str(build), '--parallel', '2'])
        self.assertEqual(subprocess.run([str(build/'owned.exe')]).returncode, 42)
        header.write_text('#define OWNED_VALUE 43\n', encoding='utf-8')
        execute(['cmake', '--build', str(build), '--parallel', '2'])
        self.assertEqual(subprocess.run([str(build/'owned.exe')]).returncode, 43)
        from windows_ninja_safety import check_dependencies
        check_dependencies(build, require_compiled=True)


if __name__ == '__main__':
    unittest.main()
