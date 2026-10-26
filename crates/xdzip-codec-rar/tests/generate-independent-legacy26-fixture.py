"""Optional Windows-only original sparse input and official RAR2.90 recipe.

The regular/CI streaming test needs no author executable or plaintext file.
This recipe writes only beneath build/ and never runs the generated input.
"""
import argparse
import ctypes
import hashlib
import json
import msvcrt
import pathlib
import struct
import subprocess

SIZE = 2 * 1024**3 + 1
AUTHOR_SHA = '3f938b2f4cc13b9b85d61417b36af45a970dbee0e09e9d9e13f4e00350485112'
ORIGINAL_SHA = 'd1ea9e8a1dd77c140253106db1be9331430622ee141e57a1ccdf31bab1d5a13a'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--author', type=pathlib.Path, required=True)
    parser.add_argument('--output', type=pathlib.Path, required=True)
    args = parser.parse_args()
    author = args.author.resolve(strict=True)
    assert hashlib.sha256(author.read_bytes()).hexdigest() == AUTHOR_SHA
    repository = pathlib.Path(__file__).resolve().parents[3]
    output = args.output.resolve()
    assert output.is_relative_to(repository / 'build'), 'output must stay below repository build/'
    output.mkdir(parents=True, exist_ok=True)
    source = output / 'large-owned.bin'
    archive = output / 'large26.rar'
    if source.exists() or archive.exists():
        raise FileExistsError('recipe requires new output files')
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    control = kernel.DeviceIoControl
    control.argtypes = [ctypes.c_void_p, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_uint32,
                       ctypes.c_void_p, ctypes.c_uint32, ctypes.POINTER(ctypes.c_uint32), ctypes.c_void_p]
    control.restype = ctypes.c_int
    seek = kernel.SetFilePointerEx
    seek.argtypes = [ctypes.c_void_p, ctypes.c_int64, ctypes.c_void_p, ctypes.c_uint32]
    seek.restype = ctypes.c_int
    finish = kernel.SetEndOfFile
    finish.argtypes = [ctypes.c_void_p]
    finish.restype = ctypes.c_int
    # Native SetEndOfFile preserves sparse holes. Python's Windows truncate can
    # call a CRT extension routine that writes the whole zero-filled interval.
    with source.open('x+b') as stream:
        handle = msvcrt.get_osfhandle(stream.fileno())
        returned = ctypes.c_uint32()
        if not control(handle, 0x900C4, None, 0, None, 0, ctypes.byref(returned), None):
            raise ctypes.WinError(ctypes.get_last_error())
        if not seek(handle, SIZE, None, 0) or not finish(handle):
            raise ctypes.WinError(ctypes.get_last_error())
        stream.seek(0)
        stream.write(bytes((31*i + 7*(i >> 5) + 13) & 255 for i in range(4096)))
        stream.seek(SIZE - 4096)
        stream.write(bytes((i*i + 5*i + (i >> 3)) & 255 for i in range(4096)))
    allocation = kernel.GetCompressedFileSizeW
    allocation.argtypes = [ctypes.c_wchar_p, ctypes.POINTER(ctypes.c_uint32)]
    allocation.restype = ctypes.c_uint32
    high = ctypes.c_uint32()
    low = allocation(str(source), ctypes.byref(high))
    reported_allocated = low + (high.value << 32)
    assert reported_allocated <= 1024**2 and source.stat().st_size == SIZE
    original = hashlib.sha256()
    with source.open('rb') as stream:
        while chunk := stream.read(1024**2):
            original.update(chunk)
    assert original.hexdigest() == ORIGINAL_SHA
    command = ['a', '-cfg-', '-inul', '-y', '-m1', '-md64', '-s-', archive.name, source.name]
    made = subprocess.run([str(author), *command], cwd=output, capture_output=True, timeout=600)
    assert made.returncode == 0
    tested = subprocess.run([str(author), 't', '-cfg-', '-inul', archive.name],
                            cwd=output, capture_output=True, timeout=300)
    assert tested.returncode == 0
    packed = archive.read_bytes()
    assert len(packed) <= 2 * 1024**2 and packed[:7] == b'Rar!\x1a\x07\0'
    position = 7
    while packed[position + 2] != 0x74:
        position += struct.unpack_from('<H', packed, position + 5)[0]
    flags = struct.unpack_from('<H', packed, position + 3)[0]
    assert flags & 0x100 and packed[position + 24] == 26 and packed[position + 25] == 0x31
    size = struct.unpack_from('<I', packed, position + 11)[0]
    size += struct.unpack_from('<I', packed, position + 36)[0] << 32
    assert size == SIZE
    record = dict(author_sha256=AUTHOR_SHA, command=command, original_bytes=SIZE,
                  original_sha256=ORIGINAL_SHA, windows_reported_allocated_bytes=reported_allocated,
                  archive_bytes=len(packed), archive_sha256=hashlib.sha256(packed).hexdigest(),
                  actual_unpack_version=26, author_full_test_exit=tested.returncode,
                  scope='No header version mutation; archive timestamp differences can change archive identity.')
    (output / 'author-results.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print('Genuine version26 archive authored and full original SHA/author CRC checked.')


if __name__ == '__main__':
    main()
