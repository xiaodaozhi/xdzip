#!/usr/bin/env python3
"""Exercise the shared settings design draft; persistence is verified in Qt."""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
ROOT=Path(__file__).resolve().parents[1]
OUTPUT=ROOT/'build/review/html-settings'

def main():
    OUTPUT.mkdir(parents=True,exist_ok=True)
    with sync_playwright() as p:
        browser=p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM','/usr/bin/chromium'),args=['--no-sandbox','--disable-dev-shm-usage'])
        holder=browser.new_page()
        holder.set_content((ROOT/'dist/xdzip-hifi-design.html').read_text(encoding='utf-8'))
        canvas=holder.evaluate('window.XDZIP_CANVAS')
        page=browser.new_page(viewport={'width':1600,'height':1300})
        errors=[]
        page.on('pageerror',lambda error:errors.append(str(error)))
        page.set_content(canvas)
        page.evaluate('document.fonts.ready')
        page.locator('#title-settings').click()
        modal=page.locator('#modal')
        expect(modal).to_have_attribute('data-kind','settings')
        expect(modal).to_have_attribute('data-modeless','true')
        expect(page.locator('#setting-language option')).to_have_text(['中文 (简体)','English (US)'])
        original=page.evaluate('state.theme')
        expect(modal.locator('.modal-header')).to_be_visible()
        expect(modal.locator('.theme-card')).to_have_count(3)
        expect(modal.locator('.notice')).to_contain_text('不启用遥测')
        normal=modal.locator('.modal-header h1').evaluate('el=>parseFloat(getComputedStyle(el).fontSize)')
        layout=json.loads((ROOT/'docs/visual/layout-scale.json').read_text(encoding='utf-8'))
        spec=json.loads((ROOT/'docs/visual/settings-reference.json').read_text(encoding='utf-8'))
        ratio=layout['main']['source_to_logical'][0]/layout['main']['source_to_logical'][1]
        assert abs(modal.bounding_box()['width']*ratio-spec['logical_width'])<=1
        for selector in ['#setting-text-scale-trigger','#setting-language-trigger']:
            field=page.locator(selector)
            assert abs(field.bounding_box()['height']*ratio-layout['operation']['controls']['bodyButtonHeight'])<=1
            assert abs(field.evaluate('el=>parseFloat(getComputedStyle(el).fontSize)')*ratio-spec['body_size'])<=.1
        modal.screenshot(path=str(OUTPUT/'settings-light.png'))
        page.locator('input[name=setting-theme][value=dark]').check()
        expect(page.locator('html')).to_have_attribute('data-theme','dark')
        modal.screenshot(path=str(OUTPUT/'settings-dark.png'))
        page.locator('#clear-recent').click()
        expect(page.locator('#clear-recent')).to_have_text('保存后清空')
        expect(page.locator('#clear-recent')).to_be_disabled()
        page.locator('#setting-text-scale-trigger').click()
        page.keyboard.press('End')
        page.keyboard.press('Enter')
        expect(page.locator('#setting-text-scale')).to_have_value('2')
        assert modal.locator('.modal-header h1').evaluate('el=>parseFloat(getComputedStyle(el).fontSize)')==normal*2
        modal.locator('[data-close]').click()
        expect(modal).not_to_be_visible()
        assert page.evaluate('state.theme')==original
        page.locator('#title-settings').click()
        expect(page.locator('#setting-text-scale')).to_have_value('1')
        expect(page.locator('#clear-recent')).to_be_enabled()
        page.locator('#setting-density-trigger').click()
        page.keyboard.press('End')
        page.keyboard.press('Enter')
        expect(page.locator('#setting-density')).to_have_value('compact')
        assert page.locator('#file-rows td').first.evaluate('el=>parseFloat(getComputedStyle(el).height)') < 60
        page.locator('#recent-enabled').uncheck()
        page.locator('#setting-language-trigger').click()
        page.keyboard.press('End')
        page.keyboard.press('Enter')
        expect(page.locator('#setting-language')).to_have_value('en-US')
        page.locator('#settings-save').click()
        expect(modal).not_to_be_visible()
        page.locator('#title-settings').click()
        expect(page.locator('#setting-language')).to_have_value('en-US')
        expect(page.locator('#setting-density')).to_have_value('compact')
        expect(page.locator('#recent-enabled')).not_to_be_checked()
        page.locator('#settings-defaults').click()
        expect(page.locator('#setting-language')).to_have_value('zh-CN')
        modal.locator('[data-close]').click()
        page.locator('#title-settings').click()
        expect(page.locator('#setting-language')).to_have_value('en-US')
        assert not errors,errors
        browser.close()
    (OUTPUT/'validation.json').write_text(json.dumps({'status':'passed','checks':['Real settings entry','Independent modeless design','Both supported language autonyms','Normal logical width','Shared field height and font','Theme preview','Text-size preview','Cancel rollback','Saved draft reopened','Defaults modify draft only'],'limits':['HTML keeps demo settings in memory; Qt verifies on-disk and restart behavior']},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print('Settings design: draft interactions and normal-density checks passed')

if __name__=='__main__':main()
