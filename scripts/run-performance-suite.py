#!/usr/bin/env python3
"""Owned-fixture Release observations, never automatic reference-host approval."""
import argparse
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import sys
import tempfile
from deployment_audit import ROOT, digest, within


def checked(command, log, env=None):
    with log.open('a') as stream:
        subprocess.run([str(v) for v in command],stdout=stream,stderr=stream,env=env,check=True)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['app','metrics','core','output']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--backend',default='metal' if sys.platform=='darwin' else 'software')
    a=p.parse_args();output=a.output.resolve()
    if not within(output,ROOT/'build') or output.exists():p.error('Use a new build/ directory')
    output.mkdir(parents=True);log=output/'suite.log'
    environment=dict(os.environ)
    environment.pop('XDZIP_PROBE_RESOURCE_MANIFEST',None);environment.pop('XDZIP_EXPECT_QT_SOURCE_PROOF',None)
    if sys.platform=='darwin':environment['MACOSX_DEPLOYMENT_TARGET']='14.0'
    # Build the exact same current locked core, rather than trusting a stale
    # executable path supplied by the caller. Retain compiler artifact identity.
    build=subprocess.run(['cargo','build','--release','--locked','-p','xdzip-m3-probe','--message-format=json'],cwd=ROOT,env=environment,capture_output=True,text=True,check=True)
    artifacts=[json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
    core_artifact=next(r for r in artifacts if r.get('reason')=='compiler-artifact' and r.get('target',{}).get('name')=='xdzip-m3-probe' and r.get('executable'))
    if Path(core_artifact['executable']).resolve()!=a.core.resolve():raise ValueError('Supplied core path differs from actual current Cargo build')
    core_build={'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'core_sha256':digest(a.core),
                'cargo_lock_sha256':digest(ROOT/'Cargo.lock'),'profile':core_artifact['profile'],'fresh':core_artifact['fresh']}
    (output/'core-build.json').write_text(json.dumps(core_build,indent=2)+'\n')
    if sys.platform=='darwin':
        environment.update(QT_QPA_PLATFORM='cocoa',QSG_RHI_BACKEND='metal');environment.pop('QT_QUICK_BACKEND',None)
    common=[sys.executable,ROOT/'scripts/measure-desktop.py','--app',a.app.resolve(),'--metrics',a.metrics.resolve(),'--require-backend',a.backend]
    metadata=output/'metadata.zip';payload=output/'payload.zip'
    checked([sys.executable,ROOT/'scripts/generate-m2-fixture.py','--output',metadata],log)
    checked([sys.executable,ROOT/'scripts/generate-m3-fixture.py','--output',payload],log)
    checked(common+['--fixture',metadata,'--runs','30','--output',output/'interactive.json'],log,environment)
    checked(common+['--fixture',metadata,'--runs','1','--idle-ms','71000','--browse-ms','31000','--output',output/'steady.json'],log,environment)
    with __import__('zipfile').ZipFile(payload) as archive: records=len(archive.infolist())
    checked(common+['--fixture',payload,'--expected-records',str(records),'--runs','5','--operation-extract','--output',output/'gui-extraction.json'],log,environment)
    cores=[]
    for index in range(5):
        with tempfile.TemporaryDirectory(prefix='owned-core-',dir=output) as temp:
            result=subprocess.run([str(a.core.resolve()),str(payload),temp],capture_output=True,text=True,timeout=180,check=True)
            report=json.loads(result.stdout)
            if not report.get('independent_byte_check'):raise ValueError('Core byte verification missing')
            cores.append(report)
    (output/'core-extraction.json').write_text(json.dumps(cores,indent=2)+'\n')
    gui=json.loads((output/'gui-extraction.json').read_text());interactive=json.loads((output/'interactive.json').read_text());steady=json.loads((output/'steady.json').read_text())
    identity=interactive['identity']
    if identity['configuration']!='Release' or identity['dirty']:raise ValueError('Suite requires a clean Release build')
    if any(r['identity']['revision']!=identity['revision'] for r in gui['runs']+steady['runs']):raise ValueError('Mixed suite identity')
    median_core=statistics.median(r['extract_ms'] for r in cores);median_gui=gui['summary']['extraction_ms']['median']
    environment_record={'os':platform.platform(),'architecture':platform.machine(),'power_device_freeze_approved':False}
    if sys.platform=='darwin':
        for key,cmd in [('model',['sysctl','-n','hw.model']),('memory_bytes',['sysctl','-n','hw.memsize']),('power',['pmset','-g','batt'])]:
            environment_record[key]=subprocess.check_output(cmd,text=True).strip()
        display=json.loads(subprocess.check_output(['system_profiler','-json','SPDisplaysDataType'],text=True))
        environment_record['gpu']=[{key:value for key,value in item.items() if key in ['sppci_model','spdisplays_vendor','spdisplays_metal','spdisplays_vram','spdisplays_vram_shared']}
                                   for item in display.get('SPDisplaysDataType',[])]
    result={'schema_version':1,'status':'automated-observations-complete-reference-approval-pending','identity':identity,
            'app_sha256':digest(a.app),'core_sha256':digest(a.core),'core_build_sha256':digest(output/'core-build.json'),'fixture_sha256':{'metadata':digest(metadata),'payload':digest(payload)},
            'environment':environment_record,'interactive_samples':30,'throughput_samples_per_driver':5,
            'native_idle':steady['runs'][0]['idle_cpu_one_core_percent'],
            'idle_observed_ms':steady['runs'][0]['idle_interval_ms'],
            'throughput':{'core_median_ms':median_core,'gui_median_ms':median_gui,'observed_gui_overhead_fraction':median_gui/median_core-1,
                          'order':'GUI 5 runs followed by core 5 runs, warm cache uncontrolled; no causal UI-overhead/budget assertion'},
            'reports':[{'file':f.name,'sha256':digest(f)} for f in sorted(output.glob('*.json'))],
            'limits':['Read-only native OS sampling and real owned-fixture extraction on this host, not a signed stable reference-host qualification.',
                      'OS I/O accounting, shared memory and frame notifications retain the limits in each report.',
                      'No screen-reader, IME, physical multi-monitor or reference-budget approval is inferred.']}
    (output/'performance-suite.json').write_text(json.dumps(result,indent=2)+'\n');print('30 interactive samples, steady idle/browse and paired-driver extraction observations complete')

if __name__=='__main__':main()
