#!/usr/bin/env python3
"""Exercise the relocated Linux bundle in a runtime-only Docker filesystem."""
import argparse
import json
from pathlib import Path
import re
import subprocess
from deployment_audit import digest

ROOT=Path(__file__).resolve().parents[1]


def check_events(output):
    events=[json.loads(line[len('XDZIP_PROBE '):]) for line in output.splitlines() if line.startswith('XDZIP_PROBE ')]
    by_name={e['event']:e for e in events}
    required={'first_frame','open_ready','cancel_requested','cancel_settled','quit_requested'}
    if not required<=by_name.keys():raise ValueError('Isolated product probe did not finish all stages')
    if by_name['open_ready']['records']!=100000:raise ValueError('Isolated open did not read the owned fixture')
    if not by_name['cancel_requested']['pending'] or not by_name['cancel_settled']['cancelled']:
        raise ValueError('Isolated cancellation did not interrupt an active open')
    if not by_name['first_frame']['runtime_available']:raise ValueError('Runtime unavailable in isolated app')
    return events


def check_https(output):
    records=[json.loads(line[len('XDZIP_UPDATE_HTTPS '):]) for line in output.splitlines() if line.startswith('XDZIP_UPDATE_HTTPS ')]
    if len(records)!=1:raise ValueError('SDK-free HTTPS probe did not produce exactly one result')
    record=records[0]
    if record.get('ssl_supported') is not True or record.get('ssl_backend')!='openssl':
        raise ValueError('SDK-free HTTPS probe did not load the OpenSSL backend')
    if record.get('state') not in {'up_to_date','available'} or record.get('error'):
        raise ValueError('SDK-free HTTPS probe did not complete the real GitHub update request')
    return record


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,required=True);parser.add_argument('--fixture',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();payload=args.root.resolve()
    manifest=json.loads((payload/'share/deployment.json').read_text())
    packages=manifest['system_packages']
    if not packages or any(not re.fullmatch(r'[a-z0-9][a-z0-9+.-]*(?::amd64)?',p) for p in packages):
        raise SystemExit('Invalid bounded runtime package list')
    image='xdzip-linux-runtime-check'
    build=subprocess.run(['docker','build','--build-arg','XDZIP_RUNTIME_PACKAGES='+' '.join(packages),
        '-f',str(ROOT/'apps/desktop/packaging/linux/ci-runtime.Dockerfile'),'-t',image,
        str(ROOT/'apps/desktop/packaging/linux')],capture_output=True,text=True)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.with_suffix('.build.log').write_text(build.stdout+build.stderr)
    if build.returncode:raise SystemExit('Runtime-only container image build failed; see build log')
    # A new mount location with a space verifies relative paths, not the CI build layout.
    result=subprocess.run(['docker','run','--rm','--network=none','--read-only','--cap-drop=ALL',
        '--security-opt=no-new-privileges','--user','1000:1000','--tmpfs','/tmp:exec,mode=1777',
        '--volume',str(payload)+':/opt/xdzip runtime:ro','--volume',str(args.fixture.resolve())+':/fixture.zip:ro',
        '--volume',str(ROOT/'apps/desktop/packaging/linux/verify-runtime.sh')+':/verify-runtime.sh:ro',
        image,'/bin/sh','/verify-runtime.sh'],capture_output=True,text=True,timeout=60)
    args.output.with_suffix('.runtime.log').write_text(result.stdout+result.stderr)
    if result.returncode:raise SystemExit('SDK-free relocated application check failed; see runtime log')
    try:events=check_events(result.stdout)
    except ValueError as error:raise SystemExit(str(error))
    # Network remains disabled for archive/runtime validation. A separate process
    # proves QtNetwork HTTPS and the system CA store without mounting the Qt SDK.
    https=subprocess.run(['docker','run','--rm','--read-only','--cap-drop=ALL',
        '--security-opt=no-new-privileges','--user','1000:1000','--tmpfs','/tmp:exec,mode=1777',
        '--volume',str(payload)+':/opt/xdzip runtime:ro',
        '--volume',str(ROOT/'apps/desktop/packaging/linux/verify-https.sh')+':/verify-https.sh:ro',
        image,'/bin/sh','/verify-https.sh'],capture_output=True,text=True,timeout=80)
    args.output.with_suffix('.https.log').write_text(https.stdout+https.stderr)
    if https.returncode:raise SystemExit('SDK-free HTTPS update request failed; see HTTPS log')
    try:https_record=check_https(https.stdout)
    except ValueError as error:raise SystemExit(str(error))
    args.output.with_suffix('.https.json').write_text(json.dumps(https_record,indent=2)+'\n')
    report={'schema_version':1,'status':'passed','baseline':manifest['baseline'],
        'executable_sha256':digest(payload/'bin/xdzip'),'read_only_payload':True,'network':'none',
        'system_qt_and_compiler':'absent','rendering':'Xvfb X11, Qt Quick software; not physical GPU acceptance',
        'events':events,'limits':['Runtime-only container is not a full desktop clean-install, IME or reader acceptance.',
                                 'Installer, file associations, other distributions and Wayland remain unverified.']}
    report['https']={'network':'GitHub HTTPS, separate SDK-free process','result':https_record}
    args.output.write_text(json.dumps(report,indent=2)+'\n')
    print('Linux SDK-free runtime passed: version, X11 startup, 100k open, active cancel and shutdown')

if __name__=='__main__':main()
