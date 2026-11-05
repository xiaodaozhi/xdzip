"""Check invalid creation names against the actual native worker protocol."""
import struct
import subprocess
import sys


def frame(kind, payload):
    return struct.pack('<IB', len(payload) + 1, kind) + payload


def run(worker, name):
    # One 7z creation record, with enough memory for the worker to start.
    start = struct.pack('<BBQIIQB', 2, 7, 512 * 1024 * 1024, 1, 4096, 1024, 0)
    start += struct.pack('<IQI', 0, 0, 0)  # empty password, selection, volumes
    start += struct.pack('<BBBBI', 0, 0, 0, 0, 1)
    row = struct.pack('<BQQQI', 0, 0, 0, 0xffffffffffffffff, len(name)) + name
    return subprocess.run([worker], input=frame(1, start) + frame(13, row),
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)


def check(worker, name):
    result = run(worker, name)
    expected = frame(14, b'\x02')
    if result.returncode != 1 or result.stdout != expected:
        raise AssertionError(f'invalid name {name.hex()}: exit={result.returncode}, '
                             f'output={result.stdout.hex()}, stderr={result.stderr!r}')


def main():
    valid = run(sys.argv[1], b'valid.txt')
    if len(valid.stdout) < 5 or valid.stdout[4] == 14:
        raise AssertionError(f'valid control did not reach backend: exit={valid.returncode}, '
                             f'output={valid.stdout.hex()}, stderr={valid.stderr!r}')
    for name in (b'\x80', b'\xc2', b'\xe0\x80\x80', b'\xed\xa0\x80',
                 b'\xf4\x90\x80\x80', b'valid\xf0\x90'):
        check(sys.argv[1], name)
    print('native Unicode protocol: invalid names produce error 2')


if __name__ == '__main__':
    main()
