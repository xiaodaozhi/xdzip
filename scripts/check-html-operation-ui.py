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
    ratio = spec['source_to_logical'][0] / spec['source_to_logical'][1]
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
                    page.evaluate('([platform,theme])=>{XdzipDesign.platform(platform);XdzipDesign.theme(theme);document.body.dataset.standalone="true";setCanvasLogicalScale("extract")}', [platform,theme])
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
                        tag = f'{platform}/{theme}/DPR{dpr}/{kind}'
                        metrics = modal.evaluate('''n=>({
                            width:n.getBoundingClientRect().width,
                            client:n.clientWidth, scroll:n.scrollWidth,
                            buttons:[...n.querySelectorAll('.modal-footer .button')].map(b=>parseFloat(getComputedStyle(b).fontSize)),
                            notes:[...n.querySelectorAll('.modal-footer .ready-label,.modal-footer .info-line')].map(b=>parseFloat(getComputedStyle(b).fontSize)),
                            footer:n.querySelector('.modal-footer').getBoundingClientRect().height
                        })''')
                        expected_width = spec['logical_widths'].get(kind, spec['logical_widths']['default'])
                        check(tag+'/width', metrics['width'] * ratio, expected_width, 1)
                        check(tag+'/horizontal overflow', max(0,metrics['scroll']-metrics['client']), 0, 1)
                        assert metrics['buttons'] and metrics['notes'], tag
                        for i, size in enumerate(metrics['buttons']):
                            check(tag+f'/footer action {i}', size * ratio, spec['footer']['fontSize'])
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
                  environment='Chromium offline artboard; CSS source bounds multiplied by 1280/1548; DPR changes raster only')
    (OUTPUT / 'validation.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
    for c in checks:
        if not c['passed']: print(json.dumps(c))
    print(f"Operation dialogs: {report['passed']} passed, {report['failed']} failed")
    if report['failed'] or errors or requests: raise SystemExit(1)


if __name__ == '__main__':
    main()
