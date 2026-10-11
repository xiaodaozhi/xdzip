#!/usr/bin/env python3
"""Export design assets; these tools do not form part of the product runtime."""
from pathlib import Path
from playwright.sync_api import sync_playwright
import os, json, hashlib, re, base64, mimetypes
root = Path(__file__).resolve().parents[1] / 'design/assets/v1'
if os.environ.get('XDZIP_FONTCONFIG_FILE'):
    os.environ['FONTCONFIG_FILE'] = os.environ['XDZIP_FONTCONFIG_FILE']
with sync_playwright() as p:
    b = p.chromium.launch(executable_path='/usr/bin/chromium', args=['--no-sandbox', '--disable-dev-shm-usage'])
    page = b.new_page()
    for src in sorted((root / 'svg').rglob('*.svg')):
        for scale in [1, 2, 3]:
            size = 24 * scale
            page.set_viewport_size({'width': size, 'height': size})
            page.set_content('<style>html,body{margin:0;background:transparent}svg{width:100vw;height:100vh;display:block}</style>' + src.read_text())
            page.screenshot(path=str(root / 'png' / src.parent.name / f'{src.stem}@{scale}x.png'), omit_background=True)
    page.set_viewport_size({'width': 1200, 'height': 1000})
    page.set_content(re.sub('src="([^"]+)"', lambda m: 'src="data:' + mimetypes.guess_type(m[1])[0] + ';base64,' + base64.b64encode((root / m[1]).read_bytes()).decode() + '"', (root / 'index.html').read_text()))
    page.screenshot(path=str(root / 'preview/asset-contact-sheet.png'), full_page=True)
    b.close()
manifest = json.loads((root / 'manifest.json').read_text())
manifest['assets'] = [a for a in manifest['assets'] if a['kind'] != 'vector-rasterization']
for path in sorted((root / 'png').rglob('*.png')):
    scale = int(path.stem[-2])
    manifest['assets'].append(dict(id='raster/' + path.parent.name + '/' + path.stem, file=str(path.relative_to(root)), kind='vector-rasterization', size=[24 * scale, 24 * scale], scale=scale, logical_size=[24, 24], source=f'svg/{path.parent.name}/{path.stem[:-3]}.svg', sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
(root / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
print('Exported 162 transparent PNGs and contact sheet')
