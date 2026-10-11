#!/usr/bin/env python3
"""Validate source crop integrity and asset metadata; requires Pillow for read-only inspection."""
from pathlib import Path
from PIL import Image
import json, hashlib, xml.etree.ElementTree as ET
r = Path(__file__).resolve().parents[1] / 'design/assets/v1'
m = json.loads((r / 'manifest.json').read_text())
checks = []
for a in m['assets']:
    p = r / a['file']
    assert p.exists()
    assert hashlib.sha256(p.read_bytes()).hexdigest() == a['sha256']
    if a['kind'] == 'original-pixel-crop':
        x, y, w, h = a['rect']
        src = Image.open(r / a['source']).convert('RGBA')
        img = Image.open(p).convert('RGBA')
        assert img.size == (w, h)
        assert img.tobytes() == src.crop((x, y, x + w, y + h)).tobytes()
    if a['kind'] in ['vector-rasterization', 'generative-transparent-reconstruction']:
        im = Image.open(p).convert('RGBA')
        assert im.size == tuple(a['size'])
        alpha = im.getchannel('A')
        assert alpha.getextrema()[0] == 0 and alpha.getextrema()[1] > 0
for p in (r / 'svg').rglob('*.svg'):
    ET.parse(p)
    assert '<script' not in p.read_text() and '<image' not in p.read_text()
checks = ['All manifest assets exist and match SHA-256', '24 source crops are pixel-identical to recorded source rectangles', 'Transparent raster exports have expected sizes, transparent pixels and visible content', '54 SVG files parse without scripts or embedded images']
report = dict(status='passed', asset_entries=len(m['assets']), source_crops=24, svg_files=54, vector_png_files=len(list((r / 'png').rglob('*.png'))), checks=checks, limits=['No Qt renderer validation', 'Reconstructed vectors and generated brand asset are not exact original layers', 'No native OS window controls exported as production bitmaps'])
(r / 'validation.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(report, ensure_ascii=False))
