#!/usr/bin/env python3
"""Compare owned Rust LH5 writer output with pinned independent complete readers."""
from pathlib import Path
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
    work=Path(tempfile.mkdtemp(prefix='lh5-writer-',dir=base)).resolve()
    assert work.is_relative_to((ROOT/'build').resolve())
    archives=work/'archives'
    env=dict(os.environ,XDZIP_LH5_REFERENCE_OUTPUT=str(archives))
    p=subprocess.run(['cargo','test','-p','xdzip-compression','--locked','--offline',
        'lzh_encode::tests::export_owned_lh5_reference_outputs','--','--ignored','--exact'],
        env=env,cwd=ROOT,capture_output=True,timeout=120)
    if p.returncode:raise ValueError('Owned writer export failed: '+p.stderr.decode(errors='replace')[-2000:])
    expected=dict(empty=b'',tail1=b'A',allbytes=bytes(range(256)),spaces=b' '*100000,
        random=ref.pattern(65000),long=b'abracadabra abracadabra\n'*(6*4096),window=ref.pattern(8192)*9)
    rows=[]
    for level in (1,6,9):
        for name,plain in expected.items():
            archive=archives/f'{name}-{level}.lzh';data=archive.read_bytes()
            assert len(data)>=34 and data[2:7]==b'-lh5-' and data[20]==0
            assert data[0]==31 and data[1]==sum(data[2:33])%256 and data[-1]==0
            assert int.from_bytes(data[7:11],'little')==len(data)-34
            assert int.from_bytes(data[11:15],'little')==len(plain)
            assert int.from_bytes(data[31:33],'little')==ref.crc16(plain)
            decoded=work/f'{name}-{level}.bin'
            ref.run([*java,'decode',archive,decoded]);assert decoded.read_bytes()==plain
            extracted=work/f'lhasa-{name}-{level}';extracted.mkdir()
            p=subprocess.run([str(lhasa),'xq',str(archive)],cwd=extracted,capture_output=True,timeout=20,creationflags=0x08000000)
            assert p.returncode==0,(archive.name,'Lhasa',p.returncode)
            assert (extracted/'hello.bin').read_bytes()==plain
            p=subprocess.run([str(seven),'x','-so',str(archive)],capture_output=True,timeout=20,creationflags=0x08000000)
            assert p.returncode==0 and p.stdout==plain,(archive.name,'7-Zip',p.returncode)
            p=subprocess.run([str(seven),'t',str(archive)],capture_output=True,timeout=20,creationflags=0x08000000)
            assert p.returncode==0,(archive.name,'7-Zip CRC',p.returncode)
            rows.append(dict(name=archive.name,level=level,method='lh5',packed=len(data)-34,decoded=len(plain),
                archive_sha256=hashlib.sha256(data).hexdigest(),decoded_sha256=hashlib.sha256(plain).hexdigest(),
                jlha_exact=True,lhasa_exact=True,sevenzip_exact=True,sevenzip_crc=True))
    report=dict(schema_version=1,scope='raw LH5 writer, owned level-0 test framing; product creation is separate',
        generated_directory=work.relative_to(ROOT).as_posix(),cases=rows)
    (base/'m8-lh5-writer-reference.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(f'{len(rows)} LH5 writer cases passed complete jLHA, Lhasa and 7-Zip reads and independent header/CRC checks')


if __name__=='__main__':
    main()
