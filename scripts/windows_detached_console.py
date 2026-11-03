"""Run a non-interactive command in its own guarded Windows build console."""

import argparse
import ctypes
from ctypes import wintypes
from pathlib import Path
import subprocess
import sys
import uuid

from windows_ninja_safety import utf8_console


class BasicLimit(ctypes.Structure):
    _fields_ = [('PerProcessUserTimeLimit', ctypes.c_int64),
                ('PerJobUserTimeLimit', ctypes.c_int64),
                ('LimitFlags', wintypes.DWORD),
                ('MinimumWorkingSetSize', ctypes.c_size_t),
                ('MaximumWorkingSetSize', ctypes.c_size_t),
                ('ActiveProcessLimit', wintypes.DWORD),
                ('Affinity', ctypes.c_size_t),
                ('PriorityClass', wintypes.DWORD),
                ('SchedulingClass', wintypes.DWORD)]


class IoCounters(ctypes.Structure):
    _fields_ = [(name, ctypes.c_uint64) for name in
                ('ReadOperationCount', 'WriteOperationCount', 'OtherOperationCount',
                 'ReadTransferCount', 'WriteTransferCount', 'OtherTransferCount')]


class ExtendedLimit(ctypes.Structure):
    _fields_ = [('BasicLimitInformation', BasicLimit), ('IoInfo', IoCounters),
                ('ProcessMemoryLimit', ctypes.c_size_t),
                ('JobMemoryLimit', ctypes.c_size_t),
                ('PeakProcessMemoryUsed', ctypes.c_size_t),
                ('PeakJobMemoryUsed', ctypes.c_size_t)]


def windows_api():
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.CreateJobObjectW.argtypes = [ctypes.c_void_p, wintypes.LPCWSTR]
    kernel.CreateJobObjectW.restype = wintypes.HANDLE
    kernel.SetInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int,
                                               ctypes.c_void_p, wintypes.DWORD]
    kernel.AssignProcessToJobObject.argtypes = [wintypes.HANDLE, wintypes.HANDLE]
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CreateEventW.argtypes = [ctypes.c_void_p, wintypes.BOOL, wintypes.BOOL, wintypes.LPCWSTR]
    kernel.CreateEventW.restype = wintypes.HANDLE
    kernel.OpenEventW.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.LPCWSTR]
    kernel.OpenEventW.restype = wintypes.HANDLE
    kernel.SetEvent.argtypes = [wintypes.HANDLE]
    kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    return kernel


def checked(value, message):
    if not value:
        raise OSError(ctypes.get_last_error(), message)
    return value


def worker(gate_name, done_name, command):
    kernel = windows_api()
    gate = checked(kernel.OpenEventW(0x00100000, False, gate_name), 'Cannot open worker gate')
    done = None
    try:
        done = checked(kernel.OpenEventW(0x0002, False, done_name), 'Cannot open completion signal')
        if kernel.WaitForSingleObject(gate, 15000) != 0:
            raise OSError('Supervisor did not start the worker')
        if (kernel.GetConsoleCP(), kernel.GetConsoleOutputCP()) != (0, 0):
            raise OSError('Worker was not detached from its caller console')
        with utf8_console():
            result = subprocess.run(command, check=False).returncode
        if (kernel.GetConsoleCP(), kernel.GetConsoleOutputCP()) != (0, 0):
            raise OSError('Worker did not release its temporary console')
        checked(kernel.SetEvent(done), 'Cannot signal worker completion')
        return result if 0 <= result <= 255 else 1
    finally:
        if done:
            kernel.CloseHandle(done)
        kernel.CloseHandle(gate)


def run_isolated(command, timeout):
    kernel = windows_api()
    token = uuid.uuid4().hex
    gate_name = 'Local\\xdzip-build-gate-' + token
    done_name = 'Local\\xdzip-build-done-' + token
    handles = []
    process = None
    try:
        for name in (gate_name, done_name):
            ctypes.set_last_error(0)
            handle = checked(kernel.CreateEventW(None, True, False, name), 'Cannot create worker signal')
            handles.append(handle)
            if ctypes.get_last_error() == 183:
                raise OSError('Worker signal already exists')
        gate, done = handles
        job = checked(kernel.CreateJobObjectW(None, None), 'Cannot create worker job')
        handles.append(job)
        limits = ExtendedLimit()
        limits.BasicLimitInformation.LimitFlags = 0x2000  # Kill owned processes when the job closes.
        checked(kernel.SetInformationJobObject(job, 9, ctypes.byref(limits), ctypes.sizeof(limits)),
                'Cannot configure worker job')
        base_python = getattr(sys, '_base_executable', None) or sys.executable
        if not Path(base_python).is_file():
            raise OSError('Cannot find the base Python interpreter')
        startup = subprocess.STARTUPINFO()
        startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = subprocess.SW_HIDE
        process = subprocess.Popen([base_python, '-u', str(Path(__file__).resolve()), '--worker',
                                    gate_name, done_name, *command],
                                   stdin=sys.stdin, stdout=sys.stdout, stderr=sys.stderr,
                                   creationflags=subprocess.DETACHED_PROCESS, startupinfo=startup)
        checked(kernel.AssignProcessToJobObject(job, wintypes.HANDLE(int(process._handle))),
                'Cannot assign worker to its job')
        checked(kernel.SetEvent(gate), 'Cannot start worker')
        try:
            result = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            print('Isolated build timed out; stopping its process tree.', file=sys.stderr)
            return 124
        if kernel.WaitForSingleObject(done, 0) != 0:
            print('Isolated build did not report completion.', file=sys.stderr)
            return 1
        return result if 0 <= result <= 255 else 1
    finally:
        # Close the job before waiting, including on timeout or supervisor exceptions.
        for handle in reversed(handles):
            kernel.CloseHandle(handle)
        if process and process.poll() is None:
            process.kill()
            process.wait()


def main():
    if sys.platform != 'win32':
        raise OSError('Isolated build consoles are available only on Windows')
    if len(sys.argv) >= 5 and sys.argv[1] == '--worker':
        return worker(sys.argv[2], sys.argv[3], sys.argv[4:])
    parser = argparse.ArgumentParser(description='Run a non-interactive command in an isolated Windows console')
    parser.add_argument('--timeout', type=int, default=7200, help='maximum seconds, default: 7200')
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command or args.timeout < 1:
        parser.error('a command and a positive timeout are required')
    import msvcrt
    kernel = windows_api()
    kernel.GetConsoleMode.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
    for stream in (sys.stdin, sys.stdout, sys.stderr):
        if stream is None:
            parser.error('standard input, output and error must be redirected')
        mode = wintypes.DWORD()
        if kernel.GetConsoleMode(msvcrt.get_osfhandle(stream.fileno()), ctypes.byref(mode)):
            parser.error('standard input, output and error must be redirected')
    return run_isolated(command, args.timeout)


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except OSError as error:
        print('Isolated build failed: '+str(error), file=sys.stderr)
        raise SystemExit(1)
