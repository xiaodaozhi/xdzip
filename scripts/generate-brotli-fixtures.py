#!/usr/bin/env python3
"""Generate authored Brotli cases and cross-check full output using the pinned author codec."""
import argparse
import ast
import hashlib
import json
import lzma
from pathlib import Path
import re
import struct
import sys
import zipfile
import zlib

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / 'tests/fixtures/brotli'
WHEEL_SHA = 'f5342b8256476554b4045db8b7bf4dc4044d0405485f220f0cd80764bc0d7faf'
RFC_SHA = '394abd1879e016f4eac89512c2acaf92f7b2d67feb8ab58b6b118c213be07146'
ORDER = [1,2,3,4,0,5,17,6,16,7,8,9,10,11,12,13,14,15]


class Wire:
    def __init__(self):
        self.data = bytearray()
        self.used = 0

    def put(self, value, width):
        assert 0 <= value < 1 << width
        for n in range(width):
            if not self.used:
                self.data.append(0)
            self.data[-1] |= ((value >> n) & 1) << self.used
            self.used = (self.used + 1) % 8

    def code(self, value, width):
        for n in reversed(range(width)):
            self.put((value >> n) & 1, 1)

    def align(self):
        if self.used:
            self.put(0, 8-self.used)


def fixed(wire, alphabet):
    """A complete canonical tree, derived from alphabet size alone."""
    width = (alphabet-1).bit_length()
    short = (1 << width)-alphabet
    lengths = [width-1]*short+[width]*(alphabet-short)
    values = sorted(set(lengths))
    wire.put(0, 2)
    last = 17 if len(values)==1 else max(ORDER.index(n) for n in values)
    for symbol in ORDER[:last+1]:
        wire.put(0b0111, 4) if symbol in values else wire.put(0, 2)
    if len(values)==2:
        for n in lengths:
            wire.put(values.index(n), 1)
    return lambda symbol: wire.code(symbol if symbol<short else symbol+short, width-1 if symbol<short else width)


def plain_header(wire, last, length, uncompressed=False):
    wire.put(int(last),1)
    if last:
        wire.put(0,1)
    width=max(4, ((length-1).bit_length()+3)//4)
    wire.put(width-4,2)
    wire.put(length-1,width*4)
    if not last:
        wire.put(int(uncompressed),1)


def number(wire, value):
    if value==1:
        wire.put(0,1)
    else:
        wire.put(1,1)
        width=(value-1).bit_length()-1
        wire.put(width,3)
        if width:wire.put(value-(1<<width)-1,width)


def singleton(wire, alphabet, value):
    wire.put(1,2);wire.put(0,2);wire.put(value,(alphabet-1).bit_length())


def mapping(wire, values, trees):
    number(wire,trees)
    if trees==1:return
    wire.put(1,1);wire.put(6,4)  # RLEMAX 7, followed by inverse MTF.
    emit=fixed(wire,trees+7)
    mtf=list(range(256));indexes=[]
    for value in values:
        index=mtf.index(value);indexes.append(index);mtf.pop(index);mtf.insert(0,value)
    at=0
    while at<len(indexes):
        if indexes[at]:
            emit(indexes[at]+7);at+=1
        else:
            end=at+1
            while end<len(indexes) and indexes[end]==0:end+=1
            count=end-at
            while count:
                if count==1:emit(0);count=0
                else:
                    width=min(7,count.bit_length()-1);repeat=min(count,(1<<(width+1))-1)
                    emit(width);wire.put(repeat-(1<<width),width);count-=repeat
            at=end
    wire.put(1,1)


def literal_context_case(rfc, mode):
    text=rfc.decode('ascii')
    tables=[[int(n) for n in re.findall(r'\d+',re.search(f'Lut{i} :='+r'\s*([0-9,\s]+)',text)[1])] for i in range(3)]
    expected=bytearray([0xc3,0xa9]);values=[97,32,195]
    for _ in range(256):
        last,before=expected[-1],expected[-2]
        context=[last&63,last>>2,tables[0][last]|tables[1][before],(tables[2][last]<<3)|tables[2][before]][mode]
        expected.append(values[context%3])
    wire=Wire();wire.put(0,1);plain_header(wire,False,2,True);wire.align();wire.put(0xc3,8);wire.put(0xa9,8)
    plain_header(wire,True,256)
    for _ in range(3):number(wire,1)
    wire.put(0,2);wire.put(0,4);wire.put(mode,2)
    mapping(wire,[n%3 for n in range(64)],3);mapping(wire,[0]*4,1)
    for value in values:singleton(wire,256,value)
    singleton(wire,704,456)  # insert 194+62 literals, copy omitted at block end
    singleton(wire,64,0)
    wire.put(62,7)
    wire.align();return bytes(wire.data),bytes(expected)


def block_switch_case():
    wire=Wire();wire.put(0,1);plain_header(wire,False,1,True);wire.align();wire.put(88,8)
    plain_header(wire,True,96)
    for _ in range(3):
        number(wire,2);singleton(wire,4,1);singleton(wire,26,0);wire.put(0,2)
    wire.put(0,2);wire.put(1,4);wire.put(0,2);wire.put(0,2)
    mapping(wire,[0]*64+[1]*64,2);mapping(wire,[0]*4+[1]*4,2)
    for value in [65,66]:singleton(wire,256,value)
    for _ in range(2):singleton(wire,704,136)
    for _ in range(2):singleton(wire,65,16)
    expected=bytearray(b'X')
    for at in range(32):
        # Switch command, literal and distance categories independently after
        # the initial count-one block. Each singleton count code has 2 extra bits.
        if at:
            wire.put(0,2);wire.put(0,2);wire.put(0,2)
        expected.extend(bytes([65+(at%2)])*3)
    wire.align();return bytes(wire.data),bytes(expected)


def distance_code(value, postfix, direct):
    if value <= direct:
        return 15+value, 0, 0
    for code in range(16+direct,16+direct+(48<<postfix)):
        delta=code-direct-16; high=delta>>postfix; low=delta&((1<<postfix)-1)
        width=1+(high>>1); offset=((2+(high&1))<<width)-4
        minimum=(offset<<postfix)+low+direct+1
        difference=value-minimum
        if difference>=0 and difference%(1<<postfix)==0:
            extra=difference>>postfix
            if extra<1<<width:
                return code,extra,width
    raise ValueError('No normative distance code')


def dictionary_cases(rfc, postfix=0, direct_field=0, all_words=False):
    text=rfc.decode('ascii')
    a=text[text.index('Appendix A.  Static Dictionary Data'):text.index('Appendix B.  List of Word Transformations')]
    dictionary=bytes.fromhex(''.join(re.findall(r'^      ([0-9a-f]{20,})\s*$',a,re.M)))
    depths=[int(n) for n in re.findall(r'\d+',re.search(r'NDBITS :=\s*([0-9,\s]+)',text)[1])]
    b=text[text.index('Appendix B.  List of Word Transformations'):text.index('Appendix C.  Computing CRC-32 Check Values')]
    pattern=r'^\s+(\d+)\s+("(?:\\.|[^"\\])*")\s+(Identity|FermentFirst|FermentAll|OmitFirst[1-9]|OmitLast[1-9])\s+("(?:\\.|[^"\\])*")\s*$'
    transforms=re.findall(pattern,b,re.M);assert len(transforms)==121
    records=[];expected=bytearray()
    for length in range(4,25) if all_words else [4,8,24]:
        start=sum(n*(1<<depths[n]) for n in range(4,length))
        for index in [0,(1<<depths[length])-1] if all_words else [0]:
            for transform,prefix,op,suffix in transforms:
                word=bytearray(dictionary[start+length*index:start+length*(index+1)])
                if op.startswith('OmitFirst'):
                    word=word[int(op[-1]):]
                elif op.startswith('OmitLast'):
                    word=word[:max(0,len(word)-int(op[-1]))]
                elif op.startswith('Ferment'):
                    at=0
                    while at<len(word):
                        if word[at]<192:
                            if 97<=word[at]<=122:
                                word[at]^=32
                            advance=1
                        elif word[at]<224:
                            if at+1<len(word):word[at+1]^=32
                            advance=2
                        else:
                            if at+2<len(word):word[at+2]^=5
                            advance=3
                        if op=='FermentFirst':break
                        at+=advance
                payload=ast.literal_eval('b'+prefix)+word+ast.literal_eval('b'+suffix)
                distance=min(len(expected)+1,65520)+(int(transform)<<depths[length])+index+1
                records.append((length,distance))
                expected.append(33);expected.extend(payload)
    wire=Wire();wire.put(0,1)  # Standard 16-bit window.
    plain_header(wire,True,len(expected))
    for _ in range(3):wire.put(0,1)  # one block type per category
    wire.put(postfix,2);wire.put(direct_field,4);wire.put(0,2)
    wire.put(0,1);wire.put(0,1)  # one literal/distance context tree
    wire.put(1,2);wire.put(0,2);wire.put(33,8)  # singleton literal
    command=fixed(wire,704)
    direct=direct_field<<postfix
    distance=fixed(wire,16+direct+(48<<postfix))
    copy_widths=[0]*8+[1,1,2,2,3,3,4,4,5,5,6,7,8,9,10,24]
    for length,value in records:
        base=2
        for symbol,width in enumerate(copy_widths):
            if length<base+(1<<width):break
            base+=1<<width
        command((136 if symbol<8 else 200)+(symbol%8))
        wire.put(length-base,width)
        code,extra,width=distance_code(value,postfix,direct)
        distance(code);wire.put(extra,width)
    wire.align()
    return bytes(wire.data),bytes(expected)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference',type=Path,required=True)
    parser.add_argument('--rfc',type=Path,required=True)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args();reference=args.reference.resolve()
    wheel=reference/'brotli-1.2.1-cp312-cp312-win_amd64.whl'
    if hashlib.sha256(wheel.read_bytes()).hexdigest()!=WHEEL_SHA:raise ValueError('Unexpected Brotli reference identity')
    with zipfile.ZipFile(wheel) as archive:
        for info in archive.infolist():
            path=(reference/'python'/info.filename).resolve()
            if not path.is_relative_to(reference/'python'):raise ValueError('Invalid reference path')
            if not info.is_dir() and path.read_bytes()!=archive.read(info):raise ValueError('Modified reference component')
    cache=reference/'python/__pycache__'
    for bytecode in cache.glob('brotli.*.pyc'):
        if not bytecode.resolve().is_relative_to(reference/'python'):
            raise ValueError('Reference bytecode escapes its fixed root')
        bytecode.unlink()
    sys.dont_write_bytecode=True
    sys.path.insert(0,str(reference/'python'));import brotli
    if brotli.__version__!='1.2.1':raise ValueError('Unexpected loaded Brotli version')
    rfc=args.rfc.read_bytes()
    if hashlib.sha256(rfc).hexdigest()!=RFC_SHA:raise ValueError('Unexpected RFC identity')
    phrase=b'abracadabra abracadabra\n'*6;cases=[]
    for quality in range(12):
        for mode,label in [(0,'generic'),(1,'text'),(2,'font')]:
            cases.append((f'q{quality}-{label}.br',brotli.compress(phrase,mode=mode,quality=quality,lgwin=16),phrase))
    for window in range(10,25):
        cases.append((f'window-{window}.br',brotli.compress(phrase,quality=5,lgwin=window),phrase))
    varied=(b'<p>Time and transformation, static dictionary words.</p>\n'*700+'自有 Brotli 输入，跨字节上下文。\n'.encode()*500+bytes(range(256))*200)
    samples=[('empty',b''),('all-bytes',bytes(range(256))),('varied',varied),('repeated',phrase*8000),('integers',b''.join(struct.pack('<hhhh',i%30000,-(i%30000),i%256,0) for i in range(32768)))]
    for label,payload in samples:
        cases.append((label+'.br',brotli.compress(payload,quality=11,mode=2 if label=='integers' else 1),payload))
    compressor=brotli.Compressor(quality=9,lgwin=16)
    payload=phrase*1000+bytes(range(256))*300+varied
    parts=[phrase*1000,bytes(range(256))*300,varied]
    packed=b''.join(compressor.process(part)+compressor.flush() for part in parts)+compressor.finish()
    cases.append(('meta-blocks.br',packed,payload))
    for mode in range(4):
        data,payload=literal_context_case(rfc,mode)
        cases.append((f'context-{mode}.br',data,payload))
    data,payload=block_switch_case();cases.append(('block-switches.br',data,payload))
    data,payload=dictionary_cases(rfc,all_words=True)
    cases.append(('dictionary-all.br',data,payload))
    for postfix in range(4):
        for field in [0,1,15]:
            data,payload=dictionary_cases(rfc,postfix,field)
            cases.append((f'distance-{postfix}-{field}.br',data,payload))
    for label,metadata,body,last in [('metadata',b'owned opaque metadata',phrase,False),('empty-meta',b'xyz',b'',True),('raw',b'',b'Z',False)]:
        wire=Wire();wire.put(0,1)
        if metadata:
            wire.put(int(last),1)
            if last:wire.put(0,1)
            wire.put(3,2);wire.put(0,1);wire.put(1,2);wire.put(len(metadata)-1,8);wire.align()
            for byte in metadata:wire.put(byte,8)
        if body:
            plain_header(wire,False,len(body),uncompressed=True);wire.align()
            for byte in body:wire.put(byte,8)
        if not last:wire.put(1,1);wire.put(1,1)
        wire.align();cases.append((label+'.br',bytes(wire.data),body))
    for name,payload in [
        ('files.tar.br',lzma.decompress((ROOT/'tests/fixtures/streams/files.txz').read_bytes())),
        ('bad-header.br',lzma.decompress((ROOT/'tests/fixtures/streams/bad-header.txz').read_bytes())),
        ('empty.tar.br',bytes(1024)),('empty-tar.br',bytes(1024)),('zeros.br',bytes(4096)),
    ]:
        cases.append((name,brotli.compress(payload,quality=5,lgwin=16),payload))
    payload=bytearray(512);payload[254:259]=b'ustar'
    wire=Wire();wire.put(0,1);plain_header(wire,False,len(payload),True);wire.align()
    for byte in payload:wire.put(byte,8)
    wire.put(1,1);wire.put(1,1);wire.align()
    assert wire.data[257:262]==b'ustar'
    cases.append(('raw-tar-marker.br',bytes(wire.data),bytes(payload)))
    identities={};DEST.mkdir(parents=True,exist_ok=True)
    for name,data,payload in cases:
        if brotli.decompress(data)!=payload:raise ValueError('Independent reference mismatch: '+name)
        path=DEST/name
        if args.check:
            if path.read_bytes()!=data:raise ValueError('Stale owned input: '+name)
        else:path.write_bytes(data)
        identities[name]=dict(packed=len(data),decoded=len(payload),sha256=hashlib.sha256(data).hexdigest(),decoded_sha256=hashlib.sha256(payload).hexdigest(),decoded_crc32=zlib.crc32(payload))
    out=ROOT/'build/review/format-expansion/m7-brotli-reference.json';out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(identities,indent=2)+'\n',encoding='utf-8')
    rust=['// Generated independent fixture expectations; no product decoder was used.', '#[rustfmt::skip]', 'pub(super) const CASES: &[(&str, usize, u32)] = &[']
    rust.extend(f'    ("{name}", {row["decoded"]}, 0x{row["decoded_crc32"]:08x}),' for name,row in identities.items())
    rust.append('];')
    expected=('\n'.join(rust)+'\n').encode()
    target=ROOT/'crates/xdzip-compression/src/brotli/fixture_cases.rs'
    if args.check:
        if target.read_bytes()!=expected:raise ValueError('Stale fixture expectations')
    else:target.write_bytes(expected)
    print(f'{len(cases)} authored inputs verified by pinned Brotli 1.2.1')


if __name__=='__main__':
    main()
