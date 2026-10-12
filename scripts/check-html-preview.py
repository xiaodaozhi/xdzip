#!/usr/bin/env python3
"""Check preview fit and reachable actions in the offline desktop artboard."""
from pathlib import Path
import json
import os
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'dist/html-preview-review'
OUTPUT.mkdir(parents=True, exist_ok=True)
METRICS = '''() => {
    const panel = document.querySelector('#inspector');
    const button = document.querySelector('#inspector-extract');
    const rect = panel.getBoundingClientRect();
    const action = button.getBoundingClientRect();
    return {width: panel.clientWidth, scrollWidth: panel.scrollWidth,
        height: panel.clientHeight, scrollHeight: panel.scrollHeight,
        buttonVisible: action.bottom <= rect.top + panel.clientHeight
            && action.top >= rect.top};
}'''
checks, errors = [], []
with sync_playwright() as playwright:
    browser = playwright.chromium.launch(
        executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
        args=['--no-sandbox', '--disable-dev-shm-usage'])
    try:
        for width in [1588, 1200, 390]:
            page = browser.new_page(viewport={'width': width, 'height': 1000})
            page.on('pageerror', lambda error: errors.append(str(error)))
            page.route('**/*', lambda route: route.abort())
            page.set_content((ROOT / 'dist/xdzip-hifi-design.html').read_text())
            panel = page.frame_locator('#design-frame').locator('#inspector')
            expect(panel.locator('#inspector-extract')).to_be_visible()
            frame = next(frame for frame in page.frames if frame.parent_frame)
            for classic in [False, True]:
                # Reserve classic scrollbar space without relying on host settings.
                frame.eval_on_selector('#inspector', '(p, enabled) => p.style.scrollbarGutter = enabled ? "stable" : "auto"', classic)
                for theme in ['light', 'dark']:
                    page.select_option('#theme', theme)
                    expect(panel.locator('h2')).to_have_text('冰川湖.jpg')
                    metrics = frame.evaluate(METRICS)
                    assert metrics['scrollWidth'] <= metrics['width'], metrics
                    assert metrics['scrollHeight'] <= metrics['height'], metrics
                    assert metrics['buttonVisible'], metrics
                    checks.append({'outerWidth': width, 'classicGutter': classic,
                        'theme': theme, **metrics})
            if width == 1588:
                page.select_option('#theme', 'light')
                frame.eval_on_selector('#inspector', '(p) => p.style.scrollbarGutter = "auto"')
                panel.screenshot(path=str(OUTPUT / 'preview.png'))
                frame.locator('.stage').screenshot(path=str(OUTPUT / 'main.png'))
                frame.eval_on_selector('#inspector', '(p) => p.style.scrollbarGutter = "stable"')
                # Exercise other metadata layouts through actual row selection.
                for row in ['guide', 'photos']:
                    frame.locator(f'tr[data-id="{row}"]').click()
                    metrics = frame.evaluate(METRICS)
                    assert metrics['scrollWidth'] <= metrics['width'], metrics
                    assert metrics['buttonVisible'], metrics
                    checks.append({'selection': row, **metrics})
                frame.locator('tr[data-id="guide"] input').check()
                expect(panel.locator('h2')).to_have_text('13 个文件')
                metrics = frame.evaluate(METRICS)
                assert metrics['scrollWidth'] <= metrics['width'], metrics
                assert metrics['buttonVisible'], metrics
                checks.append({'selection': 'multiple files', **metrics})
                frame.locator('tr[data-id="photos"] input').uncheck()
                frame.locator('tr[data-id="guide"] input').uncheck()
                expect(panel.locator('h2')).to_have_text('北境旅行.zip')
                metrics = frame.evaluate(METRICS)
                assert metrics['scrollWidth'] <= metrics['width'], metrics
                assert metrics['buttonVisible'], metrics
                checks.append({'selection': 'archive metadata', **metrics})
                # Long names and metadata may scroll vertically, but actions remain reachable.
                frame.eval_on_selector('#inspector h2', '(e) => e.textContent = "长文件名".repeat(60) + ".pdf"')
                frame.eval_on_selector('.property dd', '(e) => e.textContent = "长属性值".repeat(40)')
                frame.eval_on_selector('#inspector', '(p) => {p.style.scrollbarGutter="stable"; p.scrollTop=p.scrollHeight}')
                metrics = frame.evaluate(METRICS)
                assert metrics['scrollWidth'] <= metrics['width'], metrics
                assert metrics['buttonVisible'], metrics
                checks.append({'selection': 'long metadata', **metrics})
                frame.locator('#inspector-extract').click()
                expect(frame.locator('dialog[open]')).to_be_visible()
            page.close()
        assert not errors, errors
    finally:
        browser.close()
        (OUTPUT / 'validation.json').write_text(json.dumps(
            {'checks': checks, 'page_errors': errors,
             'scope': 'Offline Chromium layout; classic scrollbar space simulated'},
            ensure_ascii=False, indent=2) + '\n')
print(f'{len(checks)} preview layout checks passed')
