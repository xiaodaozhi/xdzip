#!/usr/bin/env python3
"""Compile a per-user, non-default-taking installer from an audited Windows runtime."""
import argparse
import json
from pathlib import Path
import re
import subprocess
from deployment_audit import digest, within
from installer_payload import verify_payload

ROOT=Path(__file__).resolve().parents[1]
EXTENSIONS=['zip','tar','tgz','gz']


def render(payload,audit,output,chinese,qa=None):
    if audit['status']!='passed' or audit['platform']!='windows' or audit['mode']!='deployed' or audit['identity']['dirty']:
        raise ValueError('Installer requires clean deployed Windows audit')
    verify_payload(payload,audit)
    identity=audit['identity'];revision=identity['revision']
    if not re.fullmatch('[0-9a-f]{12}',revision):raise ValueError('Invalid revision')
    runtime=revision+'-'+digest(payload/'xdzip.exe')[:12]+('-qa'+qa if qa else '')
    associations=[]
    for ext in EXTENSIONS:
        associations += [f'Root: HKCU; Subkey: "Software\\Classes\\.{ext}\\OpenWithProgids"; ValueType: none; ValueName: "xdzip.Archive"; Flags: uninsdeletevalue',
            f'Root: HKCU; Subkey: "Software\\Classes\\Applications\\xdzip.exe\\SupportedTypes"; ValueType: string; ValueName: ".{ext}"',
            f'Root: HKCU; Subkey: "Software\\xdzip\\Capabilities\\FileAssociations"; ValueType: string; ValueName: ".{ext}"; ValueData: "xdzip.Archive"']
    values={'VERSION':identity['version'],'OUTPUT':str(output.parent.resolve()),'FILENAME':output.stem,
        'FILEVERSION':json.loads((ROOT/'VERSION.json').read_text(encoding='utf-8'))['version']+'.'+str(identity['build']),
        'APPICON':str((ROOT/'design/assets/brand/xdzip.ico').resolve()),'PAYLOAD':str(payload.resolve()),
        'CHINESE':str(chinese.resolve()),'RUNTIME':runtime,'ASSOCIATIONS':'\n'.join(associations)}
    text=(ROOT/'apps/desktop/packaging/windows/installer.iss.in').read_text(encoding='utf-8')
    for name,value in values.items():
        if name!='ASSOCIATIONS' and any(c in value for c in ['"','\r','\n']):raise ValueError('Unsafe installer substitution')
        text=text.replace('@'+name+'@',value)
    return text,{'runtime_id':runtime,'identity':identity,'executable_sha256':digest(payload/'xdzip.exe'),'qa_fixture':qa is not None}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,required=True);parser.add_argument('--audit',type=Path,required=True)
    parser.add_argument('--iscc',type=Path,required=True);parser.add_argument('--chinese',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True);parser.add_argument('--report',type=Path,required=True)
    parser.add_argument('--qa-variant',choices=['1','2']);args=parser.parse_args()
    if not within(args.output,ROOT/'build'):parser.error('Installer belongs under build/')
    text,result=render(args.root,json.loads(args.audit.read_text(encoding='utf-8')),args.output,args.chinese,args.qa_variant)
    args.output.parent.mkdir(parents=True,exist_ok=True);source=args.output.with_suffix('.iss');source.write_text(text,encoding='utf-8-sig')
    subprocess.run([str(args.iscc),str(source)],check=True)
    result.update(file=args.output.name,sha256=digest(args.output),compiler='Inno Setup 6.7.3',scope='current user',signed=False)
    args.report.write_text(json.dumps(result,indent=2)+'\n')
    print('Windows current-user installer compiled, defaults preserved')

if __name__=='__main__':main()
