#!/usr/bin/env python3
"""Prepare or check the pinned security-patched QtNetwork in an existing SDK."""
import argparse
import json
from pathlib import Path
import sys
from qt_network_security import ROOT, identity, prepare
from windows_ninja_safety import utf8_console

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--qt-prefix', type=Path, required=True)
parser.add_argument('--work-dir', type=Path, default=ROOT/'build/toolchain/qt-network-security'/
                    {'darwin': 'macos', 'win32': 'windows'}.get(sys.platform, 'linux'))
parser.add_argument('--cache', type=Path, default=ROOT/'build/source-cache')
parser.add_argument('--parallel', type=int, default=3)
parser.add_argument('--check-only', action='store_true')
args = parser.parse_args()
if args.check_only:
    record = identity(args.qt_prefix)
else:
    with utf8_console():
        record = prepare(args.qt_prefix, args.work_dir, args.cache, args.parallel)
print(json.dumps({'status': 'passed', 'variant': record['variant'], 'network_sha256': record['network_sha256']}))
