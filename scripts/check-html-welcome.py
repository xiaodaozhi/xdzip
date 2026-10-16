#!/usr/bin/env python3
"""Verify W01 visual anchors and demo navigation, without reading archive bytes."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Thread
import re
import json
import os
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'build/review/html-welcome'
OUT.mkdir(parents=True, exist_ok=True)

class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass

server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
Thread(target=server.serve_forever, daemon=True).start()
base = f'http://127.0.0.1:{server.server_port}/'
checks, errors = [], []

with sync_playwright() as p:
    browser = p.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'), headless=True)
    try:
        for dpr in [1, 1.25, 2]:
            context = browser.new_context(viewport={'width': 1200, 'height': 1000}, device_scale_factor=dpr)
            page = context.new_page()
            page.on('pageerror', lambda error: errors.append(str(error)))
            for view, width, height in [('welcome', 1024, 622), ('welcome-narrow', 832, 544)]:
                for theme in ['light', 'dark']:
                    page.goto(base + f'design/html/index.html?view={view}&theme={theme}&zoom=1')
                    frame = page.frame_locator('#design-frame')
                    expect(frame.locator('#welcome')).to_be_visible()
                    frame.locator('body').evaluate('() => document.fonts.ready')
                    expect(frame.locator('.toolbar')).to_be_hidden()
                    expect(frame.locator('.workspace')).to_be_hidden()
                    expect(frame.locator('.welcome-tab')).to_contain_text('起始页')
                    expect(frame.locator('#selection-status')).to_have_text('文件在本机处理')
                    rects = frame.locator('.window').evaluate('el => {const f=parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--ui-source-factor"));const rect=e=>{const r=e.getBoundingClientRect();return {width:r.width/f,height:r.height/f}};return {window:rect(el),titlebar:rect(el.querySelector(".titlebar")),status:rect(el.querySelector(".statusbar")),cards:[...el.querySelectorAll(".welcome-action")].map(rect),overflow:el.scrollWidth>el.clientWidth}}')
                    assert abs(rects['window']['width'] - width) <= 1 and abs(rects['window']['height'] - height) <= 1, rects
                    assert abs(rects['titlebar']['height'] - 48) <= 1 and abs(rects['status']['height'] - 32) <= 1, rects
                    assert not rects['overflow'], rects
                    assert all(abs(card['width'] - 296) <= 1 and abs(card['height'] - 124) <= 1 for card in rects['cards']), rects
                    frame.locator('.window').screenshot(path=str(OUT / f'{view}-{theme}-dpr{dpr}.png'))
                    page.locator('#reference-toggle').click()
                    if theme == 'light' or view == 'welcome':
                        expect(page.locator('#reference-image')).to_be_visible()
                        page.wait_for_function('document.querySelector("#reference-image").complete && document.querySelector("#reference-image").naturalWidth > 0')
                    else:
                        expect(page.locator('#reference-image')).to_be_hidden()
                    checks.append({'view': view, 'theme': theme, 'dpr': dpr, 'geometry': rects})
            context.close()

        page = browser.new_page(viewport={'width': 1200, 'height': 1000})
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.goto(base + 'design/html/index.html?view=main&zoom=1')
        frame = page.frame_locator('#design-frame')
        expect(frame.locator('#file-rows tr')).to_have_count(9)
        header_metrics = "el=>{const r=el.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height,font:getComputedStyle(el).fontSize}}"
        header_before = {selector: frame.locator(selector).first.evaluate(header_metrics) for selector in ['.titlebar', '.app-brand', '.archive-tab', '#title-settings']}
        page.locator('#view').select_option('welcome')
        expect(frame.locator('#welcome-open')).to_be_visible()
        header_after = {selector: frame.locator(selector).first.evaluate(header_metrics) for selector in header_before}
        assert header_before == header_after, (header_before, header_after)
        checks.append('Welcome reuses main titlebar, brand, tab and settings geometry and fonts unchanged')
        open_button = frame.locator('#welcome-open')
        open_button.hover()
        assert open_button.evaluate('el=>getComputedStyle(el).backgroundColor') == 'rgb(18, 75, 224)'
        page.mouse.down()
        assert open_button.evaluate('el=>getComputedStyle(el).backgroundColor') == 'rgb(11, 56, 181)'
        page.mouse.up()
        expect(frame.locator('#modal')).to_be_visible()
        frame.locator('.os-close').click()
        expect(frame.locator('#welcome')).to_be_visible()
        expect(open_button).to_be_focused()
        frame.locator('#welcome-create').click()
        expect(frame.locator('#modal')).to_have_attribute('data-kind', 'create')
        frame.locator('.os-close').click()
        expect(frame.locator('#welcome')).to_be_visible()
        frame.locator('#welcome-create').focus()
        page.keyboard.press('Enter')
        expect(frame.locator('#modal')).to_have_attribute('data-kind', 'create')
        frame.locator('.os-close').click()
        open_button.focus()
        page.keyboard.press('Meta+o')
        expect(frame.locator('#modal')).to_have_attribute('data-kind', 'picker')
        frame.locator('#pick-sample').click()
        expect(frame.locator('#welcome-opening')).to_be_visible()
        expect(frame.locator('#welcome-opening [role=progressbar]')).not_to_have_attribute('aria-valuenow', '100')
        assert frame.locator('.welcome-content').evaluate('el=>el.inert')
        frame.locator('#welcome-opening-cancel').click()
        page.wait_for_timeout(650)  # A cancelled demo timer must not open its sample later.
        expect(frame.locator('#welcome')).to_be_visible()
        expect(frame.locator('#welcome-opening')).to_have_count(0)
        expect(frame.locator('#welcome-open')).to_be_focused()
        checks.append('Opening overlay cancellation suppresses stale completion and restores welcome focus')
        page.locator('#view').select_option('welcome-narrow')
        expect(frame.locator('body')).to_have_class(re.compile(r'.*welcome-narrow.*'))
        child = next(f for f in page.frames if f.parent_frame)
        child.evaluate('()=>{beginWelcomeOpening("preview.zip",()=>{throw new Error("Stale preset completion")});XdzipDesign.open("main")}')
        expect(frame.locator('#welcome-opening')).to_have_count(0)
        expect(frame.locator('#welcome')).to_be_hidden()
        assert not child.evaluate('document.body.classList.contains("welcome-narrow")')
        assert abs(frame.locator('.window').bounding_box()['width'] - 1024) <= 1
        page.locator('#view').select_option('welcome')
        expect(frame.locator('#welcome-open')).to_be_visible()
        checks.append('Main review preset bypasses demo loading and cancels pending preview callbacks')
        frame.locator('#welcome-open').click()
        frame.locator('#pick-sample').click()
        expect(frame.locator('#welcome')).to_be_hidden()
        expect(page.locator('#view')).to_have_value('main')
        frame.locator('#close-archive').click()
        frame.locator('#close-example-tab').click()
        expect(frame.locator('#welcome')).to_be_visible()
        expect(page.locator('#view')).to_have_value('welcome')
        expect(frame.locator('#welcome-open')).to_be_focused()
        checks.append('Open/create, cancel, focus restore, keyboard and last-tab welcome round trip')

        for names, accepted in [(['sample.tar.gz'], True), (['sample.txt'], False), (['a.zip', 'b.zip'], False)]:
            transfer = frame.locator('body').evaluate_handle('(el,names)=>{const dt=new DataTransfer();for(const name of names)dt.items.add(new File([],name));return dt}', names)
            frame.locator('#welcome').dispatch_event('dragenter', {'dataTransfer': transfer})
            expect(frame.locator('#welcome-drop')).to_have_class('welcome-drop drag-over' if accepted else 'welcome-drop')
            frame.locator('#welcome').dispatch_event('drop', {'dataTransfer': transfer})
            if accepted:
                expect(frame.locator('#modal')).to_have_attribute('data-kind', 'picker')
                frame.locator('.os-close').click()
            else:
                expect(frame.locator('.welcome-feedback')).to_contain_text('请拖入单个')
            transfer.dispose()
        checks.append('Drag accepts a single supported extension, rejects multiple and unsupported files without reading content')
        page.locator('#platform').select_option('windows')
        expect(frame.locator('.welcome-shortcut').first).to_have_text('Ctrl+O')
        expect(frame.locator('.traffic')).to_be_hidden()
        frame.locator('#tasks').click()
        menu = frame.locator('[role=menu]')
        expect(menu).to_be_visible()
        menu_metrics = menu.evaluate('el=>({width:el.getBoundingClientRect().width,font:parseFloat(getComputedStyle(el.querySelector("[role^=menuitem]")||el).fontSize)})')
        assert abs(menu_metrics['width'] * (1024 / 1548) - 240) <= 1 and abs(menu_metrics['font'] * (1024 / 1548) - 13) < .01, menu_metrics
        checks.append({'menu': menu_metrics, 'menu_to_body_font_ratio': 1})
        page.goto(base + 'design/hifi/sources/welcome.html?external=1')
        external = page.frame_locator('iframe')
        expect(external.locator('#welcome-opening')).to_be_visible()
        expect(external.locator('.welcome-content')).to_be_hidden()
        expect(external.locator('.titlebar')).to_be_visible()
        # The hifi wrapper deliberately disables pointer interaction with its iframe.
        external.locator('#welcome-opening-cancel').evaluate('button => button.click()')
        expect(external.locator('.welcome-content')).to_be_visible()
        expect(external.locator('#welcome-opening')).to_have_count(0)
        checks.append('External-open hifi shows only cancellable loading content, then restores welcome on cancellation')
        page.close()

        # No network access is permitted while testing the self-contained delivery.
        page = browser.new_page(viewport={'width': 1200, 'height': 1000})
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.route('http://**/*', lambda route: route.abort())
        page.route('https://**/*', lambda route: route.abort())
        page.goto((ROOT / 'dist/xdzip-welcome-design.html').as_uri() + '?zoom=1')
        offline = page.frame_locator('#design-frame')
        expect(offline.locator('#welcome-open')).to_be_visible()
        offline.locator('#welcome-create').click()
        expect(offline.locator('#modal')).to_have_attribute('data-kind', 'create')
        offline.locator('.os-close').click()
        expect(offline.locator('#welcome-open')).to_be_visible()
        checks.append('Offline bundle welcome/create/cancel with network blocked')
        assert not errors, errors
    finally:
        browser.close()
        server.shutdown()
        (OUT / 'validation.json').write_text(json.dumps({'checks': checks, 'errors': errors, 'browser_zoom': '100%; actual 125% is covered separately by the CI scale suite'}, ensure_ascii=False, indent=2))
print(f'W01: {len(checks)} checks passed')
