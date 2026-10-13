#!/usr/bin/env python3
"""Capture the canonical HTML reviewer at logical 100%, without image resampling."""
from functools import partial
from hashlib import sha256
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import subprocess
from threading import Thread

from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'docs/qa/m3/html'


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
    worker = Thread(target=server.serve_forever, daemon=True)
    worker.start()
    captures = []
    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path='/usr/bin/chromium',
                                                args=['--no-sandbox', '--disable-dev-shm-usage'])
            page = browser.new_page(viewport={'width': 1800, 'height': 1800}, device_scale_factor=1)
            for view in ['extract', 'progress', 'conflict']:
                page.goto(f'http://127.0.0.1:{server.server_port}/design/html/index.html?view={view}&zoom=1')
                frame = page.frame_locator('#design-frame')
                modal = frame.locator('dialog[open]')
                expect(modal).to_be_visible()
                if view == 'conflict':
                    # The public conflict scenario first asks for extraction intent.
                    # Freeze its illustrative task explicitly before capturing it.
                    expect(modal).to_have_attribute('data-kind', 'extract')
                    frame.locator('body').evaluate('() => { XdzipDesign.open("progress"); clearInterval(timer); state.task.status="conflict"; showConflict(); }')
                expect(modal).to_have_attribute('data-kind', view)
                frame.locator('body').evaluate('() => document.fonts.ready')
                expect(page.locator('#status')).to_contain_text('100% 设计尺寸')
                # The reviewer applies the canonical outer scale; screenshot pixels
                # therefore equal logical pixels at DPR 1, without postprocessing.
                path = OUTPUT / f'{view}.png'
                modal.screenshot(path=str(path))
                captures.append({'view': view, 'file': path.name,
                                 'logical_bounds': modal.bounding_box(),
                                 'sha256': sha256(path.read_bytes()).hexdigest()})
            browser.close()
    finally:
        server.shutdown()
        server.server_close()
        worker.join()
    receipt = {'source_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
               'environment': 'Chromium / Linux, DPR 1, canonical HTML reviewer logical 100%',
               'processing': 'Native screenshot only; no resampling, crops or compositing after capture',
               'captures': captures}
    (OUTPUT / 'receipt.json').write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(receipt, ensure_ascii=False))


if __name__ == '__main__':
    main()
