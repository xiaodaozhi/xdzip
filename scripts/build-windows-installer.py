#!/usr/bin/env python3
"""Compile a per-user, non-default-taking installer from an audited Windows runtime."""
import argparse
import hashlib
import os
import json
from pathlib import Path
import re
import subprocess
from deployment_audit import digest, within
from installer_payload import verify_payload

ROOT=Path(__file__).resolve().parents[1]
EXTENSIONS=['zip','tar','tgz','gz']


MANIFEST_NAME='install-manifest.txt'


def payload_manifest(audit):
    entries=[]
    seen=set()
    for item in sorted(audit['files'],key=lambda f:f['path'].casefold()):
        path=item['path'].replace('/', '\\')
        parts=path.split('\\')
        if (not parts or any(not p or p in ('.','..') or p[-1] in '. ' for p in parts)
                or any(p.split('.')[0].casefold() in {'con','prn','aux','nul','clock$',*(f'com{i}' for i in range(1,10)),*(f'lpt{i}' for i in range(1,10))} for p in parts)
                or any(ord(c)<32 or c in '/:*?"<>|' for c in path)
                or len(path)>1024 or path.casefold() in seen
                or parts[0].casefold() in ('versions', MANIFEST_NAME)
                or parts[0].casefold().startswith('unins')
                or not re.fullmatch('[0-9a-f]{64}',item['sha256'])):
            raise ValueError('Unsafe or reserved installer payload path')
        seen.add(path.casefold())
        entries.append(item['sha256']+'\t'+path)
    text='xdzip-install-files-v1\n'+'\n'.join(entries)+'\n'
    if len(entries)>16384 or len(text.encode('utf-8'))>2097152:
        raise ValueError('Installer manifest exceeds bounded format')
    return text


def render(payload,audit,output,chinese,qa=None,legacy=False):
    if audit['status']!='passed' or audit['platform']!='windows' or audit['mode']!='deployed' or audit['identity']['dirty']:
        raise ValueError('Installer requires clean deployed Windows audit')
    verify_payload(payload,audit)
    identity=audit['identity'];revision=identity['revision']
    if not re.fullmatch('[0-9a-f]{12}',revision):raise ValueError('Invalid revision')
    runtime=revision+'-'+digest(payload/'xdzip.exe')[:12]+('-qa'+qa if qa else '')
    manifest=payload_manifest(audit)
    runtime_dir=f'{{app}}\\versions\\{runtime}' if legacy else '{app}'
    manifest_path=output.with_suffix('.files').resolve()
    associations=[]
    for ext in EXTENSIONS:
        associations += [f'Root: HKCU; Subkey: "Software\\Classes\\.{ext}\\OpenWithProgids"; ValueType: string; ValueName: "xdzip.Archive"; Flags: uninsdeletevalue',
            f'Root: HKCU; Subkey: "Software\\Classes\\Applications\\xdzip.exe\\SupportedTypes"; ValueType: string; ValueName: ".{ext}"',
            f'Root: HKCU; Subkey: "Software\\xdzip\\Capabilities\\FileAssociations"; ValueType: string; ValueName: ".{ext}"; ValueData: "xdzip.Archive"']
    values={'VERSION':identity['version'],'OUTPUT':str(output.parent.resolve()),'FILENAME':output.stem,
        'FILEVERSION':json.loads((ROOT/'VERSION.json').read_text(encoding='utf-8'))['version']+'.'+str(identity['build']),
        'APPICON':str((ROOT/'design/assets/brand/xdzip.ico').resolve()),'PAYLOAD':str(payload.resolve()),
        'CHINESE':str(chinese.resolve()),'RUNTIME_DIR':runtime_dir,'ASSOCIATIONS':'\n'.join(associations),
        'MANIFEST_FILES':'' if legacy else (f'Source: "{manifest_path}"; DestName: "xdzip-package-files.txt"; Flags: dontcopy\n'
            f'Source: "{manifest_path}"; DestDir: "{{app}}"; DestName: "{MANIFEST_NAME}"; Flags: ignoreversion'),
        'CLEANUP_CODE':'' if legacy else (ROOT/'apps/desktop/packaging/windows/installed-files.iss').read_text(encoding='utf-8')}
    text=(ROOT/'apps/desktop/packaging/windows/installer.iss.in').read_text(encoding='utf-8')
    for name,value in values.items():
        if name not in ('ASSOCIATIONS','MANIFEST_FILES','CLEANUP_CODE') and any(c in value for c in ['"','\r','\n']):raise ValueError('Unsafe installer substitution')
        text=text.replace('@'+name+'@',value)
    return text,{'runtime_id':runtime,'identity':identity,'executable_sha256':digest(payload/'xdzip.exe'),'qa_fixture':qa is not None or legacy,'layout':'legacy-qa' if legacy else 'flat',
        'executable_relative_path':('versions/'+runtime+'/' if legacy else '')+'xdzip.exe',
        'payload_manifest_sha256':hashlib.sha256(manifest.encode('utf-8')).hexdigest()}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,required=True);parser.add_argument('--audit',type=Path,required=True)
    parser.add_argument('--iscc',type=Path,required=True);parser.add_argument('--chinese',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True);parser.add_argument('--report',type=Path,required=True)
    parser.add_argument('--qa-variant',choices=['1','2'])
    parser.add_argument('--qa-legacy-layout',action='store_true',help='CI-only historical layout fixture, never a product package')
    args=parser.parse_args()
    if args.qa_legacy_layout and os.environ.get('CI')!='true':parser.error('Legacy layout is restricted to disposable CI fixtures')
    if not within(args.output,ROOT/'build'):parser.error('Installer belongs under build/')
    audit=json.loads(args.audit.read_text(encoding='utf-8'))
    toolchain=json.loads((ROOT/'scripts/installer-toolchain.json').read_text())['inno_setup']
    notice_records=[]
    for item in toolchain['materials']:
        if item['file'] == 'ChineseSimplified.isl':continue
        path=args.root/'share/notices'/item['file']
        if not path.is_file() or digest(path)!=item['sha256']:
            raise ValueError('Installer payload lacks exact upstream engine notices')
        notice_records.append({'file':'share/notices/'+item['file'],'sha256':item['sha256'],'source':item['url']})
    credits=args.root/'share/notices/Installer-CREDITS.txt'
    if not credits.is_file() or 'RemObjects Pascal Script (https://www.remobjects.com/ps)' not in credits.read_text(encoding='utf-8'):
        raise ValueError('Installer payload lacks Pascal Script documentation acknowledgement')
    text,result=render(args.root,audit,args.output,args.chinese,args.qa_variant,args.qa_legacy_layout)
    args.output.parent.mkdir(parents=True,exist_ok=True);source=args.output.with_suffix('.iss')
    args.output.with_suffix('.files').write_bytes(payload_manifest(audit).encode('utf-8'))
    source.write_text(text,encoding='utf-8-sig')
    subprocess.run([str(args.iscc),str(source)],check=True)
    result.update(file=args.output.name,sha256=digest(args.output),compiler='Inno Setup 6.7.3',scope='current user',signed=False)
    result.update(compiler_source_commit=toolchain['source_commit'],compiler_package_sha256=toolchain['sha256'],
                  engine_notice_files=notice_records,pascal_script_acknowledgement={'file':'share/notices/Installer-CREDITS.txt','sha256':digest(credits)})
    args.report.write_text(json.dumps(result,indent=2)+'\n')
    print('Windows current-user installer compiled, defaults preserved')

if __name__=='__main__':main()
