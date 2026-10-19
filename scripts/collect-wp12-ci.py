#!/usr/bin/env python3
"""Collect exact-commit GitHub CI JSON and selected artifacts without uploading data.

Authentication is sent only to the fixed repository API, never storage redirects.
Large development artifacts are read by bounded ranges. No remote binaries run.
"""
import argparse
import io
import json
from pathlib import Path
import subprocess
import urllib.error
import urllib.request
import zipfile
from deployment_audit import ROOT, digest, within

REPO='xiaodaozhi/xdzip'
WANTED={'build-version.json','deployment-audit.json','runtime-attribution.json','source-build.cdx.json','artifact-files.cdx.json',
        'qt-sdk-correspondence.json','qt-binary-differences.json','linux-deployment.json','runtime-resources.json','linkage-evidence.json','desktop-performance.json','desktop-input-accessibility.json',
        'manual-desktop.json','native-redistribution.json','macos-association.json','linux-runtime.json','linux-install-lifecycle.json','windows-install-lifecycle.json',
        'linux-installer.json','windows-installer.json','settings-compat.json','previous-product.json','build.json',
        'qt-source-build.json','qt-source-product.json'}


def api(path):
    return json.loads(subprocess.check_output(['gh','api','repos/'+REPO+'/'+path],text=True))


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self,*args,**kwargs):return None


def archive_url(artifact):
    token=subprocess.check_output(['gh','auth','token'],text=True).strip()
    request=urllib.request.Request('https://api.github.com/repos/'+REPO+'/actions/artifacts/'+str(artifact)+'/zip',headers={'Authorization':'Bearer '+token,'Accept':'application/vnd.github+json'})
    try:
        urllib.request.build_opener(NoRedirect).open(request,timeout=30)
        raise ValueError('Expected artifact storage redirect')
    except urllib.error.HTTPError as error:
        if error.code!=302:raise ValueError('Artifact API download failed') from None
        location=error.headers['Location']
        if not location.startswith('https://'):raise ValueError('Non-HTTPS artifact storage rejected')
        return location


class RemoteZip(io.RawIOBase):
    def __init__(self,url):
        self.url=url;self.position=0
        with urllib.request.urlopen(urllib.request.Request(url,method='HEAD'),timeout=30) as response:self.size=int(response.headers['Content-Length'])
    def seekable(self):return True
    def readable(self):return True
    def tell(self):return self.position
    def seek(self,offset,whence=0):
        self.position=offset if whence==0 else self.position+offset if whence==1 else self.size+offset
        return self.position
    def read(self,count=-1):
        count=min(count if count>=0 else self.size-self.position,self.size-self.position)
        if count<=0:return b''
        start=self.position
        with urllib.request.urlopen(urllib.request.Request(self.url,headers={'Range':f'bytes={start}-{start+count-1}'}),timeout=60) as response:
            if response.status!=206:raise ValueError('Artifact server ignored bounded range')
            data=response.read(count)
        self.position+=len(data);return data


def download_records(artifact,output,packages=False):
    output.mkdir(parents=True,exist_ok=True);found={}
    with zipfile.ZipFile(RemoteZip(archive_url(artifact))) as archive:
        for item in archive.infolist():
            name=Path(item.filename).name
            package=packages and name in {'xdzip-windows-x64-setup.exe','xdzip-ubuntu24.04-amd64.deb'}
            if name not in WANTED and not package:continue
            if item.file_size>(128*1024*1024 if package else 32*1024*1024):raise ValueError('Evidence entry exceeds declared bound')
            data=archive.read(item)
            if not package:json.loads(data)
            target=output/name
            if target.exists() and target.read_bytes()!=data:raise ValueError('Ambiguous evidence basename')
            target.write_bytes(data);found[name]=digest(target)
    return found


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--run',type=int,required=True);p.add_argument('--source-run',type=int)
    p.add_argument('--output',type=Path,required=True);p.add_argument('--packages',action='store_true');a=p.parse_args()
    out=a.output.resolve()
    if out.exists() or not within(out,ROOT/'build'):p.error('Use a new build/ directory')
    current=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    run=api('actions/runs/'+str(a.run));jobs=api('actions/runs/'+str(a.run)+'/jobs?per_page=100')['jobs']
    if run.get('name')!='Qt components':raise ValueError('Unexpected product workflow')
    if run['head_sha']!=current or run['conclusion']!='success' or len(jobs)!=3 or any(j['conclusion']!='success' for j in jobs):raise ValueError('Exact current three-platform product run has not passed')
    out.mkdir(parents=True);receipts=[]
    for category,run_id in [('product',a.run)]+([('qt-source',a.source_run)] if a.source_run else []):
        info=api('actions/runs/'+str(run_id))
        if info['head_sha']!=current or info['conclusion']!='success':raise ValueError('Mixed or incomplete CI run')
        for artifact in api('actions/runs/'+str(run_id)+'/artifacts')['artifacts']:
            name=artifact['name'];platform='macos' if 'macOS' in name else 'windows' if 'Windows' in name else 'linux' if 'Linux' in name else None
            if not platform:continue
            target=out/platform if category=='product' else out/'source'/platform
            files=download_records(artifact['id'],target,a.packages and category=='product')
            receipts.append({'category':category,'platform':platform,'artifact_id':artifact['id'],'run_id':run_id,'files':files})
    checks=api('actions/runs?head_sha='+current+'&event=push&per_page=30')['workflow_runs']
    required={'Documentation','Foundation probes'}
    successful={r['name'] for r in checks if r['conclusion']=='success'}
    if not required<=successful:raise ValueError('Current documentation/foundation checks incomplete')
    report={'schema_version':1,'status':'passed','revision':current,'repository':REPO,'run_id':a.run,'source_run_id':a.source_run,
            'all_jobs_succeeded':True,'product_platforms':['macos','windows','linux'],'artifacts':receipts,
            'limits':['CI and artifact records, not human desktop, formal signature or legal approval.']}
    (out/'ci-evidence.json').write_text(json.dumps(report,indent=2)+'\n');print('Collected exact current CI identities and selected bound evidence')

if __name__=='__main__':main()
