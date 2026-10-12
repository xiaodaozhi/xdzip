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
OUTPUT = ROOT / 'docs/qa/html-header'
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
        for selector, expected in spec['dom_rect_targets'].items():
            rect = page.locator(selector).bounding_box()
            actual = [rect[key] for key in ['x', 'y', 'width', 'height']]
            delta = max(abs(a - b) for a, b in zip(actual, expected))
            measurements.append({'kind':'DOM rect', 'selector':selector, 'expected':expected, 'actual':actual, 'maximum_delta_px':delta})
            assert delta <= 1, measurements[-1]
        for selector, expected in spec['dom_center_targets'].items():
            rect = page.locator(selector).bounding_box()
            actual = [rect['x'] + rect['width']/2, rect['y'] + rect['height']/2]
            delta = max(abs(a - b) for a, b in zip(actual, expected))
            measurements.append({'kind':'DOM center', 'selector':selector, 'expected':expected, 'actual':actual, 'maximum_delta_px':delta})
            assert delta <= 1, measurements[-1]
        shot = Image.open(io.BytesIO(page.screenshot())).convert('RGB')
        original = Image.open(source).convert('RGB')
        for name, region in {'active-close':[501,52,26,26], 'inactive-close':[766,52,26,26],
                             'plus':[820,45,32,34], 'settings':[1514,44,36,35],
                             'open':[45,108,36,38], 'new':[170,108,36,38],
                             'test':[316,108,36,38], 'more':[446,117,34,20]}.items():
            expected, actual = ink_bounds(original, region), ink_bounds(shot, region)
            delta = max(abs(a - b) for a, b in zip(actual, expected))
            measurements.append({'kind':'painted ink', 'name':name, 'expected':expected, 'actual':actual, 'maximum_delta_px':delta})
            assert delta <= 2, measurements[-1]
        crop = (20,30,1568,167)
        reference, implementation = original.crop(crop), shot.crop(crop)
        reference.save(OUTPUT / 'reference.png')
        implementation.save(OUTPUT / 'after.png')
        Image.blend(reference, implementation, .5).save(OUTPUT / 'overlay.png')
        ImageChops.difference(reference, implementation).save(OUTPUT / 'difference.png')
        comparison = Image.new('RGB', (1548, 290), '#e2e6ed')
        comparison.paste(reference, (0,0)); comparison.paste(implementation, (0,153))
        comparison.save(OUTPUT / 'comparison.png')
        shot.save(OUTPUT / 'main.png')
        # Record type contours separately; the original font is not recoverable.
        for name, region in {'brand':[173,44,69,38], 'active-label':[345,48,138,33],
                             'inactive-label':[594,48,136,33]}.items():
            measurements.append({'kind':'type contour (review only)', 'name':name,
                'expected':ink_bounds(original, region), 'actual':ink_bounds(shot, region)})
        # Verify actual handlers and close the popup before changing platform state.
        page.locator('#title-settings').click()
        assert page.locator('dialog[open][data-kind="settings"]').count() == 1
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
