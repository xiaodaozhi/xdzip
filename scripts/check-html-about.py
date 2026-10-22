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
errors=[];records=[];update_records=[]
UPDATE_STATES = [{"state": state} for state in (
    'idle', 'checking', 'up_to_date', 'available', 'downloading', 'verifying',
    'downloaded', 'waiting_tasks', 'preparing', 'installing', 'error',
    'recovery_required', 'updated', 'unavailable')]
UPDATE_STATES += [{"state": "available", "canInstall": False, "trusted": trusted}
                  for trusted in (False, True)]
UPDATE_STATES += [{"state": "error", "errorCode": code} for code in (
    'network', 'tls', 'timeout', 'rate_limited', 'disk', 'minimum_system',
    'verification_hash', 'authorization_cancelled', 'authorization_failed')]
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
                        for index, fixture in enumerate(UPDATE_STATES):
                            fixture = dict(fixture, downloaded=fixture['state']=='downloaded')
                            page.evaluate('fixture=>XdzipAboutUpdatePreview(fixture)', fixture)
                            page.evaluate('()=>new Promise(resolve=>requestAnimationFrame(resolve))')
                            status=modal.locator('#about-update-status')
                            assert status.text_content() and not status.text_content().startswith('update.')
                            metrics=page.evaluate('''()=>{
                                const modal=document.querySelector('#modal'),body=modal.querySelector('.about-content'),
                                    section=modal.querySelector('.about-update-section'),status=modal.querySelector('#about-update-status'),
                                    footer=modal.querySelector('.modal-footer'),box=modal.getBoundingClientRect(),
                                    inset=Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-source-factor'));
                                return {font:Number.parseFloat(getComputedStyle(status).fontSize)/inset,
                                    height:box.height/inset,overflow:body.scrollWidth-body.clientWidth,
                                    footerBottom:footer.getBoundingClientRect().bottom-box.bottom,
                                    progressHeight:section.querySelector('progress')?.getBoundingClientRect().height/inset,
                                    buttons:[...section.querySelectorAll('button')].map(button=>{
                                        const a=button.getBoundingClientRect(),b=section.getBoundingClientRect();
                                        return {left:(a.left-b.left)/inset,right:(a.right-b.right)/inset,overflow:button.scrollWidth-button.clientWidth};
                                    })};
                            }''')
                            assert abs(metrics['font']-13*scale)<.1,(tag,fixture,metrics)
                            assert metrics['height']<=640.5 and metrics['overflow']<=1 and metrics['footerBottom']<=1,(tag,fixture,metrics)
                            assert all(button['left']>=-1 and button['right']<=1 and button['overflow']<=1 for button in metrics['buttons']),(tag,fixture,metrics)
                            busy=fixture['state'] in ('checking','downloading','verifying','waiting_tasks','preparing','installing')
                            expect(modal.locator('#about-check-update')).to_be_disabled() if busy else expect(modal.locator('#about-check-update')).to_be_enabled()
                            expect(modal.locator('#about-cancel-update')).to_have_count(int(busy and fixture['state']!='installing'))
                            progress=fixture['state'] in ('downloading','verifying','preparing')
                            expect(modal.locator('.about-update-section progress')).to_have_count(int(progress))
                            if progress: assert abs(metrics['progressHeight']-6)<.1
                            action=modal.locator('#about-install-update')
                            if action.count():
                                action.focus()
                                expect(action).to_be_in_viewport()
                                if dpr==1 and scale==1.5 and index in (3,14):
                                    modal.screenshot(path=str(OUT/f'{tag}-update-{index}.png'))
                                action.click()
                                expect(status).to_have_text(page.evaluate('language=>window.XDZIP_UI_MESSAGES[language].messages["update.preview"]',language))
                            update_records.append({'language':language,'dark':dark,'scale':scale,'dpr':dpr,'fixture':fixture,'metrics':metrics})
                            if dpr==1 and scale==1.5 and index in (4,11):
                                status.scroll_into_view_if_needed()
                                modal.screenshot(path=str(OUT/f'{tag}-update-{index}.png'))
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
(OUT/'review.json').write_text(json.dumps({'cases':records,'update_cases':update_records,'limits':'Browser page zoom is 100%; CSS text scale is an application preference, not browser zoom. Update actions use read-only state fixtures and never download or install. Native Qt/Git fields are not invented in this design preview.'},indent=2)+'\n')
print(f'About HTML: {len(records)} language/theme/text-scale/DPR cases and {len(update_records)} update state cases passed')
