#!/usr/bin/env python3
"""Generate an owned real ZIP64 archive; large fixtures stay in build directories."""
import argparse
from pathlib import Path
import zipfile
import random

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--entries', type=int, default=100_000)
parser.add_argument('--hierarchical', action='store_true')
parser.add_argument('--shuffle', action='store_true')
args = parser.parse_args()
if not 0 <= args.entries <= 100_001:
    raise SystemExit('Fixture count is outside the supported test range')
args.output.parent.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(args.output, 'w', allowZip64=True) as archive:
    order = list(range(args.entries))
    if args.shuffle:
        random.Random(20261013).shuffle(order)
    for i in order:
        name = f'item-{i:06}.txt'
        if args.hierarchical:
            name = f'folder-{i // 1000:03}/' + name
        entry = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
        entry.compress_type = zipfile.ZIP_STORED
        archive.writestr(entry, b'')
with zipfile.ZipFile(args.output) as reference:
    assert len(reference.infolist()) == args.entries
print(f'Owned ZIP metadata fixture: {args.entries} records, {args.output.stat().st_size} bytes')
