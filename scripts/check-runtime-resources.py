#!/usr/bin/env python3
"""Bind actual compiled Qt resources to source paths and existing notices."""
import argparse
import json
import os
import sys
from pathlib import Path
import xml.etree.ElementTree as ET
from deployment_audit import ROOT, digest, within


def check(report):
    events=[e for r in report['runs'] for e in r['events'] if e['event']=='resource_manifest']
    if not events:raise ValueError('Actual runtime resource manifest missing')
    actual={r['resource']:r['sha256'] for r in events[0]['files']}
    source=ROOT/'apps/desktop/ui.qrc';expected={};files=[]
    for group in ET.parse(source).getroot().findall('qresource'):
        prefix=group.get('prefix','')
        for file in group.findall('file'):
            path=(source.parent/file.text).resolve()
            if not within(path,ROOT):raise ValueError('Resource source escapes repository')
            alias=file.get('alias',file.text);resource=':'+prefix+'/'+alias
            expected[resource]=digest(path)
            files.append({'resource':resource,'source':path.relative_to(ROOT).as_posix(),'sha256':expected[resource],
                          'license_reference':'design/assets/fonts/OFL.txt' if '/fonts/' in path.as_posix() else 'LICENSE',
                          'scope':'Source/compiled-byte correspondence and existing project/font notice, not third-party or generative-rights approval'})
    if actual!=expected:raise ValueError('Runtime resource bytes or set differ from source')
    return files


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--probe',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    if a.output.exists() or not within(a.output.resolve(),ROOT/'build'):p.error('Use a new build/ file')
    report=json.loads(a.probe.read_text());files=check(report)
    result={'schema_version':1,'platform':'macos' if sys.platform=='darwin' else 'windows' if os.name=='nt' else 'linux','status':'source-runtime-resource-bytes-matched','identity':report['identity'],'app_sha256':report['app_sha256'],
            'probe_sha256':digest(a.probe),'files':files,'limits':['All project qrc resources enumerated; Qt internal resources and final legal attribution remain separate.']}
    a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(result,indent=2)+'\n');print('Actual compiled resources match '+str(len(files))+' declared source inputs')

if __name__=='__main__':main()
