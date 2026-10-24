# scripts/check-add-dialog.py

"""Measure the C01-derived addition dialog and exercise its HTML interaction states."""
import argparse
from contextlib import contextmanager
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import math
import os
from pathlib import Path
import tempfile
from threading import Thread

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = json.loads((ROOT / 'docs/visual/add-reference.json').read_text(encoding='utf-8'))
PALETTE = json.loads((ROOT / 'docs/visual/tokens.json').read_text(encoding='utf-8'))


def qt_cases(directory):
    cases = []
    for language in ['zh-CN', 'en-US']:
        for factor in [1, 1.25, 1.5]:
            for dark in [False, True]:
                name = f'{language}{factor:g}{int(dark)}'
                capture = json.loads((directory / (name + '.json')).read_text(encoding='utf-8'))
                nodes = {node['name']: node for node in capture['nodes'] if node['name']}
                assert capture['width'] == REFERENCE['logical_width'], name
                assert capture['height'] <= min(736, capture['availableScreenHeight'] - 60), name
                for key in ['updateInputPanel', 'updateDestinationCard', 'updateOptionsCard']:
                    assert abs(nodes[key]['x'] - REFERENCE['content_inset']) <= 1, (name, key)
                    assert abs(nodes[key]['width'] - (491 - 2 * REFERENCE['content_inset'])) <= 1, (name, key)
                assert nodes['updateHeading']['fontPixelSize'] == math.floor(20 * factor + .5), name
                assert nodes['updateInputPanel']['color'].upper() == PALETTE['dark' if dark else 'light']['operationCard'].upper(), name
                assert nodes['updateInputSize-0']['text'] == '11 B', name
                assert nodes['updateRemoveInput-0']['width'] == 28, name
                assert nodes['updateRemoveInput-0Icon']['width'] == 24, name
                assert nodes['updateRemoveInput-0Icon']['source'].endswith('/remove.png'), name
                for key in ['updateCancel', 'startUpdate']:
                    node = nodes[key]
                    assert node['y'] >= nodes['updateFooter']['y'], (name, key)
                    assert node['y'] + node['height'] <= capture['height'] + 1, (name, key)
                assert abs(nodes['updateFooter']['y'] + nodes['updateFooter']['height'] - capture['height']) <= 1, name
                if factor == 1:
                    assert abs(nodes['updateInputRow-0']['height'] - REFERENCE['input_row_height']) <= 1, name
                    assert abs(nodes['startUpdate']['height'] - REFERENCE['footer_action_height']) <= 1, name
                    assert nodes['updateConflictPolicyBackground']['radius'] == 5, name
                cases.append({'name': name, 'width': capture['width'], 'height': capture['height'], 'dpr': capture['dpr'], 'platform': capture['platform'], 'renderer': capture.get('requestedQuickBackend')})
    return cases


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


@contextmanager
def server_url():
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
    Thread(target=server.serve_forever, daemon=True).start()
    try:
        yield f'http://127.0.0.1:{server.server_port}/design/hifi/sources/archive-add.html'
    finally:
        server.shutdown()
        server.server_close()


def geometry(page):
    return page.locator('#add-dialog').evaluate('''dialog => {
        const box = dialog.getBoundingClientRect(), footer = dialog.querySelector('footer').getBoundingClientRect();
        const body = dialog.querySelector('main');
        return {width: box.width, height: box.height, footerGap: box.bottom - footer.bottom,
                footerOutside: footer.left < box.left - 1 || footer.right > box.right + 1,
                horizontalOverflow: dialog.scrollWidth > dialog.clientWidth + 1,
                bodyOverflow: body.scrollHeight > body.clientHeight + 1,
                missingImages: [...dialog.querySelectorAll('img')].some(img => !img.complete || !img.naturalWidth),
                titleSize: parseFloat(getComputedStyle(dialog.querySelector('h1')).fontSize),
                clippedText: [...dialog.querySelectorAll('.readonly-field span')].some(el => el.scrollWidth > el.clientWidth + 1)};
    }''')


def check_geometry(record):
    assert abs(record['width'] - 491) <= 1, record
    assert record['height'] <= 736.5, record
    assert abs(record['footerGap']) <= 1, record
    assert not any(record[key] for key in ['footerOutside', 'horizontalOverflow', 'missingImages', 'clippedText']), record


def html_cases(output):
    from playwright.sync_api import expect, sync_playwright
    records = []
    with server_url() as url, sync_playwright() as p:
        options = {'channel': os.environ['XDZIP_BROWSER_CHANNEL']} if os.environ.get('XDZIP_BROWSER_CHANNEL') else {}
        if os.environ.get('XDZIP_CHROMIUM'):
            options = {'executable_path': os.environ['XDZIP_CHROMIUM']}
        browser = p.chromium.launch(**options)
        try:
            for dpr in [1, 1.25, 2]:
                page = browser.new_page(viewport={'width': 1200, 'height': 1000}, device_scale_factor=dpr)
                errors = []
                page.on('pageerror', lambda error: errors.append(str(error)))
                for language in ['zh-CN', 'en-US']:
                    for dark in [False, True]:
                        for factor in [1, 1.25, 1.5]:
                            tag = f'{language}-{dark}-{factor}-{dpr}'
                            page.goto(f'{url}?export&lang={language}&theme={"dark" if dark else "light"}&textScale={factor}')
                            page.evaluate('document.fonts.ready')
                            record = {'name': tag, 'dpr': dpr, 'browserZoom': 1, 'language': language, 'theme': dark, 'textScale': factor, 'browserVersion': browser.version, **geometry(page)}
                            check_geometry(record)
                            assert abs(record['titleSize'] - 20 * factor) < .01, record
                            assert page.locator('section.field-group').count() == 3
                            expect(page.locator('#input-summary')).to_have_text('3 个文件，27.4 MB' if language == 'zh-CN' else '3 files, 27.4 MB')
                            page.locator('.remove-input').first.focus()
                            page.keyboard.press('Enter')
                            assert page.locator('.create-row').count() == 2
                            page.locator('#add-folder').click()
                            page.locator('#add-folder').click()
                            assert page.locator('.create-row').count() == 3
                            page.locator('#conflict-policy-trigger').click()
                            page.keyboard.press('End')
                            page.keyboard.press('Enter')
                            assert page.locator('#conflict-policy').evaluate('e=>e.selectedIndex') == 3
                            while page.locator('.remove-input').count():
                                page.locator('.remove-input').first.click()
                            expect(page.locator('#confirm-add')).to_be_disabled()
                            page.locator('#add-files').click()
                            expect(page.locator('#confirm-add')).to_be_enabled()
                            page.locator('#cancel-add').click()
                            if dpr == 1:
                                page.goto(f'{url}?export&lang={language}&theme={"dark" if dark else "light"}&textScale={factor}&long')
                                page.evaluate('document.fonts.ready')
                                long_record = geometry(page)
                                check_geometry(long_record)
                                page.locator('#add-body').evaluate('e=>e.scrollTop=e.scrollHeight')
                                page.locator('#add-dialog').screenshot(path=str(output / (tag + '.png')))
                                record['longNames'] = long_record
                            assert not errors, errors
                            records.append(record)
                page.close()
        finally:
            browser.close()
    return records


def native_zoom_cases(output):
    from native_browser_zoom import NativeZoom, isolated_display
    from playwright.sync_api import sync_playwright
    records = []
    with server_url() as url, isolated_display() as display, sync_playwright() as p:
        keyboard = NativeZoom(display)
        try:
            for dpr in [1, 1.25, 2]:
                with tempfile.TemporaryDirectory(prefix='xdzip-add-browser-') as profile:
                    context = p.chromium.launch_persistent_context(profile,
                        executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
                        headless=False, no_viewport=True, env={**os.environ, 'DISPLAY': display},
                        args=['--no-sandbox', '--disable-dev-shm-usage', '--window-size=1400,1100',
                              f'--force-device-scale-factor={dpr}'])
                    page = context.pages[0]
                    for zoom in [1, 1.25]:
                        for language in ['zh-CN', 'en-US']:
                            for theme in ['light', 'dark']:
                                page.goto(f'{url}?export&long&lang={language}&theme={theme}&textScale=1.5')
                                keyboard.set_zoom(page, dpr, zoom)
                                page.evaluate('document.fonts.ready')
                                record = {'dpr': dpr, 'browserZoom': zoom, 'language': language, 'theme': theme, 'textScale': 1.5, 'effectiveDpr': page.evaluate('devicePixelRatio'), 'visualViewportScale': page.evaluate('visualViewport.scale'), **geometry(page)}
                                check_geometry(record)
                                records.append(record)
                    context.close()
        finally:
            keyboard.close()
    return records


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-input', type=Path)
    parser.add_argument('--html', action='store_true')
    parser.add_argument('--native-zoom', action='store_true')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert args.qt_input or args.html, 'Select Qt capture checks or HTML checks'
    assert hashlib.sha256((ROOT / REFERENCE['source']).read_bytes()).hexdigest() == REFERENCE['sha256']
    args.output.mkdir(parents=True, exist_ok=True)
    qt = qt_cases(args.qt_input) if args.qt_input else []
    html = html_cases(args.output) if args.html else []
    native = native_zoom_cases(args.output) if args.native_zoom else []
    (args.output / 'report.json').write_text(json.dumps({'passed': True, 'qt': qt, 'html': html, 'nativeZoom': native,
        'source': REFERENCE['source'], 'source_sha256': REFERENCE['sha256'],
        'limits': ['Qt offscreen captures do not approve native OS decorations or manual desktop interaction.',
                   'Actual browser page zoom is verified only in nativeZoom cases; headless DPR does not substitute for it.']},
        ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'Addition dialog: {len(qt)} Qt captures, {len(html)} HTML cases, {len(native)} real browser zoom cases passed')


if __name__ == '__main__':
    main()
