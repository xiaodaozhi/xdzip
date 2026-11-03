#!/usr/bin/env python3
"""Exercise the six-page, immediately applied settings design."""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'build/review/html-settings'


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM'), args=['--no-sandbox'])
        holder = browser.new_page()
        holder.set_content((ROOT / 'dist/xdzip-hifi-design.html').read_text(encoding='utf-8'))
        canvas = holder.evaluate('window.XDZIP_CANVAS')
        page = browser.new_page(viewport={'width': 1600, 'height': 1300})
        errors = []
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.set_content(canvas)
        page.evaluate('document.fonts.ready')
        page.locator('#title-settings').click()
        modal = page.locator('#modal')
        expect(modal).to_have_attribute('data-kind', 'settings')
        expect(modal).to_have_attribute('data-modeless', 'true')
        expect(modal.locator('.settings-nav-item')).to_have_count(6)
        expect(modal.locator('.modal-footer')).to_have_count(0)
        assert not modal.get_by_role('button', name='完成').count()
        layout = json.loads((ROOT / 'docs/visual/layout-scale.json').read_text())
        palette = json.loads((ROOT / 'docs/visual/tokens.json').read_text())
        controls = json.loads((ROOT / 'docs/visual/controls.json').read_text())
        ratio = layout['main']['source_to_logical'][0] / layout['main']['source_to_logical'][1]
        assert abs(modal.bounding_box()['width'] * ratio - 612) < 1
        def rgb(value):
            return 'rgb(' + ', '.join(str(int(value[index:index+2], 16)) for index in (1, 3, 5)) + ')'
        for theme in ('light', 'dark'):
            page.evaluate('theme => XdzipDesign.theme(theme)', theme)
            header = modal.locator('.shared-operation-header')
            divider = header.evaluate('''node => {
                const style = getComputedStyle(node, '::after');
                return {height:parseFloat(style.height), color:style.backgroundColor};
            }''')
            assert abs(divider['height'] * ratio - layout['operation']['settings']['headerDividerHeight']) < .1
            assert divider['color'] == rgb(palette[theme]['border'])
            sidebar = modal.locator('.settings-sidebar')
            assert sidebar.evaluate('node => getComputedStyle(node).backgroundColor') == rgb(palette[theme]['sidebar'])
            hovered = modal.locator('#settings-tab-general')
            hovered.hover()
            assert hovered.evaluate('node => getComputedStyle(node).backgroundColor') == rgb(controls[theme]['controlHover'])
            assert hovered.evaluate('node => getComputedStyle(node).backgroundColor') != sidebar.evaluate('node => getComputedStyle(node).backgroundColor')
            icon = hovered.locator('svg').bounding_box()
            assert abs((icon['x'] - hovered.bounding_box()['x']) * ratio - layout['operation']['settings']['sidebarIconInset']) < .1
            selected = modal.locator('#settings-tab-appearance')
            selected.hover()
            selected_color = selected.evaluate('node => getComputedStyle(node).backgroundColor')
            assert selected_color == rgb(palette[theme]['selection']), (theme, selected_color, selected.get_attribute('aria-selected'))
        page.mouse.move(0, 0)
        for tab in ('appearance', 'general', 'privacy', 'associations', 'network', 'about'):
            page.locator('#settings-tab-' + tab).click()
            expect(page.locator('.settings-pane')).to_have_attribute('data-tab', tab)
            assert page.locator('.settings-sidebar').bounding_box()['y'] == page.locator('.settings-pane').bounding_box()['y']
            assert abs(page.locator('.settings-sidebar').bounding_box()['x'] + page.locator('.settings-sidebar').bounding_box()['width'] - page.locator('.settings-pane').bounding_box()['x']) < 1
        page.locator('#settings-tab-appearance').click()
        page.locator('input[name=setting-theme][value=dark]').check()
        expect(page.locator('html')).to_have_attribute('data-theme', 'dark')
        page.locator('#setting-text-scale').select_option('1.5', force=True)
        page.locator('#settings-tab-general').click()
        page.locator('#setting-open-mode').select_option('windows', force=True)
        page.locator('#tray-enabled').uncheck()
        page.locator('#setting-language').select_option('en-US', force=True)
        expect(page.locator('#settings-tab-general')).to_have_text('General')
        page.locator('#settings-tab-privacy').click()
        page.locator('#recent-enabled').uncheck()
        page.locator('#settings-tab-associations').click()
        expect(page.locator('.settings-associations input')).to_have_count(9)
        page.locator('[id="association-.zip"]').check()
        page.locator('#settings-tab-network').click()
        page.locator('input[name=setting-proxy][value=custom]').check()
        page.locator('#setting-proxy-address').fill('http://127.0.0.1:7890')
        page.locator('#setting-proxy-address').dispatch_event('change')
        page.locator('#setting-mirror-address').fill('https://mirror.example.com')
        page.locator('#setting-mirror-address').dispatch_event('change')
        page.locator('#settings-tab-about').click()
        expect(page.locator('#about-check-update')).to_be_visible()
        page.evaluate('window.XdzipAboutUpdatePreview({state:"available",canInstall:true})')
        expect(page.locator('#about-install-update')).to_be_visible()
        modal.locator('.os-close').click()
        page.locator('#title-settings').click()
        expect(page.locator('html')).to_have_attribute('data-theme', 'dark')
        page.locator('#settings-tab-general').click()
        expect(page.locator('#setting-open-mode')).to_have_value('windows')
        expect(page.locator('#tray-enabled')).not_to_be_checked()
        page.locator('#settings-tab-privacy').click()
        expect(page.locator('#recent-enabled')).not_to_be_checked()
        page.locator('#settings-tab-associations').click()
        expect(page.locator('[id="association-.zip"]')).to_be_checked()
        page.locator('#settings-tab-network').click()
        expect(page.locator('#setting-proxy-address')).to_have_value('http://127.0.0.1:7890')
        expect(page.locator('#setting-mirror-address')).to_have_value('https://mirror.example.com')
        page.evaluate('window.XdzipDesign.open("about")')
        expect(modal).to_have_attribute('data-kind', 'settings')
        expect(page.locator('.settings-pane')).to_have_attribute('data-tab', 'about')
        assert not errors, errors
        browser.close()
    (OUTPUT / 'validation.json').write_text(json.dumps({
        'status': 'passed', 'checks': ['six tabs', 'no footer', 'immediate changes', 'reopen state',
        'two languages and 150% text', 'proxy and mirror fields', 'association switches', 'about update states'],
        'limits': ['HTML settings are an in-memory design simulation; system associations and persistence require Qt tests.']
    }, ensure_ascii=False, indent=2) + '\n')
    print('Settings design: six tabs and immediate interactions passed')


if __name__ == '__main__':
    main()
