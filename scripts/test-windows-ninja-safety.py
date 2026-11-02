#!/usr/bin/env python3
"""Check Windows console restoration and Ninja dependency rejection."""
import sys
import subprocess
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import windows_ninja_safety as safety


class Console:
    def __init__(self):
        self.input, self.output = 936, 65001

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
                source.write_text('int main() { return 1; }\n')
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


if __name__ == '__main__':
    unittest.main()
