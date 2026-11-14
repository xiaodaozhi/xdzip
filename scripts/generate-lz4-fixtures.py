#!/usr/bin/env python3
"""Generate owned standard LZ4 inputs and full XXH32 vectors with the author CLI."""
import argparse
import io
from pathlib import Path
import struct
import subprocess
import tarfile
import tempfile

ROOT=Path(__file__).resolve().parents[1]
OUTPUT=ROOT/'tests/fixtures/streams'
PHRASE=b'abracadabra abracadabra\n'*6


def pattern(length):
    return bytes((i*73^(i>>8))&255 for i in range(length))


def random_prefix():
    state=0x726f6f74
    result=bytearray()
    for _ in range(65535):
        state ^= state << 13
        state ^= (state & 0xffffffff) >> 17
        state ^= (state & 0xffffffff) << 5
        state &= 0xffffffff
        result.append(state >> 24)
    return bytes(result)


def samples(tool):
    with tempfile.TemporaryDirectory(prefix='lz4-',dir=ROOT/'build/test-tmp') as scratch:
        directory=Path(scratch)
        def command(*args,data=None):
            return subprocess.check_output([str(tool),*map(str,args)],input=data,stderr=subprocess.PIPE,timeout=60)
        def compress(data,*options):
            path=directory/'owned-input.bin';path.write_bytes(data)
            return command('-z','-c','-q','--single-thread','--no-mmap',*options,path)
        result={
            'phrase.lz4':compress(PHRASE,'-B4','-BI','--content-size'),
            'phrase-block-checksum.lz4':compress(PHRASE,'-B4','-BI','-BX'),
            'phrase-no-checksum.lz4':compress(PHRASE,'-B4','--no-frame-crc'),
            'empty.lz4':compress(b'','--content-size'),
            'linked.lz4':compress(random_prefix()*5,'-B4','-BD','-BX','--content-size'),
            'large-block.lz4':compress(PHRASE*10000,'-B7','-BI','-BX'),
            'legacy.lz4':compress(PHRASE,'-l'),
        }
        skip=struct.pack('<II',0x184d2a57,8)+b'metadata'
        result['multiple.lz4']=result['phrase.lz4']+skip+result['empty.lz4']+result['phrase-no-checksum.lz4']
        result['leading-skip.lz4']=skip+result['phrase.lz4']
        result['only-skip.lz4']=skip
        tar=io.BytesIO()
        with tarfile.open(fileobj=tar,mode='w',format=tarfile.USTAR_FORMAT) as archive:
            item=tarfile.TarInfo('hello.txt');item.size=len(PHRASE);archive.addfile(item,io.BytesIO(PHRASE))
        result['files.tar.lz4']=compress(tar.getvalue(),'-BX')
        broken=bytearray(tar.getvalue());broken[0]^=1
        result['bad-header.tar.lz4']=compress(broken,'-BX')
        expected={name:PHRASE for name in result}
        expected.update({'empty.lz4':b'','linked.lz4':random_prefix()*5,'large-block.lz4':PHRASE*10000,
                         'multiple.lz4':PHRASE*2,'only-skip.lz4':b'',
                         'files.tar.lz4':tar.getvalue(),'bad-header.tar.lz4':bytes(broken)})
        for name,data in result.items():
            assert command('-d','-c','-q',data=data)==expected[name],name
        checks=bytearray()
        for length in [0,1,3,15,16,17,31,32,33,255,4096,65536]:
            data=compress(pattern(length))
            assert data[4]&4
            checks+=struct.pack('<I',length)+data[-4:]
        result['lz4-xxh32.bin']=bytes(checks)
        return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lz4',required=True,type=Path)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    for name,data in samples(args.lz4.resolve()).items():
        path=OUTPUT/name
        if args.check:
            assert path.read_bytes()==data,'Stale LZ4 fixture: '+name
        else:path.write_bytes(data)
    print('Owned LZ4 fixtures are current' if args.check else 'Owned LZ4 fixtures independently decoded and generated')


if __name__=='__main__':main()
