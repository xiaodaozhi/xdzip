#!/usr/bin/env python3
"""Measure actual operation dialogs and footers against the logical specification."""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'dist/html-operation-review'


def main():
    spec = json.loads((ROOT / 'docs/visual/layout-scale.json').read_text())['operation']
    OUTPUT.mkdir(parents=True, exist_ok=True)
    checks, errors, requests = [], [], []

    def check(name, actual, expected, tolerance=.1):
        passed = abs(actual - expected) <= tolerance
        checks.append(dict(name=name, actual=actual, expected=expected, passed=passed))

    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
                                    args=['--no-sandbox', '--disable-dev-shm-usage'])
        holder = browser.new_page()
        holder.set_content((ROOT / 'dist/xdzip-hifi-design.html').read_text())
        canvas = holder.evaluate('window.XDZIP_CANVAS')
        holder.close()
        for dpr in [1, 1.25, 2]:
            page = browser.new_page(viewport={'width':1588, 'height':1600}, device_scale_factor=dpr)
            page.route('**/*', lambda r: (requests.append(r.request.url), r.abort()))
            page.on('pageerror', lambda e: errors.append(str(e)))
            page.set_content(canvas)
            page.evaluate('document.fonts.ready')
            for platform in ['macos', 'windows', 'linux']:
                for theme in ['light', 'dark']:
                    page.evaluate('([platform,theme])=>{XdzipDesign.platform(platform);XdzipDesign.theme(theme);document.body.dataset.standalone="true";}', [platform,theme])
                    for view in ['extract', 'create', 'settings', 'progress', 'conflict', 'result']:
                        if view == 'conflict':
                            page.evaluate('()=>{XdzipDesign.open("progress");state.task.status="conflict";showConflict()}')
                        elif view == 'result':
                            page.evaluate('()=>{XdzipDesign.open("progress");finishTask("success")}')
                        else:
                            page.evaluate('view=>XdzipDesign.open(view)', view)
                        modal = page.locator('dialog[open]')
                        expect(modal).to_be_visible()
                        expect(modal).to_have_attribute('data-kind', view)
                        kind = modal.get_attribute('data-kind')
                        conversion = spec.get('source_to_logical_by_view', {}).get(kind, spec['source_to_logical'])
                        ratio = conversion[0] / conversion[1]
                        page.evaluate('kind=>setCanvasLogicalScale(kind)', kind)
                        tag = f'{platform}/{theme}/DPR{dpr}/{kind}'
                        metrics = modal.evaluate('''n=>({
                            width:n.getBoundingClientRect().width,
                            client:n.clientWidth, scroll:n.scrollWidth,
                            buttons:[...n.querySelectorAll('.modal-footer .button')].map(b=>parseFloat(getComputedStyle(b).fontSize)),
                            actions:[...n.querySelectorAll('.modal-footer .button')].map(b=>({height:b.getBoundingClientRect().height,radius:parseFloat(getComputedStyle(b).borderRadius)})),
                            notes:[...n.querySelectorAll('.modal-footer .ready-label,.modal-footer .info-line')].map(b=>parseFloat(getComputedStyle(b).fontSize)),
                            footer:n.querySelector('.modal-footer').getBoundingClientRect().height
                        })''')
                        expected_width = spec['logical_widths'].get(kind, spec['logical_widths']['default'])
                        check(tag+'/width', metrics['width'] * ratio, expected_width, 1)
                        check(tag+'/horizontal overflow', max(0,metrics['scroll']-metrics['client']), 0, 1)
                        assert metrics['buttons'] and metrics['notes'], tag
                        for i, size in enumerate(metrics['buttons']):
                            check(tag+f'/footer action {i}', size * ratio, spec['footer']['fontSize'])
                        for i, action in enumerate(metrics['actions']):
                            check(tag+f'/footer height {i}', action['height'] * ratio, spec['controls']['footerButtonHeight'], 1)
                            check(tag+f'/footer radius {i}', action['radius'] * ratio, spec['controls']['radius'])
                        if kind == 'create':
                            assert modal.locator('#create-size').inner_text() == '3 个文件，27.4 MB'
                            group = modal.locator('#create-format')
                            expect(group).to_have_attribute('role', 'radiogroup')
                            radios = group.locator('[role=radio]')
                            formats = ['ZIP', 'TAR', 'TAR.GZ', '7z']
                            expect(radios).to_have_count(len(formats))
                            expect(radios).to_have_text(formats)
                            expect(radios.nth(0)).to_have_attribute('aria-checked', 'true')
                            assert all(radios.nth(i).is_enabled() for i in range(len(formats)))
                            segment_metrics = radios.evaluate_all('''nodes=>nodes.map(n=>{
                                const r=n.getBoundingClientRect(),s=getComputedStyle(n);
                                return {left:r.left,right:r.right,width:r.width,corners:[s.borderTopLeftRadius,s.borderTopRightRadius,s.borderBottomRightRadius,s.borderBottomLeftRadius].map(parseFloat)};
                            })''')
                            check(tag+'/equal segment widths', (max(s['width'] for s in segment_metrics)-min(s['width'] for s in segment_metrics))*ratio, 0, 1)
                            for index, segment in enumerate(segment_metrics):
                                for corner, expected in enumerate([(4,0,0,4), (0,0,0,0), (0,0,0,0), (0,4,4,0)][index]):
                                    check(tag+f'/segment {index}/corner {corner}', segment['corners'][corner]*ratio, expected, .15)
                                if index:
                                    check(tag+f'/segment {index}/seam', (segment['left']-segment_metrics[index-1]['right'])*ratio, 0, .15)
                            for selector in ['#create-name','#create-destination','#create-volume-trigger','#create-level-trigger']:
                                field = modal.locator(selector)
                                shape = field.evaluate('n=>({radius:parseFloat(getComputedStyle(n).borderRadius),height:n.getBoundingClientRect().height})')
                                check(tag+selector+'/radius',shape['radius']*ratio,spec['controls']['radius'])
                                check(tag+selector+'/height',shape['height']*ratio,28,1)
                            for selector in ['#create-volume-trigger','#create-level-trigger']:
                                gap = modal.locator(selector).evaluate('n=>n.getBoundingClientRect().right-n.querySelector("svg").getBoundingClientRect().right')
                                check(tag+selector+'/right icon inset',gap*ratio,spec['controls']['selectIndicatorRightInset'],.15)
                            check(tag+'/segment height',group.bounding_box()['height']*ratio,28,1)
                            radios.nth(0).focus()
                            for index, suffix in [(1,'.tar'), (2,'.tar.gz'), (3,'.7z')]:
                                page.keyboard.press('ArrowRight')
                                expect(radios.nth(index)).to_be_focused()
                                expect(radios.nth(index)).to_have_attribute('aria-checked','true')
                                expect(modal.locator('#create-name')).to_have_value('旅行资料'+suffix)
                                assert modal.locator('#create-volume-trigger').is_enabled() == (index == 3)
                                assert modal.locator('#create-use-password').is_enabled() == (index == 3)
                            expect(modal.locator('#create-solid')).not_to_be_checked()
                            expect(modal.locator('#create-encrypt-header')).to_be_disabled()
                            modal.locator('#create-use-password').check()
                            expect(modal.locator('#create-encrypt-header')).to_be_enabled()
                            modal.locator('#create-encrypt-header').check()
                            modal.locator('#create-solid').check()
                            modal.locator('#create-use-password').uncheck()
                            expect(modal.locator('#create-encrypt-header')).not_to_be_checked()
                            expect(modal.locator('#create-encrypt-header')).to_be_disabled()
                            radios.nth(0).click()
                            expect(modal.locator('#create-header-row')).to_be_hidden()
                            expect(modal.locator('#create-solid-row')).to_be_hidden()
                            radios.nth(3).click()
                            expect(modal.locator('#create-solid')).not_to_be_checked()
                            expect(modal.locator('#create-encrypt-header')).not_to_be_checked()
                        for i, size in enumerate(metrics['notes']):
                            check(tag+f'/footer note {i}', size * ratio, spec['footer']['noteSize'])
                        if kind == 'progress':
                            edges = modal.locator('.info-line').evaluate_all('(nodes)=>nodes.map(n=>{const icon=n.querySelector("svg").getBoundingClientRect();return icon.right+parseFloat(getComputedStyle(n).gap)})')
                            check(tag+'/progress hint text edge', edges[0] * ratio, edges[1] * ratio, .01)
                        # Long extraction scopes may scroll; the footer must remain reachable.
                        modal.evaluate('n=>n.scrollTop=n.scrollHeight')
                        footer = modal.locator('.modal-footer')
                        assert footer.bounding_box()['height'] > 0
                        if dpr == 1 and platform == 'macos' and theme == 'light':
                            modal.screenshot(path=str(OUTPUT / f'{kind}.png'))
                    # Shared footer styles also apply to auxiliary real dialogs.
                    page.evaluate('()=>{XdzipDesign.open("main");state.task=null;document.body.dataset.standalone="false";setCanvasLogicalScale("main");confirmCreateOverwrite()}')
                    for action in ['overwrite', 'picker', 'archive-test']:
                        if action == 'picker':
                            page.evaluate('()=>{XdzipDesign.open("main");XdzipMain.dispatch("open")}')
                        elif action == 'archive-test':
                            page.evaluate('()=>{XdzipDesign.open("main");XdzipMain.dispatch("test")}')
                        expect(page.locator('dialog[open]')).to_have_attribute('data-kind', {'overwrite':'create-overwrite','picker':'picker','archive-test':'archive-test'}[action])
                        sizes = page.locator('dialog[open] .modal-footer .button').evaluate_all('(nodes)=>nodes.map(n=>parseFloat(getComputedStyle(n).fontSize))')
                        for i, size in enumerate(sizes):
                            check(f'{platform}/{theme}/DPR{dpr}/{action}/footer {i}', size * (1024 / 1548), spec['footer']['fontSize'])
                        if action == 'archive-test':
                            page.evaluate('()=>{XdzipMain.resetJobs();closeModal()}')
            page.close()
        browser.close()
    report = dict(checks=checks, passed=sum(c['passed'] for c in checks),
                  failed=sum(not c['passed'] for c in checks), page_errors=errors, external_requests=requests,
                  environment='Chromium offline artboard; CSS source bounds use per-view source-to-logical conversion; DPR changes raster only')
    (OUTPUT / 'validation.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
    for c in checks:
        if not c['passed']: print(json.dumps(c))
    print(f"Operation dialogs: {report['passed']} passed, {report['failed']} failed")
    if report['failed'] or errors or requests: raise SystemExit(1)


if __name__ == '__main__':
    main()
