# scripts/check-tab-strip.py

"""Exercise browser-style tab overflow, sorting and loading cancellation in the design."""
from functools import partial
from http.server import SimpleHTTPRequestHandler
from html_review_server import ReviewHTTPServer
import json
import os
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'build/review/tab-strip'


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    server = ReviewHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
    Thread(target=server.serve_forever, daemon=True).start()
    checks, errors = [], []
    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'), headless=True)
            try:
                for dpr in [1, 1.25, 2]:
                    context = browser.new_context(viewport={'width':1600, 'height':1040}, device_scale_factor=dpr)
                    page = context.new_page()
                    page.on('pageerror', lambda error: errors.append(str(error)))
                    for language in ['zh-CN', 'en-US']:
                        for scale in [1, 1.25, 1.5]:
                            page.goto(f'http://127.0.0.1:{server.server_port}/design/html/canvas.html?view=main')
                            page.wait_for_function('window.XdzipMain && document.querySelector(".tab-viewport")')
                            page.evaluate('document.fonts.ready')
                            width = 832 if scale > 1 else 1024
                            page.evaluate('''([language,scale,width,dpr])=>{
                                document.documentElement.lang=language;
                                document.documentElement.style.setProperty('--settings-text-scale',scale);
                                document.body.dataset.platform=dpr===2?'macos':'windows';
                                document.querySelector('.window').style.width=width*parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--ui-source-factor'))+'px';
                                for(let index=0;index<12;index++)XdzipMain.openSample('sample-'+index);
                            }''', [language, scale, width, dpr])
                            expect(page.locator('#tabs-list')).to_be_visible()
                            page.wait_for_function('''()=>{const v=document.querySelector('.tab-viewport'),a=v.querySelector('[aria-selected=true]'),r=v.getBoundingClientRect(),t=a.getBoundingClientRect();return t.left>=r.left-.5&&t.right<=r.right+.5}''')
                            geometry = page.evaluate('''()=>{const v=document.querySelector('.tab-viewport'),b=document.querySelector('#tabs-list').getBoundingClientRect(),s=document.querySelector('#title-settings').getBoundingClientRect();return {viewport:v.clientWidth,content:v.scrollWidth,controlsRight:b.right,settingsLeft:s.left,activeWeight:getComputedStyle(v.querySelector('[aria-selected=true]')).fontWeight,inactiveWeight:getComputedStyle(v.querySelector('.inactive')).fontWeight}}''')
                            assert geometry['content'] > geometry['viewport'] and geometry['controlsRight'] <= geometry['settingsLeft']
                            assert geometry['activeWeight'] == '600' and geometry['inactiveWeight'] == '400', geometry
                            position = page.locator('.tab-viewport').evaluate('el=>el.scrollLeft')
                            page.locator('#tabs-previous').click()
                            page.wait_for_function('(previous)=>document.querySelector(".tab-viewport").scrollLeft<previous-.5', arg=position)
                            page.locator('#tabs-list').click()
                            assert not errors, errors
                            menu = page.locator('.ui-menu')
                            expect(menu).to_be_visible()
                            ratio = menu.evaluate('el=>parseFloat(getComputedStyle(el).fontSize)/parseFloat(getComputedStyle(document.querySelector(".archive-tab")).fontSize)')
                            assert .95 <= ratio <= 1.10, ratio
                            page.keyboard.press('Home')
                            page.keyboard.press('ArrowDown')
                            page.keyboard.press('ArrowDown')
                            page.keyboard.press('Enter')
                            page.wait_for_function('XdzipMain.menuState().activeTab===XdzipMain.menuState().tabs[2].id')
                            if dpr == 1:
                                page.locator('.titlebar').screenshot(path=str(OUT/f'{language}-{scale}.png'))
                            checks.append(dict(language=language, scale=scale, logical_width=width, dpr=dpr, geometry=geometry, menu_font_ratio=ratio))
                    # Real pointer drag, keyboard reorder and Escape, preserving active identity.
                    page.evaluate('''()=>{document.documentElement.style.setProperty('--settings-text-scale',1);document.querySelector('.window').style.width='1548px';const all=XdzipMain.menuState().tabs;all.slice(3).forEach(t=>XdzipMain.closeTab(t.id));XdzipMain.switchTab(all[2].id,true)}''')
                    nodes = page.locator('.tab-viewport [data-tab]')
                    expect(nodes).to_have_count(3)
                    before = page.evaluate('XdzipMain.menuState()')
                    first, last = nodes.nth(0).bounding_box(), nodes.nth(2).bounding_box()
                    page.mouse.move(first['x']+first['width']/2,first['y']+first['height']/2)
                    page.mouse.down()
                    page.mouse.move(last['x']+last['width']/2,last['y']+last['height']/2,steps=10)
                    expect(page.locator('.tab-drag-preview')).to_be_visible()
                    page.mouse.up()
                    after = page.evaluate('XdzipMain.menuState()')
                    assert [t['id'] for t in after['tabs']] == [before['tabs'][1]['id'],before['tabs'][2]['id'],before['tabs'][0]['id']]
                    assert after['activeTab'] == before['activeTab']
                    page.keyboard.press('Control+Shift+PageUp')
                    assert page.evaluate('XdzipMain.menuState().tabs[0].id===XdzipMain.menuState().activeTab')
                    before = page.evaluate('XdzipMain.menuState()')
                    first, last = nodes.nth(0).bounding_box(), nodes.nth(2).bounding_box()
                    page.mouse.move(first['x']+first['width']/2,first['y']+first['height']/2)
                    page.mouse.down(); page.mouse.move(last['x']+last['width']/2,last['y']+last['height']/2,steps=10)
                    page.keyboard.press('Escape'); page.mouse.up()
                    assert page.evaluate('XdzipMain.menuState()')['tabs'] == before['tabs'], (before['tabs'],page.evaluate('XdzipMain.menuState()')['tabs'])
                    page.evaluate('''()=>{XdzipMain.enterWelcome();document.documentElement.style.setProperty('--settings-text-scale',1.5);beginWelcomeOpening('long-archive-name.zip');document.querySelector('#welcome').style.height='250px'}''')
                    cancel = page.locator('#welcome-opening-cancel')
                    expect(cancel).to_be_visible()
                    card, footer = page.locator('.welcome-opening-card').bounding_box(), page.locator('.welcome-opening-footer').bounding_box()
                    assert footer['y']+footer['height'] <= card['y']+card['height']
                    expect(cancel).to_have_text('Cancel loading')
                    page.locator('#welcome-opening').screenshot(path=str(OUT/f'loading-cancel-{dpr}.png'))
                    cancel.click()
                    expect(page.locator('#welcome-opening')).to_have_count(0)
                    expect(page.locator('#welcome-open')).to_be_focused()
                    context.close()
                assert not errors, errors
            finally:
                browser.close()
    finally:
        server.shutdown()
    (OUT/'result.json').write_text(json.dumps({'cases':checks,'errors':errors,'scope':'HTML pointer, keyboard, geometry and language checks, not native desktop acceptance'},indent=2)+'\n',encoding='utf-8')
    print(f'Tab strip: {len(checks)} geometry cases and pointer, keyboard, menu and cancellation paths passed')


if __name__ == '__main__':
    main()
