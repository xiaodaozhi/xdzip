#!/usr/bin/env python3
"""Exercise About within settings, including update state fixtures."""
from functools import partial
from http.server import SimpleHTTPRequestHandler
from html_review_server import ReviewHTTPServer
from pathlib import Path
from threading import Thread
import json
import os
from playwright.sync_api import sync_playwright, expect
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'build/review/about/html'
OUT.mkdir(parents=True, exist_ok=True)


class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *_):
        pass


def main():
    server = ReviewHTTPServer(('127.0.0.1', 0), partial(Quiet, directory=str(ROOT)))
    Thread(target=server.serve_forever, daemon=True).start()
    states = ['idle', 'checking', 'up_to_date', 'available', 'downloading', 'verifying',
              'downloaded', 'waiting_tasks', 'preparing', 'installing', 'error',
              'recovery_required', 'updated', 'unavailable']
    records, errors = [], []
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM'), args=['--no-sandbox'])
            try:
                for language in ('zh-CN', 'en-US'):
                    for dark in (False, True):
                        page = browser.new_page(viewport={'width': 1600, 'height': 1500}, permissions=['clipboard-read', 'clipboard-write'])
                        page.on('pageerror', lambda error: errors.append(str(error)))
                        page.add_init_script('''Object.defineProperty(navigator, "clipboard", {
                            value: { writeText: async value => { window.__copiedText = value; } }
                        });''')
                        page.goto(f'http://127.0.0.1:{server.server_port}/design/html/canvas.html?view=about&lang={language}&standalone=1')
                        page.evaluate('document.fonts.ready')
                        page.evaluate('dark => { XdzipDesign.theme(dark ? "dark" : "light"); XdzipDesign.open("about"); }', dark)
                        modal = page.locator('#modal')
                        expect(modal).to_have_attribute('data-kind', 'settings')
                        expect(modal.locator('.settings-pane')).to_have_attribute('data-tab', 'about')
                        expect(modal.locator('.modal-footer')).to_have_count(0)
                        expect(modal.locator('.settings-about-brand')).to_contain_text(page.evaluate('XDZIP_VERSION.display_version'))
                        page.locator('#about-copy').click()
                        expect(page.locator('#about-copy')).to_have_text('已复制' if language == 'zh-CN' else 'Copied')
                        clipboard = json.loads(page.evaluate('window.__copiedText'))
                        assert clipboard['version'] == page.evaluate('XDZIP_VERSION.display_version') and 'path' not in clipboard
                        for state in states:
                            page.evaluate('state => XdzipAboutUpdatePreview({state,downloaded:state==="downloaded"})', state)
                            status = modal.locator('#about-update-status')
                            assert status.text_content() and not status.text_content().startswith('update.')
                            busy = state in ('checking', 'downloading', 'verifying', 'waiting_tasks', 'preparing', 'installing')
                            if busy: expect(modal.locator('#about-check-update')).to_be_disabled()
                            else: expect(modal.locator('#about-check-update')).to_be_enabled()
                            expect(modal.locator('#about-cancel-update')).to_have_count(int(busy and state != 'installing'))
                            expect(modal.locator('.about-update-section progress')).to_have_count(int(state in ('downloading', 'verifying', 'preparing')))
                            if state in ('available', 'downloaded'):
                                modal.locator('#about-install-update').click()
                                expect(status).to_have_text(page.evaluate('lang=>XDZIP_UI_MESSAGES[lang].messages["update.preview"]', language))
                            records.append({'language': language, 'dark': dark, 'state': state})
                        modal.screenshot(path=str(OUT / f'{language}-{dark}.png'))
                        page.close()
                assert not errors, errors
            finally:
                browser.close()
    finally:
        server.shutdown()
        server.server_close()
    (OUT / 'review.json').write_text(json.dumps({'cases': records, 'limits': 'Update actions and clipboard use read-only HTML fixtures, not native update or desktop acceptance.'}, indent=2) + '\n')
    print(f'About settings: {len(records)} update state and language/theme cases passed')


if __name__ == '__main__':
    main()
