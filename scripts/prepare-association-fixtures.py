#!/usr/bin/env python3
"""Prepare bounded archive inputs for isolated native association checks."""
import argparse
import gzip
import io
from pathlib import Path
import shutil
import tarfile
import zlib

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--zip', required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(ROOT / 'build') or output == ROOT / 'build' or output.exists():
        parser.error('Use a new child directory under build/')
    output.mkdir(parents=True)
    shutil.copyfile(args.zip, output / 'sample.zip')
    for suffix in ['xpi', 'apk', 'ipa', 'cbz']:
        shutil.copyfile(args.zip, output / ('sample.' + suffix))
    shutil.copyfile(ROOT / 'tests/fixtures/streams/zipx-deflate64.zipx', output / 'sample.zipx')
    shutil.copyfile(ROOT / 'tests/fixtures/streams/phrase-crc64.xz', output / 'sample.xz')
    shutil.copyfile(ROOT / 'tests/fixtures/streams/phrase.lz4', output / 'sample.lz4')
    shutil.copyfile(ROOT / 'tests/fixtures/streams/files.tar.lz4', output / 'sample.tar.lz4')
    for suffix in ['tar.xz', 'txz']:
        shutil.copyfile(ROOT / 'tests/fixtures/streams/files.txz', output / ('sample.' + suffix))
    shutil.copyfile(ROOT / 'tests/fixtures/streams/phrase.zst', output / 'sample.zst')
    shutil.copyfile(ROOT / 'tests/fixtures/deb/control-gz-data-zst.deb', output / 'sample.deb')
    shutil.copyfile(ROOT / 'tests/fixtures/aescrypt/v3-phrase.aes', output / 'sample.aes')
    shutil.copyfile(ROOT / 'tests/fixtures/brotli/q11-text.br', output / 'sample.br')
    shutil.copyfile(ROOT / 'tests/fixtures/brotli/files.tar.br', output / 'sample.tar.br')
    shutil.copyfile(ROOT / 'tests/fixtures/asar/ordinary.asar', output / 'sample.asar')
    shutil.copyfile(ROOT / 'tests/fixtures/pma/pm2-boundary-1024.pma', output / 'sample.pma')
    shutil.copyfile(ROOT / 'tests/fixtures/arj/m1-phrase.arj', output / 'sample.arj')
    shutil.copyfile(ROOT / 'tests/fixtures/arc/m8-phrase.arc', output / 'sample.arc')
    bh_payload = b'xdzip BH association fixture\n'
    encoder = zlib.compressobj(level=6, wbits=-15)
    compressed = encoder.compress(bh_payload) + encoder.flush()
    bh = bytearray(b'BH\x05\x07')
    name = b'hello.txt'
    bh.extend((37 + len(name)).to_bytes(2, 'little'))
    bh.extend(b'\x25\xd8\x0c\x00\x08')
    bh.extend((0x12345678).to_bytes(4, 'little'))
    bh.extend(len(compressed).to_bytes(4, 'little'))
    bh.extend(len(bh_payload).to_bytes(4, 'little'))
    bh.extend((zlib.crc32(bh_payload) & 0xffffffff).to_bytes(4, 'little'))
    bh.extend(b'\x20' + b'\x00' * 5)
    bh.extend(len(name).to_bytes(4, 'little'))
    bh.extend(name)
    bh.extend(compressed)
    (output / 'sample.bh').write_bytes(bh)
    alz_payload = b'xdzip ALZ association fixture\n'
    alz_compressed = zlib.compressobj(level=6, wbits=-15)
    alz_packed = alz_compressed.compress(alz_payload) + alz_compressed.flush()
    alz = bytearray(b'ALZ\x01\x01\x00\x01\x00')
    name = b'hello.txt'
    alz.extend(b'BLZ\x01')
    alz.extend(len(name).to_bytes(2, 'little'))
    alz.extend(b'\x20')
    alz.extend((0x12345678).to_bytes(4, 'little'))
    alz.extend(b'\x40\x00\x02\x00')
    alz.extend((zlib.crc32(alz_payload) & 0xffffffff).to_bytes(4, 'little'))
    alz.extend(len(alz_packed).to_bytes(4, 'little'))
    alz.extend(len(alz_payload).to_bytes(4, 'little'))
    alz.extend(name)
    alz.extend(alz_packed)
    alz.extend(b'CLZ\x02' + b'\x00' * 16)
    (output / 'sample.alz').write_bytes(alz)
    egg_payload = b'xdzip EGG association fixture\n'
    egg_encoder = zlib.compressobj(level=6, wbits=-15)
    egg_packed = egg_encoder.compress(egg_payload) + egg_encoder.flush()
    end = b'\x22\x82\xe2\x08'
    egg = bytearray(b'EGGA\x00\x01\x01\x00\x00\x00\x00\x00\x00\x00' + end)
    name = b'hello.txt'
    egg.extend(b'\xe3\x90\x85\x0a' + (0).to_bytes(4, 'little') + len(egg_payload).to_bytes(8, 'little'))
    egg.extend(b'\xac\x91\x85\x0a\x00' + len(name).to_bytes(2, 'little') + name + end)
    egg.extend(b'\x13\x0c\xb5\x02\x01\x00' + len(egg_payload).to_bytes(4, 'little') + len(egg_packed).to_bytes(4, 'little'))
    egg.extend((zlib.crc32(egg_payload) & 0xffffffff).to_bytes(4, 'little') + end + egg_packed + end)
    (output / 'sample.egg').write_bytes(egg)
    pea_payload = b'xdzip PEA association fixture\n'
    pea = bytearray(bytes([0xEA, 1, 6, 0, 0, 0, 0, 1, 0x21, 0]))
    pea.extend(b'\x00\x00POD\x00\x00\x00\x00\x00')
    name = b'hello.txt'
    pea.extend(len(name).to_bytes(2, 'little'))
    pea.extend(name)
    pea.extend((0).to_bytes(4, 'little'))
    pea.extend((0).to_bytes(4, 'little'))
    pea.extend(len(pea_payload).to_bytes(8, 'little'))
    pea.extend(pea_payload)
    pea.extend(b'\x00\x00EOA\x00')
    (output / 'sample.pea').write_bytes(pea)
    for suffix in ['lha','lzh']:
        shutil.copyfile(ROOT / 'tests/fixtures/lha/lh5-h2.lzh', output / ('sample.' + suffix))
    shutil.copyfile(ROOT / 'tests/fixtures/compress/phrase.Z', output / 'sample.z')
    for suffix in ['tar.z', 'taz']:
        shutil.copyfile(ROOT / 'tests/fixtures/compress/files.taz', output / ('sample.' + suffix))
    for suffix, fixture in [('uu', 'spaces.uu'), ('uue', 'ordinary.uue'), ('xxe', 'ordinary.xxe')]:
        shutil.copyfile(ROOT / 'tests/fixtures/uu' / fixture, output / ('sample.' + suffix))
    for suffix in ['tar.zst','tzst']:
        shutil.copyfile(ROOT / 'tests/fixtures/streams/files.tar.zst', output / ('sample.' + suffix))
    content = b'xdzip association fixture\n'
    (output / 'sample.gz').write_bytes(gzip.compress(content, mtime=0))
    for name, mode in [('sample.tar', 'w'), ('sample.tar.gz', 'w:gz')]:
        with tarfile.open(output / name, mode) as archive:
            item = tarfile.TarInfo('hello.txt')
            item.size = len(content)
            archive.addfile(item, io.BytesIO(content))
    shutil.copyfile(output / 'sample.tar.gz', output / 'sample.tgz')
    for fixture, suffixes in [
        ('phrase.bz2', ['bz2', 'bz']), ('files.tbz2', ['tar.bz2', 'tar.bz', 'tbz', 'tbz2']),
        ('phrase.lzma', ['lzma']), ('files.tar.lzma', ['tar.lzma', 'tlz']),
        ('phrase.lz', ['lz']), ('files.tlz', ['tar.lz']),
    ]:
        for suffix in suffixes:
            shutil.copyfile(ROOT / 'tests/fixtures/streams' / fixture, output / ('sample.' + suffix))
    for source, name in [
        ('crates/xdzip-codec-7z/tests/fixtures/copy.7z', 'sample.7z'),
        ('tests/fixtures/native-archives/rar7-store-small.rar', 'sample.rar'),
        ('tests/fixtures/cab-iso/named.cab', 'sample.cab'),
    ]:
        shutil.copyfile(ROOT / source, output / name)
    for source, name in [('iso9660.iso.gz', 'sample.iso'), ('udf-regular.iso.gz', 'sample.udf')]:
        with gzip.open(ROOT / 'tests/fixtures/cab-iso' / source, 'rb') as archive:
            data = archive.read(64 * 1024 * 1024 + 1)
        if not data or len(data) > 64 * 1024 * 1024:
            raise ValueError('Optical fixture exceeds its bounded output')
        (output / name).write_bytes(data)
    print('Prepared ZIP and derived candidates, TAR, compressed TAR, GZIP, 7z, RAR, CAB, ISO and UDF inputs')


if __name__ == '__main__':
    main()
