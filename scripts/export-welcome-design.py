# scripts/export-welcome-design.py

"""Export top-aligned W01 design states without replacing historical references."""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
SCENES = [('welcome-top-light.png', 'theme=light'),
          ('welcome-top-dark.png', 'theme=dark'),
          ('welcome-top-narrow.png', 'narrow=1'),
          ('welcome-top-error.png', 'error=1')]


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write-design', action='store_true')
    args = parser.parse_args()
    output = ROOT/'build/review/welcome-design'
    output.mkdir(parents=True, exist_ok=True)
    target = ROOT/'design/hifi/screens' if args.write_design else output
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
    Thread(target=server.serve_forever, daemon=True).start()
    spec = json.loads((ROOT/'docs/visual/welcome-reference.json').read_text(encoding='utf-8'))
    checks, errors = [], []
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM'), headless=True)
            try:
                page = browser.new_page(viewport={'width':1600,'height':1050}, device_scale_factor=1)
                page.on('pageerror', lambda error: errors.append(str(error)))
                for filename, query in SCENES:
                    page.goto(f'http://127.0.0.1:{server.server_port}/design/hifi/sources/welcome.html?export=1&'+query)
                    frame = page.frames[1]
                    expect(frame.locator('#welcome')).to_be_visible()
                    frame.evaluate('document.fonts.ready')
                    if 'error=1' in query:
                        expect(frame.locator('.welcome-feedback')).to_be_visible()
                    metrics = frame.evaluate('''()=>{const f=parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--ui-source-factor')),body=document.querySelector('#welcome').getBoundingClientRect(),content=document.querySelector('.welcome-content').getBoundingClientRect();return {top:(content.top-body.top)/f,title:document.querySelector('.titlebar').getBoundingClientRect().height/f}}''')
                    assert abs(metrics['top']-spec['content_top_inset']) <= .1, metrics
                    assert abs(metrics['title']-spec['titlebar_height']) <= .1, metrics
                    frame.evaluate('Promise.all([...document.images].map(image=>image.decode()))')
                    page.locator('iframe').screenshot(path=str(target/filename))
                    checks.append(dict(file=filename,metrics=metrics))
                assert not errors, errors
            finally:
                browser.close()
    finally:
        server.shutdown()
    (output/'validation.json').write_text(json.dumps(dict(checks=checks,errors=errors,
        limits='Editable HTML design exports at DPR 1; not native desktop acceptance.'),indent=2)+'\n',encoding='utf-8')
    print(f'Welcome design: {len(checks)} top-aligned states passed')


if __name__ == '__main__':
    main()
