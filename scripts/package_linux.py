#!/usr/bin/env python3
"""Archive our validated Linux payload preserving execute bits across CI downloads."""
import argparse
import gzip
import json
from pathlib import Path
import tarfile
from deployment_audit import digest, within


def archive_payload(root,output):
    root=root.resolve()
    if within(output,root):raise ValueError('Archive must be outside its payload')
    entries=sorted(root.rglob('*'))
    if any(p.is_symlink() and not within(p,root) for p in entries):raise ValueError('Archive symlink escapes payload')
    if any(not (p.is_file() or p.is_dir() or p.is_symlink()) for p in entries):raise ValueError('Special output object rejected')
    output.parent.mkdir(parents=True,exist_ok=True)
    with output.open('wb') as raw, gzip.GzipFile(filename='',fileobj=raw,mode='wb',mtime=0) as compressed:
        with tarfile.open(fileobj=compressed,mode='w|',format=tarfile.PAX_FORMAT) as archive:
            for path in [root]+entries:
                name='xdzip-linux-x86_64'+('/'+path.relative_to(root).as_posix() if path!=root else '')
                info=archive.gettarinfo(str(path),arcname=name)
                info.uid=info.gid=info.mtime=0;info.uname=info.gname='';info.pax_headers={}
                info.mode=0o755 if info.isdir() or path.stat().st_mode&0o111 else 0o644
                if info.isfile():
                    with path.open('rb') as file:archive.addfile(info,file)
                else:archive.addfile(info)
    return {'schema_version':1,'file':output.name,'bytes':output.stat().st_size,'sha256':digest(output),
            'executable_sha256':digest(root/'bin/xdzip'),'format':'tar.gz','permissions':'execute bits preserved',
            'limits':['Experimental Ubuntu 24.04 x86_64 X11 runtime directory; not a signed installer or universal Linux package.']}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,required=True);parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--report',type=Path,required=True);args=parser.parse_args()
    result=archive_payload(args.root,args.output)
    args.report.write_text(json.dumps(result,indent=2)+'\n')
    print('Linux tar.gz archive created with preserved executable permissions and SHA-256')

if __name__=='__main__':main()
