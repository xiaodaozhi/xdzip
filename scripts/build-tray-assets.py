#!/usr/bin/env python3
"""Render dedicated native status icons from the canonical xdzip mark."""
import io
import json
from pathlib import Path
from PIL import Image
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]

def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--browser', help='Optional Chromium executable')
    args = parser.parse_args()
    spec = json.loads((ROOT/'docs/visual/desktop-integration.json').read_text())['icon']
    mark = (ROOT/'design/assets/brand/mark.svg').read_text()
    paths = mark[mark.index('<path'):mark.rindex('</svg>')]
    output = ROOT/'design/assets/tray'
    output.mkdir(exist_ok=True)
    with sync_playwright() as p:
        browser = p.chromium.launch(**({'executable_path':args.browser} if args.browser else {}))
        page = browser.new_page(device_scale_factor=1)
        page.route('**/*', lambda route: route.abort())
        for variant, color in [('black','#20232a'),('white','#ffffff'),('contrast','#ffffff')]:
            outline = ' stroke="#20232a" stroke-width="24" stroke-linejoin="round"' if variant == 'contrast' else ''
            svg = f'<svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 22 22" fill="{color}"><svg x="5" y="2" width="12" height="18" viewBox="407 325 440 660"{outline}>{paths}</svg></svg>'
            (output/f'{variant}.svg').write_text(svg+'\n')
            for size in spec['rasterSizes']:
                page.set_viewport_size({'width':size,'height':size})
                page.set_content('<style>html,body{margin:0;background:transparent}svg{display:block;width:100vw;height:100vh}</style>'+svg)
                raw = page.screenshot(omit_background=True)
                with Image.open(io.BytesIO(raw)) as image:
                    assert image.getbbox() and image.size == (size,size)
                    image.save(output/f'{variant}-{size}.png')
        browser.close()

if __name__ == '__main__': main()
