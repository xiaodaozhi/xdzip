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
binary = args.binary.resolve()
native_temp = subprocess.check_output([str(binary), '--native-temp-base'], text=True).strip()
with tempfile.TemporaryDirectory(prefix='xdzip-m3-measure-', dir=native_temp) as temporary:
    owned = Path(temporary)
    fixture = owned / 'deflate.zip'
    generated = subprocess.check_output([
        os.sys.executable, str(ROOT / 'scripts/generate-m3-fixture.py'),
        '--output', str(fixture)], text=True)
    target = owned / 'output'
    # POSIX private mode; Windows inherits the owned parent ACL rather than
    # Python-version-specific mode=0700 ACL synthesis. Product ACL checks remain.
    target.mkdir(mode=0o700 if platform.system() != 'Windows' else 0o777)
    command = [str(binary), str(fixture), str(target)]
    if platform.system() == 'Darwin':
        command = ['/usr/bin/time', '-l'] + command
    completed = subprocess.run(command, capture_output=True, text=True, timeout=120)
    if completed.returncode:
        # Only the freshly owned CI target is examined; no user paths are logged.
        if platform.system() == 'Windows':
            shell = os.environ.get('XDZIP_CI_POWERSHELL', 'powershell.exe')
            quoted = str(target).replace("'", "''")
            acl = subprocess.run([shell, '-NoProfile', '-Command', f"(Get-Acl -LiteralPath '{quoted}').Sddl"], capture_output=True, text=True, timeout=10)
            if acl.returncode == 0:
                print('Owned benchmark target ACL:', acl.stdout.strip())
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
