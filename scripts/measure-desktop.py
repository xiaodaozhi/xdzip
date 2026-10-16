#!/usr/bin/env python3
"""Measure fresh product processes using an owned ZIP and OS process counters.

No cache purge, dependency installation, archive writes or release assertions.
RSS is sampled and summed across descendants, including shared-page double count.
"""
import argparse
import json
import math
import os
from pathlib import Path
import platform
import statistics
import shutil
import tempfile
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[1]

def cpu_seconds(value):
    days, _, rest = value.rpartition('-')
    parts = [float(p) for p in rest.split(':')]
    return (int(days) * 86400 if days else 0) + sum(v * 60 ** n for n, v in enumerate(reversed(parts)))

def processes():
    if os.name == 'nt':
        command = ['powershell', '-NoProfile', '-Command', 'Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,WorkingSetSize,KernelModeTime,UserModeTime | ConvertTo-Json -Compress']
        result = json.loads(subprocess.check_output(command, text=True))
        return {int(p['ProcessId']): (int(p['ParentProcessId']), int(p.get('WorkingSetSize') or 0), (int(p.get('KernelModeTime') or 0) + int(p.get('UserModeTime') or 0)) / 1e7) for p in result}
    lines = subprocess.check_output(['ps', '-axo', 'pid=,ppid=,rss=,time='], text=True).splitlines()
    return {int(pid): (int(ppid), int(rss) * 1024, cpu_seconds(cpu)) for pid, ppid, rss, cpu in (line.split() for line in lines)}

def descendants(table, pid):
    found = {pid} if pid in table else set()
    while True:
        expanded = found | {child for child, (parent, _, _) in table.items() if parent in found}
        if expanded == found: return found
        found = expanded

def percentile(values, fraction):
    values = sorted(values)
    return values[max(0, math.ceil(len(values) * fraction) - 1)]

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=ROOT / 'build/review/private-test/desktop-performance.json')
    parser.add_argument('--expected-records', type=int, default=100000)
    parser.add_argument('--runs', type=int, default=5)
    args = parser.parse_args()
    if not 1 <= args.runs <= 20: parser.error('--runs must be 1–20')
    args.output.parent.mkdir(parents=True, exist_ok=True)
    runs = []
    for index in range(args.runs):
        events, samples = [], []
        profile = Path(tempfile.mkdtemp(prefix='probe-profile-', dir=args.output.parent))
        env = dict(os.environ, XDZIP_SETTINGS_PATH=str(profile / 'settings.ini'))
        started = time.perf_counter()
        process = subprocess.Popen([str(args.app.resolve()), '--desktop-probe', str(args.fixture.resolve())], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=env)
        stderr = []
        def read_events():
            for line in process.stdout:
                if line.startswith('XDZIP_PROBE '):
                    data = json.loads(line[len('XDZIP_PROBE '):]);data['received_ms'] = (time.perf_counter() - started) * 1000;events.append(data)
        reader = threading.Thread(target=read_events);reader.start()
        err_reader = threading.Thread(target=lambda:stderr.extend(process.stderr.readlines()));err_reader.start()
        try:
            while process.poll() is None:
                if time.perf_counter() - started > 40:
                    process.kill();raise RuntimeError('Product probe exceeded 40-second deadline')
                table = processes();pids = descendants(table, process.pid)
                phase = events[-1]['event'] if events else 'starting'
                samples.append({'elapsed_ms':(time.perf_counter()-started)*1000,'phase':phase,'rss_bytes':sum(table[p][1] for p in pids),'cpu_seconds':sum(table[p][2] for p in pids),'processes':len(pids)})
                time.sleep(.2)
            process.wait();reader.join();err_reader.join()
            exited = (time.perf_counter()-started)*1000
            by_name = {event['event']:event for event in events}
            required = {'first_frame','open_ready','cancel_requested','cancel_settled','quit_requested'}
            if process.returncode or not required <= set(by_name): raise RuntimeError('Product probe failed or did not finish all stages')
            if by_name['open_ready']['records'] != args.expected_records: raise RuntimeError('Archive record count does not match the fixture contract')
            if not by_name['cancel_requested']['pending'] or not by_name['cancel_settled']['cancelled']: raise RuntimeError('Cancellation did not interrupt an in-flight open')
            idle = [s for s in samples if s['phase']=='first_frame']
            idle_cpu = None
            if len(idle) >= 2:
                idle_cpu = 100 * (idle[-1]['cpu_seconds']-idle[0]['cpu_seconds']) / ((idle[-1]['elapsed_ms']-idle[0]['elapsed_ms'])/1000)
            runs.append({'index':index,'events':events,'samples':samples,'launch_to_frame_ms':by_name['first_frame']['received_ms'],'open_ms':by_name['open_ready']['elapsed_ms'],'cancel_ms':by_name['cancel_settled']['elapsed_ms'],'quit_upper_bound_ms':exited-by_name['quit_requested']['received_ms'],'sampled_peak_tree_rss_bytes':max(s['rss_bytes'] for s in samples),'idle_cpu_one_core_percent':idle_cpu})
        finally:
            if process.poll() is None:process.kill();process.wait()
            shutil.rmtree(profile)
    metrics = ['launch_to_frame_ms','open_ms','cancel_ms','quit_upper_bound_ms','sampled_peak_tree_rss_bytes']
    report = {'schema_version':1,'platform':platform.platform(),'machine':platform.machine(),'fixture_bytes':args.fixture.stat().st_size,'runs':runs,'summary':{key:{'median':statistics.median([r[key] for r in runs]),'p95_nearest_rank':percentile([r[key] for r in runs],.95)} for key in metrics},'limits':['Fresh processes on a running host; filesystem/GPU caches are uncontrolled, not a disk-cold benchmark.','Debug/release identity must be recorded with the build. No release performance budget is asserted.','Launch-to-frame includes process launch, event emission and parent pipe delivery.','RSS is sampled at about 200 ms plus counter command latency, and can miss peaks; summed shared pages may be counted more than once.','Quit latency is an upper bound including sampling/poll delay.','CPU counter resolution depends on OS; Linux ps time may have one-second resolution.','Internal stage durations have integer-millisecond resolution.',
               'OS disk I/O bytes, physical GPU verification, other-platform manual checks and screen readers remain separate.']}
    args.output.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(report['summary'],indent=2))

if __name__ == '__main__':main()
