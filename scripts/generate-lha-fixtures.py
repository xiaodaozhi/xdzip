#!/usr/bin/env python3
"""Generate authored LHA inputs through verified public Java APIs, with independent readers."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import zipfile

ROOT=Path(__file__).resolve().parents[1]
COMMIT='0a6348d96e9bbb1ac5ee8b3f6ff1c2c75b17d0d9'
SOURCE_SHA='ef9feb1c59f55aff69b00c0d69dee5edaefb45e86950a99a6728beb35e70ff3a'
JDK_SHA='98518ecf554465057ee5509462631deeee374bc368c12a411d225cd29381f84b'
LHASA_SHA='5925e6a7b615d784e381e18240a1537d55a42cddad91d21e13c604a40974fe28'
INPUT_SHA='8866cb95bfac04a90895c5e426dc8689bba7a5e508d8d294803a7f79849ee20b'


def run(command):
    p=subprocess.run(list(map(str,command)),capture_output=True,timeout=40,creationflags=0x08000000)
    if p.returncode:raise ValueError('Owned independent-reference command failed: '+Path(command[0]).name)
    return p.stdout


def pinned(path,sha):
    if path.is_symlink() or not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest()!=sha:
        raise ValueError('Reference identity mismatch: '+path.name)


def setup(base):
    ref=base/'jlha-0.0.11';pinned(ref/('jlha-'+COMMIT+'.zip'),SOURCE_SHA)
    with zipfile.ZipFile(ref/('jlha-'+COMMIT+'.zip')) as z:
        for e in z.infolist():
            if e.is_dir():continue
            parts=Path(e.filename).parts;name=Path(*parts[1:]).as_posix()
            if name.startswith('src/main/') or name in ['LICENSE','README.md','pom.xml','CHANGELOG.md']:
                path=(ref/'source'/name).resolve()
                if not path.is_relative_to((ref/'source').resolve()) or path.is_symlink() or path.read_bytes()!=z.read(e):raise ValueError('Modified Java reference source')
    java_root=base/'temurin-17.0.21'
    pinned(java_root/'OpenJDK17U-jdk_x64_windows_hotspot_17.0.21_9.zip',JDK_SHA)
    with zipfile.ZipFile(java_root/'OpenJDK17U-jdk_x64_windows_hotspot_17.0.21_9.zip') as z:
        for e in z.infolist():
            if e.is_dir():continue
            path=(java_root/'runtime'/e.filename).resolve()
            if not path.is_relative_to((java_root/'runtime').resolve()) or path.is_symlink() or path.read_bytes()!=z.read(e):raise ValueError('Modified Java runtime component')
    jdk=next((java_root/'runtime').iterdir());classes=(ref/'classes').resolve();classes.mkdir(exist_ok=True)
    sources=[*sorted((ref/'source/src/main/java').resolve().rglob('*.java')),ROOT/'tests/fixtures/lha/Reference.java']
    args=ref/'java-sources.txt';args.write_text('\n'.join('"'+p.as_posix()+'"' for p in sources)+'\n',encoding='utf-8')
    run([jdk/'bin/javac.exe','--release','17','-encoding','UTF-8','-d',classes,'@'+str(args.resolve())])
    for path in (ref/'source/src/main/resources').rglob('*'):
        if path.is_file():
            target=classes/path.relative_to(ref/'source/src/main/resources');target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(path,target)
    lhasa_root=base/'lhasa-0.6.0';pinned(lhasa_root/'lhasa-0.6.0-win32.zip',LHASA_SHA)
    lhasa=lhasa_root/'runtime/lha.exe'
    pinned(lhasa,'93cf8100eb520e49708c5076694253d52caf241d6013be4e37c11ec8e89bbe85')
    seven=base/'7zip-26.06-full/tools/7z.exe'
    pinned(seven,'dc0493429fe28136fcb177a0e77c9875cbb3b0bff5f2f8eeba4df302e98cef76')
    pinned(seven.with_name('7z.dll'),'500db9eb17f3c21b729ba41dd02ecfe1823da05028c7e4ba129084cc74cc364e')
    return [jdk/'bin/java.exe','-Duser.timezone=UTC','-cp',classes,'Reference'],lhasa,seven


def pattern(count):
    seed=0x6d2b79f5;result=bytearray()
    for _ in range(count):
        seed^=(seed<<13)&0xffffffff;seed^=seed>>17;seed^=(seed<<5)&0xffffffff;result.append(seed>>24)
    return bytes(result)


def crc16(data):
    value=0
    for byte in data:
        value^=byte
        for _ in range(8):value=(value>>1)^(0xa001 if value&1 else 0)
    return value


def fields(values):
    bits=''.join(f'{value:0{width}b}' for value,width in values)
    bits+='0'*(-len(bits)%8)
    return int(bits,2).to_bytes(len(bits)//8,'big')


def framed(method,raw,payload):
    name=b'hello.bin'
    header=bytearray([24+len(name)-2,0])+('-'+method+'-').encode()
    header+=len(raw).to_bytes(4,'little')+len(payload).to_bytes(4,'little')
    header+=bytes.fromhex('aab16e572000')+bytes([len(name)])+name+crc16(payload).to_bytes(2,'little')
    header[1]=sum(header[2:])&255
    return bytes(header)+raw+b'\0',len(header)


def initial_vectors():
    # Wire-authored initial references distinguish prefilled history from
    # dictionaries learned from actual preceding output.
    return [
        ('lh5-initial.lzh','lh5',fields([(1,16),(0,5),(0,5),(0,9),(256,9),(0,4),(0,4)]),b' '*3),
        ('lzs-initial.lzh','lzs',fields([(0,1),(0,11),(15,4)]),b' '*17),
        ('lz5-initial.lzh','lz5',bytes([0,0,15]),bytes(13)+bytes([1])*5),
    ]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--references',type=Path,default=ROOT/'build/toolchain/references')
    parser.add_argument('--check',action='store_true');args=parser.parse_args()
    java,lhasa,seven=setup(args.references.resolve())
    methods=['lh0','lh1','lh2','lh3','lh4','lh5','lh6','lh7','lzs','lz4','lz5']
    phrase=b'abracadabra abracadabra\n'*6
    cases=[(f'{method}-h{level}.lzh',method,level,phrase) for method in methods for level in range(3)]
    for method in methods:
        if method in ['lh0','lz4']:continue
        for label,payload in [('empty',b''),('tail1',b'A'),('spaces',bytes([32])*4096),('random',pattern(65000)),('long',phrase*4096)]:
            cases.append((f'{method}-{label}.lzh',method,2,payload))
    dest=ROOT/'tests/fixtures/lha';dest.mkdir(parents=True,exist_ok=True)
    report=ROOT/'build/review/format-expansion';report.mkdir(parents=True,exist_ok=True);identities={}
    with tempfile.TemporaryDirectory(prefix='lha-reference-',dir=report) as temporary:
        work=Path(temporary).resolve()
        if not work.is_relative_to(report.resolve()):raise ValueError('Reference scratch directory escapes build/')
        source=work/'owned.bin';archive=work/'owned.lzh';decoded=work/'decoded.bin'
        for name,method,level,payload in cases:
            source.write_bytes(payload)
            fields=list(map(int,run([*java,'encode',method,str(level),source,archive]).decode('ascii').split()))
            header,packed,size,crc=fields;data=archive.read_bytes()
            if size!=len(payload) or data[2:7]!=('-'+method+'-').encode() or len(data)!=header+packed+1:raise ValueError('Unexpected reference framing')
            run([*java,'decode',archive,decoded])
            if decoded.read_bytes()!=payload:raise ValueError('Independent Java output mismatch: '+name)
            target=work/name;target.mkdir()
            p=subprocess.run([str(lhasa),'xq',str(archive)],cwd=target,capture_output=True,timeout=15,creationflags=0x08000000)
            lhasa_exact=p.returncode==0 and (target/'hello.bin').is_file() and (target/'hello.bin').read_bytes()==payload
            if method in ['lh2','lh3']:
                if p.returncode!=1:raise ValueError('Changed expected Lhasa method limitation')
            elif not lhasa_exact:raise ValueError('Independent Lhasa output mismatch: '+name)
            p=subprocess.run([str(seven),'x','-so',str(archive)],capture_output=True,timeout=15,creationflags=0x08000000)
            seven_exact=p.returncode==0 and p.stdout==payload
            if method in ['lh0','lh4','lh5','lh6','lh7','lz4']:
                if not seven_exact:raise ValueError('Independent 7-Zip output mismatch: '+name)
            elif p.returncode!=2:raise ValueError('Changed expected 7-Zip method limitation')
            target=dest/name
            if args.check:
                if target.read_bytes()!=data:raise ValueError('Stale owned input: '+name)
            else:target.write_bytes(data)
            identities[name]=dict(method=method,level=level,header=header,packed=packed,decoded=size,crc16=crc,sha256=hashlib.sha256(data).hexdigest(),jlha_exact=True,lhasa_exact=lhasa_exact,sevenzip_exact=seven_exact)
        for name,method,raw,payload in initial_vectors():
            data,header=framed(method,raw,payload);archive.write_bytes(data)
            run([*java,'decode',archive,decoded]);java_exact=decoded.read_bytes()==payload
            if java_exact!=(method=='lz5'):raise ValueError('Changed expected Java initial history behavior')
            target=work/name;target.mkdir()
            p=subprocess.run([str(lhasa),'xq',str(archive)],cwd=target,capture_output=True,timeout=15,creationflags=0x08000000)
            if p.returncode or (target/'hello.bin').read_bytes()!=payload:raise ValueError('Independent initial history mismatch: '+name)
            p=subprocess.run([str(seven),'x','-so',str(archive)],capture_output=True,timeout=15,creationflags=0x08000000)
            if p.returncode!=2:raise ValueError('Changed expected 7-Zip initial history limitation')
            target=dest/name
            if args.check:
                if target.read_bytes()!=data:raise ValueError('Stale owned initial history input: '+name)
            else:target.write_bytes(data)
            identities[name]=dict(method=method,level=0,header=header,packed=len(raw),decoded=len(payload),crc16=crc16(payload),sha256=hashlib.sha256(data).hexdigest(),jlha_exact=java_exact,lhasa_exact=True,sevenzip_exact=False)
    identity=hashlib.sha256()
    for name in sorted(identities):identity.update(name.encode('utf-8')+b'\0'+(dest/name).read_bytes())
    if len(identities)!=81 or identity.hexdigest()!=INPUT_SHA:raise ValueError('Owned input set does not match documented identity')
    rust=['// Generated independent LHA fixture expectations, not decoder output.','#[rustfmt::skip]','pub(crate) const CASES: &[(&str, &str, usize, usize, usize, u16)] = &[']
    rust.extend(f'    ("{name}", "{v["method"]}", {v["header"]}, {v["packed"]}, {v["decoded"]}, 0x{v["crc16"]:04x}),' for name,v in identities.items());rust.append('];')
    target=ROOT/'crates/xdzip-compression/src/lzh/fixture_cases.rs';data=('\n'.join(rust)+'\n').encode()
    if args.check:
        if target.read_bytes()!=data:raise ValueError('Stale fixture expectations')
    else:target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
    (report/'m8-lha-reference.json').write_text(json.dumps(identities,indent=2)+'\n',encoding='utf-8')
    print(f'{len(identities)} authored LHA inputs verified; reference method and initial-history limitations recorded separately')


if __name__=='__main__':
    main()
