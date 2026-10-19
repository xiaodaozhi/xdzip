#!/usr/bin/env python3
"""Review shared progress composition at real browser zoom and fractional DPR."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import hashlib
import json
import os
from pathlib import Path
import tempfile
from threading import Thread

from playwright.sync_api import sync_playwright, expect
from native_browser_zoom import isolated_display, NativeZoom

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'build/review/integrity-progress'


class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


def main():
    reference = json.loads((ROOT / 'docs/visual/integrity-reference.json').read_text(encoding='utf-8'))
    assert hashlib.sha256((ROOT / reference['source']).read_bytes()).hexdigest() == reference['sha256']
    OUTPUT.mkdir(parents=True, exist_ok=True)
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Quiet, directory=str(ROOT)))
    Thread(target=server.serve_forever, daemon=True).start()
    checks, errors = [], []

    def check(name, actual, expected, tolerance=1):
        passed = abs(actual - expected) <= tolerance
        checks.append(dict(name=name, actual=actual, expected=expected, passed=passed))
        assert passed, checks[-1]

    try:
        with isolated_display() as display, sync_playwright() as p:
            keyboard = NativeZoom(display)
            try:
                for dpr in [1, 1.25, 2]:
                    with tempfile.TemporaryDirectory(prefix='xdzip-integrity-browser-') as profile:
                        browser = p.chromium.launch_persistent_context(
                            profile, executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
                            headless=False, no_viewport=True, env={**os.environ, 'DISPLAY': display},
                            args=['--no-sandbox', '--disable-dev-shm-usage', '--window-size=1300,1300',
                                  f'--force-device-scale-factor={dpr}'])
                        page = browser.pages[0]
                        page.on('pageerror', lambda e: errors.append(str(e)))
                        for zoom in [1, 1.25]:
                            page.goto(f'http://127.0.0.1:{server.server_port}/design/html/index.html?view=archive-test&zoom=1')
                            frame = page.frame_locator('#design-frame')
                            modal = frame.locator('dialog[open]')
                            expect(modal).to_have_attribute('data-kind', 'archive-test')
                            child = next(f for f in page.frames if f.parent_frame)
                            keyboard.set_zoom(page, dpr, zoom)
                            for theme in ['light', 'dark']:
                                child.evaluate('value => XdzipDesign.theme(value)', theme)
                                tag = f'DPR{dpr}/zoom{zoom}/{theme}'
                                ratio = child.evaluate('XDZIP_DISPLAY_SPEC.views["archive-test"].sourceToLogical')
                                check(tag + '/width', modal.bounding_box()['width'], reference['logical_width'])
                                metrics = modal.evaluate('''n => ({
                                    title: parseFloat(getComputedStyle(n.querySelector('h1')).fontSize),
                                    metrics: n.querySelectorAll('.task-metrics>div').length,
                                    nativeBars: n.querySelectorAll('progress').length,
                                    fonts: [...n.querySelectorAll('.modal-footer .button')].map(b => parseFloat(getComputedStyle(b).fontSize)),
                                    hints: [...n.querySelectorAll('.info-line')].map(b => {
                                        const icon = b.querySelector('svg').getBoundingClientRect();
                                        return icon.right + parseFloat(getComputedStyle(b).gap);
                                    })
                                })''')
                                check(tag + '/title', metrics['title'] * ratio, reference['title_size'])
                                check(tag + '/metrics columns', metrics['metrics'], 3, 0)
                                check(tag + '/no native progress bar', metrics['nativeBars'], 0, 0)
                                check(tag + '/hint text alignment', metrics['hints'][0], metrics['hints'][1], .01)
                                for i, size in enumerate(metrics['fonts']):
                                    check(tag + f'/footer font {i}', size * ratio, reference['footer_font_size'], .1)
                                if dpr == 1 and zoom == 1:
                                    modal.screenshot(path=str(OUTPUT / f'integrity-{theme}.png'))
                            # Close preserves the demonstration; cancel is an explicit state transition.
                            frame.locator('#close-archive-test').click()
                            assert child.evaluate('state.testJob.status') == 'running'
                            child.evaluate('XdzipMain.dispatch("tasks")')
                            frame.locator('#cancel-archive-test').click()
                            assert child.evaluate('state.testJob.status') == 'cancelled'
                            expect(modal.locator('h1')).to_have_text('检查已取消')
                            expect(modal.locator('#cancel-archive-test')).to_have_count(0)
                            child.evaluate('XdzipMain.resetJobs(); closeModal()')
                        browser.close()
                assert not errors, errors
            finally:
                keyboard.close()
    finally:
        server.shutdown()
        server.server_close()
    (OUTPUT / 'measurements.json').write_text(json.dumps(dict(checks=checks, page_errors=errors), ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'Integrity progress: {len(checks)} layout checks and six close/cancel flows passed')


if __name__ == '__main__':
    main()
