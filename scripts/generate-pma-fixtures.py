#!/usr/bin/env python3
"""Generate owned PMA data with pinned original tools and independent complete readers."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import zipfile

ROOT=Path(__file__).resolve().parents[1]
COMMIT='275c977c3b3dd058586f4acf49d467c69fcf3969'
SOURCE_SHA='c4ec97fe9d5580b20c389d22efdce8d17eee3c7b5af075b68e5f2090c8d27c80'
COMPILER_SHA='a30b01810e44edaa465cc51f1dffae6267cb0092cc653d71b881970866039cca'
PMARC_SHA='28aa664d70d6c464d459a7e2db8be5b2a22d62d4d41485e5acd58f27b7ce32da'
PMEXT_SHA='5d4c0cf8b10c2ed4fa27998bcaabec7018139da5ca31fee0d1c36c96073a1f9e'
LHASA_SHA='93cf8100eb520e49708c5076694253d52caf241d6013be4e37c11ec8e89bbe85'
INPUT_SHA='df253fd3cdb37337bd6fa25d6cebbc22fee69c287b04b38ab225237f959763f3'


def pinned(path,expected):
    if path.is_symlink() or hashlib.sha256(path.read_bytes()).hexdigest()!=expected:
        raise ValueError('Modified reference component: '+path.name)


def command(args,*,cwd=None,env=None):
    p=subprocess.run(list(map(str,args)),cwd=cwd,env=env,input=b'',capture_output=True,timeout=45,creationflags=0x08000000)
    if p.returncode:raise ValueError('Reference command failed: '+Path(args[0]).name+' '+str(p.returncode))
    return p


def setup():
    refs=ROOT/'build/toolchain/references';base=refs/'cpmemu-275c977c'
    pinned(base/'source.zip',SOURCE_SHA)
    prefix='cpmemu-'+COMMIT+'/'
    with zipfile.ZipFile(base/'source.zip') as z:
        for entry in z.infolist():
            if entry.is_dir() or not entry.filename.startswith(prefix):continue
            name=entry.filename.removeprefix(prefix)
            if not (name.startswith('src/') or name in ['LICENSE','README.md','docs/usage.md']):continue
            target=(base/'source'/name).resolve()
            if not target.is_relative_to((base/'source').resolve()) or target.is_symlink():raise ValueError('Reference path escaped')
            data=z.read(entry)
            if target.exists() and target.read_bytes()!=data:raise ValueError('Modified reference source')
            if not target.exists():target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
    compiler=ROOT/'build/toolchain/strawberry-perl-5.42.3.1/c/bin/g++.exe';pinned(compiler,COMPILER_SHA)
    emulator=base/'cpmemu-verified.exe'
    command([compiler,'-std=c++11','-O2','-static','-Wl,--no-insert-timestamp','-I.',
        'cpmemu.cc','gdbstub.cc','os/windows/platform.cc','qkz80.cc','qkz80_errors.cc','qkz80_mem.cc','qkz80_reg_set.cc',
        '-lws2_32','-o',emulator],cwd=base/'source/src')
    if emulator.stat().st_size<100000 or emulator.read_bytes()[:2]!=b'MZ':raise ValueError('Missing compiled reference executable')
    pmarc=refs/'pmarc-2.0/original/pmarc2.com';pinned(pmarc,PMARC_SHA)
    pinned(refs/'pmarc-2.0/pmarc200-original.pma','e735229dc938358230af86c74235fb4b358dfdd9012b1311816371798a30d294')
    pmext=refs/'pmarc-2.0/PMEXT.CPM';pinned(pmext,PMEXT_SHA)
    pinned(refs/'pmarc-2.0/pmarcpc.zip','668a81cc3d77eee41255874812a543ac28d4adf9991f04e5d48bcbe1d7fdbff5')
    lhasa=refs/'lhasa-0.6.0/runtime/lha.exe';pinned(lhasa,LHASA_SHA)
    return emulator,pmarc,pmext,lhasa


def cpm(emulator,program,directory,args,files):
    directory.mkdir()
    (directory/'TOOL.COM').write_bytes(program.read_bytes())
    for name,data in files.items():(directory/name).write_bytes(data);os.utime(directory/name,(1700000000,1700000000))
    config='program = TOOL.COM\ncd = .\ndrive_A = .\n'
    config+=''.join(f'{name} = {name} binary\n' for name in dict.fromkeys([*files,'RESULT.PMA','HELLO.BIN']))
    (directory/'owned.cfg').write_text(config,encoding='utf-8')
    env={k:v for k,v in os.environ.items() if not k.upper().startswith('CPM_')}
    p=command([emulator,directory/'owned.cfg',*args],cwd=directory,env=env)
    (directory/'stdout.bin').write_bytes(p.stdout);(directory/'stderr.txt').write_bytes(p.stderr)


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    spec=importlib.util.spec_from_file_location('lha_fields',ROOT/'scripts/generate-lha-fixtures.py')
    fields=importlib.util.module_from_spec(spec);spec.loader.exec_module(fields)
    emulator,pmarc,pmext,lhasa=setup()
    base=ROOT/'build/review/format-expansion';base.mkdir(parents=True,exist_ok=True)
    work=Path(tempfile.mkdtemp(prefix='pma-fixtures-',dir=base)).resolve()
    assert work.is_relative_to((ROOT/'build').resolve())
    cycle=bytes(range(256))+b'abracadabra abracadabra\n'*128
    def repeated(size):return (cycle*((size+len(cycle)-1)//len(cycle)))[:size]
    cases=[(f'pm2-boundary-{n}.pma',2,repeated(n)) for n in [128,256,512,1024,2048,4096,8192,16384]]
    cases += [('pm2-spaces.pma',2,b' '*8192),('pm2-zeros.pma',2,bytes(65536)),
        ('pm2-allbytes.pma',2,bytes(range(256))*256),('pm2-long.pma',2,repeated(131072)),
        ('pm0-allbytes.pma',0,bytes(range(256))),('empty.pma',255,b''),
        ('pm2-deep28.pma',2,bytes(range(39,31,-1))*16),('pm2-prologue-zero.pma',2,repeated(512))]
    results={};dest=ROOT/'tests/fixtures/pma';dest.mkdir(exist_ok=True)
    emulator_sha=hashlib.sha256(emulator.read_bytes()).hexdigest();generated={}
    for name,method,plain in cases:
        pinned(emulator,emulator_sha)
        encoded=work/name.removesuffix('.pma')
        wire=name in ['pm2-deep28.pma','pm2-prologue-zero.pma']
        if name=='pm2-deep28.pma':
            values=[(0,1),(29,5),(1,3),(5,3)]+[(n,5) for n in [28,*range(1,28),28]]+[(0,3)]*5+[((1<<28)-2,28),(7,3)]*128
            raw=fields.fields(values);h=bytearray(generated['pm2-boundary-128.pma'][:33])
            h[7:11]=len(raw).to_bytes(4,'little');h[11:15]=len(plain).to_bytes(4,'little');h[31:33]=fields.crc16(plain).to_bytes(2,'little');h[1]=sum(h[2:])%256
            data=bytes(h)+raw+b'\0';data+=b'\x1a'*(-len(data)%128);encoded.mkdir();(encoded/'RESULT.PMA').write_bytes(data)
        elif name=='pm2-prologue-zero.pma':
            data=bytearray(generated['pm2-boundary-512.pma']);assert data[33]&0x80!=0;data[33]^=0x80;encoded.mkdir();(encoded/'RESULT.PMA').write_bytes(data)
        else:
            cpm(emulator,pmarc,encoded,['RESULT.PMA','HELLO.BIN/B'+('N' if method==0 else '')],{'HELLO.BIN':plain})
        data=(encoded/'RESULT.PMA').read_bytes()
        generated[name]=data
        if method==255:
            assert data==b'\0'+b'\x1a'*127
            header=packed=decoded=0;tail=data
        else:
            header=data[0]+2;packed=int.from_bytes(data[7:11],'little');decoded=int.from_bytes(data[11:15],'little');tail=data[header+packed:]
            assert header==33 and data[2:7]==f'-pm{method}-'.encode() and data[20]==0 and decoded==len(plain)
            assert data[1]==sum(data[2:header])%256 and data[22:31]==b'HELLO.BIN'
            assert int.from_bytes(data[31:33],'little')==fields.crc16(plain)
        assert len(data)%128==0 and 1<=len(tail)<=128 and tail==b'\0'+b'\x1a'*(len(tail)-1)
        target=work/(name+'-lhasa');target.mkdir()
        command([lhasa,'xq',encoded/'RESULT.PMA'],cwd=target)
        output=[p for p in target.iterdir() if p.is_file()]
        assert (not output) if method==255 else (len(output)==1 and output[0].read_bytes()==plain)
        target=work/(name+'-pmext')
        cpm(emulator,pmext,target,['SOURCE.PMA','/C' if method==255 else 'HELLO.BIN/B'],{'SOURCE.PMA':data})
        if method==255:assert not (target/'HELLO.BIN').exists()
        else:assert (target/'HELLO.BIN').is_file() and (target/'HELLO.BIN').read_bytes()==plain
        if args.check:
            if (dest/name).read_bytes()!=data:raise ValueError('Stale PMA input: '+name)
        else:(dest/name).write_bytes(data)
        results[name]=dict(method=method,header=header,packed=packed,decoded=decoded,crc16=fields.crc16(plain),
            origin='wire-authored' if wire else 'original-author-encoder',archive_sha256=hashlib.sha256(data).hexdigest(),decoded_sha256=hashlib.sha256(plain).hexdigest(),lhasa_exact=True,pmext_exact=True)
    identity=hashlib.sha256()
    for name in sorted(results):identity.update(name.encode()+b'\0'+(dest/name).read_bytes())
    if INPUT_SHA and identity.hexdigest()!=INPUT_SHA:raise ValueError('PMA input set differs from its admitted identity')
    rust=['// Generated independently verified owned PMA expectations, not decoder output.','#[rustfmt::skip]','pub(crate) const CASES: &[(&str, u8, usize, usize, usize, u16)] = &[']
    rust.extend(f'    ("{name}", {v["method"]}, {v["header"]}, {v["packed"]}, {v["decoded"]}, 0x{v["crc16"]:04x}),' for name,v in results.items());rust.append('];')
    output=ROOT/'crates/xdzip-compression/src/pm2/fixture_cases.rs';data=('\n'.join(rust)+'\n').encode()
    if args.check:
        if output.read_bytes()!=data:raise ValueError('Stale PMA expectations')
    else:output.parent.mkdir(parents=True,exist_ok=True);output.write_bytes(data)
    report=dict(emulator_source_commit=COMMIT,emulator_sha256=emulator_sha,generated_directory=work.relative_to(ROOT).as_posix(),input_set_sha256=identity.hexdigest(),cases=results)
    (base/'m8-pma-reference.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(f'{len(results)} owned PMA inputs (14 original-author, 2 wire-authored) verified with complete Lhasa and PMEXT reads; set SHA-256 {identity.hexdigest()}')


if __name__=='__main__':
    main()
