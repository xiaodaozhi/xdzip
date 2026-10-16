#!/usr/bin/env python3
"""Exercise A01 version provenance, languages, scaling and clipboard actions."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Thread
import json
import os
from playwright.sync_api import sync_playwright, expect
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'build/review/about/html';OUT.mkdir(parents=True,exist_ok=True)
class Quiet(SimpleHTTPRequestHandler):
    def log_message(self,*_):pass
server=ThreadingHTTPServer(('127.0.0.1',0),partial(Quiet,directory=str(ROOT)))
Thread(target=server.serve_forever,daemon=True).start()
errors=[];records=[]
with sync_playwright() as p:
    browser=p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM','/usr/bin/chromium'),args=['--no-sandbox'])
    try:
        for dpr in (1,1.25,2):
            context=browser.new_context(viewport={'width':1600,'height':1500},device_scale_factor=dpr,permissions=['clipboard-read','clipboard-write'])
            for language in ('zh-CN','en-US'):
                page=context.new_page();page.on('pageerror',lambda error:errors.append(str(error)))
                page.goto(f'http://127.0.0.1:{server.server_port}/design/html/canvas.html?view=about&lang={language}&standalone=1')
                page.evaluate('document.fonts.ready')
                for dark in (False,True):
                    for scale in (1,1.25,1.5):
                        page.evaluate('([dark,scale])=>{XdzipDesign.theme(dark?"dark":"light");document.documentElement.style.setProperty("--settings-text-scale",scale);XdzipDesign.open("about")}',[dark,scale])
                        modal=page.locator('#modal');expect(modal).to_be_visible();expect(modal).to_have_attribute('data-modeless','true')
                        version=page.evaluate('window.XDZIP_VERSION.display_version');expect(modal.locator('dd').first).to_have_text(version)
                        ratio=page.evaluate('1/Number(getComputedStyle(document.documentElement).getPropertyValue("--ui-source-factor"))')
                        bounds=modal.bounding_box();assert abs(bounds['width']*ratio-491)<=1
                        buttons=modal.locator('.modal-footer .button');a,b=[buttons.nth(i).bounding_box() for i in range(2)];assert abs(a['y']-b['y'])<=1
                        assert b['y']+b['height']<=bounds['y']+bounds['height']+1
                        expect(modal.locator('.os-title')).to_have_text('关于 xdzip' if language=='zh-CN' else 'About xdzip')
                        page.locator('#about-copy').click();expect(page.locator('#about-copy')).to_have_text('已复制' if language=='zh-CN' else 'Copied')
                        clipboard=json.loads(page.evaluate('navigator.clipboard.readText()'));assert clipboard['version']==version and 'path' not in clipboard
                        tag=f'{language}-{dark}-{scale}-{dpr}';modal.screenshot(path=str(OUT/f'{tag}.png'));records.append({'language':language,'dark':dark,'scale':scale,'dpr':dpr,'logical_width':bounds['width']*ratio})
                page.evaluate('XdzipDesign.open("settings")')
                page.locator('.theme-card').nth(1).click()
                page.evaluate('()=>{const select=document.querySelector("#setting-text-scale");select.value="1.25";select.dispatchEvent(new Event("change",{bubbles:true}));}')
                page.locator('#settings-about').click()
                expect(page.locator('#modal')).to_have_attribute('data-kind','about')
                page.locator('#modal [data-close]').click()
                expect(page.locator('#modal')).to_have_attribute('data-kind','settings')
                expect(page.locator('input[name="setting-theme"][value="light"]')).to_be_checked()
                expect(page.locator('#setting-text-scale')).to_have_value('1.25')
                page.locator('#modal [data-close]').click()
                page.close()
            context.close()
        assert not errors,errors
    finally:browser.close();server.shutdown()
(OUT/'review.json').write_text(json.dumps({'cases':records,'limits':'Browser page zoom is 100%; CSS text scale is an application preference, not browser zoom. Native Qt/Git fields are not invented in this design preview.'},indent=2)+'\n')
print(f'About HTML: {len(records)} language/theme/text-scale/DPR cases passed')
