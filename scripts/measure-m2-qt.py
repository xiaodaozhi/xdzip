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
args.output.parent.mkdir(parents=True, exist_ok=True)
log_path = args.output.with_suffix('.log').resolve()
# QTest's explicit file logger works for Windows GUI-subsystem executables too;
# inherited stdout may be absent even when the test itself succeeds.
log_path.unlink(missing_ok=True)
start = time.monotonic()
process = subprocess.run([str(args.binary.resolve()), 'm2LargeArchiveLatestQueryAndSourceChange',
                          '-o', str(log_path) + ',txt'],
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
elapsed = time.monotonic() - start
test_log = log_path.read_text(encoding='utf-8', errors='replace') if log_path.exists() else ''
if process.stdout:
    test_log += '\n' + process.stdout
log_path.write_text(test_log, encoding='utf-8')
peak = None
if os.name == 'posix':
    import resource
    peak = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    if platform.system() == 'Darwin':
        peak /= 1024
match = re.search(r'M2_QT_METRICS open_ms=(\d+) rows=(\d+) delegates=(\d+) full_sort_ms=(\d+)', test_log)
metrics = {key: int(value) for key, value in zip(['open_ms', 'rows', 'delegates', 'full_sort_ms'], match.groups())} if match else {}
report = dict(os=platform.system(), architecture=platform.machine(),
              qpa=os.environ.get('QT_QPA_PLATFORM'), renderer=os.environ.get('QT_QUICK_BACKEND'),
              metrics=metrics, elapsed_s=elapsed, peak_rss_kib=peak, returncode=process.returncode,
              passed=process.returncode == 0 and bool(metrics))
args.output.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report))
if not report['passed']:
    raise SystemExit(1)
