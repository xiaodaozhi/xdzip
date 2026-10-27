#!/usr/bin/env python3
"""Compare live extraction controls with the immutable hifi reference and entry points."""
from functools import partial
from hashlib import sha256
from http.server import SimpleHTTPRequestHandler
from html_review_server import ReviewHTTPServer
import json
import os
from pathlib import Path
from threading import Thread

from playwright.sync_api import sync_playwright, expect
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'build/operation-repair/html'


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


def main():
    reference = json.loads((ROOT / 'docs/visual/extract-reference.json').read_text())
    assert sha256((ROOT / reference['source']).read_bytes()).hexdigest() == reference['sha256']
    source_ratio = reference['source_to_logical'][0] / reference['source_to_logical'][1]
    OUTPUT.mkdir(parents=True, exist_ok=True)
    checks, errors, scenes = [], [], {}

    def check(name, actual, expected, tolerance=1):
        passed = abs(actual - expected) <= tolerance if isinstance(actual, (int, float)) else actual == expected
        checks.append(dict(name=name, actual=actual, expected=expected, passed=passed))

    original = Image.open(ROOT / reference['source']).convert('RGB')
    color = reference['colors']['card']
    expected_rgb = tuple(int(color[i:i+2], 16) for i in (1,3,5))
    for point in reference['sample_pixels']:
        distance = max(abs(a-b) for a,b in zip(original.getpixel(tuple(point)), expected_rgb))
        check('hifi/card sample '+str(point), distance, 0, reference['color_channel_tolerance'])

    server = ReviewHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'), args=['--no-sandbox', '--disable-dev-shm-usage'])
            for dpr in (1, 1.25, 2):
                for theme in ('light', 'dark'):
                    for entry in ('main', 'extract'):
                        page = browser.new_page(viewport={'width':1600, 'height':1800}, device_scale_factor=dpr)
                        page.on('pageerror', lambda error: errors.append(str(error)))
                        page.goto(f'http://127.0.0.1:{server.server_port}/design/html/index.html?view={entry}&theme={theme}&zoom=1')
                        frame = page.frame_locator('#design-frame')
                        frame.locator('body').evaluate('()=>{state.selected=new Set(["lake","sunrise","guide"]);render();showExtract()}')
                        modal = frame.locator('dialog[open]')
                        expect(modal).to_have_attribute('data-kind', 'extract')
                        tag = f'{entry}/{theme}/DPR{dpr}'
                        box = modal.bounding_box()
                        check(tag+'/width', box['width'], reference['source_client_width'] * source_ratio)
                        check(tag+'/path inside field', frame.locator('.destination-field #change-target').count(), 1)
                        expect(frame.locator('#target-base')).to_have_value('下载 / 北境旅行')
                        frame.locator('#same-folder').uncheck()
                        expect(frame.locator('#target-base')).to_have_value('下载')
                        frame.locator('#same-folder').check()
                        expect(frame.locator('#target-base')).to_have_value('下载 / 北境旅行')
                        check(tag+'/shared checkbox', frame.locator('#same-folder').evaluate('n=>getComputedStyle(n).appearance'), 'none')
                        checkbox_mark = frame.locator('.destination-check .destination-mark').bounding_box()
                        status_mark = frame.locator('#target-valid .destination-mark').bounding_box()
                        checkbox_text = frame.locator('.destination-check .destination-text').bounding_box()
                        status_text = frame.locator('#target-valid .destination-text').bounding_box()
                        check(tag+'/destination marker columns', checkbox_mark['x'] + checkbox_mark['width']/2, status_mark['x'] + status_mark['width']/2, .1)
                        check(tag+'/destination text columns', checkbox_text['x'], status_text['x'], .1)
                        for row, mark, text in [('checkbox', checkbox_mark, checkbox_text), ('status', status_mark, status_text)]:
                            check(tag+'/'+row+' vertical center', mark['y'] + mark['height']/2, text['y'] + text['height']/2, .5)
                        check(tag+'/native select hidden', frame.locator('#conflict-policy').is_visible(), False)
                        metrics = modal.evaluate('''n=>[n,n.querySelector('h1'),...n.querySelectorAll('.modal-footer .button')].map(e=>({
                            selector:e===n?'DIALOG.':e.tagName+'.',w:e.getBoundingClientRect().width,
                            font:parseFloat(getComputedStyle(e).fontSize),text:e.textContent.slice(0,80)
                        }))''')
                        factor = source_ratio if entry == 'extract' else 1024 / 1548
                        for node in metrics:
                            node['w'] *= factor
                            node['font'] *= factor
                        check(tag+'/title size', metrics[1]['font'], reference['source_title_font'] * source_ratio)
                        for index, node in enumerate(metrics[2:]):
                            check(tag+f'/footer action {index}', node['font'], reference['footer_font_size'], .1)
                        field = frame.locator('.destination-field')
                        footer_box = modal.locator('.modal-footer').bounding_box()
                        check(tag+'/footer inside window', footer_box['y'] + footer_box['height'] <= box['y'] + box['height'] + 1, True)
                        for button in modal.locator('.modal-footer .button').all():
                            bounds = button.bounding_box()
                            check(tag+'/action inside footer', bounds['y'] + bounds['height'] <= footer_box['y'] + footer_box['height'] + 1, True)
                        check(tag+'/field height', field.bounding_box()['height'], reference['source_field_height'] * source_ratio)
                        cards = modal.locator('.field-group')
                        expected_color = reference['colors']['card'] if theme == 'light' else '#202630'
                        actual = cards.first.evaluate('n=>getComputedStyle(n).backgroundColor')
                        rgb = tuple(int(expected_color[i:i+2], 16) for i in (1,3,5))
                        check(tag+'/card fill', actual, f'rgb{rgb}')
                        trigger = frame.locator('#conflict-policy-trigger')
                        trigger.focus()
                        trigger.press('ArrowDown')
                        options = frame.get_by_role('listbox')
                        expect(options).to_be_visible()
                        expect(options.get_by_role('option')).to_have_count(4)
                        menu_box, trigger_box = options.bounding_box(), trigger.bounding_box()
                        gap = menu_box['y'] - (trigger_box['y'] + trigger_box['height']) if menu_box['y'] >= trigger_box['y'] else trigger_box['y'] - (menu_box['y'] + menu_box['height'])
                        check(tag+'/popup gap', gap, 4, .2)
                        options.get_by_role('option').first.press('End')
                        options.get_by_role('option').last.press('Enter')
                        expect(trigger).to_contain_text('覆盖已有文件')
                        expect(trigger).not_to_be_focused()
                        expect(frame.locator('#policy-note')).to_contain_text('将替换已有文件内容')
                        trigger.press('ArrowDown')
                        options.get_by_role('option').last.press('Escape')
                        expect(trigger).not_to_be_focused()
                        expect(modal).to_be_visible()
                        target = frame.locator('#target-base')
                        for key in ['Escape', 'Enter']:
                            target.focus()
                            target.press(key)
                            expect(modal).to_be_visible()
                            check(tag+'/'+key+'/target loses focus', target.evaluate('n=>n!==document.activeElement'), True)
                        target.focus()
                        modal.locator('h1').click()
                        check(tag+'/outside click/target loses focus', target.evaluate('n=>n!==document.activeElement'), True)
                        check(tag+'/keyboard selection and Escape', True, True)
                        if entry == 'extract' and dpr == 1:
                            trigger.press('ArrowDown')
                            options.get_by_role('option').first.press('Home')
                            options.get_by_role('option').first.press('Enter')
                            modal.locator('h1').click()
                            modal.screenshot(path=str(OUTPUT / f'extract-{theme}.png'))
                            if theme == 'light':
                                scenes['extract'] = metrics
                                trigger.click()
                                modal.screenshot(path=str(OUTPUT / 'extract-dropdown.png'))
                                options.get_by_role('option').first.press('Escape')
                                frame.locator('#start-extract').click()
                                expect(modal).to_have_attribute('data-kind', 'progress')
                                expect(page.locator('#view')).to_have_value('progress')
                                check(tag+'/transition to progress width', modal.bounding_box()['width'], 491)
                        page.close()
            browser.close()
    finally:
        server.shutdown()
        server.server_close()
        thread.join()
    report = dict(checks=checks, failed=sum(not c['passed'] for c in checks), page_errors=errors,
                  reference=reference['source'], reference_sha256=reference['sha256'],
                  environment='Chromium / DPR 1, 1.25, 2 / normal reviewer scale / native screenshots',
                  html_sha256=sha256((ROOT / 'design/html/hifi.css').read_bytes()).hexdigest(), scenes=scenes)
    (OUTPUT / 'validation.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
    for c in checks:
        if not c['passed']:
            print(json.dumps(c, ensure_ascii=False))
    print(f"Extraction fidelity: {len(checks)} checks, {report['failed']} failed")
    raise SystemExit(bool(report['failed'] or errors))


if __name__ == '__main__':
    main()
