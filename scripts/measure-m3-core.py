#!/usr/bin/env python3
"""Measure actual extraction and independent byte verification into owned temporary files."""
import argparse
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
args.output.parent.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='xdzip-m3-measure-') as temporary:
    owned = Path(temporary)
    fixture = owned / 'deflate.zip'
    generated = subprocess.check_output([
        os.sys.executable, str(ROOT / 'scripts/generate-m3-fixture.py'),
        '--output', str(fixture)], text=True)
    target = owned / 'output'
    target.mkdir(mode=0o700)
    command = [str(args.binary.resolve()), str(fixture), str(target)]
    if platform.system() == 'Darwin':
        command = ['/usr/bin/time', '-l'] + command
    completed = subprocess.run(command, capture_output=True, text=True, timeout=120)
    if completed.returncode:
        raise SystemExit(completed.stderr)
    report = json.loads(completed.stdout)
    if platform.system() == 'Darwin':
        match = re.search(r'(\d+)\s+maximum resident set size', completed.stderr)
        report['peak_rss_kib'] = int(match.group(1)) / 1024 if match else None
    report.update(os=platform.system(), architecture=platform.machine(),
                  fixture=json.loads(generated), profile='release',
                  scope='64 Deflate files / 8 directories, actual decode+fsync+publication; single CI run, warm-cache effects possible')
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))
