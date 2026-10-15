#!/usr/bin/env python3
"""Measure the area below the toolbar against independently recorded source anchors.

The reference file contains independent source-image measurements and explicitly
authored design revisions. Revisions keep the original comparison as evidence;
they never change the approved image or silently relax its tolerance.
A failed comparison still writes images and measurements for visual review.
Text contours are review evidence: the raster reference's original font is unknown.
"""
from ui_terminology import revised_source_label
from pathlib import Path
import argparse
import hashlib
import io
import json
import os
from PIL import Image, ImageChops
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
BODY_CROP = (20, 167, 1568, 970)
DEFAULT_REGIONS = {
    'full': {'label': '全部内容区域', 'rect': [20, 167, 1548, 803]},
    'sidebar': {'label': '左侧归档与目录树', 'rect': [20, 167, 292, 749]},
    'list': {'label': '路径、文件列表与选择状态', 'rect': [312, 167, 866, 749]},
    'preview': {'label': '右侧文件预览', 'rect': [1178, 167, 390, 749]},
    'status': {'label': '底部状态栏', 'rect': [20, 916, 1548, 54]},
}


def painted_bounds(picture, region, mode='dark'):
    """Find visible ink within a deliberately isolated source-image region."""
    x, y, width, height = map(int, region)
    crop = picture.crop((x, y, x + width, y + height)).convert('RGB')
    if mode == 'blue':
        predicate = lambda rgb: rgb[2] > 140 and rgb[2] - rgb[0] > 40 and rgb[2] - rgb[1] > 15
    elif mode == 'white':
        predicate = lambda rgb: min(rgb) > 180 and max(rgb) - min(rgb) < 80
    elif mode == 'red':
        predicate = lambda rgb: rgb[0] > 150 and rgb[0] - rgb[1] > 70 and rgb[0] - rgb[2] > 50
    elif mode == 'gray':
        # Include the source's pale paper outlines while excluding white fill.
        predicate = lambda rgb: min(rgb) > 70 and max(rgb) < 240 and max(rgb) - min(rgb) < 50
    elif mode == 'rule':
        predicate = lambda rgb: min(rgb) > 200 and max(rgb) < 251 and max(rgb) - min(rgb) < 14
    else:
        predicate = lambda rgb: sum(rgb) / 3 < 180
    mask = Image.new('L', crop.size)
    mask.putdata([255 if predicate(rgb) else 0 for rgb in crop.getdata()])
    bounds = mask.getbbox()
    if bounds is None:
        return None
    left, top, right, bottom = bounds
    return [x + left, y + top, right - left, bottom - top]


def record_geometry(measurements, failures, kind, name, expected, actual, tolerance):
    delta = None if actual is None else max(abs(a - b) for a, b in zip(actual, expected))
    passed = delta is not None and delta <= tolerance
    item = {'kind': kind, 'name': name, 'expected': expected, 'actual': actual,
            'maximum_delta_px': delta, 'tolerance_px': tolerance, 'passed': passed}
    measurements.append(item)
    if not passed:
        failures.append(f'{name}: {delta if delta is not None else "missing"} px; tolerance {tolerance} px')


def normalize_target(value, key, default_tolerance):
    if isinstance(value, dict):
        return value[key], value.get('tolerance', default_tolerance)
    return value, default_tolerance


def review_regions(spec):
    regions = dict(DEFAULT_REGIONS)
    for name, value in spec.get('review_regions', {}).items():
        regions[name] = value if isinstance(value, dict) else {'label': name, 'rect': value}
        regions[name].setdefault('label', name)
    return regions


def save_images(output, original, shot, regions):
    reference, implementation = original.crop(BODY_CROP), shot.crop(BODY_CROP)
    reference.save(output / 'reference.png')
    implementation.save(output / 'after.png')
    Image.blend(reference, implementation, .5).save(output / 'overlay.png')
    ImageChops.difference(reference, implementation).save(output / 'difference.png')
    comparison = Image.new('RGB', (reference.width, reference.height * 2 + 12), '#e2e6ed')
    comparison.paste(reference, (0, 0))
    comparison.paste(implementation, (0, reference.height + 12))
    comparison.save(output / 'comparison.png')
    for name, region in regions.items():
        if name == 'full':
            continue
        x, y, width, height = map(int, region['rect'])
        box = (x, y, x + width, y + height)
        first, second = original.crop(box), shot.crop(box)
        detail = Image.new('RGB', (width, height * 2 + 12), '#e2e6ed')
        detail.paste(first, (0, 0))
        detail.paste(second, (0, height + 12))
        detail.save(output / f'{name}-comparison.png')
    shot.save(output / 'main.png')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, default=ROOT / 'design/html/body-reference.json')
    parser.add_argument('--output', type=Path, default=ROOT / 'build/review/html-body')
    parser.add_argument('--bundle', type=Path, default=ROOT / 'dist/xdzip-hifi-design.html')
    parser.add_argument('--rect-tolerance', type=float, default=2)
    parser.add_argument('--center-tolerance', type=float, default=2)
    parser.add_argument('--ink-tolerance', type=float, default=2)
    args = parser.parse_args()
    spec = json.loads(args.reference.read_text())
    if not any(spec.get(key) for key in ['dom_rect_targets', 'dom_center_targets', 'ink_regions']):
        raise SystemExit('The reference must contain at least one independently measured geometry or ink target')
    source = ROOT / spec['source']
    if hashlib.sha256(source.read_bytes()).hexdigest() != spec['sha256']:
        raise SystemExit('Reference changed: remeasure it explicitly before running this check')
    args.output.mkdir(parents=True, exist_ok=True)
    original = Image.open(source).convert('RGB')
    measurements, failures, page_errors, requests = [], [], [], []
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(
            executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
            args=['--no-sandbox', '--disable-dev-shm-usage'])
        try:
            outer = browser.new_page()
            outer.route('**/*', lambda route: route.abort())
            outer.set_content(args.bundle.read_text())
            canvas = outer.evaluate('window.XDZIP_CANVAS')
            page = browser.new_page(viewport={'width': 1588, 'height': 1000}, device_scale_factor=1)
            page.on('pageerror', lambda error: page_errors.append(str(error)))

            def block_external(route):
                requests.append(route.request.url)
                route.abort()

            page.route('**/*', block_external)
            page.set_content(canvas)
            page.evaluate('document.fonts.ready')
            loaded_font = page.evaluate('[...document.fonts].some(f => f.family === "Xdzip Design Sans SC" && f.status === "loaded")')
            if not loaded_font:
                failures.append('Embedded comparison font did not load')
            # Approved floating-title revision moves top-anchored content; footer stays fixed.
            layout = json.loads((ROOT / 'docs/visual/layout-scale.json').read_text())
            title_delta = layout['main']['titlebar']['height'] * 1548 / 1024 - 57
            for selector, target in spec.get('dom_rect_targets', {}).items():
                expected, tolerance = normalize_target(target, 'rect', args.rect_tolerance)
                tolerance = spec.get('dom_rect_tolerances', {}).get(selector, tolerance)
                locator = page.locator(selector)
                rect = locator.bounding_box() if locator.count() else None
                actual = [rect[key] for key in ['x', 'y', 'width', 'height']] if rect else None
                # Preserve source measurements; revised controls follow the logical interaction contract.
                if selector in ['.file-tools .view-active', '.inspector-head .icon-button']:
                    measurements.append({'kind': 'source rect (design revised)', 'name': selector,
                                         'expected': expected, 'actual': actual, 'review_only': True,
                                         'design_revision': 'qt-interaction-sync'})
                    factor = 1548 / 1024
                    size = layout['main']['interaction']['iconButtonSize'] * factor
                    header_center = 167 + 69 / 2
                    if selector == '.file-tools .view-active':
                        expected = [1178 - 26 - 3 * size - 2 * 6.4 * factor,
                                    header_center - size / 2, size, size]
                    else:
                        expected = [1568 - 18 - size, header_center - size / 2, size, size]
                    kind = 'logical interaction rect'
                revision = spec.get('dom_rect_revisions', {}).get(selector)
                kind = 'logical interaction rect' if selector in ['.file-tools .view-active', '.inspector-head .icon-button'] else 'DOM rect'
                if revision:
                    if revision['design_revision'] not in spec.get('design_revisions', {}):
                        failures.append(f'{selector}: unrecorded rect revision')
                        continue
                    measurements.append({'kind': 'source rect (design revised)', 'name': selector,
                                         'expected': expected, 'actual': actual, 'review_only': True,
                                         'design_revision': revision['design_revision']})
                    expected = revision['target']
                    kind = 'revised design rect'
                expected = list(expected)
                if selector not in ['.storage-bar', '.statusbar']:
                    expected[1] += title_delta
                    if len(expected) == 4 and selector in ['.sidebar', '.files-pane', '.inspector']:
                        expected[3] -= title_delta
                record_geometry(measurements, failures, kind, selector, expected, actual, tolerance)
            for selector, target in spec.get('dom_center_targets', {}).items():
                expected, tolerance = normalize_target(target, 'center', args.center_tolerance)
                tolerance = spec.get('dom_center_tolerances', {}).get(selector, tolerance)
                locator = page.locator(selector)
                rect = locator.bounding_box() if locator.count() else None
                actual = [rect['x'] + rect['width'] / 2, rect['y'] + rect['height'] / 2] if rect else None
                revision = None if selector == '.inspector-head .icon-button' else spec.get('dom_center_revisions', {}).get(selector)
                kind = 'DOM center'
                if selector == '.inspector-head .icon-button':
                    measurements.append({'kind': 'source center (design revised)', 'name': selector, 'expected': expected, 'actual': actual, 'review_only': True, 'design_revision': 'qt-interaction-sync'})
                    expected = [1568 - 18 - layout['main']['interaction']['iconButtonSize'] * 1548 / 1024 / 2, 167 + 69 / 2]
                    kind = 'logical interaction center'
                if revision:
                    if revision['design_revision'] not in spec.get('design_revisions', {}):
                        failures.append(f'{selector}: unrecorded center revision')
                        continue
                    measurements.append({'kind': 'source center (design revised)', 'name': selector,
                                         'expected': expected, 'actual': actual, 'review_only': True,
                                         'design_revision': revision['design_revision']})
                    expected = revision['target']
                    kind = 'revised design center'
                expected = list(expected)
                if selector not in ['.storage-bar', '.statusbar']:
                    expected[1] += title_delta
                    if len(expected) == 4 and selector in ['.sidebar', '.files-pane', '.inspector']:
                        expected[3] -= title_delta
                record_geometry(measurements, failures, kind, selector, expected, actual, tolerance)
            shot = Image.open(io.BytesIO(page.screenshot())).convert('RGB')
            save_images(args.output, original, shot, review_regions(spec))
            for item in spec.get('ink_regions', []):
                mode = item.get('mode', 'dark')
                expected = painted_bounds(original, item['region'], mode)
                region = list(item.get('target_region', item['region']))
                moving = item['region'][1] < 915
                if moving:
                    region[1] += title_delta
                actual = painted_bounds(shot, region, mode)
                if moving and expected:
                    expected[1] += title_delta
                if expected is None:
                    failures.append(f'{item["name"]}: source ink region is empty; remeasure it')
                    continue
                if item.get('interaction_revision'):
                    measurements.append({'kind': 'source ink (design revised)', 'name': item['name'],
                                         'expected': expected, 'actual': actual, 'review_only': True,
                                         'design_revision': item['interaction_revision'],
                                         'validation': 'check-html-density.py: logical slot dimensions and icon centers'})
                    continue
                revision = item.get('design_revision')
                if revision:
                    if revision not in spec.get('design_revisions', {}):
                        failures.append(f'{item["name"]}: unrecorded design revision {revision}')
                        continue
                    source_actual = painted_bounds(shot, item['region'], mode)
                    measurements.append({
                        'kind': 'source ink (design revised)', 'name': item['name'],
                        'expected': expected, 'actual': source_actual, 'review_only': True,
                        'maximum_delta_px': None if source_actual is None else max(abs(a - b) for a, b in zip(source_actual, expected)),
                        'design_revision': revision,
                        'original_tolerance_px': item.get('tolerance', args.ink_tolerance),
                    })
                    record_geometry(measurements, failures, 'revised design ink', item['name'],
                                    [v + title_delta if i == 1 and moving else v for i, v in enumerate(item['target_ink_rect'])], actual, item.get('tolerance', args.ink_tolerance))
                else:
                    record_geometry(measurements, failures, 'painted ink', item['name'], expected, actual,
                                    item.get('tolerance', args.ink_tolerance))
            for item in spec.get('type_regions', []):
                measurements.append({'kind': 'type contour (review only)', 'name': item['name'],
                                     'expected': painted_bounds(original, item['region']),
                                     'actual': painted_bounds(shot, item['region']), 'review_only': True})
            for item in spec.get('color_samples', []):
                point = tuple(item['point'])
                expected, actual = original.getpixel(point), shot.getpixel(point)
                if item.get('interaction_revision'):
                    measurements.append({'kind': 'source ink (design revised)', 'name': item['name'],
                                         'expected': expected, 'actual': actual, 'review_only': True,
                                         'design_revision': item['interaction_revision'],
                                         'validation': 'check-html-density.py: logical slot dimensions and icon centers'})
                    continue
                revision = item.get('design_revision')
                kind = 'RGB sample'
                if revision:
                    if revision not in spec.get('design_revisions', {}):
                        failures.append(f'{item["name"]}: unrecorded color sample revision')
                        continue
                    measurements.append({'kind': 'source RGB sample (design revised)', 'name': item['name'],
                                         'expected': expected, 'actual': actual, 'review_only': True,
                                         'point': list(point), 'design_revision': revision})
                    actual = shot.getpixel(tuple(item['target_point']))
                    kind = 'revised design RGB sample'
                sample = list(item.get('target_point', item['point']))
                if sample[1] < 915:
                    sample[1] += title_delta
                actual = shot.getpixel(tuple(round(v) for v in sample))
                record_geometry(measurements, failures, kind, item['name'], expected, actual,
                                item.get('tolerance', 6))
            for selector, expected in spec.get('text_targets', {}).items():
                historical_expected = expected
                expected = revised_source_label(expected)
                values = page.locator(selector).all_text_contents()
                actual = [value.strip() for value in values]
                if isinstance(expected, str):
                    actual = actual[0] if len(actual) == 1 else actual
                passed = actual == expected
                measurements.append({'kind': 'source text' if expected == historical_expected else 'approved copy revision',
                                     'name': selector, 'historical_expected': historical_expected,
                                     'expected': expected, 'actual': actual, 'passed': passed})
                if not passed:
                    failures.append(f'{selector}: source label/order differs: {actual!r}')
            for row_id, kind, date in spec.get('row_metadata', []):
                row = page.locator(f'tr[data-id="{row_id}"]')
                actual = ([row.locator('.type-col').inner_text(), row.locator('.date-col').inner_text()]
                          if row.count() else None)
                passed = actual == [kind, date]
                measurements.append({'kind': 'source row metadata', 'name': row_id,
                                     'expected': [kind, date], 'actual': actual, 'passed': passed})
                if not passed:
                    failures.append(f'{row_id}: source type/date differs: {actual!r}')
            # Keep data semantics correct where the image contains contradictory states.
            semantics = page.evaluate('''() => ({
                partialSelection: document.querySelector('#select-all').indeterminate,
                savings: document.querySelector('#saving').textContent.trim(),
                savingsFill: document.querySelector('#saving-bar').getBoundingClientRect().width /
                    document.querySelector('.storage-bar').getBoundingClientRect().width
            })''')
            semantic_passed = (semantics['partialSelection'] and semantics['savings'] == '26%'
                               and abs(semantics['savingsFill'] - .26) < .005)
            measurements.append({'kind': 'semantic exceptions', 'name': 'partial selection and savings',
                                 'actual': semantics, 'passed': semantic_passed})
            if not semantic_passed:
                failures.append(f'Functional semantic exceptions regressed: {semantics}')
            # A visual capture can reveal scrollbars that element geometry alone misses.
            for selector in spec.get('no_overflow_selectors', []):
                overflow = page.locator(selector).evaluate('(el) => ({horizontal:el.scrollWidth-el.clientWidth,vertical:el.scrollHeight-el.clientHeight})')
                passed = overflow['horizontal'] <= 1 and overflow['vertical'] <= 1
                measurements.append({'kind': 'overflow', 'name': selector, 'expected': {'horizontal': 0, 'vertical': 0},
                                     'actual': overflow, 'passed': passed})
                if not passed:
                    failures.append(f'{selector}: content overflows {overflow}')
        finally:
            browser.close()
    if page_errors:
        failures.append('Browser page errors were recorded')
    if requests:
        failures.append('The offline canvas requested external resources')
    regions = review_regions(spec)
    report = {'source_sha256': spec['sha256'], 'source': spec['source'], 'viewport': [1588, 1000],
              'device_scale_factor': 1, 'body_crop': list(BODY_CROP),
              'font': 'Xdzip Design Sans SC embedded variable subset',
              'source_measurements': spec, 'review_regions': regions,
              'measurements': measurements, 'page_errors': page_errors, 'blocked_requests': requests,
              'passed': not failures, 'failures': failures,
              'limits': ['Source font is unknown; type contours are review evidence, not exact-font assertions.',
                         'Geometry and isolated ink checks do not prove pixel-perfect full-body fidelity.',
                         'The difference image includes antialiasing, source glow, shading and font differences.']}
    (args.output / 'measurements.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    (args.output / 'measurements.js').write_text('window.XDZIP_BODY_REPORT=' + json.dumps(report, ensure_ascii=False) + ';\n')
    if failures:
        raise SystemExit('\n'.join(failures))
    strict = sum('passed' in measurement for measurement in measurements)
    contours = sum(measurement['kind'] == 'type contour (review only)' for measurement in measurements)
    revisions = sum(measurement['kind'].startswith('revised design ') for measurement in measurements)
    print(f'{strict} strict body checks passed ({revisions} authored revision checks); '
          f'{contours} type contours recorded for review')


if __name__ == '__main__':
    main()
