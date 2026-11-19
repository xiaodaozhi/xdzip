#!/usr/bin/env python3
"""Verify edited archives exported by the runtime tests with independent readers."""
import argparse
import io
from pathlib import Path
import struct
import subprocess
import tarfile
import zipfile


def run(*command):
    return subprocess.check_output([str(part) for part in command])


def verify_tar(root, payload):
    decoders = {
        'edit.txz': ('xz', '-dc'),
        'edit.tar.xz': ('xz', '-dc'),
        'edit.tzst': ('zstd', '-dcq'),
        'edit.tar.zst': ('zstd', '-dcq'),
        'edit.tar.lz4': ('lz4', '-dcq'),
        'edit.tar.bz2': ('bzip2', '-dc'),
        'edit.tar.lzma': ('xz', '--format=lzma', '-dc'),
        'edit.tar.lz': ('xz', '--format=lzip', '-dc'),
        'edit.tar.Z': ('gzip', '-dc'),
        'edit.tar.br': ('brotli', '--decompress', '--stdout'),
    }
    for filename, decoder in decoders.items():
        tar_bytes = run(*decoder, root / filename)
        with tarfile.open(fileobj=io.BytesIO(tar_bytes), mode='r:') as archive:
            assert {member.name for member in archive if member.isdir()} == {
                'project', 'project/empty', 'project/target',
            }, filename
            assert {member.name: archive.extractfile(member).read()
                    for member in archive if member.isfile()} == {
                'project/target/renamed.bin': payload,
                'project/target/extra.txt': b'replacement bytes',
            }, filename


def packed_zip_entry(path, name):
    with zipfile.ZipFile(path) as archive, path.open('rb') as source:
        entry = archive.getinfo(name)
        source.seek(entry.header_offset)
        header = source.read(30)
        assert header[:4] == b'PK\x03\x04', path
        name_len, extra_len = struct.unpack_from('<HH', header, 26)
        source.seek(name_len + extra_len, 1)
        return entry.compress_type, source.read(entry.compress_size)


def verify_members(root, seven_zip, payload):
    for filename in ('edit.lha', 'edit.lzh'):
        path = root / filename
        run(seven_zip, 't', path)
        for name, content in [('renamed.bin', payload), ('extra.txt', b'replacement bytes')]:
            assert run(seven_zip, 'x', '-so', path, 'project/target/' + name) == content, path
    source_root = Path(__file__).resolve().parents[1] / 'tests/fixtures/streams'
    for method in ('deflate64', 'bzip2', 'lzma', 'xz'):
        for suffix in ('', '-descriptor'):
            path = root / f'zipx-{method}{suffix}.zipx'
            run(seven_zip, 't', path)
            assert packed_zip_entry(source_root / path.name, 'hello.txt') == packed_zip_entry(
                path, 'target/renamed.txt'), path
            assert run(seven_zip, 'x', '-so', path, 'target/renamed.txt') == b'abracadabra abracadabra\n' * 6, path
            assert run(seven_zip, 'x', '-so', path, 'target/extra.txt') == b'new', path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--7zz', dest='seven_zip', required=True)
    args = parser.parse_args()
    payload = (args.input / 'expected.bin').read_bytes()
    verify_tar(args.input, payload)
    verify_members(args.input, args.seven_zip, payload)
    print('Verified 10 edited TAR archives, two LHA/LZH archives and eight ZIPX profiles')


if __name__ == '__main__':
    main()
