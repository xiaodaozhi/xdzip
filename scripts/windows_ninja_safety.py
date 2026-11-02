"""Protect Windows MSVC/Ninja builds from lost localized include dependencies."""
from contextlib import contextmanager
import argparse
import ctypes
from pathlib import Path
import re
import shutil
import subprocess
import sys


@contextmanager
def utf8_console():
    """Use UTF-8 only for this console, restoring the caller's original code pages."""
    if sys.platform != 'win32':
        yield
        return
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.CreateMutexW.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_wchar_p]
    kernel.CreateMutexW.restype = ctypes.c_void_p
    kernel.WaitForSingleObject.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
    kernel.ReleaseMutex.argtypes = [ctypes.c_void_p]
    kernel.CloseHandle.argtypes = [ctypes.c_void_p]
    kernel.GetConsoleWindow.restype = ctypes.c_void_p
    initial = (kernel.GetConsoleCP(), kernel.GetConsoleOutputCP())
    console = kernel.GetConsoleWindow() if any(initial) else None
    mutex = None
    if any(initial):
        mutex = kernel.CreateMutexW(None, False, 'Local\\xdzip-ninja-utf8-console-'+str(console or 'unknown'))
        if not mutex:
            raise OSError('Cannot coordinate Windows build consoles')
    acquired = False
    try:
        if mutex:
            result = kernel.WaitForSingleObject(mutex, 30000)
            if result not in (0, 0x80):  # Acquired, including an abandoned mutex.
                raise OSError('Another Windows build is using the console; retry later')
            acquired = True
        original = (kernel.GetConsoleCP(), kernel.GetConsoleOutputCP())
        allocated = False
        try:
            if not any(original):
                if not kernel.AllocConsole():
                    raise OSError('Cannot create a temporary console for MSVC')
                allocated = True
                user = ctypes.WinDLL('user32', use_last_error=True)
                user.ShowWindow.argtypes = [ctypes.c_void_p, ctypes.c_int]
                user.ShowWindow(kernel.GetConsoleWindow(), 0)
            elif not all(original):
                raise OSError('Cannot determine both Windows console code pages')
            if not kernel.SetConsoleCP(65001) or not kernel.SetConsoleOutputCP(65001):
                raise OSError('Cannot select UTF-8 for the MSVC console')
            yield
        finally:
            if allocated:
                if not kernel.FreeConsole():
                    raise OSError('Cannot release the temporary MSVC console')
            elif all(original):
                restored_input = kernel.SetConsoleCP(original[0])
                restored_output = kernel.SetConsoleOutputCP(original[1])
                if not restored_input or not restored_output:
                    raise OSError('Cannot restore the original console code pages')
    finally:
        released = kernel.ReleaseMutex(mutex) if acquired else True
        closed = kernel.CloseHandle(mutex) if mutex else True
        if not released or not closed:
            raise OSError('Cannot release the Windows build console lock')


_ENTRY = re.compile(r'^(.+\.(?:obj|o)): #deps (\d+),', re.IGNORECASE)
_TARGET = re.compile(r'^(.+\.(?:obj|o)):\s', re.IGNORECASE)
_INCLUDE = re.compile(r'^\s*#\s*include\s*[<"]', re.MULTILINE)


def make_program(build):
    cache = build/'CMakeCache.txt'
    if not cache.is_file():
        raise ValueError('Ninja build configuration is missing')
    lines = cache.read_text(encoding='utf-8', errors='replace').splitlines()
    values = [line.split('=', 1)[1] for line in lines
              if line.startswith(('CMAKE_MAKE_PROGRAM:FILEPATH=', 'CMAKE_MAKE_PROGRAM:STRING=',
                                  'CMAKE_MAKE_PROGRAM:UNINITIALIZED='))]
    if len(values) != 1:
        raise ValueError('Cannot identify the configured Ninja executable')
    executable = values[0] if Path(values[0]).is_file() else shutil.which(values[0])
    if not executable:
        raise ValueError('Cannot identify the configured Ninja executable')
    return executable


def check_dependencies(build, *, require_compiled=False):
    """Reject existing C/C++ objects with includes but no Ninja dependency records."""
    if sys.platform != 'win32':
        return
    build = Path(build)
    if not (build/'build.ninja').is_file():
        if require_compiled or (build.exists() and any(build.rglob('*.obj'))):
            raise ValueError('Ninja build graph is missing')
        return
    ninja = make_program(build)
    def run(*args):
        result = subprocess.run([ninja, '-C', str(build), '-t', *args],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                encoding='utf-8', errors='replace', check=False)
        if result.returncode:
            raise ValueError('Cannot inspect Windows Ninja dependencies')
        return result.stdout
    records = {}
    for line in run('deps').splitlines():
        match = _ENTRY.match(line)
        if match:
            records[match.group(1).replace('\\', '/').lower()] = int(match.group(2))
    targets = []
    for line in run('targets', 'all').splitlines():
        match = _TARGET.match(line)
        if match and (build/match.group(1)).is_file():
            targets.append(match.group(1))
    for target in targets:
        if records.get(target.replace('\\', '/').lower(), 0):
            continue
        query = subprocess.run([ninja, '-C', str(build), '-t', 'query', target],
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               encoding='utf-8', errors='replace', check=False)
        if query.returncode:
            raise ValueError('Cannot inspect a Windows Ninja object target')
        sources = [line.strip() for line in query.stdout.splitlines()
                   if line.startswith('    ') and not line.strip().startswith(('|', '||'))
                   and line.strip().lower().endswith(('.c', '.cc', '.cpp', '.cxx'))]
        for source in sources:
            path = Path(source)
            if not path.is_absolute():
                path = build/path
            if not path.is_file():
                raise ValueError('Cannot inspect a Windows Ninja source dependency')
            if _INCLUDE.search(path.read_text(encoding='utf-8', errors='replace')):
                raise ValueError('Windows Ninja omitted header dependencies; use a new build directory')


def main():
    parser = argparse.ArgumentParser(description='Run a build command with Windows Ninja safeguards')
    parser.add_argument('--build-dir', required=True)
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command:
        parser.error('a command is required after --')
    with utf8_console():
        check_dependencies(args.build_dir)
        subprocess.run(command, check=True)
        check_dependencies(args.build_dir)


if __name__ == '__main__':
    main()
