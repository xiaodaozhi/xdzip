#!/usr/bin/env python3
"""Run package lifecycle verification in the previously built SDK-free runtime image."""
import argparse
import json
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--packages',type=Path,required=True);parser.add_argument('--fixture',type=Path,required=True)
parser.add_argument('--formats',type=Path,required=True)
parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
result=subprocess.run(['docker','run','--rm','--network=none','--security-opt=no-new-privileges',
    '--volume',str(args.packages.resolve())+':/packages:ro','--volume',str(args.fixture.resolve())+':/fixture.zip:ro',
    '--volume',str(args.formats.resolve())+':/formats:ro',
    '--volume',str(ROOT/'scripts/check-linux-install.py')+':/check.py:ro','xdzip-linux-runtime-check','python3','/check.py'],capture_output=True,text=True,timeout=180)
args.output.with_suffix('.log').write_text(result.stdout+result.stderr)
if result.returncode:raise SystemExit('Linux installer lifecycle failed; see installer log')
line=next((line for line in result.stdout.splitlines() if line.startswith('XDZIP_INSTALL ')),None)
if not line:raise SystemExit('Installer lifecycle evidence missing')
args.output.write_text(json.dumps(json.loads(line.removeprefix('XDZIP_INSTALL ')),indent=2)+'\n')
print('Linux install/upgrade/rollback/busy protection/GIO association/uninstall passed as non-root app user')
