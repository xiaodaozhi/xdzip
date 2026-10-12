#!/usr/bin/env python3
"""Reject untracked, stale or reference-only raster assets in the Qt resource bundle."""
from pathlib import Path
import hashlib
import json
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
assets = {}
for folder in ('design/assets/v1', 'design/hifi/v2'):
    base = ROOT / folder
    manifest = json.loads((base / 'manifest.json').read_text())
    for entry in manifest['assets']:
        assets[(base / entry['file']).resolve()] = (base, entry)

checked = 0
for node in ET.parse(ROOT / 'apps/desktop/ui.qrc').iter('file'):
    path = (ROOT / 'apps/desktop' / node.text).resolve()
    if path.suffix != '.png':
        continue
    assert path in assets, f'Untracked runtime image: {node.text}'
    base, entry = assets[path]
    assert entry['kind'] == 'vector-rasterization', f'Reference/candidate cannot ship: {node.text}'
    assert hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256'], node.text
    source = (base / entry['source']).resolve()
    assert source in assets, f'Missing vector provenance: {node.text}'
    assert hashlib.sha256(source.read_bytes()).hexdigest() == assets[source][1]['sha256'], str(source)
    checked += 1
print(f'Checked {checked} runtime aliases against vector/raster manifests. Visual fidelity requires separate review.')
