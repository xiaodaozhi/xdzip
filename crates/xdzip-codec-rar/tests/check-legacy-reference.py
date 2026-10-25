"""Full-byte legacy RAR candidate comparison, with explicit fixture identities."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import threading

ROOT=Path(__file__).resolve().parents[3]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--worker',type=Path,required=True)
    parser.add_argument('--probe',type=Path,required=True)
    parser.add_argument('--additional',type=Path)
    parser.add_argument('--unrar',type=Path)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    output=args.output.resolve()
    if not any(output.is_relative_to(ROOT/name) for name in ['build','dist']):raise SystemExit('evidence_output_scope')
    spec=importlib.util.spec_from_file_location('native',ROOT/'scripts/check-native-codecs.py')
    native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native)
    base=ROOT/'tests/fixtures/native-archives'
    manifest=json.loads((base/'sources.json').read_text(encoding='utf-8'))
    paths=[base/'test_read_format_rar_compress_normal.rar']
    identities={row['file']:row['sha256'] for row in manifest['fixtures']}
    if args.additional:
        extra=json.loads((args.additional/'sources.json').read_text(encoding='utf-8'))
        for row in (extra if isinstance(extra,list) else extra['fixtures']):
            paths.append(args.additional/row['file']);identities[row['file']]=row['archive_sha256']
    results=[]
    env=os.environ.copy();env.pop('XDZIP_RAR_PROBE_PASSWORD',None)
    for path in paths:
        data=path.read_bytes();assert hashlib.sha256(data).hexdigest()==identities[path.name],'fixture_identity'
        rows,_=native.run_worker(args.worker,[data],format_id=4)
        for row in rows:
            if row['flags']&5:continue
            if row['size']>8*1024*1024:
                assert args.unrar is not None,'large_reference_requires_streaming_unrar'
                assert hashlib.sha256(args.unrar.read_bytes()).hexdigest()=='0d3715001790f0fd18d3e850f947b540530b2d2deb9a2e6a9e84f2ed7b234235','official_streaming_tool_identity'
                child=subprocess.Popen([str(args.unrar),'p','-cfg-','-idq','-y',str(path),row['name']],stdout=subprocess.PIPE,stderr=subprocess.DEVNULL)
                timer=threading.Timer(120,child.kill);timer.start()
                count=0;digest=hashlib.sha256()
                try:
                    while True:
                        block=child.stdout.read(65536)
                        if not block:break
                        count+=len(block);assert count<=8*1024**3,'reference_output_limit';digest.update(block)
                    assert child.wait(timeout=10)==0,'official_streaming_exit'
                finally:
                    timer.cancel();child.stdout.close()
                    if child.poll() is None:child.kill();child.wait(timeout=10)
                run=subprocess.run([str(args.probe),'digest',str(row['index']),str(path)],env=env,capture_output=True,timeout=120)
                assert run.returncode==0,(path.name,row['index'],run.stderr.decode(errors='replace'))
                actual_count,actual_digest=run.stdout.decode('ascii').strip().split('\t')
                assert int(actual_count)==count==row['size'] and actual_digest==digest.hexdigest(),'full_streaming_digest'
                expected_bytes=count;expected_digest=digest.hexdigest();comparison='full_streaming_digest'
            else:
                _,expected=native.run_worker(args.worker,[data],format_id=4,selected=row['index'])
                run=subprocess.run([str(args.probe),'decode',str(row['index']),str(path)],env=env,capture_output=True,timeout=90)
                assert run.returncode==0,(path.name,row['index'],run.stderr.decode(errors='replace'))
                assert run.stdout==expected,(path.name,row['index'],'full_reference_bytes')
                expected_bytes=len(expected);expected_digest=hashlib.sha256(run.stdout).hexdigest();comparison='full_bytes'
            results.append(dict(case=path.name,entry=row['index'],name=row['name'],method=row['method'],
                bytes=expected_bytes,sha256=expected_digest,comparison=comparison,status='passed',independent_original_input=False))
    report=dict(scope='Legacy candidate full-byte comparison; no product-route, all-filter or old-generation claim.',
        probe_sha256=hashlib.sha256(args.probe.read_bytes()).hexdigest(),
        reference_sha256=hashlib.sha256(args.worker.read_bytes()).hexdigest(),results=results)
    output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(cases=len(results),bytes=sum(row['bytes'] for row in results))))


if __name__=='__main__':main()
