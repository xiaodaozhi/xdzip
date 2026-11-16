#!/usr/bin/env python3
"""Generate small owned BZip2/LZMA/Lzip compatibility inputs with Python stdlib."""
import argparse
import bz2
import io
import lzma
from pathlib import Path
import struct
import tarfile
import zlib

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'tests/fixtures/streams'
PHRASE = b'abracadabra abracadabra\n' * 6
FILTERS = [dict(id=lzma.FILTER_LZMA1, dict_size=4096, lc=3, lp=0, pb=2)]


def lzip(data):
    raw = lzma.compress(data, format=lzma.FORMAT_RAW, filters=FILTERS)
    return b'LZIP\x01\x0c' + raw + struct.pack('<IQQ', zlib.crc32(data), len(data), len(raw) + 26)


def samples():
    tar = io.BytesIO()
    with tarfile.open(fileobj=tar, mode='w', format=tarfile.USTAR_FORMAT) as archive:
        entry = tarfile.TarInfo('hello.txt')
        entry.size = len(PHRASE)
        entry.mtime = 0
        archive.addfile(entry, io.BytesIO(PHRASE))
    alone = lzma.compress(PHRASE, format=lzma.FORMAT_ALONE, filters=FILTERS)
    corrupt_tar = bytearray(tar.getvalue())
    corrupt_tar[0] ^= 1  # Keep ustar identification, deliberately break its header checksum.
    return {
        'phrase.bz2': bz2.compress(PHRASE, compresslevel=1),
        'empty.bz2': bz2.compress(b''),
        'multiple.bz2': bz2.compress(b'') + bz2.compress(PHRASE, compresslevel=1),
        'multiblock.bz2': bz2.compress(bytes((i * 73 ^ (i >> 8)) & 255 for i in range(320000)), compresslevel=1),
        'phrase.lzma': alone,
        'known-size.lzma': alone[:5] + struct.pack('<Q', len(PHRASE)) + alone[13:],
        'empty.lzma': lzma.compress(b'', format=lzma.FORMAT_ALONE, filters=FILTERS),
        'phrase.lz': lzip(PHRASE),
        'empty.lz': lzip(b''),
        'multiple.lz': lzip(PHRASE) + lzip(b'') + lzip(PHRASE),
        'files.tbz2': bz2.compress(tar.getvalue(), compresslevel=1),
        'files.tar.lzma': lzma.compress(tar.getvalue(), format=lzma.FORMAT_ALONE, filters=FILTERS),
        'files.tlz': lzip(tar.getvalue()),
        'bad-header.tbz2': bz2.compress(corrupt_tar, compresslevel=1),
        'bad-header.tar.lzma': lzma.compress(corrupt_tar, format=lzma.FORMAT_ALONE, filters=FILTERS),
        'bad-header.tlz': lzip(corrupt_tar),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    stale = []
    for name, data in samples().items():
        path = OUTPUT / name
        if args.check:
            if not path.is_file() or path.read_bytes() != data:
                stale.append(name)
        elif not path.is_file() or path.read_bytes() != data:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    if stale:
        raise SystemExit('Regenerate stream fixtures: ' + ', '.join(stale))
    print('Owned stream fixtures are current' if args.check else 'Generated owned stream fixtures')


if __name__ == '__main__':
    main()
