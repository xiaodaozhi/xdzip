#!/usr/bin/env python3
"""Measure the actual Qt large-archive case; no fixture rows replace the runtime."""
import argparse
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
start = time.monotonic()
process = subprocess.run([str(args.binary.resolve()), 'm2LargeArchiveLatestQueryAndSourceChange'],
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
elapsed = time.monotonic() - start
peak = None
if os.name == 'posix':
    import resource
    peak = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    if platform.system() == 'Darwin':
        peak /= 1024
match = re.search(r'M2_QT_METRICS open_ms=(\d+) rows=(\d+) delegates=(\d+) full_sort_ms=(\d+)', process.stdout)
metrics = {key: int(value) for key, value in zip(['open_ms', 'rows', 'delegates', 'full_sort_ms'], match.groups())} if match else {}
report = dict(os=platform.system(), architecture=platform.machine(),
              qpa=os.environ.get('QT_QPA_PLATFORM'), renderer=os.environ.get('QT_QUICK_BACKEND'),
              metrics=metrics, elapsed_s=elapsed, peak_rss_kib=peak, returncode=process.returncode,
              passed=process.returncode == 0 and bool(metrics))
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(report, indent=2) + '\n')
args.output.with_suffix('.log').write_text(process.stdout)
print(json.dumps(report))
if not report['passed']:
    raise SystemExit(1)
