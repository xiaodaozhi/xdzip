#!/usr/bin/env python3
"""Build an experimental Ubuntu 24.04 amd64 package from the audited runtime."""
import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess
from deployment_audit import digest, within
from installer_payload import verify_payload

ROOT=Path(__file__).resolve().parents[1]


def package_tree(payload,audit,stage,qa=None):
    if audit['status']!='passed' or audit['platform']!='linux' or audit['mode']!='deployed':
        raise ValueError('Installer requires independently deployed Linux audit')
    if audit['identity']['dirty']:raise ValueError('Installer requires a clean build identity')
    identity=audit['identity'];revision=identity['revision']
    if not re.fullmatch('[0-9a-f]{12}',revision):raise ValueError('Invalid build revision')
    verify_payload(payload,audit)
    app_hash=digest(payload/'bin/xdzip');runtime_id=revision+'-'+app_hash[:12]+('-qa'+qa if qa else '')
    version=identity['version'].replace('-', '~',1)+'+git'+revision+('+qa'+qa if qa else '')
    if not re.fullmatch(r'[0-9][a-zA-Z0-9.+~]*',version):raise ValueError('Invalid Debian version')
    if stage.exists():raise ValueError('Refusing existing package stage')
    runtime=stage/'opt/xdzip/versions'/runtime_id
    shutil.copytree(payload,runtime)
    bindir=stage/'usr/bin';bindir.mkdir(parents=True)
    launcher=bindir/'xdzip'
    launcher.write_text('#!/bin/sh\nexec "/opt/xdzip/versions/'+runtime_id+'/xdzip" "$@"\n');launcher.chmod(0o755)
    share=stage/'usr/share';(share/'applications').mkdir(parents=True)
    desktop=(ROOT/'apps/desktop/packaging/linux/xdzip.desktop').read_text(encoding='utf-8').replace('Exec=xdzip-browser','Exec=/usr/bin/xdzip')
    (share/'applications/xdzip.desktop').write_text(desktop,encoding='utf-8')
    icons=share/'icons/hicolor/512x512/apps';icons.mkdir(parents=True)
    shutil.copy2(ROOT/'design/assets/brand/xdzip.png',icons/'xdzip.png')
    tmpfiles=stage/'usr/lib/tmpfiles.d';tmpfiles.mkdir(parents=True)
    (tmpfiles/'xdzip.conf').write_text('d /run/xdzip 0755 root root -\nf /run/xdzip/usage.lock 0644 root root -\n')
    control=stage/'DEBIAN';control.mkdir()
    manifest=json.loads((payload/'share/deployment.json').read_text(encoding='utf-8'))
    packages={p.removesuffix(':amd64') for p in manifest['system_packages']}
    if any(not re.fullmatch(r'[a-z0-9][a-z0-9+.-]*',p) for p in packages):raise ValueError('Invalid runtime dependency')
    packages.update(['desktop-file-utils','shared-mime-info','util-linux'])
    depends=', '.join('libc6 (>= 2.39)' if p=='libc6' else p for p in sorted(packages))
    (control/'control').write_text(f'Package: xdzip\nVersion: {version}\nArchitecture: amd64\nMaintainer: xdzip contributors <noreply@xdzip.invalid>\nSection: utils\nPriority: optional\nX-Xdzip-Settings-Schema: 1\nDepends: {depends}\nDescription: Experimental ZIP and TAR archive manager\n Ubuntu 24.04 x86_64 X11 developer installer, unsigned; release review pending.\n')
    hook=(ROOT/'apps/desktop/packaging/linux/package-hook.sh').read_text(encoding='utf-8')
    for name in ['preinst','prerm','postinst','postrm']:
        path=control/name
        path.write_text(hook.replace('hook=$1\nshift',f'hook={name}'),encoding='utf-8')
        path.chmod(0o755)
    doc=share/'doc/xdzip';doc.mkdir(parents=True)
    shutil.copy2(ROOT/'LICENSE',doc/'copyright')
    return {'version':version,'runtime_id':runtime_id,'executable_sha256':app_hash,'qa_fixture':qa is not None}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,required=True);parser.add_argument('--audit',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True);parser.add_argument('--report',type=Path,required=True)
    parser.add_argument('--qa-variant',choices=['1','2'])
    args=parser.parse_args();stage=ROOT/'build/installer-staging'/args.output.stem
    if not within(args.output,ROOT/'build'):parser.error('Generated installer belongs under build/')
    result=package_tree(args.root,json.loads(args.audit.read_text(encoding='utf-8')),stage,args.qa_variant)
    subprocess.run(['dpkg-deb','--root-owner-group','--build',str(stage),str(args.output)],check=True)
    result.update(file=args.output.name,sha256=digest(args.output),baseline='Ubuntu 24.04 amd64 X11',signed=False)
    args.report.write_text(json.dumps(result,indent=2)+'\n')
    print('Debian installer created with explicit runtime dependencies and alternate MIME handler')

if __name__=='__main__':main()
