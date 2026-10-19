#!/usr/bin/env python3
"""Measure real product frames/queries/tasks with read-only native OS counters.

No disk-cache purge, unsafe acceleration, reference-host or release approval.
"""
import argparse
import json
import math
import os
from pathlib import Path, PurePosixPath
import platform
import random
import shutil
import statistics
import subprocess
import tempfile
import threading
import time
import zipfile
from deployment_audit import ROOT, digest, within


def percentile(values, fraction):
    values = sorted(values)
    return values[max(0, math.ceil(len(values)*fraction)-1)]


def summarize(values):
    if not values: return None
    generator = random.Random(20261018)
    draws=[generator.choices(values,k=len(values)) for _ in range(1000)]
    medians=[statistics.median(v) for v in draws]
    tails=[percentile(v,.95) for v in draws]
    return {'n':len(values), 'median':statistics.median(values), 'p95_nearest_rank':percentile(values,.95),
            'min':min(values), 'max':max(values), 'stdev':statistics.stdev(values) if len(values)>1 else 0,
            'median_bootstrap_95_percent_interval':[percentile(medians,.025),percentile(medians,.975)] if len(values)>=5 else None,
            'p95_bootstrap_95_percent_interval':[percentile(tails,.025),percentile(tails,.975)] if len(values)>=20 else None}


def aggregate(sample):
    rows = sample['processes']
    return {**sample, 'rss_bytes':sum(p['rss_bytes'] for p in rows),
            'memory_bytes':sum(p['memory_bytes'] for p in rows),
            'cpu_seconds':sum(p['cpu_seconds'] for p in rows),
            'read_bytes':sum(p['read_bytes'] for p in rows),
            'written_bytes':sum(p['written_bytes'] for p in rows), 'process_count':len(rows)}


def verify_output(fixture, target):
    files, total = 0, 0
    with zipfile.ZipFile(fixture) as archive:
        for item in archive.infolist():
            relative = PurePosixPath(item.filename)
            if relative.is_absolute() or '..' in relative.parts:
                raise ValueError('Only owned non-traversing fixtures are accepted')
            path = target/item.filename
            if item.is_dir():
                if not path.is_dir(): raise ValueError('Output directory missing')
            else:
                if path.is_symlink() or not within(path, target): raise ValueError('Unexpected output link')
                with archive.open(item) as expected, path.open('rb') as actual:
                    while True:
                        chunk = expected.read(1024*1024)
                        if actual.read(len(chunk) if chunk else 1) != chunk:
                            raise ValueError('Independent extraction bytes differ')
                        if not chunk: break
                files += 1; total += item.file_size
    if len([p for p in target.rglob('*') if p.is_file()]) != files:
        raise ValueError('Unexpected output files')
    return {'files':files,'bytes':total,'independent_byte_crc_check':True}


def run_once(app, fixture, metrics, parent, index, idle_ms, browse_ms, extract, deadline):
    events, samples, errors = [], [], []
    with tempfile.TemporaryDirectory(prefix='owned-performance-', dir=parent) as temp:
        profile = Path(temp); env = dict(os.environ, XDZIP_SETTINGS_PATH=str(profile/'settings.ini'),
                                         XDZIP_PROBE_IDLE_MS=str(idle_ms), XDZIP_PROBE_BROWSE_MS=str(browse_ms))
        # Avoid inherited measurement mode accidentally writing files.
        env.pop('XDZIP_PROBE_OPERATION', None); env.pop('XDZIP_PROBE_EXTRACT_TARGET', None)
        target = profile/'output'
        if extract:
            target.mkdir(mode=0o700)
            env.update(XDZIP_PROBE_OPERATION='extract',XDZIP_PROBE_EXTRACT_TARGET=str(target))
        start = time.perf_counter()
        process = subprocess.Popen([str(app),'--desktop-probe',str(fixture)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,env=env)
        try:
            sampler = subprocess.Popen([str(metrics),str(process.pid),'100'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        except Exception:
            process.kill();process.wait()
            raise
        threads = []
        def event_reader():
            try:
                for line in process.stdout:
                    if line.startswith('XDZIP_PROBE '):
                        item=json.loads(line[len('XDZIP_PROBE '):]);item['received_ms']=(time.perf_counter()-start)*1000;events.append(item)
            except Exception: errors.append('Invalid product probe event')
        def sample_reader():
            try:
                for line in sampler.stdout:
                    item=aggregate(json.loads(line));item['received_ms']=(time.perf_counter()-start)*1000
                    item['phase']=events[-1]['event'] if events else 'starting';samples.append(item)
            except Exception: errors.append('Invalid native sampler event')
        def drain(stream):
            # Framework logs may include private paths. They are intentionally
            # not copied into shareable performance reports.
            for _ in stream: pass
        for function,args in [(event_reader,()),(sample_reader,()),(drain,(process.stderr,)),(drain,(sampler.stderr,))]:
            thread=threading.Thread(target=function,args=args);thread.start();threads.append(thread)
        try:
            process.wait(timeout=deadline);exited=(time.perf_counter()-start)*1000
            sampler.wait(timeout=3)
            for thread in threads:thread.join(timeout=3)
            if process.returncode or sampler.returncode or errors or any(t.is_alive() for t in threads):
                raise ValueError('Product/native measurement failed')
            by_name={e['event']:e for e in events}
            required={'first_frame','open_ready','search_frame','sort_frame','browse_stable','quit_requested'}
            required |= {'extract_finished'} if extract else {'cancel_requested','cancel_settled'}
            if not required <= set(by_name) or not samples or not any(s['processes'] for s in samples):
                raise ValueError('Measurement stage or native counters missing')
            if not extract and (not by_name['cancel_requested']['pending'] or not by_name['cancel_settled']['cancelled']):
                raise ValueError('Cancellation did not interrupt in-flight open')
            # Require complete tree counters rather than silently using a partial sum.
            if any(s['process_count']==0 for s in samples[:-1]):
                raise ValueError('Native process counters unavailable')
            first=by_name['first_frame'];first_ms=first['received_ms']
            idle=[s for s in samples if s['phase']=='first_frame' and s['received_ms']>=first_ms+10000]
            idle_cpu = 100*(idle[-1]['cpu_seconds']-idle[0]['cpu_seconds'])/((idle[-1]['received_ms']-idle[0]['received_ms'])/1000) if len(idle)>1 else None
            totals={}
            for sample in samples:
                for p in sample['processes']:
                    key=(p['pid'],p['start']);old=totals.setdefault(key,dict(read_bytes=0,written_bytes=0,cpu_seconds=0))
                    for field in old:old[field]=max(old[field],p[field])
            result={'index':index,'identity':first,'events':events,'samples':samples,
                    'launch_to_frame_ms':first_ms,'open_ms':by_name['open_ready']['elapsed_ms'],
                    'search_to_frame_ms':by_name['search_frame']['elapsed_ms'],'sort_to_frame_ms':by_name['sort_frame']['elapsed_ms'],
                    'cancel_ms':None if extract else by_name['cancel_settled']['elapsed_ms'],
                    'quit_upper_bound_ms':exited-by_name['quit_requested']['received_ms'],
                    'sampled_peak_tree_rss_bytes':max(s['rss_bytes'] for s in samples),
                    'sampled_peak_tree_memory_bytes':max(s['memory_bytes'] for s in samples),
                    'memory_kind':samples[0]['memory_kind'],'io_kind':samples[0]['io_kind'],
                    'sampled_process_lifetime_io':{f:sum(p[f] for p in totals.values()) for f in ['read_bytes','written_bytes']},
                    'idle_cpu_one_core_percent':idle_cpu,'idle_interval_ms':idle[-1]['received_ms']-idle[0]['received_ms'] if len(idle)>1 else None}
            if extract:
                result['extraction_ms']=by_name['extract_finished']['elapsed_ms'];result['extraction']=verify_output(fixture,target)
            return result
        finally:
            for child in [process,sampler]:
                if child.poll() is None:child.kill();child.wait()
            for thread in threads:thread.join(timeout=3)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app',type=Path,required=True);parser.add_argument('--fixture',type=Path,required=True)
    parser.add_argument('--metrics',type=Path,required=True);parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--expected-records',type=int,default=100000);parser.add_argument('--runs',type=int,default=30)
    parser.add_argument('--idle-ms',type=int,default=1500);parser.add_argument('--browse-ms',type=int,default=0)
    parser.add_argument('--operation-extract',action='store_true');parser.add_argument('--require-backend')
    args=parser.parse_args()
    if not 1<=args.runs<=100 or not 0<=args.idle_ms<=90000 or not 0<=args.browse_ms<=60000:parser.error('Invalid sample count or dwell time')
    if args.output.exists() or not within(args.output.resolve(),ROOT/'build'):parser.error('Use a new output file under build/')
    if not args.metrics.is_file() or not args.app.is_file() or not args.fixture.is_file():parser.error('Missing product, native metrics tool or owned fixture')
    args.output.parent.mkdir(parents=True,exist_ok=True)
    runs=[];app=args.app.resolve();fixture=args.fixture.resolve()
    for index in range(args.runs):
        r=run_once(app,fixture,args.metrics.resolve(),args.output.parent,index,args.idle_ms,args.browse_ms,args.operation_extract,220+(args.idle_ms+args.browse_ms)/1000)
        events={e['event']:e for e in r['events']}
        if events['open_ready']['records']!=args.expected_records:raise ValueError('Fixture record count mismatch')
        if args.require_backend and r['identity']['backend']!=args.require_backend:raise ValueError('Required renderer unavailable; software fallback is not accepted')
        if runs and any(r['identity'][k]!=runs[0]['identity'][k] for k in ['revision','dirty','configuration','backend']):raise ValueError('Mixed build or renderer identity')
        runs.append(r);print('Measured product run '+str(index+1),flush=True)
    keys=['launch_to_frame_ms','open_ms','search_to_frame_ms','sort_to_frame_ms','cancel_ms','quit_upper_bound_ms','sampled_peak_tree_rss_bytes','sampled_peak_tree_memory_bytes','extraction_ms','idle_cpu_one_core_percent']
    report={'schema_version':2,'platform':platform.platform(),'machine':platform.machine(),'app_sha256':digest(app),'fixture_sha256':digest(fixture),
            'fixture_bytes':fixture.stat().st_size,'sampler_sha256':digest(args.metrics),'identity':runs[0]['identity'],'runs':runs,
            'summary':{k:summarize([r[k] for r in runs if r.get(k) is not None]) for k in keys},
            'release_reference_qualified':False,
            'limits':['Fresh processes with uncontrolled filesystem/GPU caches, not cold-disk measurements.',
                      'Native 100ms snapshots may miss peaks, short-lived descendants and final process I/O. Process identity includes start time.',
                      'macOS memory is summed physical footprint, Linux PSS, Windows private committed bytes; these are not interchangeable or unique shared-memory accounting.',
                      'Windows I/O transfer counters include non-disk I/O; ETW storage attribution remains separate. Other OS counters are storage-accounting lower bounds.',
                      'Timing includes event delivery and frame notification, not display scanout. Sampler overhead and normal host load affect measurements.',
                      'Bootstrap intervals describe observed samples; not independent-tail or stable reference-host qualification. Manual power/device freeze, signed release and budget approval remain pending.']}
    args.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report['summary'],indent=2))

if __name__=='__main__':main()
