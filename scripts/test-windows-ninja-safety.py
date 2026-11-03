#!/usr/bin/env python3
"""Check Windows console restoration and Ninja dependency rejection."""
import sys
import shutil
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

import windows_ninja_safety as safety


class Console:
    def __init__(self):
        self.input, self.output = 936, 65001
        self.CreateMutexW = Mock(return_value=1)
        self.WaitForSingleObject = Mock(return_value=0)
        self.ReleaseMutex = Mock(return_value=1)
        self.CloseHandle = Mock(return_value=1)
        self.GetConsoleWindow = Mock(return_value=None)
        self.processes = [100]
        self.GetConsoleProcessList = Mock(side_effect=self.process_list)
        self.GetCurrentProcessId = Mock(return_value=100)

    def process_list(self, buffer, size):
        if len(self.processes) <= size:
            for index, process in enumerate(self.processes):
                buffer[index] = process
        return len(self.processes)

    def GetConsoleCP(self): return self.input
    def GetConsoleOutputCP(self): return self.output
    def SetConsoleCP(self, value): self.input = value; return 1
    def SetConsoleOutputCP(self, value): self.output = value; return 1


class SafetyTests(unittest.TestCase):
    def test_console_restores_both_code_pages_after_failure(self):
        console = Console()
        with patch.object(safety.sys, 'platform', 'win32'), \
             patch.object(safety.ctypes, 'WinDLL', return_value=console, create=True):
            with self.assertRaisesRegex(RuntimeError, 'sentinel'):
                with safety.utf8_console():
                    self.assertEqual((console.input, console.output), (65001, 65001))
                    raise RuntimeError('sentinel')
        self.assertEqual((console.input, console.output), (936, 65001))
        console.ReleaseMutex.assert_called_once_with(1)
        console.CloseHandle.assert_called_once_with(1)

    def test_console_lock_timeout_does_not_change_code_pages(self):
        console = Console()
        console.WaitForSingleObject.return_value = 258
        with patch.object(safety.sys, 'platform', 'win32'), \
             patch.object(safety.ctypes, 'WinDLL', return_value=console, create=True):
            with self.assertRaisesRegex(OSError, 'retry later'):
                with safety.utf8_console():
                    self.fail('must not enter on timeout')
        self.assertEqual((console.input, console.output), (936, 65001))
        console.ReleaseMutex.assert_not_called()
        console.CloseHandle.assert_called_once_with(1)

    def test_console_process_list_grows_and_requires_current_process(self):
        console = Console()
        console.processes = list(range(1, 101))
        self.assertEqual(safety.console_processes(console), console.processes)
        self.assertEqual(console.GetConsoleProcessList.call_count, 2)
        console.processes = [1, 2]
        with self.assertRaisesRegex(OSError, 'Cannot identify'):
            safety.console_processes(console)
        console.processes = []
        with self.assertRaisesRegex(OSError, 'Cannot identify'):
            safety.console_processes(console)

    def test_partial_console_lock_timeout_releases_every_handle(self):
        console = Console()
        console.processes = [200, 100]
        console.CreateMutexW.side_effect = [11, 22]
        console.WaitForSingleObject.side_effect = [0, 258]
        with self.assertRaisesRegex(OSError, 'retry later'):
            with safety.console_locks(console):
                self.fail('must not enter on timeout')
        self.assertEqual([call.args[2] for call in console.CreateMutexW.call_args_list],
                         ['Local\\xdzip-ninja-utf8-process-100',
                          'Local\\xdzip-ninja-utf8-process-200'])
        console.ReleaseMutex.assert_called_once_with(11)
        self.assertEqual([call.args[0] for call in console.CloseHandle.call_args_list], [22, 11])

    def test_abandoned_console_lock_is_released(self):
        console = Console()
        console.WaitForSingleObject.return_value = 0x80
        with safety.console_locks(console):
            pass
        console.ReleaseMutex.assert_called_once_with(1)
        console.CloseHandle.assert_called_once_with(1)

    def test_console_with_window_also_acquires_legacy_lock(self):
        console = Console()
        console.GetConsoleWindow.return_value = 12345
        console.CreateMutexW.side_effect = [11, 22]
        with patch.object(safety.sys, 'platform', 'win32'), \
             patch.object(safety.ctypes, 'WinDLL', return_value=console, create=True):
            with safety.utf8_console():
                self.assertEqual((console.input, console.output), (65001, 65001))
        self.assertEqual([call.args[2] for call in console.CreateMutexW.call_args_list],
                         ['Local\\xdzip-ninja-utf8-process-100',
                          'Local\\xdzip-ninja-utf8-console-12345'])
        self.assertEqual([call.args[0] for call in console.ReleaseMutex.call_args_list], [22, 11])
        self.assertEqual([call.args[0] for call in console.CloseHandle.call_args_list], [22, 11])

    def test_failed_console_snapshot_does_not_change_code_pages(self):
        console = Console()
        console.processes = []
        with patch.object(safety.sys, 'platform', 'win32'), \
             patch.object(safety.ctypes, 'WinDLL', return_value=console, create=True):
            with self.assertRaisesRegex(OSError, 'Cannot identify'):
                with safety.utf8_console():
                    self.fail('must not enter without console membership')
        self.assertEqual((console.input, console.output), (936, 65001))
        console.CreateMutexW.assert_not_called()

    def test_guard_rejects_missing_header_dependencies(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'build.ninja').write_text('')
            ninja = root/'ninja.exe'
            ninja.write_text('')
            (root/'CMakeCache.txt').write_text('CMAKE_MAKE_PROGRAM:UNINITIALIZED='+str(ninja)+'\n')
            obj = root/'CMakeFiles/probe.dir/main.cpp.obj'
            obj.parent.mkdir(parents=True)
            obj.write_bytes(b'object')
            source = root/'main.cpp'
            source.write_text('#include "sentinel.h"\nint main() { return 1; }\n')
            target = 'CMakeFiles/probe.dir/main.cpp.obj'
            def output(*args, **kwargs):
                command = args[0]
                if command[-1] == 'deps':
                    text = target+': #deps 0, deps mtime 1 (VALID)\n'
                elif command[-2:] == ['targets', 'all']:
                    text = target+': CXX_COMPILER\n'
                else:
                    text = target+':\n  input: rule\n    '+str(source)+'\n  outputs:\n'
                return SimpleNamespace(returncode=0, stdout=text, stderr='')
            with patch.object(safety.sys, 'platform', 'win32'), \
                 patch.object(safety.subprocess, 'run', side_effect=output):
                with self.assertRaisesRegex(ValueError, 'omitted header dependencies'):
                    safety.check_dependencies(root, require_compiled=True)
                source.write_text('#include <windows.h>\nint main() { return 1; }\n')
                safety.check_dependencies(root, require_compiled=True)
                (root/'sentinel.h').write_text('')
                source.write_text('#include <sentinel.h>\nint main() { return 1; }\n')
                with self.assertRaisesRegex(ValueError, 'omitted header dependencies'):
                    safety.check_dependencies(root, require_compiled=True)

    def test_guard_rejects_active_object_with_includes_and_no_record(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'build.ninja').write_text('')
            ninja = root/'ninja.exe'
            ninja.write_text('')
            (root/'CMakeCache.txt').write_text('CMAKE_MAKE_PROGRAM:FILEPATH='+str(ninja)+'\n')
            (root/'main.obj').write_bytes(b'object')
            source = root/'main.cpp'
            source.write_text('#include "sentinel.h"\n')
            def output(*args, **kwargs):
                command = args[0]
                if command[-1] == 'deps':
                    text = ''
                elif command[-2:] == ['targets', 'all']:
                    text = 'main.obj: CXX_COMPILER\n'
                else:
                    text = 'main.obj:\n  input: rule\n    '+str(source)+'\n  outputs:\n'
                return SimpleNamespace(returncode=0, stdout=text, stderr='')
            with patch.object(safety.sys, 'platform', 'win32'), \
                 patch.object(safety.subprocess, 'run', side_effect=output):
                for required in (False, True):
                    with self.assertRaisesRegex(ValueError, 'omitted header dependencies'):
                        safety.check_dependencies(root, require_compiled=required)

    def test_configure_probe_objects_are_not_build_targets(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'build.ninja').write_text('')
            ninja = root/'ninja.exe'
            ninja.write_text('')
            (root/'CMakeCache.txt').write_text('CMAKE_MAKE_PROGRAM:FILEPATH='+str(ninja)+'\n')
            probe = root/'CMakeFiles/ShowIncludes/main.obj'
            probe.parent.mkdir(parents=True)
            probe.write_bytes(b'object')
            result = SimpleNamespace(returncode=0, stdout='', stderr='')
            with patch.object(safety.sys, 'platform', 'win32'), \
                 patch.object(safety.subprocess, 'run', return_value=result):
                safety.check_dependencies(root)

    def test_guard_rejects_existing_objects_without_build_graph(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'main.obj').write_bytes(b'object')
            with patch.object(safety.sys, 'platform', 'win32'):
                with self.assertRaisesRegex(ValueError, 'graph is missing'):
                    safety.check_dependencies(root)

    def test_make_program_resolves_executable_on_path(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'CMakeCache.txt').write_text('CMAKE_MAKE_PROGRAM:STRING=ninja\n')
            with patch.object(safety.shutil, 'which', return_value='C:/tools/ninja.exe'):
                self.assertEqual(safety.make_program(root), 'C:/tools/ninja.exe')

    @unittest.skipUnless(sys.platform == 'win32', 'requires Windows console APIs')
    def test_detached_process_restores_no_console_and_keeps_output(self):
        code = '''import ctypes
from windows_ninja_safety import utf8_console
k = ctypes.WinDLL("kernel32")
assert (k.GetConsoleCP(), k.GetConsoleOutputCP()) == (0, 0)
with utf8_console():
    assert (k.GetConsoleCP(), k.GetConsoleOutputCP()) == (65001, 65001)
assert (k.GetConsoleCP(), k.GetConsoleOutputCP()) == (0, 0)
print("detached console restored")
'''
        result = subprocess.run([sys.executable, '-c', code], cwd=Path(__file__).parent,
                                creationflags=subprocess.DETACHED_PROCESS,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('detached console restored', result.stdout)

    @unittest.skipUnless(sys.platform == 'win32', 'requires Windows console APIs')
    def test_independent_hidden_consoles_do_not_block_each_other(self):
        worker_code = '''import ctypes, sys, time
from pathlib import Path
sys.path.insert(0, sys.argv[3])
from windows_ninja_safety import utf8_console
k = ctypes.WinDLL("kernel32")
assert not k.GetConsoleWindow()
original = (k.GetConsoleCP(), k.GetConsoleOutputCP())
assert all(original)
with utf8_console():
    assert (k.GetConsoleCP(), k.GetConsoleOutputCP()) == (65001, 65001)
    Path(sys.argv[1]).touch()
    if sys.argv[2] != "-":
        release = Path(sys.argv[2])
        end = time.monotonic() + 12
        while not release.exists() and time.monotonic() < end: time.sleep(.02)
        assert release.exists(), "holder was not released"
assert (k.GetConsoleCP(), k.GetConsoleOutputCP()) == original
'''
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            worker, release = root/'worker.py', root/'release'
            worker.write_text(worker_code)
            scripts = str(Path(__file__).parent)
            def start(entered, gate):
                return subprocess.Popen([sys.executable, str(worker), str(entered), str(gate), scripts],
                                        creationflags=subprocess.CREATE_NO_WINDOW,
                                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            def wait_for(path, process):
                end = time.monotonic() + 8
                while not path.exists() and process.poll() is None and time.monotonic() < end:
                    time.sleep(.02)
                self.assertTrue(path.exists(), 'worker failed to enter its own console')
            holder = start(root/'holder', release)
            waiter = None
            try:
                wait_for(root/'holder', holder)
                waiter = start(root/'waiter', '-')
                wait_for(root/'waiter', waiter)
                self.assertIsNone(holder.poll(), 'holder must still be in its guarded console')
            finally:
                release.touch()
                for process in (holder, waiter):
                    if process is not None:
                        out, err = process.communicate(timeout=15)
                        self.assertEqual(process.returncode, 0, out+err)

    @unittest.skipUnless(sys.platform == 'win32', 'requires Windows console APIs')
    def test_two_processes_sharing_console_restore_original_code_pages(self):
        worker_code = '''import sys, time
from pathlib import Path
sys.path.insert(0, sys.argv[3])
from windows_ninja_safety import utf8_console
marker = Path(sys.argv[2])
if sys.argv[1] == "holder":
    with utf8_console():
        print("entered", flush=True)
        for _ in range(100):
            if marker.exists(): break
            time.sleep(.05)
        else: raise RuntimeError("waiter did not start")
        time.sleep(.7)
else:
    marker.touch()
    start = time.monotonic()
    with utf8_console():
        print(time.monotonic()-start, flush=True)
'''
        broker_code = '''import ctypes, subprocess, sys
k = ctypes.WinDLL("kernel32")
assert k.SetConsoleCP(936) and k.SetConsoleOutputCP(936)
args = [sys.executable, sys.argv[1]]
holder = subprocess.Popen(args+["holder", sys.argv[2], sys.argv[3]], stdout=subprocess.PIPE, text=True)
assert holder.stdout.readline().strip() == "entered"
waiter = subprocess.run(args+["waiter", sys.argv[2], sys.argv[3]], capture_output=True, text=True, timeout=15)
assert waiter.returncode == 0, waiter.stderr
assert holder.wait(timeout=15) == 0
assert float(waiter.stdout.strip()) >= .4, waiter.stdout
assert (k.GetConsoleCP(), k.GetConsoleOutputCP()) == (936, 936)
print("shared console serialized and restored")
'''
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            worker, broker = root/'worker.py', root/'broker.py'
            worker.write_text(worker_code)
            broker.write_text(broker_code)
            result = subprocess.run([sys.executable, str(broker), str(worker), str(root/'ready'),
                                     str(Path(__file__).parent)], creationflags=subprocess.CREATE_NO_WINDOW,
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=25)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('shared console serialized and restored', result.stdout)

    @unittest.skipUnless(sys.platform == 'win32', 'requires Windows console APIs')
    def test_shared_console_timeout_and_exception_restore_code_pages(self):
        worker_code = '''import ctypes, itertools, sys, time
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0, sys.argv[3])
import windows_ninja_safety as safety
k = ctypes.WinDLL("kernel32")
marker = Path(sys.argv[2])
if sys.argv[1] == "holder":
    try:
        with safety.utf8_console():
            print("entered", flush=True)
            limit = time.monotonic()+10
            while not marker.exists() and time.monotonic()<limit: time.sleep(.02)
            assert marker.exists()
            raise RuntimeError("owned failure")
    except RuntimeError as e:
        assert str(e) == "owned failure"
    assert (k.GetConsoleCP(), k.GetConsoleOutputCP()) == (936, 936)
else:
    original = (k.GetConsoleCP(), k.GetConsoleOutputCP())
    ticks = itertools.chain([0], itertools.repeat(31))
    with patch.object(safety.time, "monotonic", side_effect=lambda: next(ticks)):
        try:
            with safety.utf8_console(): raise AssertionError("unexpected entry")
        except OSError as e: assert "retry later" in str(e)
    assert (k.GetConsoleCP(), k.GetConsoleOutputCP()) == original
    assert sys.stdin.read() == "owned input"
    sys.stdout.buffer.write("owned output 资料𠮷".encode("utf-8"))
    marker.touch()
'''
        broker_code = '''import ctypes, subprocess, sys
k = ctypes.WinDLL("kernel32")
assert k.SetConsoleCP(936) and k.SetConsoleOutputCP(936)
args = [sys.executable, sys.argv[1]]
holder = subprocess.Popen(args+["holder", sys.argv[2], sys.argv[3]], stdout=subprocess.PIPE, text=True)
assert holder.stdout.readline().strip() == "entered"
waiter = subprocess.run(args+["waiter", sys.argv[2], sys.argv[3]], input=b"owned input",
                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=15)
assert waiter.returncode == 0, waiter.stderr
assert waiter.stdout == "owned output 资料𠮷".encode("utf-8")
assert holder.wait(timeout=15) == 0
assert (k.GetConsoleCP(), k.GetConsoleOutputCP()) == (936, 936)
print("timeout and exception restored shared console")
'''
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            worker, broker = root/'worker.py', root/'broker.py'
            worker.write_text(worker_code, encoding='utf-8')
            broker.write_text(broker_code, encoding='utf-8')
            result = subprocess.run([sys.executable, str(broker), str(worker), str(root/'ready'),
                                     str(Path(__file__).parent)], creationflags=subprocess.CREATE_NO_WINDOW,
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=25)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('timeout and exception restored shared console', result.stdout)

    @unittest.skipUnless(sys.platform == 'win32' and all(shutil.which(x) for x in ('cl', 'cmake', 'ninja')),
                         'requires the Windows MSVC, CMake and Ninja development environment')
    def test_parallel_hidden_msvc_builds_retain_header_dependencies(self):
        worker_code = '''import ctypes, subprocess, sys, time
from pathlib import Path
root, mine, other = map(Path, sys.argv[1:4])
sys.path.insert(0, sys.argv[4])
from windows_ninja_safety import check_dependencies, utf8_console
k = ctypes.WinDLL("kernel32")
original = (k.GetConsoleCP(), k.GetConsoleOutputCP())
assert all(original) and not k.GetConsoleWindow()
source = root/"资料𠮷"
source.mkdir(parents=True)
header = source/"含𠮷.h"
header.write_text("#define OWNED_VALUE 42\\n", encoding="utf-8")
(source/"main.cpp").write_text('#include "含𠮷.h"\\nint main(){return OWNED_VALUE;}\\n', encoding="utf-8")
(source/"CMakeLists.txt").write_text("cmake_minimum_required(VERSION 3.24)\\n"
    "project(owned LANGUAGES CXX)\\nadd_compile_options(/utf-8)\\n"
    "add_executable(owned main.cpp)\\n", encoding="utf-8")
build = root/"out"
def run(*cmd):
    result = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if result.returncode:
        sys.stderr.buffer.write(result.stdout)
        raise RuntimeError("owned build command failed")
with utf8_console():
    mine.touch()
    end = time.monotonic()+25
    while not other.exists() and time.monotonic()<end: time.sleep(.02)
    assert other.exists(), "independent builds did not enter concurrently"
    run("cmake", "-S", str(source), "-B", str(build), "-G", "Ninja",
        "-DCMAKE_BUILD_TYPE=Release", "-DCMAKE_CXX_FLAGS_DEBUG=/Z7 /Od /RTC1")
    run("cmake", "--build", str(build), "--parallel", "2")
    assert subprocess.run([str(build/"owned.exe")]).returncode == 42
    check_dependencies(build, require_compiled=True)
    header.write_text("#define OWNED_VALUE 43\\n", encoding="utf-8")
    run("cmake", "--build", str(build), "--parallel", "2")
    assert subprocess.run([str(build/"owned.exe")]).returncode == 43
    check_dependencies(build, require_compiled=True)
assert (k.GetConsoleCP(), k.GetConsoleOutputCP()) == original
print("parallel build and rebuild ok")
'''
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            worker = root/'worker.py'
            worker.write_text(worker_code, encoding='utf-8')
            processes = []
            for index in (0, 1):
                processes.append(subprocess.Popen(
                    [sys.executable, str(worker), str(root/str(index)), str(root/f'entered-{index}'),
                     str(root/f'entered-{1-index}'), str(Path(__file__).parent)],
                    stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                    creationflags=subprocess.CREATE_NO_WINDOW))
            try:
                outputs = [process.communicate(timeout=120) for process in processes]
            finally:
                for process in processes:
                    if process.poll() is None:
                        process.kill()
                        process.communicate()
            for process, (out, err) in zip(processes, outputs):
                self.assertEqual(process.returncode, 0, out+err)
                self.assertIn(b'parallel build and rebuild ok', out)


if __name__ == '__main__':
    unittest.main()
