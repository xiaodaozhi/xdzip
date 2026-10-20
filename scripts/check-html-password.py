#!/usr/bin/env python3
"""Exercise password demo states and measure actual bounded HTML layouts."""
import argparse
import hashlib
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import tempfile
from threading import Thread

from playwright.sync_api import expect, sync_playwright
from native_browser_zoom import NativeZoom, isolated_display

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'build/review/password/html'


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


def native_zoom_cases():
    """Use real browser zoom on a private CI display, never CSS/pinch zoom."""
    records = []
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
    Thread(target=server.serve_forever, daemon=True).start()
    try:
        with isolated_display() as display, sync_playwright() as p:
            keyboard = NativeZoom(display)
            try:
                for dpr in [1, 1.25, 2]:
                    with tempfile.TemporaryDirectory(prefix='xdzip-password-browser-') as profile:
                        context = p.chromium.launch_persistent_context(profile,
                            executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
                            headless=False, no_viewport=True, env={**os.environ, 'DISPLAY': display},
                            args=['--no-sandbox', '--disable-dev-shm-usage', '--window-size=1500,1200',
                                  f'--force-device-scale-factor={dpr}'])
                        page = context.pages[0]
                        for zoom in [1, 1.25]:
                            for language in ['zh-CN', 'en-US']:
                                page.goto(f'http://127.0.0.1:{server.server_port}/design/html/index.html?view=password&zoom=1')
                                keyboard.set_zoom(page, dpr, zoom)
                                child = next(frame for frame in page.frames if frame.parent_frame)
                                child.evaluate('document.fonts.ready')
                                child.evaluate('''language=>{
                                    history.replaceState(null,'','?view=password&lang='+language);
                                    document.documentElement.style.setProperty('--settings-text-scale',1.5);
                                }''', language)
                                for dark in [False, True]:
                                    child.evaluate('''dark=>{XdzipDesign.theme(dark?'dark':'light');showPasswordPrompt(true);}''', dark)
                                    modal = page.frame_locator('#design-frame').locator('#modal')
                                    expect(modal).to_be_visible()
                                    child.evaluate('''()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))''')
                                    # Measure one layout snapshot, inside the frame. CDP's separate
                                    # boxes can straddle a layout update or page-zoom pixel rounding.
                                    # The footer meets the dialog's inner border, not its outside edge.
                                    geometry = modal.evaluate('''el=>{
                                        const box=el.getBoundingClientRect(),footer=el.querySelector('.modal-footer').getBoundingClientRect();
                                        const frame=window.frameElement,scale=frame.getBoundingClientRect().width/frame.offsetWidth;
                                        const border=parseFloat(getComputedStyle(el).borderBottomWidth);
                                        return {width:box.width*scale,footerGap:(box.bottom-border-footer.bottom)*scale,
                                            sourceBottom:box.bottom,footerBottom:footer.bottom,border,frameScale:scale,fonts:document.fonts.status};
                                    }''')
                                    case = {'dpr': dpr, 'browserZoom': zoom, 'language': language,
                                            'dark': dark, 'textScale': 1.5, **geometry,
                                            'effectiveDpr': page.evaluate('devicePixelRatio'),
                                            'reviewZoom': page.locator('#zoom').input_value()}
                                    if abs(geometry['width'] - 491) > 1 or abs(geometry['footerGap']) > 1:
                                        tag = f'zoom-failure-{dpr}-{zoom}-{language}-{dark}'
                                        (OUT / (tag + '.json')).write_text(json.dumps(case, indent=2) + '\n')
                                        modal.screenshot(path=str(OUT / (tag + '.png')))
                                    assert abs(geometry['width'] - 491) <= 1, case
                                    assert abs(geometry['footerGap']) <= 1, case
                                    assert child.locator('#extraction-password').input_value() == ''
                                    records.append(case)
                        context.close()
            finally:
                keyboard.close()
    finally:
        server.shutdown()
    return records


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native-zoom', action='store_true', help='Also check real 100/125% zoom using a private Xvfb display')
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    reference = json.loads((ROOT / 'docs/visual/password-reference.json').read_text())
    source = ROOT / reference['source']
    assert hashlib.sha256(source.read_bytes()).hexdigest() == reference['source_sha256'], 'Password design source changed without a reviewed reference update'
    records, errors = [], []
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'))
        for dpr in [1, 1.25, 2]:
            for language in ['zh-CN', 'en-US']:
                page = browser.new_page(viewport={'width': 1300, 'height': 1100}, device_scale_factor=dpr)
                page.on('pageerror', lambda error: errors.append(str(error)))
                page.goto((ROOT / 'design/html/canvas.html').as_uri() + '?view=password&standalone=1&lang=' + language)
                page.evaluate('document.fonts.ready')
                for dark in [False, True]:
                    for scale in [1, 1.25, 1.5]:
                        for state in ['request', 'retry', 'pending']:
                            page.evaluate('''([dark,scale,state])=>{
                                XdzipDesign.theme(dark?'dark':'light');
                                document.documentElement.style.setProperty('--settings-text-scale',scale);
                                showPasswordPrompt(state==='retry',state==='pending');
                            }''', [dark, scale, state])
                            modal = page.locator('#modal')
                            expect(modal).to_be_visible()
                            m = modal.evaluate('''el=>{
                                const f=Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-source-factor'));
                                const box=el.getBoundingClientRect(),footer=el.querySelector('.modal-footer');
                                return {width:box.width/f,height:(box.height-el.querySelector('.os-titlebar').getBoundingClientRect().height)/f,title:parseFloat(getComputedStyle(el.querySelector('h1')).fontSize)/f,
                                    body:parseFloat(getComputedStyle(el.querySelector('label')).fontSize)/f,note:parseFloat(getComputedStyle(el.querySelector('.password-note')).fontSize)/f,
                                    bodyButton:parseFloat(getComputedStyle(el.querySelector('#password-visibility')).fontSize)/f,
                                    footerBottom:footer.getBoundingClientRect().bottom,boxBottom:box.bottom,
                                    buttons:[...footer.querySelectorAll('button')].map(b=>{const r=b.getBoundingClientRect();return {x:r.x,y:r.y,right:r.right,clipped:b.scrollWidth>b.clientWidth+1||b.scrollHeight>b.clientHeight+1}}),left:box.x,right:box.right};
                            }''')
                            assert abs(m['width'] - 491) <= 1, m
                            assert abs(m['title'] - 20 * scale) <= .1, m
                            assert abs(m['body'] - 13 * scale) <= .1 and abs(m['note'] - 12 * scale) <= .1, m
                            assert abs(m['bodyButton'] - 13 * scale) <= .1 and m['height'] <= 600, m
                            assert abs(m['footerBottom'] - m['boxBottom']) <= 2, m
                            assert all(abs(b['y'] - m['buttons'][0]['y']) <= 1 and not b['clipped']
                                       and b['x'] >= m['left'] and b['right'] <= m['right'] + 1 for b in m['buttons']), m
                            expect(page.locator('#extraction-password')).to_have_attribute('type', 'password')
                            expect(page.locator('#extraction-password')).to_have_value('')
                            assert page.locator('#password-submit').is_disabled() == (state == 'pending')
                            assert page.locator('#password-cancel').is_enabled()
                            assert page.locator('#password-error').is_visible() == (state == 'retry')
                            if dpr == 1:
                                modal.screenshot(path=str(OUT / f'{language}-{dark}-{scale}-{state}.png'))
                            records.append({'dpr': dpr, 'language': language, 'dark': dark, 'scale': scale, 'state': state, 'width': m['width'], 'height': m['height']})
                # Synthetic inputs verify semantics; screenshots never include input values.
                page.evaluate('showPasswordPrompt()')
                field = page.locator('#extraction-password')
                field.fill('  synthetic input  ')
                page.locator('#password-visibility').click()
                expect(field).to_have_attribute('type', 'text')
                page.locator('#password-visibility').click()
                expect(field).to_have_attribute('type', 'password')
                field.fill('文' * 400)
                expect(page.locator('#password-submit')).to_be_disabled()
                field.fill('')
                page.locator('#password-submit').click()
                expect(field).to_have_value('')
                expect(field).to_be_disabled()
                expect(page.locator('#password-submit')).to_be_disabled()
                page.locator('#password-cancel').click()
                expect(page.locator('#modal')).not_to_be_visible()
                page.set_viewport_size({'width': 1000, 'height': 340})
                page.evaluate("showPasswordPrompt(true);document.querySelector('.password-file').textContent='A long synthetic entry name / '.repeat(40)")
                assert page.locator('.password-content').evaluate('el=>el.scrollHeight>el.clientHeight')
                expect(page.locator('#password-cancel')).to_be_in_viewport()
                page.locator('.os-close').click()
                expect(page.locator('#modal')).not_to_be_visible()
                page.close()
        browser.close()
    assert not errors, errors
    native = native_zoom_cases() if args.native_zoom else []
    (OUT / 'report.json').write_text(json.dumps({'passed': True, 'cases': records, 'nativeZoomCases': native,
        'limits': 'Design demo only, baseline browser zoom 100%, DPR 1/1.25/2. Real 100/125% browser zoom is reported only when --native-zoom runs. Empty password fields in all captures. Native titlebar and real decryption are outside this report.'}, indent=2) + '\n')
    print(f'Password HTML: {len(records)} layouts, pending/retry, empty submission, byte limit and cancellation passed')
    if native:
        print(f'Password HTML: {len(native)} actual browser zoom layouts passed')


if __name__ == '__main__':
    main()
