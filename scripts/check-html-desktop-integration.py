#!/usr/bin/env python3
"""Verify the native-menu interaction draft, never claim browser output is native UI."""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT/'build/review/desktop-integration/html'

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM','/usr/bin/chromium'))
        page = browser.new_page(viewport={'width':1120,'height':950}, device_scale_factor=1)
        errors = []
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.goto((ROOT/'design/html/desktop-integration.html').as_uri())
        expect(page.locator('#bar button')).to_have_text(['xdzip','文件','编辑','显示','操作','窗口','帮助'])
        expect(page.locator('#tray')).to_be_checked()
        page.locator('#menu').get_by_role('button', name='关闭窗口', exact=True).click()
        expect(page.locator('#workspace')).to_contain_text('主窗口已隐藏')
        page.locator('#tray-menu').get_by_role('button', name='显示主窗口', exact=True).click()
        expect(page.locator('#workspace')).to_contain_text('标题栏保持原样')
        page.locator('#busy').check()
        expect(page.locator('#tray-menu')).to_contain_text('2 个进行中')
        page.locator('#tray-menu').get_by_role('button',name='任务，2 个进行中 ›',exact=True).click()
        expect(page.locator('.submenu')).to_contain_text('任务 1')
        page.keyboard.press('Escape')
        page.locator('#tray-menu').get_by_role('button', name='退出 xdzip', exact=True).click()
        expect(page.locator('#status')).to_contain_text('退出确认')
        page.screenshot(path=str(OUT/'macos-light.png'), full_page=True)
        page.emulate_media(color_scheme='dark')
        page.screenshot(path=str(OUT/'macos-dark.png'), full_page=True)
        for platform in ['Windows','Linux']:
            page.locator('#platform').select_option(platform)
            expect(page.locator('#bar')).not_to_be_visible()
            page.locator('#tray').uncheck()
            expect(page.locator('#workspace')).to_contain_text('关闭托盘前恢复显示')
            page.locator('#tray').check()
        page.locator('#platform').select_option('macOS')
        expect(page.locator('#bar')).to_be_visible()
        for label in ['编辑','显示','操作','窗口','xdzip','文件','帮助']:
            page.locator('#bar').get_by_role('button',name=label,exact=True).click()
            expect(page.locator('#menu-title')).to_have_text(label)
        expect(page.locator('#menu button')).to_have_text(['使用说明','反馈问题','关于 xdzip'])
        page.locator('#menu').get_by_role('button', name='关于 xdzip', exact=True).click()
        expect(page.locator('#status')).to_have_text('已触发：关于 xdzip')
        assert not errors, errors
        browser.close()
    (OUT/'report.json').write_text(json.dumps({'passed':True,'kind':'HTML interaction draft, not native desktop evidence','platforms':['macOS','Windows','Linux'],'dpr':1,'viewport':[1120,950]},indent=2)+'\n')
    print('Desktop integration HTML: menu groups, tray lifecycle and platform previews passed')

if __name__ == '__main__': main()
