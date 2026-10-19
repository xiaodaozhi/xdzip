#!/usr/bin/env python3
"""Read trusted deployed-image metadata; do not infer redistribution rights."""
import argparse
import ctypes
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from deployment_audit import ROOT, digest, within


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--root',type=Path,required=True);p.add_argument('--audit',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    if a.output.exists() or not within(a.output.resolve(),ROOT/'build'):p.error('Use a new build/ file')
    audit=json.loads(a.audit.read_text());records=[]
    for item in audit['files']:
        relative=item['path'];path=a.root/relative
        if digest(path)!=item['sha256']:raise ValueError('Deployed bytes changed before native metadata collection')
        if os.name=='nt' and path.suffix.lower() in {'.dll','.exe'}:
            # Only our trusted, already-audited SDK/product images; no untrusted
            # archive input is ever executed or passed to a binary loader here.
            quoted=str(path.resolve()).replace("'","''")
            script="[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new(); $v=(Get-Item -LiteralPath '"+quoted+"').VersionInfo; $v | Select-Object CompanyName,ProductName,ProductVersion,FileVersion,OriginalFilename,LegalCopyright | ConvertTo-Json -Compress"
            data=json.loads(subprocess.check_output(['powershell','-NoProfile','-Command',script],text=True,encoding='utf-8-sig'))
            records.append({'file':relative,'sha256':item['sha256'],'pe_version_info':data})
        elif sys.platform.startswith('linux') and re.fullmatch(r'lib/libicuuc\.so\.\d+(?:\.\d+)*',relative):
            version=re.search(r'\.so\.(\d+)',relative)[1]
            library=ctypes.CDLL(str(path.resolve()))
            function=getattr(library,'u_getVersion_'+version)
            # ICU UVersionInfo is four uint8_t values; exact well-known exported
            # C ABI from this trusted dependency, no caller-supplied symbol name.
            data=(ctypes.c_uint8*4)();function.argtypes=[ctypes.POINTER(ctypes.c_uint8)];function.restype=None;function(data)
            records.append({'file':relative,'sha256':item['sha256'],'icu_runtime_version':'.'.join(map(str,data))})
    result={'schema_version':1,'status':'metadata-collected-no-rights-inference','platform':audit['platform'],'identity':audit['identity'],
            'deployment_audit_sha256':digest(a.audit),'files':records,
            'limits':['PE version strings and ICU runtime version are supplier metadata, not source/build equivalence or redistribution permission.',
                      'Missing original SDK publisher build/redistribution declarations remain external provenance-review questions.']}
    a.output.write_text(json.dumps(result,indent=2)+'\n');print('Collected trusted native redistribution metadata without license inference')

if __name__=='__main__':main()
