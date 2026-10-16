#!/usr/bin/env python3
"""Export the reviewable HTML design with shared SVGs and offline demo images."""
import argparse
import base64
import hashlib
import json
import mimetypes
import re
from pathlib import Path
from brand_assets import master_svg

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
    # The manifest is the production inventory; stray files never enter a bundle.
    asset_root = ROOT / 'design/assets'
    manifest = json.loads((asset_root / 'manifest.json').read_text())
    application = asset_root / 'brand/xdzip.svg'
    receipt = next(entry for entry in manifest['assets'] if entry['file'] == 'brand/xdzip.svg')
    if hashlib.sha256(application.read_bytes()).hexdigest() != receipt['sha256']:
        raise SystemExit('Application artwork differs from its canonical source')
    assets['brand'] = {'application': application.read_text().strip(),
                       'applicationMac': master_svg('application', 'macos').strip()}
    for entry in manifest['assets']:
        if entry['kind'] != 'vector-reconstruction':
            continue
        path = asset_root / entry['file']
        if hashlib.sha256(path.read_bytes()).hexdigest() != entry['sha256']:
            raise SystemExit(f'Shared SVG does not match its manifest: {entry["file"]}')
        assets['icons'][entry['theme']][path.stem] = path.read_text().strip()
    for key, name in [('lake', 'photo-lake-thumbnail'), ('lake-preview', 'photo-lake-preview'), ('sunrise', 'photo-sunrise-thumbnail')]:
        assets['photos'][key] = data_url(ROOT / 'design/assets/demo' / f'{name}.png')
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
    canvas = inline_html('canvas.html', [FONT_STYLESHEET, 'styles.css', 'hifi.css', 'controls-tokens.css', 'controls.css', 'welcome.css'],
                         ['assets.js', 'display-spec.js', 'controls-tokens.js', 'controls.js', 'ui-messages.js', 'version.js', 'app.js', 'main-controls.js'])
    references = {}
    source_root = ROOT / 'design/hifi'
    catalog = json.loads((source_root / 'catalog.json').read_text())
    for entry in catalog['screens']:
        path = source_root / entry['file']
        if hashlib.sha256(path.read_bytes()).hexdigest() != entry['sha256']:
            raise SystemExit(f'Approved reference changed: {entry["file"]}')
        references['../hifi/' + entry['file']] = data_url(path)
    review = inline_html('index.html', [FONT_STYLESHEET, 'review.css'], [])
    review = review.replace('<script defer src="review.js"></script>', '')
    review = review.replace('href="components.html"', 'href="xdzip-components.html"')
    review = review.replace('src="canvas.html"', 'src="about:blank"')
    config = 'window.XDZIP_CANVAS=' + json.dumps(canvas, ensure_ascii=False) + ';window.XDZIP_REFERENCES=' + json.dumps(references) + ';'
    review = review.replace('<script defer src="display-spec.js"></script>', '')
    license_notice = '<script type="text/plain" id="xdzip-font-license">' + (FONTS / 'OFL.txt').read_text() + '</script>'
    review = review.replace('</body>', license_notice + script(config) + script((SOURCE / 'display-spec.js').read_text()) + script((SOURCE / 'review.js').read_text()) + '</body>')
    output = ROOT / 'dist/xdzip-hifi-design.html'
    output.parent.mkdir(exist_ok=True)
    output.write_text(review)
    welcome_review = review.replace('<option value="main" selected>', '<option value="main">', 1).replace('<option value="welcome">', '<option value="welcome" selected>', 1)
    output.with_name('xdzip-welcome-design.html').write_text(welcome_review)
    # Give the corrected logical-size delivery a distinct download name.
    output.with_name('xdzip-hifi-design-1to1.html').write_text(review)
    components = inline_html('components.html', [FONT_STYLESHEET, 'controls-tokens.css', 'controls.css', 'components.css'],
                             ['assets.js', 'controls-tokens.js', 'controls.js', 'components.js'])
    components = components.replace('</body>', license_notice + '</body>')
    output.with_name('xdzip-components.html').write_text(components)
    print(f'{output}: {output.stat().st_size} bytes; no external runtime assets')


if __name__ == '__main__':
    main()
