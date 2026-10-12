#!/usr/bin/env python3
"""Verify reference images in served, offline, and opaque-origin HTML reviews."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Thread
import json
import os
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'dist/html-reference-review'
OUTPUT.mkdir(parents=True, exist_ok=True)

class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass

server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
Thread(target=server.serve_forever, daemon=True).start()
checks, errors = [], []
try:
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(
            executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
            args=['--no-sandbox', '--disable-dev-shm-usage'])
        for mode in ['served', 'offline', 'opaque-origin']:
            page = browser.new_page(viewport={'width': 1200, 'height': 1000})
            page.on('pageerror', lambda error: errors.append(str(error)))
            if mode == 'served':
                page.goto(f'http://127.0.0.1:{server.server_port}/design/html/index.html')
            else:
                page.route('**/*', lambda route: route.abort())
                html = (ROOT / 'dist/xdzip-hifi-design.html').read_text()
                if mode == 'opaque-origin':
                    html = html.replace('<iframe id="design-frame"', '<iframe sandbox="allow-scripts" id="design-frame"')
                page.set_content(html)
            page.click('#reference-toggle')
            for view in ['main', 'extract', 'create', 'progress', 'conflict', 'settings']:
                page.select_option('#view', view)
                page.wait_for_function('''() => {
                    const image = document.querySelector('#reference-image');
                    return !image.hidden && image.complete && image.naturalWidth > 0;
                }''')
                checks.append(f'{mode}: {view} reference decoded')
            for view in ['empty', 'damaged']:
                page.select_option('#view', view)
                expect(page.locator('#reference-image')).to_be_hidden()
                expect(page.locator('#reference-message')).to_contain_text('暂无')
                checks.append(f'{mode}: {view} explicitly has no reference')
            page.select_option('#view', 'extract')
            expect(page.frame_locator('#design-frame').locator('dialog[open]')).to_be_visible()
            checks.append(f'{mode}: review configures the canvas')
            if mode == 'offline':
                page.locator('#reference').screenshot(path=str(OUTPUT / 'reference-extract.png'))
            page.close()
        assert not errors, errors
        browser.close()
finally:
    server.shutdown()
    (OUTPUT / 'validation.json').write_text(json.dumps(
        {'checks': checks, 'page_errors': errors}, ensure_ascii=False, indent=2) + '\n')
print(f'{len(checks)} reference checks passed')
