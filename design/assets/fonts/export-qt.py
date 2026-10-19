#!/usr/bin/env python3
"""Export the reviewed WOFF2 subset as native TTF without changing outlines or metrics."""
import argparse
import hashlib
import io
from pathlib import Path
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--check', action='store_true')
args = parser.parse_args()
font = TTFont(ROOT / 'xdzip-design-sans-sc.woff2', recalcTimestamp=False)
font.flavor = None
output = io.BytesIO()
font.save(output)
path = ROOT / 'xdzip-design-sans-sc.ttf'
data = output.getvalue()
if args.check:
    if not path.exists() or path.read_bytes() != data:
        raise SystemExit('Qt font is stale; run design/assets/fonts/export-qt.py')
else:
    path.write_bytes(data)
print(f'Qt UI font: {len(data)} bytes, SHA-256 {hashlib.sha256(data).hexdigest()}')
