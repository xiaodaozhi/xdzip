#!/usr/bin/env python3
"""Export the reviewable HTML design with shared SVGs and offline demo images."""
import argparse
import base64
import hashlib
import json
import mimetypes
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'design/html'
FONTS = ROOT / 'design/assets/fonts'
FONT_STYLESHEET = '../assets/fonts/fonts.css'


def data_url(path):
    return f'data:{mimetypes.guess_type(path.name)[0]};base64,' + base64.b64encode(path.read_bytes()).decode()


def script(code):
    return '<script>' + code.replace('</script', '<\\/script') + '</script>'


def inline_html(name, styles, scripts):
    page = (SOURCE / name).read_text()
    for filename in styles:
        css = (SOURCE / filename).read_text()
        if filename == FONT_STYLESHEET:
            css = css.replace('url("./xdzip-design-sans-sc.woff2")', 'url("' + data_url(FONTS / 'xdzip-design-sans-sc.woff2') + '")')
        page = re.sub(r'<link\s+rel="stylesheet"\s+href="' + re.escape(filename) + r'"\s*/?>',
                      lambda _: '<style>' + css + '</style>', page)
    for filename in scripts:
        page = re.sub(r'<script\s+defer\s+src="' + re.escape(filename) + r'"></script>', '', page)
    return page.replace('</body>', ''.join(script((SOURCE / name).read_text()) for name in scripts) + '</body>')


def main():
    assets = {'icons': {'light': {}, 'dark': {}}, 'photos': {}}
    for base in ('design/assets/v1', 'design/hifi/v2'):
        for theme in assets['icons']:
            for path in sorted((ROOT / base / 'svg' / theme).glob('*.svg')):
                assets['icons'][theme][path.stem] = path.read_text().strip()
    for key, name in [('lake', 'photo-lake-thumbnail'), ('lake-preview', 'photo-lake-preview'), ('sunrise', 'photo-sunrise-thumbnail')]:
        assets['photos'][key] = data_url(ROOT / 'design/assets/v1/slices' / f'{name}.png')
    generated = '// Generated from reviewed source assets; edit SVGs, not this file.\nwindow.XDZIP_ASSETS = ' + json.dumps(assets, ensure_ascii=False) + ';\n'
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.check:
        if (SOURCE / 'assets.js').read_text() != generated:
            raise SystemExit('HTML shared assets are stale; run scripts/build-html-design.py')
        font_manifest = json.loads((FONTS / 'manifest.json').read_text())
        for entry in ['subset', 'license']:
            item = font_manifest[entry]
            if hashlib.sha256((FONTS / item['path']).read_bytes()).hexdigest() != item['sha256']:
                raise SystemExit(f'HTML font {entry} does not match its manifest')
        print('HTML shared assets match reviewed SVGs and source photos')
        return
    (SOURCE / 'assets.js').write_text(generated)
    canvas = inline_html('canvas.html', [FONT_STYLESHEET, 'styles.css', 'hifi.css'], ['assets.js', 'app.js'])
    references = {}
    for path in sorted((ROOT / 'design/hifi').glob('v*/screens/*.png')):
        references['../hifi/' + str(path.relative_to(ROOT / 'design/hifi'))] = data_url(path)
    review = inline_html('index.html', [FONT_STYLESHEET, 'review.css'], [])
    review = review.replace('<script defer src="review.js"></script>', '')
    review = review.replace('src="canvas.html"', 'src="about:blank"')
    config = 'window.XDZIP_CANVAS=' + json.dumps(canvas, ensure_ascii=False) + ';window.XDZIP_REFERENCES=' + json.dumps(references) + ';'
    review = review.replace('<script defer src="display-spec.js"></script>', '')
    license_notice = '<script type="text/plain" id="xdzip-font-license">' + (FONTS / 'OFL.txt').read_text() + '</script>'
    review = review.replace('</body>', license_notice + script(config) + script((SOURCE / 'display-spec.js').read_text()) + script((SOURCE / 'review.js').read_text()) + '</body>')
    output = ROOT / 'dist/xdzip-hifi-design.html'
    output.parent.mkdir(exist_ok=True)
    output.write_text(review)
    # Give the corrected logical-size delivery a distinct download name.
    output.with_name('xdzip-hifi-design-1to1.html').write_text(review)
    print(f'{output}: {output.stat().st_size} bytes; no external runtime assets')


if __name__ == '__main__':
    main()
