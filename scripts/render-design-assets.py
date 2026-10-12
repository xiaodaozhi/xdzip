#!/usr/bin/env python3
"""Export all canonical SVGs on transparent, fixed 24/48/72-pixel canvases."""
from __future__ import annotations

import base64
import argparse
import hashlib
import json
import mimetypes
import os
import re
from pathlib import Path

from PIL import Image
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1] / 'design/assets'


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def visible_bounds(path: Path, scale: int) -> list[float]:
    """Report pixel-edge bounds, excluding nearly transparent antialias tails."""
    with Image.open(path) as image:
        mask = image.convert('RGBA').getchannel('A').point(lambda alpha: 255 if alpha > 8 else 0)
        bounds = mask.getbbox()
    if not bounds:
        raise ValueError(f'Empty transparent export: {path.name}')
    return [round(edge / scale, 4) for edge in bounds]


def embedded_gallery() -> str:
    def embed(match: re.Match) -> str:
        path = ROOT / match[1]
        payload = base64.b64encode(path.read_bytes()).decode('ascii')
        return f'src="data:{mimetypes.guess_type(path.name)[0]};base64,{payload}"'
    return re.sub(r'src="([^"]+)"', embed, (ROOT / 'index.html').read_text())


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--standalone', type=Path, help='Export a self-contained HTML asset gallery')
    parser.add_argument('--gallery-only', action='store_true', help='Export existing assets without rasterizing them again')
    args = parser.parse_args()
    if args.gallery_only and not args.standalone:
        parser.error('--gallery-only requires --standalone')
    if args.gallery_only:
        export_gallery(args.standalone)
        return
    if os.environ.get('XDZIP_FONTCONFIG_FILE'):
        os.environ['FONTCONFIG_FILE'] = os.environ['XDZIP_FONTCONFIG_FILE']
    manifest = json.loads((ROOT / 'manifest.json').read_text())
    vectors = [entry for entry in manifest['assets'] if entry['kind'] == 'vector-reconstruction']
    manifest['assets'] = [entry for entry in manifest['assets'] if entry['kind'] != 'vector-rasterization']
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(
            executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
            args=['--no-sandbox', '--disable-dev-shm-usage'])
        page = browser.new_page()
        for entry in vectors:
            source = ROOT / entry['file']
            for scale in (1, 2, 3):
                size = 24 * scale
                page.set_viewport_size({'width': size, 'height': size})
                page.set_content('<style>html,body{margin:0;background:transparent}svg{width:100vw;height:100vh;display:block}</style>' + source.read_text())
                target = ROOT / 'png' / entry['theme'] / f'{source.stem}@{scale}x.png'
                page.screenshot(path=str(target), omit_background=True)
                bounds = visible_bounds(target, scale)
                raster = {
                    'id': f'raster/{entry["theme"]}/{target.stem}',
                    'file': str(target.relative_to(ROOT)), 'kind': 'vector-rasterization',
                    'size': [size, size], 'canvas': [24, 24], 'logical_size': [24, 24],
                    'scale': scale, 'theme': entry['theme'], 'category': entry['category'],
                    'source': entry['file'], 'source_sha256': entry['sha256'], 'visible_bounds': bounds,
                    'usage': 'runtime-and-design', 'sha256': digest(target),
                }
                if 'optical_spec' in entry:
                    raster['optical_spec'] = entry['optical_spec']
                manifest['assets'].append(raster)
                if scale == 3:
                    entry['visible_bounds'] = bounds
                    entry['visible_bounds_measurement'] = '3x Chromium raster, alpha > 8, divided by scale'
        page.set_viewport_size({'width': 1200, 'height': 1000})
        page.set_content(embedded_gallery())
        page.evaluate('Promise.all([...document.images].map(image => image.decode()))')
        page.screenshot(path=str(ROOT / 'preview/asset-contact-sheet.png'), full_page=True)
        browser.close()
    (ROOT / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    if args.standalone:
        export_gallery(args.standalone)
    print(json.dumps({'transparent_pngs': len(vectors) * 3, 'source_icon_designs': len(vectors) // 2,
                      'scales': [1, 2, 3], 'visible_bounds_recorded': True}))


def export_gallery(output: Path) -> None:
    """Keep images local to the document and link supporting files to the repository."""
    page = embedded_gallery()
    repository = ROOT.parents[1]

    def documentation_link(match: re.Match) -> str:
        target = match[1]
        if target.startswith('#'):
            return match[0]
        path = (ROOT / target).resolve().relative_to(repository)
        return f'href="https://github.com/xiaodaozhi/xdzip/blob/main/{path.as_posix()}"'

    page = re.sub(r'href="([^"]+)"', documentation_link, page)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(page)
    print(f'{output}: {output.stat().st_size} bytes; gallery images are embedded')


if __name__ == '__main__':
    main()
