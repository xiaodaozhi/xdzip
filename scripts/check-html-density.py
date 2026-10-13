#!/usr/bin/env python3
"""Measure composed main-window/menu proportions under actual browser page zoom."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import tempfile
from threading import Thread

from playwright.sync_api import sync_playwright, expect
from native_browser_zoom import isolated_display, NativeZoom
from html_interaction_contract import verify

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'dist/ui-density/html'


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


def main():
    spec = json.loads((ROOT / 'docs/visual/layout-scale.json').read_text())
    controls = json.loads((ROOT / 'docs/visual/controls.json').read_text())
    reference = json.loads((ROOT / 'docs/visual/qt-main-reference.json').read_text())
    assert spec['main']['previous_logical_width'] * spec['main']['density'] == 1024
    assert spec['main']['logical_window'] == [1024, 622]
    assert reference['logical_width'] == 1024
    assert controls['size']['menuWidth'] == 240 and controls['size']['menuLabelSize'] == 13
    OUTPUT.mkdir(parents=True, exist_ok=True)
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT)))
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()
    checks, environments, errors, external = [], [], [], []
    ratio = 1024 / 1548

    def check(name, actual, expected, tolerance=.06):
        passed = (abs(actual - expected) <= tolerance if isinstance(actual, (int, float)) and isinstance(expected, (int, float)) else actual == expected)
        checks.append(dict(name=name, actual=actual, expected=expected, tolerance=tolerance, passed=passed))

    try:
        with isolated_display() as display, sync_playwright() as playwright:
            keyboard = NativeZoom(display)
            try:
                for dpr in [1, 1.25, 2]:
                    with tempfile.TemporaryDirectory(prefix='xdzip-density-browser-') as profile:
                        context = playwright.chromium.launch_persistent_context(
                            profile, executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
                            headless=False, no_viewport=True,
                            env={**os.environ, 'DISPLAY': display},
                            args=['--no-sandbox', '--disable-dev-shm-usage', '--window-size=1900,1300',
                                  f'--force-device-scale-factor={dpr}'])
                        try:
                            context.route('**/*', lambda route: route.continue_() if route.request.url.startswith(f'http://127.0.0.1:{server.server_port}/') else (external.append(route.request.url), route.abort()))
                            page = context.pages[0]
                            page.on('pageerror', lambda error: errors.append(str(error)))
                            page.goto(f'http://127.0.0.1:{server.server_port}/design/html/index.html?zoom=1')
                            frame = page.frame_locator('#design-frame')
                            expect(frame.locator('#file-rows tr')).to_have_count(9)
                            child = next(f for f in page.frames if f.parent_frame)
                            child.evaluate('document.fonts.ready')
                            for zoom in [1, 1.25]:
                                keyboard.set_zoom(page, dpr, zoom)
                                tag = f'DPR{dpr}-page{zoom}'
                                effective_dpr = page.evaluate('devicePixelRatio')
                                environments.append(dict(case=tag, base_dpr=dpr, browser_zoom=zoom,
                                                         dpr_source="Chromium --force-device-scale-factor",
                                                         effective_dpr=effective_dpr,
                                                         review_zoom=page.locator('#zoom').input_value(),
                                                         pinch_zoom=page.evaluate('visualViewport.scale')))
                                check(tag + '/effective DPR', effective_dpr, dpr * zoom, .001)
                                # Playwright bounds are already in the outer CSS coordinate
                                # system; only computed inner-frame font sizes need conversion.
                                check(tag + '/main width', child.locator('.window').bounding_box()['width'], 1024)
                                check(tag + '/main height', child.locator('.window').bounding_box()['height'], 940 * ratio)
                                title_box = child.locator('.titlebar').bounding_box()
                                tab_box = child.locator('.archive-tab:not(.inactive)').bounding_box()
                                title_spec = spec['main']['titlebar']
                                check(tag + '/title height', title_box['height'], title_spec['height'])
                                check(tag + '/tab height', tab_box['height'], title_spec['tab_height'])
                                check(tag + '/tab top gap', tab_box['y'] - title_box['y'], title_spec['tab_vertical_inset'])
                                check(tag + '/tab bottom gap', title_box['y'] + title_box['height'] - tab_box['y'] - tab_box['height'], title_spec['tab_vertical_inset'])
                                title_center = title_box['y'] + title_box['height'] / 2
                                for selector in ['#close-archive', '#close-example-tab', '#new-tab', '#title-settings']:
                                    box = child.locator(selector).bounding_box()
                                    check(tag + '/center ' + selector, box['y'] + box['height'] / 2, title_center, .5)
                                table_font = child.locator('#file-rows [data-id=lake] .entry-name').evaluate('(n)=>parseFloat(getComputedStyle(n).fontSize)') * ratio
                                check(tag + '/table font', table_font, 19 * ratio)
                                verify(child, ratio, check, tag, OUTPUT if dpr == 1 and zoom == 1 else None)
                                if dpr == 1 and zoom == 1:
                                    frame.locator('.window').screenshot(path=str(OUTPUT / 'main.png'))
                                frame.locator('#more-actions').click()
                                expect(frame.locator('.ui-menu')).to_have_count(1)
                                for kind in ['dropdown', 'submenu', 'context']:
                                    if kind == 'submenu':
                                        frame.locator('[data-command=more-view]').focus()
                                        page.keyboard.press('ArrowRight')
                                        expect(frame.locator('.ui-menu')).to_have_count(2)
                                    elif kind == 'context':
                                        page.keyboard.press('Escape')
                                        page.keyboard.press('Escape')
                                        frame.locator('#file-rows [data-id=lake]').click(button='right')
                                        expect(frame.locator('.ui-menu')).to_have_count(1)
                                    menu = child.locator('.ui-menu').last
                                    menu_font = menu.evaluate('(n)=>parseFloat(getComputedStyle(n).fontSize)') * ratio
                                    check(tag + '/' + kind + ' width', menu.bounding_box()['width'], 240)
                                    check(tag + '/' + kind + ' font', menu_font, 13)
                                    proportion = menu_font / table_font
                                    low, high = spec['verification']['menu_to_table_font_ratio']
                                    checks.append(dict(name=tag + '/' + kind + ' menu/table font ratio', actual=proportion,
                                                       expected=[low, high], passed=low <= proportion <= high))
                                    first = menu.locator('.ui-menu-item').first
                                    check(tag + '/' + kind + ' first row height', first.bounding_box()['height'], 32)
                                    if dpr == 1 and zoom == 1:
                                        frame.locator('.window').screenshot(path=str(OUTPUT / f'{kind}.png'))
                                page.keyboard.press('Escape')
                                expect(frame.locator('.ui-menu')).to_have_count(0)
                                operation_ratio = spec['operation']['source_to_logical'][0] / spec['operation']['source_to_logical'][1]
                                for view in ['extract', 'progress', 'settings']:
                                    page.locator('#view').select_option(view)
                                    expect(child.locator('dialog[open]')).to_have_attribute('data-kind', view)
                                    check(tag + '/operation width ' + view, child.locator('dialog[open]').bounding_box()['width'], spec['operation']['logical_widths'].get(view, spec['operation']['logical_widths']['default']), 1)
                                    for selector, expected in [('.modal-footer .button', spec['operation']['footer']['fontSize']), ('.modal-footer .ready-label,.modal-footer .info-line', spec['operation']['footer']['noteSize'])]:
                                        fonts = child.locator('dialog[open] ' + selector).evaluate_all('(nodes)=>nodes.map(n=>parseFloat(getComputedStyle(n).fontSize))')
                                        for i, size in enumerate(fonts):
                                            check(tag + f'/operation footer {view}/{selector}/{i}', size * operation_ratio, expected)
                                page.locator('#view').select_option('main')
                                expect(child.locator('#file-rows tr')).to_have_count(9)
                        finally:
                            context.close()
            finally:
                keyboard.close()
    finally:
        server.shutdown()
        server.server_close()
        thread.join()
    report = dict(method='Private Xvfb + XTest native Ctrl+0/Ctrl+=; actual Chromium page zoom, no CSS zoom or pinch emulation.',
                  checks=checks, environments=environments, page_errors=errors, external_requests=external,
                  passed=sum(c['passed'] for c in checks), failed=sum(not c['passed'] for c in checks))
    (OUTPUT / 'validation.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    for item in checks:
        if not item['passed']:
            print(json.dumps(item, ensure_ascii=True))
    print(f"Actual browser density: {report['passed']} passed, {report['failed']} failed")
    if report['failed'] or errors or external:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
