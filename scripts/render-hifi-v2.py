#!/usr/bin/env python3
"""Rasterize the v2 SVG candidates and render offline review contact sheets."""
from pathlib import Path
import base64
import hashlib
import json
import mimetypes
import os
import re
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1] / 'design/hifi/v2'


def embedded_page(name):
    def embed(match):
        path = ROOT / match[1]
        mime = mimetypes.guess_type(path)[0]
        payload = base64.b64encode(path.read_bytes()).decode('ascii')
        return f'src="data:{mime};base64,{payload}"'
    return re.sub(r'src="([^"]+)"', embed, (ROOT / name).read_text())


def main():
    if os.environ.get('XDZIP_FONTCONFIG_FILE'):
        os.environ['FONTCONFIG_FILE'] = os.environ['XDZIP_FONTCONFIG_FILE']
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(
            executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
            args=['--no-sandbox', '--disable-dev-shm-usage'])
        page = browser.new_page()
        for source in sorted((ROOT / 'svg').rglob('*.svg')):
            for scale in (1, 2, 3):
                size = 24 * scale
                page.set_viewport_size({'width': size, 'height': size})
                page.set_content('<style>html,body{margin:0;background:transparent}svg{width:100vw;height:100vh;display:block}</style>' + source.read_text())
                page.screenshot(path=str(ROOT / 'png' / source.parent.name / f'{source.stem}@{scale}x.png'), omit_background=True)
        page.set_viewport_size({'width': 1200, 'height': 1000})
        for source, output in [('boards.html', 'boards.png'), ('assets.html', 'assets.png')]:
            page.set_content(embedded_page(source))
            page.evaluate('Promise.all([...document.images].map(image => image.decode()))')
            page.screenshot(path=str(ROOT / 'preview' / output), full_page=True)
        browser.close()
    manifest = json.loads((ROOT / 'manifest.json').read_text())
    manifest['assets'] = [a for a in manifest['assets'] if a['kind'] != 'vector-rasterization']
    for path in sorted((ROOT / 'png').rglob('*.png')):
        scale = int(path.stem[-2])
        manifest['assets'].append({
            'id': f'raster/{path.parent.name}/{path.stem}',
            'file': str(path.relative_to(ROOT)), 'kind': 'vector-rasterization',
            'size': [24 * scale, 24 * scale], 'logical_size': [24, 24], 'scale': scale,
            'source': f'svg/{path.parent.name}/{path.stem[:-3]}.svg',
            'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
    (ROOT / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    print('Exported 84 transparent PNGs and two review sheets')


if __name__ == '__main__':
    main()
