#!/usr/bin/env python3
"""Generate small owned XZ vectors with the independent Python/liblzma tool."""
import argparse
import io
import lzma
import os
from pathlib import Path
import struct
import subprocess
import tarfile
import zlib

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'tests/fixtures/streams'
PHRASE = b'abracadabra abracadabra\n' * 6
FILTER = dict(id=lzma.FILTER_LZMA2, dict_size=4096)
INSTRUCTIONS = bytes.fromhex('e800000000e9faffffff000000eb00000048000000400000009000000094000000f000f800000000') * 300


def integer(value):
    result = bytearray()
    while value >= 128:
        result.append((value & 127) | 128)
        value >>= 7
    result.append(value)
    return bytes(result)


def multiple_blocks():
    blocks = []
    records = []
    for data in [b'', PHRASE, PHRASE * 3]:
        encoded = lzma.compress(data, format=lzma.FORMAT_XZ, check=lzma.CHECK_CRC32, filters=[FILTER])
        # An empty independent XZ stream may have no block; use nonempty blocks here.
        if not data:
            continue
        backward = struct.unpack('<I', encoded[-8:-4])[0]
        end = len(encoded) - 12 - (backward + 1) * 4
        block = encoded[12:end]
        header_size = (block[0] + 1) * 4
        raw = lzma.compress(data, format=lzma.FORMAT_RAW, filters=[FILTER])
        records.append((header_size + len(raw) + 4, len(data)))
        blocks.append(block)
    index = b'\0' + integer(len(records))
    for packed, decoded in records:
        index += integer(packed) + integer(decoded)
    index += bytes((-len(index)) % 4)
    index += struct.pack('<I', zlib.crc32(index))
    flags = b'\0\1'
    header = b'\xfd7zXZ\0' + flags + struct.pack('<I', zlib.crc32(flags))
    footer = struct.pack('<I', len(index) // 4 - 1) + flags
    return header + b''.join(blocks) + index + struct.pack('<I', zlib.crc32(footer)) + footer + b'YZ'


def samples(xz):
    result = {}
    for name, check in [('none', lzma.CHECK_NONE), ('crc32', lzma.CHECK_CRC32),
                        ('crc64', lzma.CHECK_CRC64), ('sha256', lzma.CHECK_SHA256)]:
        result['phrase-' + name + '.xz'] = lzma.compress(PHRASE, check=check, filters=[FILTER])
    result['empty.xz'] = lzma.compress(b'', filters=[FILTER])
    result['multiple.xz'] = result['phrase-crc64.xz'] + bytes(8) + result['empty.xz'] + result['phrase-sha256.xz'] + bytes(4)
    result['multiblock.xz'] = multiple_blocks()
    for name, identifier, offset in [('x86', 4, 17), ('ppc', 5, 16), ('ia64', 6, 32),
                                     ('arm', 7, 16), ('thumb', 8, 18), ('sparc', 9, 16),
                                     ('arm64', 10, 16), ('riscv', 11, 18)]:
        if identifier >= 10:
            if not xz:
                raise ValueError('ARM64/RISC-V vectors require --xz with the pinned independent XZ tool')
            result['filter-' + name + '.xz'] = subprocess.check_output([str(xz), '--threads=1', '--check=crc64', '--'+name+'=start='+str(offset), '--lzma2=dict=4KiB', '--stdout'], input=INSTRUCTIONS)
        else:
            result['filter-' + name + '.xz'] = lzma.compress(INSTRUCTIONS, filters=[dict(id=identifier, start_offset=offset), FILTER])
    result['filter-delta.xz'] = lzma.compress(INSTRUCTIONS, filters=[dict(id=lzma.FILTER_DELTA, dist=256), FILTER])
    result['filter-chain.xz'] = lzma.compress(INSTRUCTIONS, filters=[dict(id=lzma.FILTER_DELTA, dist=7), dict(id=lzma.FILTER_X86), FILTER])
    tar = io.BytesIO()
    with tarfile.open(fileobj=tar, mode='w', format=tarfile.USTAR_FORMAT) as archive:
        entry = tarfile.TarInfo('hello.txt')
        entry.size = len(PHRASE)
        archive.addfile(entry, io.BytesIO(PHRASE))
    result['files.txz'] = lzma.compress(tar.getvalue(), filters=[FILTER])
    broken = bytearray(tar.getvalue())
    broken[0] ^= 1
    result['bad-header.txz'] = lzma.compress(broken, filters=[FILTER])
    for name, encoded in result.items():
        decoded = (subprocess.check_output([str(xz), '--decompress', '--stdout'], input=encoded)
                   if name in ['filter-arm64.xz', 'filter-riscv.xz', 'multiple.xz'] else lzma.decompress(encoded))
        expected = (INSTRUCTIONS if name.startswith('filter-') else PHRASE * 4 if name == 'multiblock.xz'
                    else PHRASE * 2 if name == 'multiple.xz' else b'' if name == 'empty.xz'
                    else bytes(broken) if name == 'bad-header.txz' else tar.getvalue() if name == 'files.txz' else PHRASE)
        assert decoded == expected, name
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--xz', type=Path, default=os.environ.get('XDZIP_XZ_REFERENCE'))
    args = parser.parse_args()
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for name, data in samples(args.xz).items():
        path = OUTPUT / name
        if args.check:
            if not path.is_file() or path.read_bytes() != data:
                raise SystemExit('Stale XZ fixture: ' + name)
        else:
            path.write_bytes(data)
    print('Owned independent XZ fixtures are current' if args.check else 'Owned XZ fixtures generated and independently decoded')


if __name__ == '__main__':
    main()
