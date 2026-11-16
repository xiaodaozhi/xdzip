#!/usr/bin/env python3
"""Drive only the owned SFX's real system dialogs on an isolated Win32 desktop."""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
import uuid


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert os.name == 'nt'
    root = Path(__file__).resolve().parents[1]
    output = args.output.resolve()
    assert output.is_relative_to(root / 'build') and output != root / 'build' and not output.exists()
    output.mkdir(parents=True)
    sid = subprocess.check_output(['whoami', '/user', '/fo', 'csv', '/nh'], text=True).split(',')[1].strip('"\r\n')
    subprocess.run(['icacls', str(output), '/inheritance:r', '/grant:r', '*' + sid + ':(OI)(CI)F'], check=True, stdout=subprocess.DEVNULL)
    source = output / 'source'
    source.mkdir()
    # A large stored file makes cancellation observable while all completed files remain verifiable.
    with (source / 'payload.bin').open('wb') as stream:
        for _ in range(64):
            stream.write(bytes(range(256)) * 4096)
    (source / 'must-not-run.cmd').write_text('@echo unsafe>"%~dp0EXECUTED"\n', encoding='ascii')
    image = output / 'owned.sfx.exe'
    made = subprocess.run([str(args.cli.resolve()), 'create', str(image), str(source), '--level', 'store', '--json'], capture_output=True, timeout=120)
    assert made.returncode == 0, made.stdout
    user = c.WinDLL('user32', use_last_error=True)
    callback_type = c.WINFUNCTYPE(w.BOOL, w.HWND, w.LPARAM)
    user.CreateDesktopW.argtypes = [w.LPCWSTR, w.LPCWSTR, c.c_void_p, w.DWORD, w.DWORD, c.c_void_p]
    user.CreateDesktopW.restype = w.HANDLE
    user.CloseDesktop.argtypes = [w.HANDLE]
    user.EnumDesktopWindows.argtypes = [w.HANDLE, callback_type, w.LPARAM]
    user.EnumChildWindows.argtypes = [w.HWND, callback_type, w.LPARAM]
    user.GetWindowThreadProcessId.argtypes = [w.HWND, c.POINTER(w.DWORD)]
    user.GetClassNameW.argtypes = [w.HWND, w.LPWSTR, c.c_int]
    user.GetWindowTextW.argtypes = [w.HWND, w.LPWSTR, c.c_int]
    user.GetDlgCtrlID.argtypes = [w.HWND]
    user.PostMessageW.argtypes = [w.HWND, w.UINT, w.WPARAM, w.LPARAM]
    user.SendMessageW.argtypes = [w.HWND, w.UINT, w.WPARAM, w.LPARAM]
    user.SendMessageW.restype = w.LPARAM
    kernel = c.WinDLL('kernel32', use_last_error=True)
    class Startup(c.Structure):
        _fields_ = [('cb', w.DWORD), ('reserved', w.LPWSTR), ('desktop', w.LPWSTR),
                    ('title', w.LPWSTR), ('x', w.DWORD), ('y', w.DWORD),
                    ('width', w.DWORD), ('height', w.DWORD), ('chars_x', w.DWORD),
                    ('chars_y', w.DWORD), ('fill', w.DWORD), ('flags', w.DWORD),
                    ('show', w.WORD), ('reserved_size', w.WORD), ('reserved_data', c.c_void_p),
                    ('stdin', w.HANDLE), ('stdout', w.HANDLE), ('stderr', w.HANDLE)]
    class ProcessInfo(c.Structure):
        _fields_ = [('process', w.HANDLE), ('thread', w.HANDLE), ('pid', w.DWORD), ('tid', w.DWORD)]
    kernel.CreateProcessW.argtypes = [w.LPCWSTR, w.LPWSTR, c.c_void_p, c.c_void_p, w.BOOL,
                                     w.DWORD, c.c_void_p, w.LPCWSTR, c.POINTER(Startup), c.POINTER(ProcessInfo)]
    kernel.WaitForSingleObject.argtypes = [w.HANDLE, w.DWORD]
    kernel.GetExitCodeProcess.argtypes = [w.HANDLE, c.POINTER(w.DWORD)]
    kernel.CloseHandle.argtypes = [w.HANDLE]
    kernel.TerminateProcess.argtypes = [w.HANDLE, w.UINT]
    class Process:
        # Python STARTUPINFO does not forward lpDesktop. Use the actual Win32 field.
        def __init__(self, image, desktop, environment):
            startup = Startup(cb=c.sizeof(Startup), desktop=desktop)
            info = ProcessInfo()
            command = c.create_unicode_buffer(subprocess.list2cmdline([str(image)]))
            env = c.create_unicode_buffer('\0'.join(key + '=' + value for key, value in sorted(environment.items(), key=lambda item: item[0].upper())) + '\0\0')
            assert kernel.CreateProcessW(str(image), command, None, None, False, 0x400,
                                         env, None, c.byref(startup), c.byref(info)), c.get_last_error()
            self.handle, self.pid, self.returncode = info.process, info.pid, None
            kernel.CloseHandle(info.thread)
        def poll(self):
            if kernel.WaitForSingleObject(self.handle, 0) == 0:
                code = w.DWORD()
                assert kernel.GetExitCodeProcess(self.handle, c.byref(code))
                self.returncode = code.value
            return self.returncode
        def wait(self, timeout):
            assert kernel.WaitForSingleObject(self.handle, int(timeout * 1000)) == 0, 'SFX did not exit'
            return self.poll()
        def kill(self):
            kernel.TerminateProcess(self.handle, 1)
    name = 'xdzip-sfx-test-' + uuid.uuid4().hex
    desktop = user.CreateDesktopW(name, None, None, 0, 0x1ff, None)
    assert desktop, c.get_last_error()
    checks = []

    def children(hwnd):
        found = []
        @callback_type
        def visit(child, _):
            buffer = c.create_unicode_buffer(256)
            user.GetClassNameW(child, buffer, len(buffer))
            found.append((child, buffer.value, user.GetDlgCtrlID(child)))
            return True
        user.EnumChildWindows(hwnd, visit, 0)
        return found

    def windows(process):
        found = []
        @callback_type
        def visit(hwnd, _):
            pid = w.DWORD()
            user.GetWindowThreadProcessId(hwnd, c.byref(pid))
            if pid.value == process.pid:
                found.append((hwnd, children(hwnd)))
            return True
        user.EnumDesktopWindows(desktop, visit, 0)
        return found

    def wait(process, predicate, excluded=()):
        end = time.monotonic() + 30
        while time.monotonic() < end:
            for hwnd, controls in windows(process):
                if hwnd not in excluded and predicate(controls):
                    return hwnd, controls
            assert process.poll() is None, 'owned SFX exited before the expected native dialog: ' + str(process.returncode)
            time.sleep(.01)
        raise AssertionError('native dialog timeout: ' + repr([[(cls, ident) for _, cls, ident in items] for _, items in windows(process)]))

    def click(hwnd, controls, identifier):
        buttons = [h for h, cls, ident in controls if cls.lower() == 'button' and ident == identifier]
        task_dialog = any(cls.lower() == 'directuihwnd' for _, cls, _ in controls)
        assert task_dialog or len(buttons) == 1, [(cls, ident) for _, cls, ident in controls]
        # TaskDialog's documented button message works on a non-input desktop too.
        assert user.PostMessageW(hwnd, 0x0466 if task_dialog else 0x0111, identifier, 0 if task_dialog else buttons[0])

    def run(case):
        target = output / case
        target.mkdir()
        env = {key: value for key, value in os.environ.items() if not key.startswith(('XDZIP_', 'QT_', 'QML'))}
        env['PATH'] = str(Path(os.environ['SystemRoot']) / 'System32')
        process = Process(image, name, env)
        try:
            hwnd, controls = wait(process, lambda items: any(cls.lower() == 'edit' for _, cls, _ in items) and any(ident == 2 for _, _, ident in items))
            if case == 'folder-cancel':
                click(hwnd, controls, 2)
                assert process.wait(timeout=10) == 0
                assert not list(target.iterdir())
                return
            edit = next(h for h, cls, _ in controls if cls.lower() == 'edit')
            value = c.create_unicode_buffer(str(target))
            user.SendMessageW(edit, 0x000c, 0, c.cast(value, c.c_void_p).value)  # WM_SETTEXT
            click(hwnd, controls, 1)
            hwnd, controls = wait(process, lambda items: sum(cls.lower() == 'button' for _, cls, _ in items) == 4 and any(cls.lower() == 'directuihwnd' for _, cls, _ in items))
            click(hwnd, controls, 2 if case == 'conflict-cancel' else 100)
            conflict_window = hwnd
            if case == 'conflict-cancel':
                assert process.wait(timeout=10) == 0
                assert not list(target.iterdir())
                return
            # TaskDialog hosts its buttons in DirectUI with Win32 control ID 0.
            # Distinguish the system Cancel/Close captions, then invoke documented button IDs.
            def caption(items, wanted):
                for child, cls, _ in items:
                    if cls.lower() != 'button':
                        continue
                    buffer = c.create_unicode_buffer(256)
                    user.GetWindowTextW(child, buffer, len(buffer))
                    if any(text in buffer.value.replace('&', '') for text in wanted):
                        return True
                return False
            if case == 'progress-cancel':
                hwnd, controls = wait(process, lambda items: sum(cls.lower() == 'button' for _, cls, _ in items) == 1 and caption(items, ['Cancel', '取消']), [conflict_window])
                click(hwnd, controls, 2)
            hwnd, controls = wait(process, lambda items: caption(items, ['Close', '关闭']))
            click(hwnd, controls, 8)
            code = process.wait(timeout=30)
            assert code == (1 if case == 'progress-cancel' else 0), (case, code)
            expected = {'source/' + p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in source.iterdir()}
            actual = {p.relative_to(target).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in target.rglob('*') if p.is_file()}
            assert actual.items() <= expected.items(), actual.keys()
            if case == 'complete':
                assert actual == expected
            assert not any(p.name.startswith('.xdzip') or p.name == 'EXECUTED' for p in target.rglob('*'))
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=10)
            kernel.CloseHandle(process.handle)

    try:
        for case in ['folder-cancel', 'conflict-cancel', 'progress-cancel', 'complete']:
            run(case)
            checks.append(case)
            print(case + ' passed', flush=True)
    finally:
        user.CloseDesktop(desktop)
    (output / 'receipt.json').write_text(json.dumps({'status': 'passed', 'image_sha256': hashlib.sha256(image.read_bytes()).hexdigest(), 'checks': checks}, indent=2) + '\n', encoding='utf-8')


if __name__ == '__main__':
    main()
