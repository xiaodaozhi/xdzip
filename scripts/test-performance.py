#!/usr/bin/env python3
"""Measurement statistics, owned-output checks and real native counter protocol."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('measure',ROOT/'scripts/measure-desktop.py')
measure=importlib.util.module_from_spec(spec);spec.loader.exec_module(measure)
METRICS=None

class StatisticsTests(unittest.TestCase):
    def test_tail_and_uncertainty_are_explicit(self):
        d=measure.summarize(list(range(1,31)))
        self.assertEqual(d['n'],30);self.assertEqual(d['p95_nearest_rank'],29);self.assertEqual(d['max'],30)
        self.assertLessEqual(d['median_bootstrap_95_percent_interval'][0],d['median'])
        self.assertGreaterEqual(d['median_bootstrap_95_percent_interval'][1],d['median'])
        self.assertIsNone(measure.summarize([]));self.assertIsNone(measure.summarize([1])['median_bootstrap_95_percent_interval'])

    def test_native_memory_is_not_relabelled_rss(self):
        sample=measure.aggregate({'memory_kind':'physical_footprint','processes':[{'rss_bytes':100,'memory_bytes':50,'cpu_seconds':1,'read_bytes':7,'written_bytes':8}]})
        self.assertEqual(sample['rss_bytes'],100);self.assertEqual(sample['memory_bytes'],50)

    def test_independent_extraction_detects_extra_corrupt_and_linked_outputs(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);archive=root/'owned.zip';out=root/'out';out.mkdir()
            with zipfile.ZipFile(archive,'w') as z:z.writestr('中文.txt',b'owned')
            f=out/'中文.txt';f.write_bytes(b'owned')
            self.assertEqual(measure.verify_output(archive,out)['bytes'],5)
            f.write_bytes(b'wrong')
            with self.assertRaisesRegex(ValueError,'differ'):measure.verify_output(archive,out)
            f.write_bytes(b'owned');(out/'extra').touch()
            with self.assertRaisesRegex(ValueError,'Unexpected'):measure.verify_output(archive,out)
            (out/'extra').unlink();f.unlink()
            if os.name!='nt':
                f.symlink_to(archive)
                with self.assertRaisesRegex(ValueError,'link'):measure.verify_output(archive,out)

    def test_native_helper_observes_owned_process_and_io_counters(self):
        if not METRICS:self.skipTest('Native metrics helper not supplied')
        with tempfile.TemporaryDirectory() as temp:
            worker=Path(temp)/'worker.py';payload=Path(temp)/'payload';cpu=Path(temp)/'cpu.json'
            worker.write_text('import json,os,pathlib,sys,time\ntime.sleep(.3)\nx=bytearray(16*1024*1024)\nf=open(sys.argv[1],"wb");f.write(x);f.flush();os.fsync(f.fileno())\nend=time.process_time()+.3\nwhile time.process_time()<end:sum(range(10000))\npathlib.Path(sys.argv[2]).write_text(json.dumps({"cpu_seconds":time.process_time()}))\ntime.sleep(.5)\n')
            process=subprocess.Popen([sys.executable,str(worker),str(payload),str(cpu)])
            try:
                p=subprocess.run([str(METRICS),str(process.pid),'50'],capture_output=True,text=True,timeout=15,check=True)
                process.wait(timeout=5)
                rows=[json.loads(x) for x in p.stdout.splitlines()]
                observed=[p for row in rows for p in row['processes'] if p['pid']==process.pid]
                self.assertGreaterEqual(len(observed),2);self.assertTrue(all(p['start'] for p in observed))
                self.assertGreater(max(p['rss_bytes'] for p in observed),0)
                self.assertGreater(max(p['memory_bytes'] for p in observed),0)
                expected_cpu=json.loads(cpu.read_text())['cpu_seconds']
                observed_cpu=max(p['cpu_seconds'] for p in observed)
                self.assertGreater(observed_cpu,expected_cpu*.75)
                self.assertLess(observed_cpu,expected_cpu*1.3+.02)
                self.assertTrue(all(p['read_bytes']>=0 and p['written_bytes']>=0 for p in observed))
            finally:
                if process.poll() is None:process.kill();process.wait()

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--metrics',type=Path);args,remaining=parser.parse_known_args()
    METRICS=args.metrics.resolve() if args.metrics else None
    unittest.main(argv=[sys.argv[0]]+remaining)
