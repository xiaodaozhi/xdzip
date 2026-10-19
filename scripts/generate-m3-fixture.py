#!/usr/bin/env python3
"""Deterministic 32 MiB actual Deflate workload; no user data or large git artifact."""
import argparse, json, zipfile, hashlib
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
a.output.parent.mkdir(parents=True,exist_ok=True)
block=bytes(range(256))*2048
with zipfile.ZipFile(a.output,'w') as z:
 for i in range(64):
  entry=zipfile.ZipInfo(f'batch-{i%8:02}/payload-{i:03}.bin',(2026,10,13,0,0,0));entry.compress_type=zipfile.ZIP_DEFLATED
  z.writestr(entry,block)
print(json.dumps({'files':64,'output_bytes':32*1024*1024,'input_bytes':a.output.stat().st_size,'sha256':hashlib.sha256(a.output.read_bytes()).hexdigest()}))
