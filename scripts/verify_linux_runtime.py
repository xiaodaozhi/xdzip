#!/usr/bin/env python3
"""Exercise the relocated Linux bundle in a runtime-only Docker filesystem."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import time
from uuid import uuid4
from deployment_audit import digest
from update_https_retry import https_retry_plan

ROOT=Path(__file__).resolve().parents[1]
HTTPS_ATTEMPTS=3
HTTPS_WAIT_LIMIT=240


def archive_previous_evidence(output, *, https_only=False):
    """Keep earlier results without leaving them at the current run's paths."""
    suffixes=['.https.json','.https.runtime.json','.https.log','.https-waits.json']
    for attempt in range(1,HTTPS_ATTEMPTS+1):
        suffixes.extend(f'.https.attempt-{attempt}.{end}' for end in ('log','json','runtime.json'))
    paths=[output.with_suffix(suffix) for suffix in suffixes]
    if not https_only:
        paths=[output]+paths+[output.with_suffix('.build.log'),output.with_suffix('.runtime.log')]
    existing=[path for path in paths if path.exists() or path.is_symlink()]
    if not existing:return
    if any(not path.is_symlink() and not path.is_file() for path in existing):
        raise ValueError('A prior evidence path is not a file')
    history=output.parent/(output.stem+'.history')/uuid4().hex
    history.mkdir(parents=True)
    for path in existing:
        path.replace(history/path.name)


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


def https_record(output):
    records=[json.loads(line[len('XDZIP_UPDATE_HTTPS '):]) for line in output.splitlines() if line.startswith('XDZIP_UPDATE_HTTPS ')]
    if len(records)!=1 or not isinstance(records[0],dict):
        raise ValueError('SDK-free HTTPS probe did not produce exactly one result')
    return records[0]


def https_runtime_record(output):
    prefix='XDZIP_HTTPS_RUNTIME '
    records=[json.loads(line[len(prefix):]) for line in output.splitlines() if line.startswith(prefix)]
    required=('qt_openssl_backend_present','openssl_package_installed','system_ca_bundle_nonempty','ca_package_installed')
    if len(records)!=1 or not isinstance(records[0],dict) or any(records[0].get(key) is not True for key in required):
        raise ValueError('SDK-free HTTPS runtime capabilities are missing or invalid')
    return records[0]


def check_https(output):
    record=https_record(output)
    if record.get('ssl_supported') is not True or record.get('ssl_backend')!='openssl':
        raise ValueError('SDK-free HTTPS probe did not load the OpenSSL backend')
    if record.get('state') not in {'up_to_date','available'} or record.get('error'):
        raise ValueError('SDK-free HTTPS probe did not complete the real GitHub update request')
    if (type(record.get('http_diagnostics_schema')) is not int or record['http_diagnostics_schema'] != 1 or
        record.get('timeout') is not False or
        record.get('authorization_present') is not False or record.get('response_records_truncated') is not False or
        not isinstance(record.get('http_responses'),list) or not record['http_responses'] or
        type(record.get('request_count')) is not int or not len(record['http_responses']) <= record['request_count'] <= 32 or
        type(record.get('encrypted_connections')) is not int or record['encrypted_connections'] < 1 or
        type(record.get('https_responses')) is not int or record['https_responses'] < 1):
        raise ValueError('SDK-free HTTPS probe lacks complete anonymous TLS evidence')
    return record


def run_https_probe(command, output, *, run=None, sleep=None, monotonic=None, wall_clock=None):
    archive_previous_evidence(output,https_only=True)
    run=run or subprocess.run;sleep=sleep or time.sleep
    monotonic=monotonic or time.monotonic;wall_clock=wall_clock or time.time
    waited=0.0;waits=[]
    for attempt in range(1,HTTPS_ATTEMPTS+1):
        try:
            result=run(command,capture_output=True,text=True,timeout=80)
            log=result.stdout+result.stderr;exit_code=result.returncode
        except subprocess.TimeoutExpired as error:
            def decoded(value):
                return value.decode('utf-8','replace') if isinstance(value,bytes) else (value or '')
            log=decoded(error.stdout)+decoded(error.stderr)+'\nHTTPS probe process timed out\n'
            result=None;exit_code=None
        except OSError as error:
            log='HTTPS probe process failed: '+type(error).__name__+'\n'
            result=None;exit_code=None
        output.with_suffix(f'.https.attempt-{attempt}.log').write_text(log,encoding='utf-8')
        output.with_suffix('.https.log').write_text(log,encoding='utf-8')
        try:record=https_record(result.stdout) if result is not None else None
        except (ValueError,TypeError):record=None
        try:runtime=https_runtime_record(result.stdout) if result is not None else None
        except (ValueError,TypeError):runtime=None
        if record is not None:
            output.with_suffix(f'.https.attempt-{attempt}.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
        if runtime is not None:
            output.with_suffix(f'.https.attempt-{attempt}.runtime.json').write_text(json.dumps(runtime,indent=2)+'\n',encoding='utf-8')
        if exit_code==0:
            try:record=check_https(result.stdout)
            except ValueError as error:raise SystemExit(str(error))
            if runtime is None:raise SystemExit('SDK-free HTTPS runtime capabilities are missing or invalid')
            output.with_suffix('.https.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
            output.with_suffix('.https.runtime.json').write_text(json.dumps(runtime,indent=2)+'\n',encoding='utf-8')
            return record,attempt,runtime
        confirmed=(exit_code==13 and record is not None and runtime is not None and record.get('state')=='error' and
                   record.get('ssl_supported') is True and record.get('ssl_backend')=='openssl')
        plan=(https_retry_plan(record,attempt,wall_clock()) if confirmed
              else {'retry':False,'reason':'probe_process_failure'})
        if attempt==HTTPS_ATTEMPTS and plan.get('retry'):
            plan.update(retry=False,reason='attempt_limit_reached')
        elif plan.get('retry') and plan['delay_seconds'] > HTTPS_WAIT_LIMIT-waited:
            plan.update(retry=False,reason='wait_budget_exceeded')
        plan.update(attempt=attempt,waited_seconds=waited,process_exit_code=exit_code)
        waits.append(plan)
        output.with_suffix('.https-waits.json').write_text(json.dumps(waits,indent=2)+'\n',encoding='utf-8')
        if not plan['retry']:
            raise SystemExit(f'SDK-free HTTPS update request failed (exit {exit_code}; {plan["reason"]}); see HTTPS log')
        started=monotonic();plan['wait_status']='waiting';finished=False
        output.with_suffix('.https-waits.json').write_text(json.dumps(waits,indent=2)+'\n',encoding='utf-8')
        try:
            sleep(plan['delay_seconds']);finished=True
        finally:
            elapsed=max(0.0,monotonic()-started);waited+=elapsed
            plan.update(wait_status='finished' if finished and elapsed>=plan['delay_seconds'] else 'interrupted_or_short',
                        actual_wait_seconds=elapsed,total_actual_wait_seconds=waited)
            output.with_suffix('.https-waits.json').write_text(json.dumps(waits,indent=2)+'\n',encoding='utf-8')
        if not finished or elapsed < plan['delay_seconds'] or waited > HTTPS_WAIT_LIMIT:
            raise SystemExit('SDK-free HTTPS retry wait ended early or exceeded its budget')
    raise SystemExit('SDK-free HTTPS retry limit reached')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,required=True);parser.add_argument('--fixture',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();payload=args.root.resolve()
    args.output.parent.mkdir(parents=True,exist_ok=True)
    archive_previous_evidence(args.output)
    manifest=json.loads((payload/'share/deployment.json').read_text())
    packages=manifest['system_packages']
    if not packages or any(not re.fullmatch(r'[a-z0-9][a-z0-9+.-]*(?::amd64)?',p) for p in packages):
        raise SystemExit('Invalid bounded runtime package list')
    image='xdzip-linux-runtime-check'
    build=subprocess.run(['docker','build','--build-arg','XDZIP_RUNTIME_PACKAGES='+' '.join(packages),
        '-f',str(ROOT/'apps/desktop/packaging/linux/ci-runtime.Dockerfile'),'-t',image,
        str(ROOT/'apps/desktop/packaging/linux')],capture_output=True,text=True)
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
    https_command=['docker','run','--rm','--read-only','--cap-drop=ALL',
        '--security-opt=no-new-privileges','--user','1000:1000','--tmpfs','/tmp:exec,mode=1777',
        '--volume',str(payload)+':/opt/xdzip runtime:ro',
        '--volume',str(ROOT/'apps/desktop/packaging/linux/verify-https.sh')+':/verify-https.sh:ro',
        '--volume',str(ROOT/'apps/desktop/packaging/linux/verify-https-probe.sh')+':/verify-https-probe.sh:ro',
        image,'/bin/sh','/verify-https.sh']
    https_result,https_attempts,https_runtime=run_https_probe(https_command,args.output)
    report={'schema_version':1,'status':'passed','baseline':manifest['baseline'],
        'executable_sha256':digest(payload/'bin/xdzip'),'read_only_payload':True,'network':'none',
        'system_qt_and_compiler':'absent','rendering':'Xvfb X11, Qt Quick software; not physical GPU acceptance',
        'events':events,'limits':['Runtime-only container is not a full desktop clean-install, IME or reader acceptance.',
                                 'Installer, file associations, other distributions and Wayland remain unverified.']}
    report['https']={'network':'GitHub HTTPS, separate SDK-free process','runtime_capabilities':https_runtime,'result':https_result,
                     'attempts':https_attempts}
    args.output.write_text(json.dumps(report,indent=2)+'\n')
    print('Linux SDK-free runtime passed: version, X11 startup, 100k open, active cancel and shutdown')

if __name__=='__main__':main()
