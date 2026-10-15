#!/usr/bin/env python3
"""Check preflight design geometry and shared main row/inspector boundaries."""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT/'build/review/html-preflight'

def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM','/usr/bin/chromium'),args=['--no-sandbox','--disable-dev-shm-usage'])
        holder = browser.new_page()
        holder.set_content((ROOT/'dist/xdzip-hifi-design.html').read_text())
        page = browser.new_page(viewport={'width':1600,'height':1100})
        errors = []
        page.on('pageerror',lambda e:errors.append(str(e)))
        page.set_content(holder.evaluate('window.XDZIP_CANVAS'))
        page.evaluate('document.fonts.ready')
        close = page.locator('#hide-inspector').bounding_box()
        page.locator('#hide-inspector').click()
        toggle = page.locator('#properties-toggle').bounding_box()
        assert abs(close['x']-toggle['x'])<1 and abs(close['y']-toggle['y'])<1, (close,toggle)
        page.locator('#properties-toggle').click()
        page.evaluate("setCanvasLogicalScale('preflight'); window.XdzipDesign.open('preflight')")
        modal = page.locator('dialog[open][data-kind=preflight]')
        expect(modal).to_be_visible()
        factor = page.evaluate("parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--ui-source-factor'))")
        spec = json.loads((ROOT/'docs/visual/preflight-reference.json').read_text())
        assert abs(modal.bounding_box()['width']/factor-spec['logical_width'])<1
        assert abs(modal.locator('h1').evaluate('e=>parseFloat(getComputedStyle(e).fontSize)')/factor-spec['title_size'])<.1
        expect(modal.locator('.os-title')).to_have_text('解压预检')
        for theme in ['light','dark']:
            page.evaluate('(t)=>window.XdzipDesign.theme(t)',theme)
            modal.screenshot(path=str(OUTPUT/f'preflight-{theme}.png'))
        page.locator('#preflight-close').click()
        expect(modal).to_have_count(0)
        assert not errors,errors
        browser.close()
    print('Preflight design, inspector coordinates, typography and close action passed')

if __name__ == '__main__': main()
