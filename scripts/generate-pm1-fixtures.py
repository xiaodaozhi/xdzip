#!/usr/bin/env python3
"""Author PM1 wire inputs and verify complete owned content with two reference tools."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile

ROOT=Path(__file__).resolve().parents[1]
INPUT_SHA='f903b0359a950df4b6df4f85800cc3670681c6187ae333390ad3ef903b7165f2'
# Expanded wire alphabets, expressed independently of the reference's packed trees.
ALPHABETS=[
    '((((ab)c)d)(ef))','(((ab)(cf))(de))','(((ab)c)(d(ef)))','((a(bc))(d(ef)))',
    '((a(bd))(c(ef)))','((a(b(ef)))(cd))','((ab)((cd)(ef)))','((ab)((c(ef))d))',
    '((ab)(c(d(ef))))','(a(((bf)c)(de)))','(a(((b(ef))c)d))','(a(((bc)d)(ef)))',
    '(a((b(cf))(de)))','(a((bc)(d(ef))))','(a((b(d(ef)))c))','(a(b((cd)(ef))))',
    '(a(b(c(d(ef)))))','(((de)c)(de))','((a(be))(cd))','((ab)(c(de)))',
    '(a(((be)c)d))','(a((bc)(de)))','(a((b(de))c))','(a(b(c(de))))',
    '(((ab)c)d)','((a(bd))c)','((ab)(cd))','(a((bd)c))',
    '(a(b(cd)))','(a(bc))','(ab)','a',
]
RANKS=[(0,4),(16,4),(32,5),(64,6),(128,6),(192,6)]


def load(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'scripts'/f'{name}.py')
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module


def prefixes(shape):
    at=0;result=[]
    def node(prefix):
        nonlocal at
        if shape[at]!='(':
            result.append((shape[at],prefix));at+=1
        else:
            at+=1;node(prefix+'0');node(prefix+'1')
            assert shape[at]==')';at+=1
    node('');assert at==len(shape)
    return result


class Wire:
    def __init__(self,selector):
        self.bits=f'{selector:05b}';self.codes=prefixes(ALPHABETS[selector])
        self.recent=list(range(32,128))+list(range(32))+list(range(160,224))+list(range(128,160))+list(range(224,256))
        self.output=bytearray()
    def field(self,value,width):
        assert 0<=value<(1<<width)
        self.bits+=f'{value:0{width}b}'
    def observe(self,byte):
        self.recent.remove(byte);self.recent.insert(0,byte);self.output.append(byte)
    def literal(self,rank):
        options=[]
        for category,prefix in self.codes:
            base,width=RANKS[ord(category)-97]
            if base<=rank<base+(1<<width):
                options.append((prefix,base,width))
        if not options:raise ValueError('Rank is outside the selected alphabet')
        prefix,base,width=options[len(self.output)%len(options)]
        self.bits+=prefix;self.field(rank-base,width);self.observe(self.recent[rank])
    def literals(self,ranks):
        count=len(ranks);assert 1<=count<=216
        self.bits+='1'
        if count<=3:self.field(count-1,2)
        else:
            self.bits+='11'
            if count<=10:self.field(count-4,3)
            else:
                self.bits+='111'
                if count<=24:self.field(count-11,4)
                elif count<=88:self.bits+='1110';self.field(count-25,6)
                else:self.bits+='1111';self.field(count-89,7)
        for rank in ranks:self.literal(rank)
    def copy(self,length,offset,*,implicit=False,kind=None):
        position=len(self.output)
        if not implicit:self.bits+='0'
        if kind is None:kind=(0 if offset<64 else 1) if length==2 else (2 if offset<64 else 3 if offset<576 else 4 if offset<2624 else 5)
        assert (kind<2)==(length==2)
        if kind in (0,1,4):
            self.bits+='0'
            if position>=576:self.bits+='1' if kind==4 else '0'
            else:assert kind!=4
            if kind!=4:
                if position>=64:self.bits+='1' if kind==1 else '0'
                else:assert kind==0
        else:
            self.bits+='1'
            if position>=64:self.bits+='0' if kind==3 else '1'
            else:assert kind==2
            if kind!=3:
                if position>=2624:self.bits+='0' if kind==5 else '1'
                else:assert kind==2
        if length!=2:
            assert 3<=length<=244
            if length<=5:self.field(length-3,2)
            else:
                self.bits+='11'
                if length<=10:self.field(length-6,3)
                elif length<=14:self.bits+='101';self.field(length-11,2)
                elif length<=22:self.bits+='110';self.field(length-15,3)
                else:
                    self.bits+='111'
                    if length<=84:self.field(length-23,6)
                    elif length<=116:self.bits+='111110';self.field(length-85,5)
                    else:self.bits+='111111';self.field(length-117,7)
        if kind in (0,2):base,width=0,6
        elif kind==1:base,width=64,8
        elif kind==3:base,width=64,8 if position<320 else 9
        elif kind==4:base,width=576,8 if position<832 else 9 if position<1088 else 10 if position<1600 else 11
        else:base,width=2624,8 if position<2880 else 9 if position<3136 else 10 if position<3648 else 11 if position<4672 else 12 if position<6720 else 13
        assert 0<=offset<min(position,16384)
        self.field(offset-base,width)
        for _ in range(length):self.observe(self.output[-offset-1])
    def fill(self,size):
        assert size>=3 and len(self.output)==0
        blocks,tail=divmod(size,216)
        if tail in (1,2):blocks-=1;tail+=216
        for _ in range(blocks):self.literals([(i*37+19)%256 for i in range(216)])
        if tail:
            if tail==218:self.literals([i%256 for i in range(216)]);self.copy(2,0)
            else:self.literals([i%256 for i in range(tail-2)]);self.copy(2,0,implicit=True)
        assert len(self.output)==size
    def packed(self):
        bits=self.bits+'0'*(-len(self.bits)%8)
        return bytes(int(bits[i:i+8],2) for i in range(0,len(bits),8))


def cases():
    result=[]
    for selector,shape in enumerate(ALPHABETS):
        wire=Wire(selector)
        categories=list(dict.fromkeys(c for c,_ in prefixes(shape)))
        ranks=[]
        for i in range(216):
            base,width=RANKS[ord(categories[i%len(categories)])-97]
            ranks.append(base+((i*7)% (1<<width)))
        wire.literals(ranks);result.append((f'pm1-selector-{selector:02}.pma',wire))
    wire=Wire(0)
    for count in (1,2,3,4,10,11,24,25,88,89,215,216):
        wire.literals([(i*53+count)%256 for i in range(count)])
        if count<216:wire.copy(2,0,implicit=True)
    result.append(('pm1-literal-lengths.pma',wire))
    wire=Wire(0);wire.fill(216)
    for count in (2,3,5,6,10,11,14,15,22,23,84,85,116,117,244):wire.copy(count,63)
    result.append(('pm1-copy-lengths.pma',wire))
    for position in (63,64,65,319,320,575,576,577,831,832,1087,1088,1599,1600,2623,2624,2625,2879,2880,3135,3136,3647,3648,4671,4672,6719,6720,10816,20000):
        wire=Wire(0);wire.fill(position)
        kind=0 if position<=64 else 3 if position<=576 else 4 if position<=2624 else 5
        width=6 if kind==0 else 8 if kind==3 and position<320 else 9 if kind==3 else 8 if kind==4 and position<832 else 9 if kind==4 and position<1088 else 10 if kind==4 and position<1600 else 11 if kind==4 else 8 if position<2880 else 9 if position<3136 else 10 if position<3648 else 11 if position<4672 else 12 if position<6720 else 13
        base=0 if kind==0 else 64 if kind==3 else 576 if kind==4 else 2624
        offset=min(position-1,base+(1<<width)-1)
        wire.copy(2 if kind==0 else 244,offset,kind=kind)
        result.append((f'pm1-distance-{position}.pma',wire))
    wire=Wire(0);wire.fill(320);wire.copy(2,319,kind=1);result.append(('pm1-copy-two-far.pma',wire))
    wire=Wire(0);wire.fill(127);wire.literals([15]);result.append(('pm1-final-literal.pma',wire))
    wire=Wire(31);wire.literals([0]);wire.copy(244,0,implicit=True)
    for _ in range(100):wire.copy(244,0)
    result.append(('pm1-long-overlap.pma',wire))
    for _,wire in result:
        tail=-len(wire.output)%128
        if tail:
            if tail==1:tail+=128
            wire.copy(tail,0)
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    tools=load('generate-pma-fixtures');fields=load('generate-lha-fixtures')
    emulator,_,pmext,lhasa=tools.setup()
    base=ROOT/'build/review/format-expansion';work=Path(tempfile.mkdtemp(prefix='pm1-fixtures-',dir=base)).resolve()
    dest=ROOT/'tests/fixtures/pma';results={};expected=bytearray();identity=hashlib.sha256()
    for name,wire in cases():
        packed=wire.packed();plain=bytes(wire.output)
        header=bytearray((dest/'pm0-allbytes.pma').read_bytes()[:33])
        header[2:7]=b'-pm1-';header[7:11]=len(packed).to_bytes(4,'little');header[11:15]=len(plain).to_bytes(4,'little');header[31:33]=fields.crc16(plain).to_bytes(2,'little');header[1]=sum(header[2:])%256
        data=bytes(header)+packed+b'\0';data+=b'\x1a'*(-len(data)%128)
        source=work/name;source.write_bytes(data)
        target=work/(name+'-lhasa');target.mkdir();tools.command([lhasa,'xq',source],cwd=target)
        files=[p for p in target.iterdir() if p.is_file()]
        assert len(files)==1 and files[0].read_bytes()==plain, ('Lhasa output mismatch',name)
        target=work/(name+'-pmext');tools.cpm(emulator,pmext,target,['SOURCE.PMA','HELLO.BIN/B'],{'SOURCE.PMA':data})
        assert (target/'HELLO.BIN').read_bytes()==plain, ('PMext output mismatch',name)
        if args.check:assert (dest/name).read_bytes()==data, ('Stale PM1 archive',name)
        else:(dest/name).write_bytes(data)
        results[name]=dict(packed=len(packed),decoded=len(plain),crc16=fields.crc16(plain),plain_offset=len(expected),archive_sha256=hashlib.sha256(data).hexdigest(),decoded_sha256=hashlib.sha256(plain).hexdigest(),lhasa_exact=True,pmext_exact=True)
        expected.extend(plain);identity.update(name.encode()+b'\0'+data)
    identity.update(b'pm1-known.bin\0'+expected)
    if INPUT_SHA:assert identity.hexdigest()==INPUT_SHA, 'PM1 set identity differs'
    if args.check:assert (dest/'pm1-known.bin').read_bytes()==expected
    else:(dest/'pm1-known.bin').write_bytes(expected)
    rust=['// Generated owned PM1 expectations, independently read by PMext and Lhasa.','#[rustfmt::skip]','pub(crate) const CASES: &[(&str, usize, usize, u16, usize)] = &[']
    rust.extend(f'    ("{name}", {v["packed"]}, {v["decoded"]}, 0x{v["crc16"]:04x}, {v["plain_offset"]}),' for name,v in results.items());rust.append('];')
    output=ROOT/'crates/xdzip-compression/src/pm1/fixture_cases.rs';data=('\n'.join(rust)+'\n').encode()
    if args.check:assert output.read_bytes()==data
    else:output.parent.mkdir(parents=True,exist_ok=True);output.write_bytes(data)
    report=dict(source_blob='bb2937e382c8c8a8f5783b94a96b9b59e2c4e74b',origin='wire-authored',input_set_sha256=identity.hexdigest(),known_bytes=len(expected),cases=results)
    (base/'m8-pm1-reference.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(f'{len(results)} owned PM1 archives exactly verified by PMext and Lhasa; {len(expected)} known bytes; set SHA-256 {identity.hexdigest()}')


if __name__=='__main__':main()
