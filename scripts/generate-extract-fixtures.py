#!/usr/bin/env python3
"""Owned Store/Deflate/descriptor inputs and structural adversarial variants."""
from pathlib import Path
import io
import struct
import zipfile
import zlib

ROOT = Path(__file__).resolve().parents[1] / 'tests/fixtures/extract'
ROOT.mkdir(parents=True, exist_ok=True)

def single(name, data, compressed, method=8, crc=None):
    raw = name.encode()
    crc = zlib.crc32(data) if crc is None else crc
    local = struct.pack('<IHHHHHIIIHH', 0x04034b50, 20, 2048, method, 0, 0x5c21, crc, len(compressed), len(data), len(raw), 0)
    central_at = len(local) + len(raw) + len(compressed)
    central = struct.pack('<IHHHHHHIIIHHHHHII', 0x02014b50, 20, 20, 2048, method, 0, 0x5c21, crc, len(compressed), len(data), len(raw), 0, 0, 0, 0, 0, 0) + raw
    end = struct.pack('<IHHHHIIH', 0x06054b50, 0, 0, 1, 1, len(central), central_at, 0)
    return local + raw + compressed + central + end

data = 'xdzip 实际解压\n'.encode() * 100
compressor = zlib.compressobj(wbits=-15)
compressed = compressor.compress(data) + compressor.flush()
variants = dict(deflate=single('说明.txt', data, compressed),
                stored=single('说明.txt', data, data, 0),
                crc_bad=single('说明.txt', data, compressed, crc=zlib.crc32(data) ^ 1),
                trailing=single('说明.txt', data, compressed + b'junk'))
# BFINAL=0 stored block emits every correct byte but has no end block. This
# specifically defeats a decoder that checks only output size and CRC.
no_end = b'\x00' + struct.pack('<HH', len(data), len(data) ^ 0xffff) + data
variants['no_end'] = single('说明.txt', data, no_end)
header_bad = bytearray(variants['deflate'])
header_bad[30] ^= 1
variants['header_bad'] = bytes(header_bad)
for name, value in variants.items():
    (ROOT / (name + '.zip')).write_bytes(value)

class Streaming(io.BytesIO):
    def seekable(self): return False
    def seek(self, *args): raise io.UnsupportedOperation('owned streaming fixture')

for wide in (False, True):
    stream = Streaming()
    with zipfile.ZipFile(stream, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        with archive.open('说明.txt', 'w', force_zip64=wide) as entry:
            entry.write(data)
    value = stream.getvalue()
    name = 'descriptor64' if wide else 'descriptor'
    (ROOT / (name + '.zip')).write_bytes(value)
    damaged = bytearray(value)
    offset = damaged.index(b'PK\x07\x08') + 4
    damaged[offset] ^= 1
    (ROOT / (name + '_bad.zip')).write_bytes(damaged)

with zipfile.ZipFile(ROOT / 'tree.zip', 'w') as archive:
    archive.writestr('照片/', b'')
    archive.writestr('照片/说明.txt', data, compress_type=zipfile.ZIP_DEFLATED)
    archive.writestr('empty.bin', b'')
    archive.writestr('data.bin', bytes(range(256)) * 1024, compress_type=zipfile.ZIP_STORED)
with zipfile.ZipFile(ROOT / 'unsafe.zip', 'w') as archive:
    archive.writestr('../outside.txt', b'must never publish')
with zipfile.ZipFile(ROOT / 'aliases.zip', 'w') as archive:
    archive.writestr('A.txt', b'first')
    archive.writestr('a.txt', b'second')

def partial_result_fixture():
    """Two valid files followed by a bad CRC, for actual result-window outcomes."""
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, 'w') as archive:
        for name, payload in [('冰川湖.jpg', b'owned lake fixture'), ('山间日出.jpg', b'owned sunrise fixture'), ('行程手册.pdf', b'owned PDF fixture')]:
            info = zipfile.ZipInfo(name, (2026, 10, 8, 0, 0, 0))
            archive.writestr(info, payload)
    value = bytearray(stream.getvalue())
    with zipfile.ZipFile(stream) as archive:
        entry = archive.infolist()[-1]
        struct.pack_into('<I', value, entry.header_offset + 14, entry.CRC ^ 1)
    central = value.index(b'PK\x01\x02', entry.header_offset)
    while value[central:central+4] == b'PK\x01\x02':
        name_len, extra_len, comment_len = struct.unpack_from('<HHH', value, central+28)
        if value[central+46:central+46+name_len] == '行程手册.pdf'.encode():
            struct.pack_into('<I', value, central+16, entry.CRC ^ 1)
            break
        central += 46 + name_len + extra_len + comment_len
    return value

(ROOT / 'partial_crc.zip').write_bytes(partial_result_fixture())
print('Generated owned extraction fixtures')
