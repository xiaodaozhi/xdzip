#!/usr/bin/env python3
"""Wrap independent compressed payloads in owned deterministic ZIP records."""
import argparse
import lzma
from pathlib import Path
import struct
import subprocess
import tempfile
import zipfile
import zlib

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'tests/fixtures/streams'
PHRASE = b'abracadabra abracadabra\n' * 6


def archive(records, descriptor=False):
    data = bytearray()
    central = bytearray()
    for name, method, flags, packed, plain in records:
        name = name.encode('utf-8')
        flags |= 0x800 | (8 if descriptor else 0)
        crc = zlib.crc32(plain)
        offset = len(data)
        data += struct.pack('<I5H3I2H', 0x04034b50, 63, flags, method, 0, 33,
                            0 if descriptor else crc, 0 if descriptor else len(packed),
                            0 if descriptor else len(plain), len(name), 0) + name + packed
        if descriptor:
            data += struct.pack('<4I', 0x08074b50, crc, len(packed), len(plain))
        central += struct.pack('<I6H3I5H2I', 0x02014b50, 63, 63, flags, method, 0, 33,
                               crc, len(packed), len(plain), len(name), 0, 0, 0, 0, 0, offset) + name
    end = struct.pack('<I4H2IH', 0x06054b50, 0, 0, len(records), len(records), len(central), len(data), 0)
    return bytes(data + central + end)


def run(tool, *arguments, **kwargs):
    return subprocess.check_output([str(tool), *map(str, arguments)], stderr=subprocess.PIPE, timeout=60, **kwargs)


def samples(tool):
    with tempfile.TemporaryDirectory(prefix='zipx-', dir=ROOT/'build/test-tmp') as scratch:
        directory = Path(scratch)
        (directory/'hello.txt').write_bytes(PHRASE)
        payloads = {}
        for method, name in [(9, 'Deflate64'), (12, 'BZip2'), (14, 'LZMA'), (98, 'PPMd')]:
            target = directory/(name+'.zip')
            arguments = ['a', '-tzip', '-mm='+name, '-mx=9', '-mmt=1', '-bd', '-y']
            if method == 14:
                arguments += ['-md=4k']
            run(tool, *arguments, target, 'hello.txt', cwd=directory)
            with zipfile.ZipFile(target) as zipped:
                item = zipped.infolist()[0]
                assert item.compress_type == method, (name, item.compress_type)
                raw = target.read_bytes()
                n, extra = struct.unpack_from('<HH', raw, item.header_offset+26)
                start = item.header_offset+30+n+extra
                payloads[method] = (item.flag_bits & 2, raw[start:start+item.compress_size])
        payloads[95] = (0, lzma.compress(PHRASE, filters=[dict(id=lzma.FILTER_LZMA2, dict_size=4096)]))
        # A sized 7z LZMA coder omits EOS. Keep its exact single packed stream,
        # then independently verify the corresponding ZIP framing and flag.
        sized = directory/'sized-lzma.7z'
        run(tool, 'a', '-t7z', '-m0=LZMA:lc=3:lp=0:pb=2:d=4k', '-mhc=off', '-mmt=1', '-bd', '-y', sized, 'hello.txt', cwd=directory)
        encoded = sized.read_bytes()
        packed_length = struct.unpack_from('<Q', encoded, 12)[0]
        no_eos = bytes([9,4,5,0,0x5d])+struct.pack('<I',4096)+encoded[32:32+packed_length]
        result = {}
        for method, name in [(9, 'deflate64'), (12, 'bzip2'), (14, 'lzma'), (95, 'xz'), (98, 'ppmd')]:
            flags, packed = payloads[method]
            records = [('hello.txt', method, flags, packed, PHRASE)]
            result['zipx-'+name+'.zipx'] = archive(records)
            result['zipx-'+name+'-descriptor.zipx'] = archive(records, descriptor=True)
        mixed = [('hello.txt', method, payloads[method][0], payloads[method][1], PHRASE) for method in [12,14,95,9]]
        mixed.append(('hello.txt',0,0,b'distinct physical duplicate\n',b'distinct physical duplicate\n'))
        result['zipx-mixed.zipx'] = archive(mixed, descriptor=True)
        result['zipx-lzma-no-eos.zipx'] = archive([('hello.txt',14,0,no_eos,PHRASE)])
        for name, data in result.items():
            path = directory/name
            path.write_bytes(data)
            expected = PHRASE*4+b'distinct physical duplicate\n' if name=='zipx-mixed.zipx' else PHRASE
            assert run(tool, 'x', '-so', '-bd', path) == expected, name
        return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seven-zip', required=True, type=Path)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    tool = args.seven_zip.resolve()
    for name, data in samples(tool).items():
        path = OUTPUT/name
        if args.check:
            assert path.read_bytes() == data, 'Stale ZIPX fixture: '+name
        else:
            path.write_bytes(data)
    print('Owned ZIPX fixtures are current' if args.check else 'Owned ZIPX fixtures independently decoded and generated')


if __name__ == '__main__':
    main()
