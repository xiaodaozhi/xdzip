#!/usr/bin/env python3
"""Verify LZH container writer framing, names and complete independent payload reads."""
from pathlib import Path
import base64
import hashlib
import importlib.util
import json
import os
import subprocess
import tempfile

ROOT=Path(__file__).resolve().parents[1]


def main():
    spec=importlib.util.spec_from_file_location('lha_reference',ROOT/'scripts/generate-lha-fixtures.py')
    ref=importlib.util.module_from_spec(spec);spec.loader.exec_module(ref)
    java,lhasa,seven,_=ref.setup(ROOT/'build/toolchain/references',dynamic_reference=False)
    base=ROOT/'build/review/format-expansion';base.mkdir(parents=True,exist_ok=True)
    work=Path(tempfile.mkdtemp(prefix='lzh-create-',dir=base)).resolve()
    assert work.is_relative_to((ROOT/'build').resolve())
    archives=work/'archives'
    p=subprocess.run(['cargo','test','-p','xdzip-codec-lha','--locked','--offline',
        'creation::tests::export_owned_lzh_container_reference_outputs','--','--ignored','--exact'],
        env=dict(os.environ,XDZIP_LZH_CONTAINER_REFERENCE_OUTPUT=str(archives)),cwd=ROOT,capture_output=True,timeout=120)
    if p.returncode:raise ValueError('Owned container export failed: '+p.stderr.decode(errors='replace')[-2000:])
    profiles=dict(ordinary=[('nested/',None),('nested/empty-dir/',None),('nested/hello.bin',b'abracadabra abracadabra\n'*(6*4096)),('empty.bin',b''),('allbytes.bin',bytes(range(256)))],
        unicode=[('资料/',None),('资料/说明.txt',b'owned Unicode name payload\n')])
    results=[]
    for level in (1,6,9):
        for profile,expected in profiles.items():
            archive=archives/f'{profile}-{level}.lzh';data=archive.read_bytes();offset=0
            for name,plain in expected:
                length=int.from_bytes(data[offset:offset+2],'little');header=bytearray(data[offset:offset+length])
                assert length>=31 and header[20]==2 and int.from_bytes(header[15:19],'little')==1700000000
                packed=int.from_bytes(header[7:11],'little');size=int.from_bytes(header[11:15],'little')
                assert header[2:7]==(b'-lhd-' if plain is None else b'-lh5-')
                assert size==len(plain or b'') and int.from_bytes(header[21:23],'little')==ref.crc16(plain or b'')
                at=26;count=int.from_bytes(header[24:26],'little');fields={};crc_at=None
                while count:
                    assert count>=3 and at+count<=len(header)
                    kind=header[at];assert kind not in fields;fields[kind]=bytes(header[at+1:at+count-2])
                    if kind==0:crc_at=at+1
                    count=int.from_bytes(header[at+count-2:at+count],'little');at+=len(fields[kind])+3
                assert at==len(header) or (at%256==0 and len(header)==at+1 and header[-1]==0)
                assert crc_at is not None
                checksum=int.from_bytes(header[crc_at:crc_at+2],'little');header[crc_at:crc_at+2]=b'\0\0';assert ref.crc16(header)==checksum
                path='/'.join(part.decode('utf-8') for part in fields.get(2,b'').split(b'\xff')[:-1])
                path=(path+'/' if path else '')+fields[1].decode('utf-8');assert path==name,(path,name)
                offset+=length+packed
            assert data[offset:]==b'\0'
            decoded=work/f'jlha-{profile}-{level}'
            listing=ref.run([*java,'decode-tree',archive,decoded]).decode('utf-8').splitlines()
            assert len(listing)==len(expected),(profile,listing)
            for i,((name,plain),line) in enumerate(zip(expected,listing)):
                index,encoded,method,size,crc=line.split('\t');assert int(index)==i
                assert base64.b64decode(encoded).decode('utf-8')==name
                assert method==('-lhd-' if plain is None else '-lh5-')
                assert int(size)==len(plain or b'') and int(crc)==ref.crc16(plain or b'')
                assert (decoded/f'{i}.bin').read_bytes()==(plain or b'')
            tool_names={}
            for tool in ('lhasa','sevenzip'):
                target=work/f'{tool}-{profile}-{level}';target.mkdir()
                command=[str(lhasa),'xq',str(archive)] if tool=='lhasa' else [str(seven),'x','-y',f'-o{target}',str(archive)]
                p=subprocess.run(command,cwd=target,capture_output=True,timeout=25,creationflags=0x08000000)
                assert p.returncode==0,(archive.name,tool,p.returncode,p.stderr[:500])
                actual={p.relative_to(target).as_posix():p.read_bytes() for p in target.rglob('*') if p.is_file()}
                wanted={name:plain for name,plain in expected if plain is not None}
                assert sorted(actual.values())==sorted(wanted.values()),(archive.name,tool,'complete contents')
                tool_names[tool]=actual==wanted
                for name,plain in expected:
                    if plain is None and profile=='ordinary':assert (target/name).is_dir()
            p=subprocess.run([str(seven),'t',str(archive)],capture_output=True,timeout=25,creationflags=0x08000000)
            assert p.returncode==0,(archive.name,'7-Zip check')
            results.append(dict(name=archive.name,level=level,entries=len(expected),archive_sha256=hashlib.sha256(data).hexdigest(),
                independent_headers_crc=True,jlha_complete_names_and_payloads=True,lhasa_complete_payloads=True,
                sevenzip_complete_payloads=True,sevenzip_crc=True,native_name_comparison=tool_names))
    (base/'m8-lzh-creation-reference.json').write_text(json.dumps(dict(schema_version=1,generated_directory=work.relative_to(ROOT).as_posix(),cases=results),indent=2)+'\n',encoding='utf-8')
    print(f'{len(results)} LZH containers passed independent framing/name/CRC and complete payload verification; native name details recorded')


if __name__=='__main__':
    main()
