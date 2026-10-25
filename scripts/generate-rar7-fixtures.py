# scripts/generate-rar7-fixtures.py

"""Create tiny deterministic RAR7 Store and dictionary-admission regression inputs."""
import argparse
from pathlib import Path
import struct
import zlib


def vint(value):
    result = bytearray()
    while value >= 128:
        result.append((value & 127) | 128)
        value >>= 7
    result.append(value)
    return bytes(result)


def header(body):
    block = vint(len(body)) + body
    return struct.pack('<I', zlib.crc32(block)) + block


def archive(dictionary_index):
    # Format fields follow https://www.rarlab.com/technote.htm. Store contains no
    # proprietary compression implementation; version 1 denotes RAR7 headers.
    name = b'sample.txt'
    content = b'RAR newest algorithm verification sample'
    signature = b'Rar!\x1a\x07\x01\x00'
    main = header(vint(1) + vint(0) + vint(0))
    file_body = vint(2) + vint(2) + vint(len(content))
    file_body += vint(4) + vint(len(content)) + vint(0x20) + struct.pack('<I', zlib.crc32(content))
    file_body += vint(1 | (dictionary_index << 10)) + vint(0) + vint(len(name)) + name
    end = header(vint(5) + vint(0) + vint(0))
    return signature + main + header(file_body) + content + end


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=Path('build/native-codec/fixtures'))
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    for name, dictionary in [('rar7-store-small.rar', 0), ('rar7-dictionary-limit.rar', 16)]:
        (args.output / name).write_bytes(archive(dictionary))
    print('Generated two deterministic RAR7 container regression inputs')
