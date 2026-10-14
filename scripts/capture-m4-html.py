#!/usr/bin/env python3
"""Capture reviewed creation/progress HTML at logical 100%; keep older evidence intact."""
from functools import partial
from hashlib import sha256
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import subprocess
from threading import Thread
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass

def main():
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
    worker = Thread(target=server.serve_forever, daemon=True)
    worker.start()
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(executable_path='/usr/bin/chromium', args=['--no-sandbox','--disable-dev-shm-usage'])
            page = browser.new_page(viewport={'width':1800,'height':1800}, device_scale_factor=1)
            for view, directory in [('create','m4'),('progress','progress-density')]:
                output = ROOT / 'docs/qa' / directory / 'html'
                output.mkdir(parents=True, exist_ok=True)
                page.goto(f'http://127.0.0.1:{server.server_port}/design/html/index.html?view={view}&zoom=1')
                frame = page.frame_locator('#design-frame')
                modal = frame.locator('dialog[open]')
                expect(modal).to_have_attribute('data-kind', view)
                frame.locator('body').evaluate('()=>{clearInterval(timer);return document.fonts.ready;}')
                expect(page.locator('#status')).to_contain_text('100% 设计尺寸')
                path = output / (view+'.png')
                modal.screenshot(path=str(path))
                receipt = dict(base_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                               source='Current working-tree HTML, per-view logical 100%, Chromium/Linux/DPR1',
                               view=view, logical_bounds=modal.bounding_box(), sha256=sha256(path.read_bytes()).hexdigest(),
                               processing='Native browser capture; no resampling or postprocessing')
                (output/'receipt.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
            browser.close()
    finally:
        server.shutdown(); server.server_close(); worker.join()

if __name__ == '__main__':
    main()
