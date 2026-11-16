#!/usr/bin/env python3
"""Check the changed creation profiles in the HTML reference; no product I/O is performed."""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright, expect


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--browser', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    output = args.output.resolve()
    assert output.is_relative_to(root/'build') and output != root/'build'
    output.mkdir(parents=True, exist_ok=True)
    class Handler(SimpleHTTPRequestHandler):
        def log_message(self, *_):
            pass
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Handler, directory=str(root)))
    Thread(target=server.serve_forever, daemon=True).start()
    checks, errors = [], []
    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path=str(args.browser), headless=True)
            page = browser.new_page(viewport={'width': 1280, 'height': 1024})
            page.on('pageerror', lambda error: errors.append(str(error)))
            page.goto(f'http://127.0.0.1:{server.server_port}/design/html/canvas.html?view=create')
            for profile, suffix, level in [('iso', '.iso', '0'), ('zip-sfx', '.sfx.exe', '2'), ('zip', '.zip', '2')]:
                page.locator('#create-format-trigger').click()
                page.get_by_role('option', name={'iso':'ISO','zip-sfx':'ZIP SFX','zip':'ZIP'}[profile], exact=True).click()
                expect(page.locator('#create-name')).to_have_value('旅行资料' + suffix)
                expect(page.locator('#create-level')).to_have_value(level)
                if profile != 'zip':
                    expect(page.locator('#create-use-password')).to_be_disabled()
                    expect(page.locator('#create-volume')).to_be_disabled()
                if profile == 'iso':
                    expect(page.locator('#create-level')).to_be_disabled()
                    expect(page.locator('#create-volume-note')).to_contain_text('Joliet')
                if profile == 'zip-sfx':
                    expect(page.locator('#create-volume-note')).to_contain_text('绝不自动运行')
                checks.append(profile)
            assert not errors, errors
            browser.close()
    finally:
        server.shutdown()
        server.server_close()
    (output/'receipt.json').write_text(json.dumps({'status':'passed','checks':checks,'scope':'HTML design interactions only'}, indent=2)+'\n', encoding='utf-8')
    print('Creation HTML profiles: ISO, Windows SFX and switching back to ZIP passed')


if __name__ == '__main__':
    main()
