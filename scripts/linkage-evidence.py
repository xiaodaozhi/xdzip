#!/usr/bin/env python3
"""Capture actual product link inputs and symbols without inventing license ownership."""
import argparse
import json
import sys
import re
import os
import shutil
from pathlib import Path
import subprocess
from deployment_audit import ROOT, digest, within


def mac_objects(text):
    objects, retained = {}, {}
    section = ''
    for line in text.splitlines():
        if line.startswith('# Object files:'):section='objects';continue
        if line.startswith('# Symbols:'):section='symbols';continue
        if line.startswith('# Address'):continue
        if line.startswith('#'):section='';continue
        if section=='objects':
            m=re.match(r'\[\s*(\d+)\]\s+(.+)',line)
            if m:
                # Keep archive/object basenames, never developer profile paths.
                value=m[2].replace('\\','/');objects[m[1]]=value.rsplit('/',1)[-1]
        elif section=='symbols':
            m=re.match(r'0x[0-9A-Fa-f]+\s+0x([0-9A-Fa-f]+)\s+\[\s*(\d+)\]',line)
            if m and int(m[1],16)>0:
                retained[m[2]]=retained.get(m[2],0)+int(m[1],16)
    return [{'object':objects[k], 'retained_symbol_bytes':retained[k]} for k in sorted(retained) if k in objects]


def other_objects(text):
    # GNU map discarded sections precede this marker; never count them as live.
    if 'Linker script and memory map' in text:
        live=text.split('Linker script and memory map',1)[1]
        result={}
        for line in live.splitlines():
            fields=line.split()
            for i in range(len(fields)-2):
                if re.fullmatch(r'0x[0-9a-fA-F]+',fields[i]) and re.fullmatch(r'0x[0-9a-fA-F]+',fields[i+1]):
                    name=' '.join(fields[i+2:])
                    if name.endswith(('.o',')')) and int(fields[i+1],16)>0:
                        name=name.replace('\\','/').rsplit('/',1)[-1]
                        result[name]=result.get(name,0)+int(fields[i+1],16)
                    break
        return [{'object':k,'retained_section_bytes':v} for k,v in sorted(result.items())]
    # MSVC publics/static symbols name actual contributing Lib:Object entries.
    result={}
    for line in text.splitlines():
        if re.match(r'\s*[0-9a-fA-F]{4}:[0-9a-fA-F]+\s+',line):
            fields=line.split()
            if len(fields)>=4 and re.fullmatch(r'[0-9a-fA-F]{8,16}',fields[2]):
                name=fields[-1].replace('\\','/').rsplit('/',1)[-1]
                if name.endswith(('.o','.obj','.dll')):
                    result[name]=result.get(name,0)+1
    return [{'object':k,'retained_public_symbols':v} for k,v in sorted(result.items())]


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['app','map','rust-library','output']:p.add_argument('--'+name,type=Path,required=True)
    a=p.parse_args()
    if not within(a.output.resolve(),ROOT/'build') or a.output.exists():p.error('Use a new build/ file')
    identity=subprocess.check_output([str(a.app.resolve()),'--version'],text=True).strip()
    if 'dirty' in identity or 'unknown' in identity:raise ValueError('Clean product identity required')
    text=a.map.read_text(errors='replace');objects=mac_objects(text) if '# Object files:' in text else other_objects(text)
    if not objects:raise ValueError('No actual retained linker inputs parsed')
    if os.name=='nt' and not shutil.which('llvm-ar'):
        output=subprocess.check_output(['lib.exe','/nologo','/list',str(a.rust_library.resolve())],text=True)
        entries=[line.strip() for line in output.splitlines() if line.strip().endswith(('.o','.obj'))]
    else:
        tool='llvm-ar' if os.name=='nt' else 'ar'
        entries=subprocess.check_output([tool,'t',str(a.rust_library.resolve())],text=True).splitlines()
    entries=[line.replace('\\','/').rsplit('/',1)[-1] for line in entries]
    if not entries:raise ValueError('Static library member inventory empty')
    cargo=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1'],cwd=ROOT,text=True))
    packages={p['name'].replace('-','_'):p for p in cargo['packages']}
    linked=[]
    for item in objects:
        name=item['object'];member=name[name.find('(')+1:-1] if '(' in name and name.endswith(')') else name.rsplit(':',1)[-1]
        # Match real retained object basenames against actual staticlib entries.
        # Monomorphized and compiler-generated code can cross crate boundaries;
        # these candidates must not be called complete per-byte ownership.
        if member not in entries:continue
        candidate=next((packages[k] for k in sorted(packages,key=len,reverse=True) if member.startswith(k+'-')),None)
        linked.append({**item,'archive_member':member,'cargo_package':candidate['name'] if candidate else None,
                       'cargo_version':candidate['version'] if candidate else None})
    result={'schema_version':1,'platform':'macos' if sys.platform=='darwin' else 'windows' if os.name=='nt' else 'linux','status':'link-input-evidence-not-complete-attribution','identity':identity,
            'app_sha256':digest(a.app),'link_map_sha256':digest(a.map),'rust_staticlib_sha256':digest(a.rust_library),
            'staticlib_members':entries,'retained_objects':objects,'retained_rust_candidates':linked,
            'platform_map_interpreted':bool(objects),
            'limits':['Actual archive member list and linker map bound to product hashes, not exact retained Rust/native source/license ownership.',
                      'Mach-O live symbols, GNU live sections and MSVC public/static object references are parsed; contributions are not exact source-code ownership.',
                      'Cross-crate monomorphization, inlining, bundled C code and compiler runtimes still require review; no license choice is made.']}
    a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(result,indent=2)+'\n');print('Captured actual linker/staticlib evidence and product identity')

if __name__=='__main__':main()
