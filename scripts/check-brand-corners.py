#!/usr/bin/env python3
"""Check production SVG corner symmetry, tangency and curvature continuity.

The optional raster check isolates the production tile path so directional
lighting and the exterior drop shadow cannot disguise a contour defect.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
import os
from pathlib import Path
import re
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
SVG = '{http://www.w3.org/2000/svg}'


def subtract(a, b):
    return (a[0] - b[0], a[1] - b[1])


def cross(a, b):
    return a[0] * b[1] - a[1] * b[0]


def cubic_ends(curve):
    a, b, c, d = curve
    start, end = subtract(b, a), subtract(d, c)
    second_start = (c[0] - 2 * b[0] + a[0], c[1] - 2 * b[1] + a[1])
    second_end = (d[0] - 2 * c[0] + b[0], d[1] - 2 * c[1] + b[1])
    # First derivatives have factor 3 and second derivatives factor 6.
    curvature = lambda v, s: 2 * cross(v, s) / (3 * math.hypot(*v) ** 3)
    return start, end, curvature(start, second_start), curvature(end, second_end)


def curvature_at(curve, t):
    a, b, c, d = curve
    velocity = tuple(3 * ((1 - t) ** 2 * (b[i] - a[i])
                          + 2 * t * (1 - t) * (c[i] - b[i])
                          + t ** 2 * (d[i] - c[i])) for i in (0, 1))
    acceleration = tuple(6 * ((1 - t) * (c[i] - 2 * b[i] + a[i])
                              + t * (d[i] - 2 * c[i] + b[i])) for i in (0, 1))
    return cross(velocity, acceleration) / math.hypot(*velocity) ** 3


def parse_curves(path):
    """Read the restricted production contour; unsupported commands fail closed."""
    tokens = re.findall(r'[A-Za-z]|[-+]?(?:\d*\.\d+|\d+)(?:[eE][-+]?\d+)?', path)
    position = (0.0, 0.0)
    curves, points = [], []
    index = 0
    while index < len(tokens):
        command = tokens[index]
        index += 1
        count = {'M': 2, 'H': 1, 'V': 1, 'C': 6, 'Z': 0}.get(command)
        assert count is not None, f'Unexpected contour command: {command}'
        values = [float(value) for value in tokens[index:index + count]]
        assert len(values) == count
        index += count
        if command == 'M':
            position = tuple(values)
        elif command == 'H':
            position = (values[0], position[1])
        elif command == 'V':
            position = (position[0], values[0])
        elif command == 'C':
            curve = (position, tuple(values[:2]), tuple(values[2:4]), tuple(values[4:]))
            curves.append(curve)
            points.extend(curve)
            position = curve[-1]
        points.append(position)
    assert len(curves) == 8, 'Expected two cubic segments per corner'
    return curves, points


def inspect(path):
    root = ET.fromstring(path.read_text())
    tiles = [node for node in root.findall(SVG + 'path')
             if node.get('fill', '').endswith('-blue)')]
    assert len(tiles) == 1, f'Missing or duplicate blue contour: {path}'
    tile = tiles[0]
    assert tile.get('stroke', 'none') == 'none', 'Contour must not have a second outline'
    curves, points = parse_curves(tile.attrib['d'])
    low = (min(p[0] for p in points), min(p[1] for p in points))
    high = (max(p[0] for p in points), max(p[1] for p in points))
    assert math.isclose(high[0] - low[0], high[1] - low[1]), 'Non-square shell'
    centre = ((low[0] + high[0]) / 2, (low[1] + high[1]) / 2)
    rotate = lambda p: (centre[0] - (p[1] - centre[1]),
                        centre[1] + (p[0] - centre[0]))
    for corner in range(4):
        first, second = curves[corner * 2:corner * 2 + 2]
        _, tangent_a, outer_a, inner_a = cubic_ends(first)
        tangent_b, _, inner_b, outer_b = cubic_ends(second)
        assert math.isclose(outer_a, 0, abs_tol=1e-12)
        assert math.isclose(outer_b, 0, abs_tol=1e-12)
        assert abs(cross(tangent_a, tangent_b)) < 1e-9, 'Corner tangent discontinuity'
        assert sum(a * b for a, b in zip(tangent_a, tangent_b)) > 0
        assert math.isclose(inner_a, inner_b, rel_tol=1e-9, abs_tol=1e-12), 'Corner curvature discontinuity'
        curvature = [curvature_at(first, sample / 128) for sample in range(129)]
        assert all(later >= earlier - 1e-10 for earlier, later in zip(curvature, curvature[1:])), 'Curvature trough creates a beveled corner'
        next_corner = curves[((corner + 1) % 4) * 2:((corner + 1) % 4) * 2 + 2]
        for current, following in zip((first, second), next_corner):
            for a, b in zip(current, following):
                assert math.dist(rotate(a), b) < 1e-8, 'Corners differ after 90-degree rotation'
    if tile.get('id') == 'app-tile' or tile.get('fill') == 'url(#app-brand-blue)':
        assert not root.findall(SVG + 'rect'), 'Separate rounded rectangle reintroduced'
        for node in root.findall(SVG + 'path'):
            assert node.get('stroke', 'none') == 'none', 'Open highlight stroke reintroduced'
            assert node.attrib['d'] == tile.attrib['d'], 'Surface and tile contours diverged'
    return {'file': str(path.relative_to(ROOT)),
            'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'bounds': [*low, *high], 'corners': 4, 'g2_continuous': True}, tile.attrib['d']


def render_contour(path, viewbox):
    from PIL import Image, ImageChops, ImageFilter
    from playwright.sync_api import sync_playwright

    checks = []
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(
            executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
            args=['--no-sandbox', '--disable-dev-shm-usage'])
        try:
            page = browser.new_page(device_scale_factor=1)
            page.route('**/*', lambda route: route.abort())
            for size in (16, 24, 32, 48, 64, 128, 512, 1024):
                page.set_viewport_size({'width': size, 'height': size})
                page.set_content('<style>html,body{margin:0;background:transparent}svg{display:block;width:100vw;height:100vh}</style>'
                                 f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{viewbox}"><path d="{path}" fill="black"/></svg>')
                alpha = Image.open(io.BytesIO(page.screenshot(omit_background=True))).getchannel('A')
                delta = ImageChops.difference(alpha, alpha.transpose(Image.Transpose.ROTATE_90))
                maximum = delta.getextrema()[1]
                # Skia covers horizontal and vertical subpixel edges differently.
                # Report that coverage difference instead of calling it geometry.
                # The actual 50%-coverage contour may move at most one pixel.
                mask = alpha.point(lambda value: 255 if value >= 128 else 0)
                rotated = mask.transpose(Image.Transpose.ROTATE_90)
                outside = ImageChops.subtract(mask, rotated.filter(ImageFilter.MaxFilter(3)))
                inside = ImageChops.subtract(rotated, mask.filter(ImageFilter.MaxFilter(3)))
                assert outside.getbbox() is None and inside.getbbox() is None, f'{size}px contour displacement exceeds 1 pixel'
                histogram = delta.histogram()
                checks.append({'size': size, 'maximum_alpha_delta_after_rotation': maximum,
                               'mean_alpha_delta': round(sum(i * n for i, n in enumerate(histogram)) / size ** 2, 6),
                               'contour_displacement_within_1_pixel': True})
        finally:
            browser.close()
    return checks


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--render', action='store_true', help='also verify isolated contour raster symmetry')
    parser.add_argument('--output', type=Path, help='save a JSON measurement report')
    args = parser.parse_args()
    paths = [ROOT / 'design/assets/brand/xdzip.svg', ROOT / 'design/assets/file-associations/xdzip-zip-master.svg']
    paths += [ROOT / 'design/assets/svg' / theme / f'{name}.svg'
              for theme in ('light', 'dark') for name in ('app-mark', 'archive', 'archive-grey')]
    results = [inspect(path) for path in paths]
    report = {'status': 'passed', 'svg_contours': [item[0] for item in results],
              'limits': 'Contour checks exclude lighting, drop shadows, source-logo tracing accuracy and native OS display.'}
    if args.render:
        report['master_raster_symmetry'] = render_contour(results[0][1], ET.parse(paths[0]).getroot().get('viewBox'))
        report['ui_raster_symmetry'] = render_contour(results[2][1], '0 0 24 24')
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'status': 'passed', 'svg_contours': len(results),
                      'rendered_contours': 16 if args.render else 0}))


if __name__ == '__main__':
    main()
