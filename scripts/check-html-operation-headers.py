#!/usr/bin/env python3
"""Measure shared headers and the task-history viewport in the offline design.

Controlled menu rows test layout only; they are not native archive task evidence.
"""
import argparse
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'build/review/operation-headers')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    spec = json.loads((ROOT / 'docs/visual/layout-scale.json').read_text())
    common = spec['operation']['header']
    records, errors, requests = [], [], []
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
                                    args=['--no-sandbox'])
        holder = browser.new_page()
        holder.set_content((ROOT / 'dist/xdzip-hifi-design.html').read_text())
        canvas = holder.evaluate('window.XDZIP_CANVAS')
        holder.close()
        for dpr in (1, 1.25, 2):
            page = browser.new_page(viewport={'width': 1600, 'height': 1600}, device_scale_factor=dpr)
            page.route('**/*', lambda route: (requests.append(route.request.url), route.abort()))
            page.on('pageerror', lambda error: errors.append(str(error)))
            page.set_content(canvas)
            page.evaluate('document.fonts.ready')
            for language in ('zh-CN', 'en-US'):
                for dark in (False, True):
                    for scale in (1, 1.25, 1.5):
                        for view in ('extract', 'create', 'settings', 'about', 'progress', 'conflict', 'result'):
                            page.evaluate('''([language,dark,scale,view])=>{
                                activeSettingsLanguage=language; document.documentElement.lang=language;
                                XdzipDesign.theme(dark?'dark':'light');
                                document.documentElement.style.setProperty('--settings-text-scale',scale);
                                XdzipDesign.open(view); setCanvasLogicalScale(view);
                            }''', [language, dark, scale, view])
                            header = page.locator('dialog[open] .shared-operation-header')
                            expect(header).to_have_count(1)
                            metrics = header.evaluate('''h=>{
                                const ratio=Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-source-factor'));
                                const r=h.getBoundingClientRect(),i=h.querySelector('.modal-symbol').getBoundingClientRect();
                                const t=h.querySelector('h1'),s=h.querySelector('p'),body=h.nextElementSibling;
                                return {height:r.height/ratio,icon:i.width/ratio,iconHeight:i.height/ratio,
                                    bodyGap:(body.getBoundingClientRect().top-r.bottom)/ratio,
                                    bodyPadding:parseFloat(getComputedStyle(body).paddingTop)/ratio,
                                    inset:(i.left-r.left)/ratio,textInset:(t.getBoundingClientRect().left-r.left)/ratio,
                                    title:parseFloat(getComputedStyle(t).fontSize)/ratio,
                                    subtitle:parseFloat(getComputedStyle(s).fontSize)/ratio,
                                    overflow:h.scrollWidth-h.clientWidth};
                            }''')
                            tag = f'{view}-{language}-{dark}-{scale}-DPR{dpr}'
                            expected = {'icon': common['iconSize'], 'iconHeight': common['iconSize'],
                                        'inset': common['horizontalInset'],
                                        'textInset': common['horizontalInset']+common['iconSize']+common['gap'],
                                        'title': common['titleSize']*scale, 'subtitle': common['subtitleSize']*scale,
                                        'bodyGap':common['bodyGap'],'bodyPadding':0}
                            for key, value in expected.items():
                                assert abs(metrics[key]-value) < .2, (tag, key, metrics, value)
                            assert metrics['height'] >= common['normalHeight']-.2, (tag, metrics)
                            assert metrics['overflow'] <= 1, (tag, metrics)
                            records.append(dict(case=tag, **metrics))
                            if dpr == 1 and language == 'zh-CN' and scale == 1:
                                header.screenshot(path=str(args.output / f'{view}-{dark}.png'))
            page.evaluate('XdzipDesign.open("main")')
            page.evaluate('''()=>XdzipControls.open({trigger:document.querySelector('#tasks'),label:'History',
                items:Array.from({length:80},(_,i)=>({id:'task-'+i,label:'Task '+i})),
                footer:{label:'Clear finished tasks',run:()=>window.cleared=true}})''')
            menu = page.locator('.ui-task-menu')
            expect(menu).to_be_visible()
            measurements = menu.evaluate('''m=>{
                const r=m.getBoundingClientRect(),s=m.querySelector('.ui-menu-scroll'),f=m.querySelector('.ui-menu-fixed-footer');
                const y=f.getBoundingClientRect().y;s.scrollTop=s.scrollHeight;
                return {height:r.height/Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-source-factor')),
                    scroll:s.scrollTop,footerMoved:f.getBoundingClientRect().y-y};
            }''')
            assert measurements['height'] <= spec['controls']['task_menu_maximum_height']+1, measurements
            assert measurements['scroll'] > 0 and abs(measurements['footerMoved']) < .1, measurements
            page.keyboard.press('End')
            page.keyboard.press('Tab')
            expect(page.locator('.ui-menu-clear')).to_be_focused()
            page.keyboard.press('Enter')
            expect(menu).to_have_count(0)
            assert page.evaluate('window.cleared') is True
            page.close()
        browser.close()
    assert not errors, errors
    assert not requests, requests
    (args.output / 'review.json').write_text(json.dumps(dict(cases=records, errors=errors,
        limits='Offline HTML geometry and controlled menu input only; no native platform or real archive acceptance.'), indent=2)+'\n')
    print(f'Shared HTML headers: {len(records)} cases; task viewport, footer and keyboard checks passed at 3 DPRs')


if __name__ == '__main__':
    main()
