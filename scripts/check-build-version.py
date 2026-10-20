#!/usr/bin/env python3
"""Compare the built CLI and native package metadata against VERSION.json."""
import argparse
import json
import os
from pathlib import Path
import plistlib
import runpy
import subprocess
from desktop_product import platform_executable_name

ROOT=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--app',type=Path,required=True)
parser.add_argument('--output',type=Path)
args=parser.parse_args()
product_name=platform_executable_name()
if args.app.name != product_name:
    raise SystemExit('Built product filename must be '+product_name)
info=runpy.run_path(str(ROOT/'scripts/sync-version.py'))['version_info']()
# --version must work before platform plugin / GUI initialization.
env=dict(os.environ,QT_QPA_PLATFORM='xdzip-invalid-platform')
result=subprocess.run([str(args.app.resolve()),'--version'],capture_output=True,text=True,env=env,check=True)
expected=f'xdzip {info["display_version"]} (build {info["build"]}, '
if not result.stdout.strip().startswith(expected):raise SystemExit('Built CLI version differs from VERSION.json')
try:
    repository = Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],cwd=ROOT,text=True,stderr=subprocess.DEVNULL).strip())
except (FileNotFoundError,subprocess.CalledProcessError):
    repository = None
if repository is not None and repository.resolve() == ROOT.resolve():
    revision = subprocess.check_output(['git','rev-parse','--short=12','HEAD'],cwd=ROOT,text=True).strip()
    dirty = bool(subprocess.check_output(['git','status','--porcelain','--untracked-files=normal'],cwd=ROOT,text=True))
    if f', {revision}' not in result.stdout or (', dirty)' in result.stdout) != dirty:
        raise SystemExit('Built revision/dirty flag differs from the current source; rebuild first')
report={'version':info['display_version'],'build':info['build'],'cli':result.stdout.strip(),'product_filename':product_name}
if args.app.parent.name=='MacOS' and args.app.parent.parent.name=='Contents':
    with (args.app.parent.parent/'Info.plist').open('rb') as file:bundle=plistlib.load(file)
    if bundle['CFBundleShortVersionString']!=info['version'] or bundle['CFBundleVersion']!=str(info['build']) or bundle['CFBundleLongVersionString']!=info['display_version']:
        raise SystemExit('macOS bundle version differs from VERSION.json')
    if bundle['CFBundleExecutable']!=product_name or bundle['CFBundleName']!='xdzip':
        raise SystemExit('macOS bundle product name differs from xdzip')
    report['macos_bundle']='matched'
if os.name=='nt':
    metadata=subprocess.check_output(['powershell','-NoProfile','-Command','[Diagnostics.FileVersionInfo]::GetVersionInfo($env:XDZIP_VERSION_EXE) | Select-Object FileVersion,ProductVersion,FileMajorPart,FileMinorPart,FileBuildPart,FilePrivatePart,IsPreRelease,OriginalFilename | ConvertTo-Json -Compress'],env=dict(os.environ,XDZIP_VERSION_EXE=str(args.app.resolve())),text=True)
    metadata=json.loads(metadata)
    values=[metadata[k] for k in ('FileMajorPart','FileMinorPart','FileBuildPart','FilePrivatePart')]
    if values!=[int(n) for n in info['version'].split('.')]+[info['build']] or any(metadata[k]!=info['display_version'] for k in ('FileVersion','ProductVersion')) or metadata['IsPreRelease']!=(info['channel']!='stable'):
        raise SystemExit('Windows VERSIONINFO differs from VERSION.json')
    if metadata['OriginalFilename']!=product_name:
        raise SystemExit('Windows OriginalFilename differs from '+product_name)
    report['windows_versioninfo']='matched'
if args.output:
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n')
print('Built CLI and native version metadata match:',info['display_version'])
