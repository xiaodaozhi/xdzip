# scripts/export-settings-design.py

"""Export the editable settings artboard; frozen geometry references stay unchanged."""
import argparse
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler
from html_review_server import ReviewHTTPServer
import json
import os
from pathlib import Path
import shutil
from threading import Thread
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write-design', action='store_true', help='Copy authored artboards to hifi/screens')
    args = parser.parse_args()
    output = ROOT/'build/review/settings-session-design'
    output.mkdir(parents=True, exist_ok=True)
    server = ReviewHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
    Thread(target=server.serve_forever, daemon=True).start()
    spec = json.loads((ROOT/'docs/visual/settings-reference.json').read_text())
    records = []
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM'))
            try:
                for theme in ['light', 'dark']:
                    for tab in ['appearance', 'general', 'privacy', 'associations', 'network', 'about']:
                        page = browser.new_page(viewport=dict(zip(['width', 'height'], spec['authoring_viewport'])),
                                                device_scale_factor=1)
                        page.goto(f'http://127.0.0.1:{server.server_port}/design/hifi/sources/settings.html?export=1&theme={theme}&tab={tab}')
                        frame = page.frame_locator('iframe')
                        modal = frame.locator('#modal[data-kind=settings]')
                        modal.wait_for(state='visible')
                        frame.locator(f'#settings-tab-{tab}[aria-selected="true"]').wait_for(state='visible')
                        page.frames[1].evaluate('document.fonts.ready')
                        page.frames[1].wait_for_function('(theme)=>document.documentElement.dataset.theme===theme', arg=theme)
                        image = output/f'settings-{tab}-{theme}.png'
                        modal.screenshot(path=str(image))
                        records.append(dict(file=image.name, sha256=hashlib.sha256(image.read_bytes()).hexdigest(),
                                            bounds=modal.bounding_box(), theme=theme, tab=tab, browser=browser.version))
                        if tab == 'appearance':
                            page.goto(f'http://127.0.0.1:{server.server_port}/design/hifi/sources/settings.html?export=1&theme={theme}&hover=general')
                            frame = page.frame_locator('iframe')
                            modal = frame.locator('#modal[data-kind=settings]')
                            frame.locator('#settings-tab-general[data-preview-hover="true"]').wait_for(state='visible')
                            page.frames[1].evaluate('document.fonts.ready')
                            page.frames[1].wait_for_function('(theme)=>document.documentElement.dataset.theme===theme', arg=theme)
                            image = output/f'settings-sidebar-hover-{theme}.png'
                            modal.screenshot(path=str(image))
                            records.append(dict(file=image.name, sha256=hashlib.sha256(image.read_bytes()).hexdigest(),
                                                bounds=modal.bounding_box(), theme=theme, tab='sidebar-hover', browser=browser.version))
                        if tab == 'network':
                            frame.locator('input[name=setting-proxy][value=custom]').check()
                            frame.locator('#setting-proxy-address').wait_for(state='visible')
                            image = output/f'settings-network-custom-{theme}.png'
                            modal.screenshot(path=str(image))
                            records.append(dict(file=image.name, sha256=hashlib.sha256(image.read_bytes()).hexdigest(),
                                                bounds=modal.bounding_box(), theme=theme, tab='network-custom', browser=browser.version))
                        if tab == 'about':
                            frame.locator('.settings-pane').evaluate('(element) => element.scrollTop = element.scrollHeight')
                            image = output/f'settings-about-details-{theme}.png'
                            modal.screenshot(path=str(image))
                            records.append(dict(file=image.name, sha256=hashlib.sha256(image.read_bytes()).hexdigest(),
                                                bounds=modal.bounding_box(), theme=theme, tab='about-details', browser=browser.version))
                            page.goto(f'http://127.0.0.1:{server.server_port}/design/hifi/sources/settings.html?export=1&theme={theme}&tab=about&update=checking')
                            frame = page.frame_locator('iframe')
                            modal = frame.locator('#modal[data-kind=settings]')
                            frame.locator('#about-cancel-check').wait_for(state='visible')
                            page.frames[1].evaluate('document.fonts.ready')
                            page.frames[1].wait_for_function('(theme)=>document.documentElement.dataset.theme===theme', arg=theme)
                            image = output/f'settings-about-checking-{theme}.png'
                            modal.screenshot(path=str(image))
                            records.append(dict(file=image.name, sha256=hashlib.sha256(image.read_bytes()).hexdigest(),
                                                bounds=modal.bounding_box(), theme=theme, tab='about-checking', browser=browser.version))
                        page.close()
            finally:
                browser.close()
    finally:
        server.shutdown()
        server.server_close()
    if args.write_design:
        for record in records:
            shutil.copy2(output/record['file'], ROOT/'design/hifi/screens'/record['file'])
    (output/'export.json').write_text(json.dumps(dict(role='code-authored-design', source='design/hifi/sources/settings.html',
        device_pixel_ratio=1, browser_zoom=1, images=records,
        limits='Design exports are not native visual acceptance or independent geometry references.'), indent=2)+'\n')
    print('Exported six settings pages in light and dark themes')


if __name__ == '__main__': main()
