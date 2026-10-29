#!/usr/bin/env python3
"""Check the Windows launcher's actual shell resources and malformed-command boundary."""
import argparse
import ctypes
from pathlib import Path
import subprocess
import sys


def check_application_resources(executable):
    shell = ctypes.WinDLL('shell32', use_last_error=True)
    shell.ExtractIconExW.argtypes = [ctypes.c_wchar_p, ctypes.c_int, ctypes.c_void_p,
                                    ctypes.c_void_p, ctypes.c_uint]
    shell.ExtractIconExW.restype = ctypes.c_uint
    if shell.ExtractIconExW(str(executable), -1, None, None, 0) in (0, 0xffffffff):
        raise RuntimeError('Association application lacks an embedded icon')
    version = ctypes.WinDLL('version', use_last_error=True)
    version.GetFileVersionInfoSizeW.argtypes = [ctypes.c_wchar_p, ctypes.POINTER(ctypes.c_uint)]
    version.GetFileVersionInfoSizeW.restype = ctypes.c_uint
    version.GetFileVersionInfoW.argtypes = [ctypes.c_wchar_p, ctypes.c_uint, ctypes.c_uint, ctypes.c_void_p]
    version.VerQueryValueW.argtypes = [ctypes.c_void_p, ctypes.c_wchar_p,
                                     ctypes.POINTER(ctypes.c_void_p), ctypes.POINTER(ctypes.c_uint)]
    ignored = ctypes.c_uint()
    size = version.GetFileVersionInfoSizeW(str(executable), ctypes.byref(ignored))
    if not size:
        raise RuntimeError('Association application lacks version identity')
    data = ctypes.create_string_buffer(size)
    if not version.GetFileVersionInfoW(str(executable), 0, size, data):
        raise RuntimeError('Cannot read association application identity')
    for name, expected in [('FileDescription', 'xdzip'), ('ProductName', 'xdzip'),
                           ('OriginalFilename', 'xdzip-association.exe')]:
        pointer, count = ctypes.c_void_p(), ctypes.c_uint()
        if not version.VerQueryValueW(data, '\\StringFileInfo\\040904b0\\' + name,
                                     ctypes.byref(pointer), ctypes.byref(count)):
            raise RuntimeError('Missing association application ' + name)
        if ctypes.wstring_at(pointer.value) != expected:
            raise RuntimeError('Wrong association application ' + name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', required=True, type=Path)
    args = parser.parse_args()
    if sys.platform != 'win32':
        parser.error('Requires Windows')
    check_application_resources(args.exe.resolve())
    for arguments in ([], ['--'], ['wrong', 'fixture.zip'], ['--', 'fixture.zip', 'extra']):
        result = subprocess.run([str(args.exe.resolve()), *arguments], timeout=10,
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if result.returncode != 2:
            raise RuntimeError('Association launcher accepted an invalid command')
    print('Windows association application icon/identity and malformed-command rejection passed')


if __name__ == '__main__':
    main()
