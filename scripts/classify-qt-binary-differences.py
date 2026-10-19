#!/usr/bin/env python3
"""Test reversible byte transformations against upstream hashes; never guess equivalence."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
from deployment_audit import ROOT, digest, within


def pe_unsigned_candidates(data):
    if len(data)<128 or data[:2]!=b'MZ':return
    pe=struct.unpack_from('<I',data,0x3c)[0]
    if pe+24+160>len(data) or data[pe:pe+4]!=b'PE\0\0':return
    optional=pe+24;magic=struct.unpack_from('<H',data,optional)[0]
    if magic not in (0x10b,0x20b):return
    directory=optional+(96 if magic==0x10b else 112)+4*8
    offset,size=struct.unpack_from('<II',data,directory)
    if not offset or size<8 or offset+size!=len(data) or offset<directory+8:return
    unsigned=bytearray(data[:offset]);unsigned[directory:directory+8]=b'\0'*8;unsigned[optional+64:optional+68]=b'\0'*4
    for trim in range(8):
        if trim and unsigned[-trim:]!=b'\0'*trim:continue
        yield ('PE certificate removal, checksum reset, trailing alignment trim '+str(trim),bytes(unsigned[:-trim] if trim else unsigned))


def match(content,expected,algorithm):
    return hashlib.new(algorithm.lower().replace('-',''),content).hexdigest()==expected.lower()


def classify(file,checks,platform,temporary):
    expected=[c for c in checks if not c['matches_sdk'] and c['algorithm'].upper().replace('-','') in {'SHA1','SHA256'}]
    if not expected:return []
    data=file.read_bytes();result=[]
    candidates=list(pe_unsigned_candidates(data)) if platform=='windows' else []
    if platform=='macos':
        # Operate on SDK image copies only, never sign or change the SDK/app.
        copy=temporary/'unsigned-image';shutil.copy2(file,copy)
        if subprocess.run(['codesign','--remove-signature',str(copy)],capture_output=True).returncode==0:
            candidates.append(('Mach-O signature removal in isolated SDK-image copy',copy.read_bytes()))
        archs=subprocess.run(['lipo','-archs',str(file)],capture_output=True,text=True)
        if archs.returncode==0:
            for arch in archs.stdout.split():
                thin=temporary/('thin-'+arch)
                if subprocess.run(['lipo',str(file),'-thin',arch,'-output',str(thin)],capture_output=True).returncode==0:
                    candidates.append(('Mach-O exact '+arch+' slice',thin.read_bytes()))
                    if subprocess.run(['codesign','--remove-signature',str(thin)],capture_output=True).returncode==0:
                        candidates.append(('Mach-O exact '+arch+' slice with signature removed',thin.read_bytes()))
    for check in expected:
        found=next((name for name,value in candidates if match(value,check['expected'],check['algorithm'])),None)
        result.append({'algorithm':check['algorithm'],'expected':check['expected'],'verified_transformation':found,
                       'status':'exact-hash-transformation-match' if found else 'unexplained-publisher-provenance-required'})
    return result


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--attribution',type=Path,required=True);p.add_argument('--qt-prefix',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    if a.output.exists() or not within(a.output.resolve(),ROOT/'build'):p.error('Use a new build/ file')
    d=json.loads(a.attribution.read_text());rows=[]
    with tempfile.TemporaryDirectory(prefix='owned-sdk-diff-',dir=a.output.parent) as temp:
        for item in d['files']:
            checks=item.get('upstream_checksums',[])
            if not any(not c['matches_sdk'] for c in checks):continue
            file=a.qt_prefix/item['sdk_path']
            if not within(file,a.qt_prefix) or digest(file)!=item['sdk_sha256']:raise ValueError('SDK bytes changed before transformation analysis')
            rows.append({'sdk_path':item['sdk_path'],'sdk_sha256':item['sdk_sha256'],'results':classify(file,checks,d['platform'],Path(temp))})
    result={'schema_version':1,'status':'transformation-analysis-complete-not-provenance-approval','platform':d['platform'],'identity':d['identity'],'attribution_sha256':digest(a.attribution),'files':rows,
            'limits':['Original upstream hash differences are retained. Only an exact reconstructed byte hash is marked explained.',
                      'No fuzzy match, publisher signature/rights inference or SDK binary build equivalence; unmatched publisher history requires external evidence.']}
    a.output.write_text(json.dumps(result,indent=2)+'\n');print('Tested reversible SDK byte transformations; original evidence retained')

if __name__=='__main__':main()
