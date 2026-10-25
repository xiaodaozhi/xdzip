#!/usr/bin/env python3
"""Exercise creation controls at real 100/125% browser zoom and DPR 1/1.25/2."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import tempfile
from threading import Thread
from playwright.sync_api import sync_playwright, expect
from native_browser_zoom import isolated_display, NativeZoom

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT/'build/create-style/html'
class Quiet(SimpleHTTPRequestHandler):
    def log_message(self,*args): pass

def main():
    OUTPUT.mkdir(parents=True,exist_ok=True)
    server = ThreadingHTTPServer(('127.0.0.1',0),partial(Quiet,directory=str(ROOT)))
    Thread(target=server.serve_forever,daemon=True).start()
    checks,envs,errors,external = [],[],[],[]
    reference = json.loads((ROOT/'docs/visual/create-reference.json').read_text(encoding="utf-8"))
    def check(name,actual,expected,tolerance=1):
        passed = abs(actual-expected)<=tolerance if isinstance(actual,(int,float)) and isinstance(expected,(int,float)) else actual==expected
        checks.append(dict(name=name,actual=actual,expected=expected,passed=passed))
    try:
        with isolated_display() as display, sync_playwright() as p:
            keyboard = NativeZoom(display)
            for dpr in [1,1.25,2]:
                with tempfile.TemporaryDirectory(prefix='xdzip-create-browser-') as profile:
                    browser = p.chromium.launch_persistent_context(profile,executable_path=os.environ.get('XDZIP_CHROMIUM','/usr/bin/chromium'),headless=False,no_viewport=True,env={**os.environ,'DISPLAY':display},args=['--no-sandbox','--disable-dev-shm-usage','--window-size=1100,1300',f'--force-device-scale-factor={dpr}'])
                    browser.route('**/*',lambda r:r.continue_() if r.request.url.startswith(f'http://127.0.0.1:{server.server_port}/') else (external.append(r.request.url),r.abort()))
                    page = browser.pages[0]
                    page.on('pageerror',lambda e:errors.append(str(e)))
                    for zoom in [1,1.25]:
                        page.goto(f'http://127.0.0.1:{server.server_port}/design/html/index.html?view=create&zoom=1')
                        frame = page.frame_locator('#design-frame')
                        expect(frame.locator('#create-size')).to_have_text(reference['summary_fixture']['display'])
                        child = next(f for f in page.frames if f.parent_frame)
                        child.evaluate('document.fonts.ready')
                        keyboard.set_zoom(page,dpr,zoom)
                        tag=f'DPR{dpr}-page{zoom}'
                        envs.append(dict(case=tag,base_dpr=dpr,page_zoom=zoom,effective_dpr=page.evaluate('devicePixelRatio'),review_zoom=page.locator('#zoom').input_value(),pinch_zoom=page.evaluate('visualViewport.scale')))
                        check(tag+'/DPR',envs[-1]['effective_dpr'],dpr*zoom,.001)
                        modal=frame.locator('dialog[open]')
                        check(tag+'/logical dialog width',modal.bounding_box()['width'],reference['logical_width'])
                        field=frame.locator('#create-level-trigger');glyph=field.locator('svg')
                        check(tag+'/chevron right inset',field.bounding_box()['x']+field.bounding_box()['width']-glyph.bounding_box()['x']-glyph.bounding_box()['width'],6,.15)
                        check(tag+'/chevron vertical center',field.bounding_box()['y']+field.bounding_box()['height']/2,glyph.bounding_box()['y']+glyph.bounding_box()['height']/2,.1)
                        for selector in ['#create-name','#create-destination','#create-level-trigger','#create-volume-trigger']:
                            check(tag+selector+'/height',frame.locator(selector).bounding_box()['height'],28)
                        radios=frame.locator('#create-format [role=radio]')
                        radios.nth(0).focus();page.keyboard.press('ArrowRight')
                        expect(radios.nth(1)).to_be_focused();expect(radios.nth(1)).to_have_attribute('aria-checked','true')
                        expect(frame.locator('#create-name')).to_have_value('旅行资料.tar')
                        expect(frame.locator('#create-level-trigger')).to_be_disabled()
                        expect(frame.locator('#create-volume-trigger')).to_be_disabled()
                        page.keyboard.press('ArrowRight')
                        expect(radios.nth(2)).to_have_attribute('aria-checked','true')
                        expect(frame.locator('#create-name')).to_have_value('旅行资料.tar.gz')
                        expect(frame.locator('#create-level-trigger')).to_be_enabled()
                        expect(frame.locator('#create-use-password')).to_be_disabled()
                        page.keyboard.press('ArrowRight')
                        expect(radios.nth(3)).to_have_attribute('aria-checked','true')
                        expect(frame.locator('#create-name')).to_have_value('旅行资料.7z')
                        expect(frame.locator('#create-volume-trigger')).to_be_enabled()
                        expect(frame.locator('#create-use-password')).to_be_enabled()
                        expect(frame.locator('#create-solid')).not_to_be_checked()
                        expect(frame.locator('#create-encrypt-header')).to_be_disabled()
                        frame.locator('#create-use-password').check()
                        expect(frame.locator('#create-encrypt-header')).to_be_enabled()
                        frame.locator('#create-encrypt-header').check()
                        frame.locator('#create-use-password').uncheck()
                        expect(frame.locator('#create-encrypt-header')).not_to_be_checked()
                        radios.nth(0).click()
                        ratio = child.evaluate('XDZIP_DISPLAY_SPEC.views.create.sourceToLogical')
                        for i, expected in enumerate([(4, 0, 0, 4), (0, 0, 0, 0), (0, 0, 0, 0), (0, 4, 4, 0)]):
                            corners = radios.nth(i).evaluate('n=>{const s=getComputedStyle(n);return [s.borderTopLeftRadius,s.borderTopRightRadius,s.borderBottomRightRadius,s.borderBottomLeftRadius].map(parseFloat)}')
                            for corner, (actual, wanted) in enumerate(zip(corners, expected)):
                                check(tag+f'/segment{i}/corner{corner}', actual * ratio, wanted, .15)
                        # bounding_box includes the review iframe's source-to-logical transform.
                        # Unlike computed CSS corner radii above, these distances are already logical.
                        group_box = frame.locator('#create-format').bounding_box()
                        first_box, last_box = radios.first.bounding_box(), radios.last.bounding_box()
                        check(tag+'/segment outer left inset', first_box['x']-group_box['x'], 1, .15)
                        check(tag+'/segment outer right inset', group_box['x']+group_box['width']-last_box['x']-last_box['width'], 1, .15)
                        check(tag+'/segment outer top inset', first_box['y']-group_box['y'], 1, .15)
                        check(tag+'/segment outer bottom inset', group_box['y']+group_box['height']-first_box['y']-first_box['height'], 1, .15)
                        selected = radios.nth(0)
                        selected.hover()
                        def painted_background():
                            return selected.evaluate('n=>{const c=document.createElement("canvas");c.width=c.height=1;const p=c.getContext("2d");p.fillStyle=getComputedStyle(n).backgroundColor;p.fillRect(0,0,1,1);return [...p.getImageData(0,0,1,1).data].slice(0,3)}')
                        hover = painted_background()
                        check(tag+'/checked segment hover color',max(abs(a-b) for a,b in zip(hover,[18,75,224])),0,1)
                        page.mouse.down()
                        pressed = painted_background()
                        check(tag+'/checked segment pressed color',max(abs(a-b) for a,b in zip(pressed,[11,56,181])),0,1)
                        page.mouse.up()
                        if dpr==1 and zoom==1:
                            modal.screenshot(path=str(OUTPUT/'create-segment-hover.png'))
                        frame.locator('h1').hover()
                        child.evaluate('()=>{document.activeElement.blur();document.querySelector("#create-name").value="北境旅行.zip";}')
                        if dpr==1 and zoom==1:
                            modal.screenshot(path=str(OUTPUT/'create-light.png'))
                            page.locator('#theme').select_option('dark')
                            expect(frame.locator('html')).to_have_attribute('data-theme','dark')
                            modal.screenshot(path=str(OUTPUT/'create-dark.png'))
                            page.locator('#theme').select_option('light')
                            field.click();expect(frame.locator('[data-command=create-level-0]')).to_be_visible()
                            modal.screenshot(path=str(OUTPUT/'create-dropdown.png'))
                            page.keyboard.press('Escape')
                        field.click();page.keyboard.press('End');page.keyboard.press('Enter')
                        expect(frame.locator('#create-level')).to_have_value('3')
                        expect(frame.locator('#create-level-trigger')).to_be_enabled()
                        frame.locator('#create-list [data-remove="0"]').click()
                        expect(frame.locator('#create-size')).to_have_text('2 个文件，19.0 MB')
                        check(tag+'/keyboard, disabled group, remove',True,True)
                    browser.close()
    finally: server.shutdown();server.server_close()
    report=dict(checks=checks,environments=envs,page_errors=errors,external_requests=external,passed=sum(c['passed'] for c in checks),failed=sum(not c['passed'] for c in checks))
    (OUTPUT/'validation.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n', encoding='utf-8')
    for c in checks:
        if not c['passed']:print(json.dumps(c,ensure_ascii=False))
    print(f"Native browser creation review: {report['passed']} passed, {report['failed']} failed")
    raise SystemExit(bool(report['failed'] or errors or external))
if __name__=='__main__':main()
