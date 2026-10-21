#!/usr/bin/env python3
"""Compare HTML header geometry and painted icons with independent source anchors."""
from pathlib import Path
import hashlib
import io
import json
import os
from PIL import Image, ImageChops
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'build/review/html-header'
OUTPUT.mkdir(parents=True, exist_ok=True)
spec = json.loads((ROOT / 'design/html/header-reference.json').read_text())
source = ROOT / spec['source']
assert hashlib.sha256(source.read_bytes()).hexdigest() == spec['sha256'], 'Reference changed: remeasure it explicitly'
errors, measurements = [], []

def ink_bounds(picture, region):
    """Measure actual dark strokes within isolated icon regions, not SVG boxes."""
    x, y, width, height = region
    crop = picture.crop((x, y, x + width, y + height)).convert('RGB')
    mask = Image.new('L', crop.size)
    pixels = crop.load()
    mask.putdata([255 if sum(pixels[column, row]) / 3 < 180 else 0
                  for row in range(crop.height) for column in range(crop.width)])
    bounds = mask.getbbox()
    assert bounds, region
    left, top, right, bottom = bounds
    return [x + left, y + top, right - left, bottom - top]

def white_ink_bounds(picture, region):
    """Isolate the white chevron from the blue split-button background."""
    x, y, width, height = region
    crop = picture.crop((x, y, x + width, y + height)).convert('RGB')
    pixels = crop.load()
    mask = Image.new('L', crop.size)
    mask.putdata([255 if min(pixels[column, row]) > 180
                  and max(pixels[column, row]) - min(pixels[column, row]) < 80 else 0
                  for row in range(height) for column in range(width)])
    left, top, right, bottom = mask.getbbox()
    return [x + left, y + top, right - left, bottom - top]

with sync_playwright() as playwright:
    browser = playwright.chromium.launch(
        executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
        args=['--no-sandbox', '--disable-dev-shm-usage'])
    try:
        outer = browser.new_page()
        outer.route('**/*', lambda route: route.abort())
        outer.set_content((ROOT / 'dist/xdzip-hifi-design.html').read_text())
        # Render the exact embedded canvas at reference scale, without reviewer zoom.
        canvas = outer.evaluate('window.XDZIP_CANVAS')
        page = browser.new_page(viewport={'width': 1588, 'height': 1000}, device_scale_factor=1)
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.route('**/*', lambda route: route.abort())
        page.set_content(canvas)
        page.evaluate('document.fonts.ready')
        assert page.evaluate('document.fonts.check(\'600 20px "Xdzip Design Sans SC"\', "北境旅行.zip")')
        assert page.evaluate('[...document.fonts].some(f => f.family === "Xdzip Design Sans SC" && f.status === "loaded")')
        layout = json.loads((ROOT / 'docs/visual/layout-scale.json').read_text())
        title = layout['main']['titlebar']
        factor = 1548 / 1024
        title_delta = title['height'] * factor - 57
        center_y = 31 + title['height'] * factor / 2
        active_tab = spec['dom_rect_targets']['.archive-tab:not(.inactive)']
        tab_step = active_tab[2] + title['tab_gap'] * factor
        for selector, expected in spec['dom_rect_targets'].items():
            expected = list(expected)
            if selector.startswith('.archive-tab'):
                expected[1] = center_y - title['tab_height'] * factor / 2
                expected[3] = title['tab_height'] * factor
                if selector == '.archive-tab.inactive':
                    expected[0] = active_tab[0] + tab_step
                    expected[2] = active_tab[2]
            elif selector != '.window':
                expected[1] += title_delta
            rect = page.locator(selector).bounding_box()
            actual = [rect[key] for key in ['x', 'y', 'width', 'height']]
            delta = max(abs(a - b) for a, b in zip(actual, expected))
            measurements.append({'kind':'DOM rect', 'selector':selector, 'expected':expected, 'actual':actual, 'maximum_delta_px':delta})
            assert delta <= 1, measurements[-1]
        for selector, expected in spec['dom_center_targets'].items():
            x = expected[0]
            if selector == '#close-example-tab svg':
                x = spec['dom_center_targets']['#close-archive svg'][0] + tab_step
            elif selector == '#new-tab svg':
                x = active_tab[0] + 2 * tab_step + 20
            expected = [x, center_y]
            rect = page.locator(selector).bounding_box()
            actual = [rect['x'] + rect['width']/2, rect['y'] + rect['height']/2]
            delta = max(abs(a - b) for a, b in zip(actual, expected))
            measurements.append({'kind':'DOM center', 'selector':selector, 'expected':expected, 'actual':actual, 'maximum_delta_px':delta})
            assert delta <= 1, measurements[-1]
        shot = Image.open(io.BytesIO(page.screenshot())).convert('RGB')
        original = Image.open(source).convert('RGB')
        arrow = spec['extract_chevron']
        expected = white_ink_bounds(original, arrow['region'])
        assert expected == arrow['white_ink_bounds'], 'Chevron reference requires remeasurement'
        region = list(arrow['region'])
        region[1] += round(title_delta)
        actual = white_ink_bounds(shot, region)
        expected[1] += title_delta
        delta = max(abs(a-b) for a,b in zip(actual,expected))
        measurements.append({'kind':'painted white ink', 'name':'extract-chevron',
            'expected':expected, 'actual':actual, 'maximum_delta_px':delta})
        assert delta <= 1, measurements[-1]
        # Capsule geometry supersedes the archived concave shoulder, without rewriting source pixels.
        for selector in ['.archive-tab:not(.inactive)', '.archive-tab.inactive']:
            radius = page.locator(selector).evaluate('(n)=>parseFloat(getComputedStyle(n).borderTopLeftRadius)')
            assert abs(radius - title['tab_radius'] * factor) < .05
            assert page.locator(selector).evaluate('(n)=>getComputedStyle(n,"::before").content') in ['none', 'normal']
        for name, region in {'active-close':[501,52,26,26], 'inactive-close':[766,52,26,26],
                             'plus':[820,45,32,34], 'settings':[1514,44,36,35],
                             'open':[45,108,36,38], 'new':[170,108,36,38],
                             'test':[316,108,36,38], 'more':[446,117,34,20]}.items():
            expected = ink_bounds(original, region)
            target_region = list(region)
            dx = (spec['dom_center_targets']['#close-archive svg'][0] + tab_step -
                  spec['dom_center_targets']['#close-example-tab svg'][0]) if name == 'inactive-close' else (
                  active_tab[0] + 2 * tab_step + 20 - spec['dom_center_targets']['#new-tab svg'][0]) if name == 'plus' else 0
            dy = title_delta if name in ['open', 'new', 'test', 'more'] else center_y - (expected[1] + expected[3] / 2)
            target_region[0] += round(dx)
            target_region[1] += round(dy)
            if name == 'inactive-close':
                page.locator('.archive-tab.inactive').hover()
                hovered = Image.open(io.BytesIO(page.screenshot())).convert('RGB')
                actual = ink_bounds(hovered, target_region)
                page.mouse.move(0, 0)
            else:
                actual = ink_bounds(shot, target_region)
            expected[0] += dx
            expected[1] += dy
            delta = max(abs(a - b) for a, b in zip(actual, expected))
            measurements.append({'kind':'painted ink', 'name':name, 'expected':expected, 'actual':actual, 'maximum_delta_px':delta})
            assert delta <= 2, measurements[-1]
        crop = (20,30,1568,round(167 + title_delta))
        reference, implementation = original.crop(crop), shot.crop(crop)
        reference.save(OUTPUT / 'reference.png')
        implementation.save(OUTPUT / 'after.png')
        Image.blend(reference, implementation, .5).save(OUTPUT / 'overlay.png')
        ImageChops.difference(reference, implementation).save(OUTPUT / 'difference.png')
        comparison = Image.new('RGB', (1548, reference.height * 2 + 16), '#e2e6ed')
        comparison.paste(reference, (0,0)); comparison.paste(implementation, (0,reference.height + 16))
        comparison.save(OUTPUT / 'comparison.png')
        for name, box in {'tabs':(268,0,854,60), 'settings':(1472,0,1548,137),
                          'tab-right-shoulder':(504,37,548,68),
                          'extract-chevron':(1480,70,1540,125)}.items():
            first, second = reference.crop(box), implementation.crop(box)
            detail = Image.new('RGB', (first.width,first.height*2+12), '#e2e6ed')
            detail.paste(first, (0,0)); detail.paste(second, (0,first.height+12))
            detail.save(OUTPUT / f'{name}-comparison.png')
        shot.save(OUTPUT / 'main.png')
        # Record type contours separately; the original font is not recoverable.
        for name, region in {'brand':[173,44,69,38], 'active-label':[345,48,138,33],
                             'inactive-label':[594,48,136,33]}.items():
            measurements.append({'kind':'type contour (review only)', 'name':name,
                'expected':ink_bounds(original, region), 'actual':None, 'note':'Floating capsule supersedes archived title text positions'})
        # Verify actual handlers and close the popup before changing platform state.
        page.locator('#extract-menu-button').click()
        assert page.locator('#extract-menu').is_visible()
        page.locator('[data-extract="selected"]').click()
        assert page.locator('dialog[open][data-kind="extract"]').count() == 1
        page.keyboard.press('Escape')
        assert page.locator('dialog[open][data-kind=extract]').count() == 1
        page.keyboard.press('Escape')
        page.locator('#title-settings').click()
        assert page.locator('dialog[open][data-kind="settings"]').count() == 1
        page.keyboard.press('Escape')
        assert page.locator('dialog[open][data-kind=settings]').count() == 1
        page.keyboard.press('Escape')
        page.locator('#new-tab').click()
        assert page.locator('dialog[open]').count() == 1
        assert not errors, errors
    finally:
        browser.close()
        (OUTPUT / 'measurements.json').write_text(json.dumps({
            'source_sha256':spec['sha256'], 'viewport':[1588,1000], 'device_scale_factor':1,
            'font':'Xdzip Design Sans SC embedded variable subset',
            'measurements':measurements, 'page_errors':errors,
            'limits':['Source font is unknown; antialiasing and generated highlights still differ.',
                      'Geometry and isolated icon checks do not prove full-page visual fidelity.']},
            ensure_ascii=False,indent=2)+'\n')
print(f'{len(measurements)} source-anchor comparisons passed; actual painted icon strokes verified')
