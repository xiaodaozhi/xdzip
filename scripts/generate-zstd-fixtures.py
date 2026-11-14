#!/usr/bin/env python3
"""Generate owned Zstandard samples using Python's independent reference binding."""
import argparse
import compression.zstd as zstd
import io
from pathlib import Path
import struct
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]
DESTINATION = ROOT / 'tests/fixtures/streams'
PHRASE = b'abracadabra abracadabra\n' * 6


def entropy_bytes(length):
    state = 0x726f6f74
    output = bytearray()
    for _ in range(length):
        state ^= (state << 13) & 0xffffffff
        state ^= state >> 17
        state ^= (state << 5) & 0xffffffff
        value = state & 255
        output.append(97 if value < 128 else 98 if value < 192 else 32 + (value & 63))
    return bytes(output)


def encode(data, *, level=6, checksum=True, sized=True):
    options = {zstd.CompressionParameter.compression_level: level,
               zstd.CompressionParameter.checksum_flag: int(checksum),
               zstd.CompressionParameter.content_size_flag: int(sized)}
    return zstd.compress(data, options=options)


def samples():
    patterns = {
        'phrase': PHRASE, 'empty': b'', 'rle': bytes(200000),
        'entropy': entropy_bytes(700000),
        'multiblock': PHRASE * 10000,
    }
    result = {}
    for name, plain in patterns.items():
        result[name + '.zst'] = (encode(plain), plain)
    result['phrase-no-checksum.zst'] = (encode(PHRASE, checksum=False), PHRASE)
    result['phrase-unknown-size.zst'] = (encode(PHRASE, sized=False), PHRASE)
    for level in [1, 9, 19]:
        plain = patterns['entropy']
        result[f'entropy-level-{level}.zst'] = (encode(plain, level=level), plain)
    result['multiple.zst'] = (result['phrase.zst'][0] + result['empty.zst'][0] + result['phrase-no-checksum.zst'][0], PHRASE * 2)
    skip = struct.pack('<II', 0x184d2a5f, 16) + bytes(range(16))
    result['leading-skip.zst'] = (skip + result['phrase.zst'][0], PHRASE)
    result['only-skip.zst'] = (skip, b'')
    tar = io.BytesIO()
    with tarfile.open(fileobj=tar, mode='w', format=tarfile.USTAR_FORMAT) as archive:
        item = tarfile.TarInfo('hello.txt')
        item.size = len(PHRASE)
        item.mtime = 0
        archive.addfile(item, io.BytesIO(PHRASE))
    result['files.tar.zst'] = (encode(tar.getvalue()), tar.getvalue())
    bad = bytearray(tar.getvalue())
    bad[0] ^= 1
    result['bad-header.tar.zst'] = (encode(bad), bytes(bad))
    for encoded, plain in result.values():
        assert zstd.decompress(encoded) == plain
    return {name: encoded for name, (encoded, _) in result.items()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--xxhash-reference', type=Path, required=True)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if zstd.zstd_version_info != (1, 5, 7):
        parser.error('Use the pinned independent Zstandard 1.5.7 binding')
    output = subprocess.check_output([str(args.xxhash_reference.resolve())], text=True)
    hashes = b''.join(struct.pack('<IQ', int(length), int(digest, 16))
                      for length, digest in (line.split() for line in output.splitlines()))
    assert len(hashes) == 18 * 12
    generated = samples()
    generated['zstd-xxh64.bin'] = hashes
    for name, data in generated.items():
        path = DESTINATION / name
        if args.check:
            if not path.is_file() or path.read_bytes() != data:
                raise SystemExit('Stale owned fixture: ' + name)
        else:
            path.write_bytes(data)
    print(f'{len(generated)} owned Zstandard/XXH64 fixtures ' + ('verified' if args.check else 'generated'))


if __name__ == '__main__':
    main()
