#!/usr/bin/env python3
"""Export current editable design artboards, preserving frozen raster measurements."""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler
from html_review_server import ReviewHTTPServer
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
from threading import Thread

from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
ARTBOARDS = (
    ('main-capsule-light.png', 'theme=light'),
    ('main-capsule-dark.png', 'theme=dark'),
    ('directory-tree-capsule.png', 'theme=light&tree=expanded'),
    ('tabs-overflow-light.png', 'theme=light&tabs=overflow'),
    ('tabs-overflow-dark.png', 'theme=dark&tabs=overflow'),
)


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write-design', action='store_true', help='Export current PNG design sources to hifi/screens')
    args = parser.parse_args()
    output = ROOT/'build/review/hifi-titlebars'
    output.mkdir(parents=True, exist_ok=True)
    target = ROOT/'design/hifi/screens' if args.write_design else output
    spec = json.loads((ROOT/'docs/visual/layout-scale.json').read_text())
    factor = spec['main']['source_to_logical'][0]/spec['main']['source_to_logical'][1]
    server = ReviewHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
    Thread(target=server.serve_forever, daemon=True).start()
    checks, errors, requests = [], [], []
    environment = {'platform': platform.system(), 'backend': 'headless Chromium',
                   'source': 'design/hifi/sources/main.html',
                   'source_sha256': hashlib.sha256((ROOT/'design/hifi/sources/main.html').read_bytes()).hexdigest(),
                   'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                   'logical_window': spec['main']['logical_window']}

    def close(actual, expected):
        assert abs(actual-expected) <= .1, (actual, expected)

    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM'), headless=True)
            environment['browser_version'] = browser.version
            try:
                for dpr in spec['verification']['device_pixel_ratio']:
                    page = browser.new_page(viewport={'width': 1600, 'height': 1050}, device_scale_factor=dpr)
                    page.on('pageerror', lambda error: errors.append(str(error)))
                    base = f'http://127.0.0.1:{server.server_port}/'
                    page.route('**/*', lambda route: route.continue_() if route.request.url.startswith(base) else (requests.append(route.request.url), route.abort()))
                    for name, query in ARTBOARDS:
                        page.goto(base+'design/hifi/sources/main.html?export=1&'+query)
                        page.wait_for_function('document.documentElement.dataset.ready === "true"')
                        frame = page.frames[1]
                        title = frame.locator('.titlebar').bounding_box()
                        tabs = frame.locator('.archive-tab')
                        title_spec = spec['main']['titlebar']
                        close(title['height']*factor, title_spec['height'])
                        for tab in tabs.all():
                            rect = tab.bounding_box()
                            close(rect['height']*factor, title_spec['tab_height'])
                            close((rect['y']-title['y'])*factor, title_spec['tab_vertical_inset'])
                            close((title['y']+title['height']-rect['y']-rect['height'])*factor, title_spec['tab_vertical_inset'])
                            close(tab.evaluate('el=>parseFloat(getComputedStyle(el).borderRadius)')*factor, title_spec['tab_radius'])
                        active = tabs.filter(has=frame.locator('#close-archive'))
                        assert active.evaluate('el=>parseFloat(getComputedStyle(el).borderBottomWidth)') > 0
                        if 'tree=expanded' in query:
                            expect(frame.locator('[data-directory="photos"]')).to_have_attribute('aria-expanded', 'true')
                            expect(frame.locator('[data-directory="ice"]')).to_have_attribute('aria-expanded', 'true')
                            expect(frame.locator('[data-directory="shore"]')).to_be_visible()
                        page.mouse.move(0, 0)
                        if dpr == 1:
                            page.locator('iframe').screenshot(path=str(target/name))
                        table_font = frame.locator('#file-rows [data-id=lake] .entry-name').evaluate('el=>parseFloat(getComputedStyle(el).fontSize)')*factor
                        frame.locator('#more-actions').click()
                        for kind in ['dropdown', 'submenu', 'context']:
                            if kind == 'submenu':
                                page.keyboard.press('Escape')
                                frame.locator('#file-rows [data-id=lake]').click(button='right')
                                frame.locator('[data-command=view-menu]').focus()
                                page.keyboard.press('ArrowRight')
                                expect(frame.locator('.ui-menu')).to_have_count(2)
                            elif kind == 'context':
                                page.keyboard.press('Escape')
                                page.keyboard.press('Escape')
                                frame.locator('#file-rows [data-id=lake]').click(button='right')
                            menu = frame.locator('.ui-menu').last
                            expect(menu).to_be_visible()
                            menu_font = menu.evaluate('el=>parseFloat(getComputedStyle(el).fontSize)')*factor
                            close(menu.bounding_box()['width']*factor, spec['controls']['menu_width'])
                            close(menu_font, spec['controls']['menu_font_size'])
                            low, high = spec['verification']['menu_to_table_font_ratio']
                            assert low <= menu_font/table_font <= high
                            checks.append({'artboard': name, 'dpr': dpr, 'menu': kind,
                                           'title_height': title['height']*factor,
                                           'tab_height': tabs.first.bounding_box()['height']*factor,
                                           'menu_width': menu.bounding_box()['width']*factor,
                                           'menu_table_font_ratio': menu_font/table_font})
                    page.close()
                assert not errors and not requests, (errors, requests)
            finally:
                browser.close()
    finally:
        server.shutdown()
        (output/'validation.json').write_text(json.dumps({'checks': checks, 'page_errors': errors,
            'external_requests': requests, 'environment': environment, 'browser_zoom': 1, 'review_zoom': 1,
            'limits': 'Editable HTML design exports only. No native Qt/decorations or actual 125% browser page zoom tested.'}, indent=2)+'\n')
    print(f'{len(checks)} artboard/titlebar/menu checks passed; exports in {target.relative_to(ROOT)}')


if __name__ == '__main__':
    main()
