#!/usr/bin/env python3
"""Generate owned ARJ method profiles with pinned tools and independent complete reads."""
import argparse
import binascii
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT=Path(__file__).resolve().parents[1]
INPUT_SHA='9797b39865134962b6caa1a4ae0c2a546e108410f7f589ec14e5f54aa85ef93d'
SOURCE_SHA='589e4c9bccc8669e7b6d8d6fcd64e01f6a2c21fe10aad56a83304ecc3b96a7db'
PACKAGE_SHA='e5cc5d8f82c5f08246e1c0a8c5be77e5f796c2dd30f987e6bbb27a2c1df1e371'
ARJ_SHA='00ac8169236b8f9910c661e6d8b82c2f2e5800338a9c7775d691e622f3521ac9'
SEVEN_SHA='dc0493429fe28136fcb177a0e77c9875cbb3b0bff5f2f8eeba4df302e98cef76'
FIXED_DOS=(43<<25)|(11<<21)|(14<<16)|(22<<11)|(13<<5)|10


def pinned(path,expected):
    assert path.is_file() and not path.is_symlink()
    assert hashlib.sha256(path.read_bytes()).hexdigest()==expected, ('Reference identity changed',path.name)


def command(args,cwd):
    env={k:v for k,v in os.environ.items() if not k.upper().startswith(('ARJ','UNARJ'))}
    result=subprocess.run(list(map(str,args)),cwd=cwd,env=env,input=b'',capture_output=True,timeout=30,creationflags=0x08000000)
    (cwd/'stdout.txt').write_bytes(result.stdout.replace(os.fsencode(os.environ.get('USERPROFILE','<profile>')),b'<user>'))
    (cwd/'stderr.txt').write_bytes(result.stderr.replace(os.fsencode(os.environ.get('USERPROFILE','<profile>')),b'<user>'))
    assert result.returncode==0, ('Reference command failed',Path(args[0]).name,result.returncode)


def random_bytes(size):
    state=0x12345678;out=bytearray()
    for _ in range(size):
        state^=(state<<13)&0xffffffff;state^=state>>17;state^=(state<<5)&0xffffffff
        out.append(state&255)
    return bytes(out)


def plaintext(kind):
    phrase=b'abracadabra abracadabra\n'
    if kind=='empty':return b''
    if kind=='phrase':return phrase*128
    if kind=='spaces':return b' '*65536
    if kind=='allbytes':return bytes(range(256))*256
    if kind=='random':return random_bytes(32768)
    if kind=='mixed':
        cycle=bytes(range(256))+phrase*128
        return (cycle*((131072+len(cycle)-1)//len(cycle)))[:131072]
    if kind=='window21k':
        prefix=random_bytes(24000);return prefix+prefix[3000:13000]
    if kind=='window14k':
        prefix=random_bytes(15800);return prefix+prefix[1000:13000]
    raise ValueError(kind)


def framing(data,normalize=False):
    data=bytearray(data);at=0;records=[]
    while True:
        assert data[at:at+2]==b'\x60\xea'
        size=int.from_bytes(data[at+2:at+4],'little')
        if size==0:
            assert at+4==len(data)
            break
        assert 30<=size<=2600 and at+8+size<=len(data)
        begin=at+4;body=bytearray(data[begin:begin+size]);main=not records
        assert binascii.crc32(body)==int.from_bytes(data[begin+size:begin+size+4],'little')
        assert body[0]==(34 if main else 46) and body[3]==11 and body[4]==16
        assert body[6]==(2 if main else 0) and body[28:30]==b'\0\0'
        assert body[body[0]:]==(b'RESULT.ARJ\0\0' if main else b'HELLO.BIN\0\0')
        if normalize:
            body[7]=0  # Unused modifier is time-derived even without encryption.
            for field in ((8,12) if main else (8,34,38)):
                body[field:field+4]=FIXED_DOS.to_bytes(4,'little')
            data[begin:begin+size]=body
            data[begin+size:begin+size+4]=binascii.crc32(body).to_bytes(4,'little')
        at+=8+size
        # These plain native profiles contain no extended-header blocks.
        assert data[at:at+2]==b'\0\0';at+=2
        packed=0 if main else int.from_bytes(body[12:16],'little')
        records.append(dict(header_offset=begin-4,payload_offset=at,first_size=body[0],method=body[5],
            packed=packed,decoded=int.from_bytes(body[16:20],'little'),crc32=int.from_bytes(body[20:24],'little')))
        assert at+packed<=len(data);at+=packed
    assert len(records)==2
    return bytes(data),records[1]


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    refs=ROOT/'build/toolchain/references';base=refs/'arj-3.10.22'
    pinned(base/'arj-3.10.22.tar.gz',SOURCE_SHA);pinned(base/'arjw_310.exe',PACKAGE_SHA)
    arj=base/'runtime/arj.exe';seven=refs/'7zip-26.06-full/tools/7z.exe';pinned(arj,ARJ_SHA);pinned(seven,SEVEN_SHA)
    work=Path(tempfile.mkdtemp(prefix='arj-fixtures-',dir=ROOT/'build/review/format-expansion')).resolve()
    generated={};results={}
    for method in range(5):
        for kind in ('empty','phrase','spaces','allbytes','random','mixed','window21k','window14k'):
            name=f'm{method}-{kind}.arj';directory=work/name.removesuffix('.arj');directory.mkdir()
            plain=plaintext(kind);source=directory/'HELLO.BIN';source.write_bytes(plain);os.utime(source,(1700000000,1700000000))
            command([arj,'a','-y','-e',f'-m{method}','-i','RESULT.ARJ','HELLO.BIN'],directory)
            original=(directory/'RESULT.ARJ').read_bytes();data,row=framing(original,True)
            assert row['decoded']==len(plain) and row['crc32']==binascii.crc32(plain)
            assert row['method'] in (0,method), ('Unexpected native method',name,row['method'])
            assert framing(data)[0]==data
            # Normalize unencrypted timestamps/unused modifier and their header CRCs.
            assert original[row['payload_offset']:row['payload_offset']+row['packed']]==data[row['payload_offset']:row['payload_offset']+row['packed']]
            (directory/'RESULT.ARJ').write_bytes(data)
            target=directory/'seven';target.mkdir();command([seven,'x','-y','-o'+str(target),directory/'RESULT.ARJ'],target)
            assert (target/'HELLO.BIN').read_bytes()==plain, ('7-Zip mismatch',name)
            target=directory/'native';target.mkdir();command([arj,'e','-y','-i',directory/'RESULT.ARJ'],target)
            assert (target/'HELLO.BIN').read_bytes()==plain, ('ARJ mismatch',name)
            generated[name]=data;results[name]=dict(requested=method,kind=kind,**row,archive_sha256=hashlib.sha256(data).hexdigest(),decoded_sha256=hashlib.sha256(plain).hexdigest(),native_exact=True,seven_zip_exact=True)
    identity=hashlib.sha256()
    for name in sorted(generated):identity.update(name.encode()+b'\0'+generated[name])
    if INPUT_SHA:assert identity.hexdigest()==INPUT_SHA, 'ARJ set identity differs'
    dest=ROOT/'tests/fixtures/arj'
    if not args.check:dest.mkdir(exist_ok=True)
    for name,data in generated.items():
        if args.check:assert (dest/name).read_bytes()==data, ('Stale owned ARJ input',name)
        else:(dest/name).write_bytes(data)
    rust=['// Generated owned ARJ expectations, verified by native ARJ and 7-Zip.',
        '#[derive(Clone, Copy)]','pub(crate) struct Case {',
        "    pub name: &'static str,", "    pub kind: &'static str,", '    pub requested: u8,', '    pub method: u8,',
        '    pub offset: usize,', '    pub packed: usize,', '    pub decoded: usize,', '    pub crc: u32,', '}',
        '#[rustfmt::skip]', 'pub(crate) const CASES: &[Case] = &[']
    rust.extend(f'    Case {{ name: "{name}", kind: "{r["kind"]}", requested: {r["requested"]}, method: {r["method"]}, offset: {r["payload_offset"]}, packed: {r["packed"]}, decoded: {r["decoded"]}, crc: 0x{r["crc32"]:08x} }},' for name,r in results.items());rust.append('];')
    output=ROOT/'crates/xdzip-compression/src/arj/fixture_cases.rs';data=('\n'.join(rust)+'\n').encode()
    if args.check:assert output.read_bytes()==data
    else:output.parent.mkdir(parents=True,exist_ok=True);output.write_bytes(data)
    report=dict(source_sha256=SOURCE_SHA,package_sha256=PACKAGE_SHA,native_sha256=ARJ_SHA,seven_zip_sha256=SEVEN_SHA,input_set_sha256=identity.hexdigest(),work=work.relative_to(ROOT).as_posix(),cases=results)
    (ROOT/'build/review/format-expansion/m8-arj-reference.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    counts={str(m):sum(r['method']==m for r in results.values()) for m in range(5)}
    print(f'{len(results)} owned ARJ inputs verified completely by native ARJ and 7-Zip; actual methods {counts}; SHA-256 {identity.hexdigest()}')


if __name__=='__main__':main()
