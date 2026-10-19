#!/usr/bin/env python3
"""Measure actual quit/restart layouts, not just declared token values."""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT/'build/review/confirmations/html'


def main():
    OUT.mkdir(parents=True,exist_ok=True)
    records=[];errors=[]
    with sync_playwright() as p:
        browser=p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM','/usr/bin/chromium'))
        for dpr in [1,1.25,2]:
            for language in ['zh-CN','en-US']:
                page=browser.new_page(viewport={'width':1300,'height':1000},device_scale_factor=dpr)
                page.on('pageerror',lambda error:errors.append(str(error)))
                page.goto((ROOT/'design/html/canvas.html').as_uri()+'?view=quit&lang='+language+'&standalone=1')
                page.evaluate('document.fonts.ready')
                for dark in [False,True]:
                    for scale in [1,1.25,1.5]:
                        for view in ['quit','stopping','restart']:
                            page.evaluate('''([dark,scale,view])=>{
                                XdzipDesign.theme(dark?'dark':'light');
                                document.documentElement.style.setProperty('--settings-text-scale',scale);
                                if(view==='stopping')showQuitConfirmation(true);else XdzipDesign.open(view);
                            }''',[dark,scale,view])
                            modal=page.locator('#modal');expect(modal).to_be_visible()
                            measurements=modal.evaluate('''el=>{
                                const factor=Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-source-factor'));
                                const box=el.getBoundingClientRect(), title=el.querySelector('h1'),footer=el.querySelector('.modal-footer');
                                const buttons=[...footer.querySelectorAll('button')].map(b=>{const r=b.getBoundingClientRect();return {x:r.x,y:r.y,w:r.width,h:r.height,font:parseFloat(getComputedStyle(b).fontSize),disabled:b.disabled,clipped:b.scrollWidth>b.clientWidth+1||b.scrollHeight>b.clientHeight+1}});
                                return {width:box.width/factor,height:box.height/factor,title:parseFloat(getComputedStyle(title).fontSize)/factor,buttons,box:{x:box.x,y:box.y,right:box.right,bottom:box.bottom},footerBottom:footer.getBoundingClientRect().bottom};
                            }''')
                            assert abs(measurements['width']-491)<=1,measurements
                            assert abs(measurements['title']-20*scale)<=.1,measurements
                            buttons=measurements['buttons'];assert len(buttons)==(2 if view=='restart' else 3)
                            assert all(abs(b['y']-buttons[0]['y'])<=1 for b in buttons),measurements
                            assert all(b['x']>=measurements['box']['x'] and b['x']+b['w']<=measurements['box']['right']+1 and not b['clipped'] for b in buttons),measurements
                            assert abs(measurements['footerBottom']-measurements['box']['bottom'])<=2,measurements
                            assert all(b['disabled']==(view=='stopping') for b in buttons)
                            if dpr==1:
                                modal.screenshot(path=str(OUT/f'{view}-{language}-{dark}-{scale}.png'))
                            records.append(dict(dpr=dpr,language=language,dark=dark,scale=scale,view=view,width=measurements['width'],height=measurements['height']))
                # The same shared body must scroll without losing the action row.
                page.set_viewport_size({'width':1000,'height':340})
                page.evaluate('''()=>{XdzipDesign.open('quit');document.querySelector('.confirmation-content p').textContent='Committed files will be kept. '.repeat(100);}''')
                expect(page.locator('#quit-return')).to_be_visible()
                assert page.locator('.confirmation-content').evaluate('el=>el.scrollHeight>el.clientHeight')
                page.locator('#quit-return').click();expect(page.locator('#modal')).not_to_be_visible()
                for action in ['wait','cancel']:
                    page.evaluate("XdzipDesign.open('quit')")
                    page.locator('#quit-'+action).click();expect(page.locator('#modal')).not_to_be_visible()
                page.close()
        browser.close()
    assert not errors,errors
    (OUT/'report.json').write_text(json.dumps({'passed':True,'cases':records,'limits':'HTML preview at browser zoom 100%; DPR and application text-scale matrix. Native geometry and actual task exit are verified separately.'},indent=2)+'\n')
    print(f'Confirmation HTML: {len(records)} layouts plus scrolling and decision actions passed')


if __name__=='__main__':main()
