#!/usr/bin/env python3
"""Verify flat SVG geometry and actual solid-fill PNG pixels in both themes."""
import json
from pathlib import Path
import xml.etree.ElementTree as ET
from PIL import Image, ImageColor

ROOT = Path(__file__).resolve().parents[1]


def main():
    spec = json.loads((ROOT / 'docs/visual/status-icons.json').read_text())
    checks = []
    for name, icon in spec['icons'].items():
        for theme in ['light', 'dark']:
            path = ROOT / 'design/assets/svg' / theme / (name + '.svg')
            root = ET.parse(path).getroot()
            nodes = list(root.iter())
            assert not any('Gradient' in n.tag or n.tag.endswith('filter') for n in nodes), path
            assert not any('url(' in v or 'opacity' in k for n in nodes for k, v in n.attrib.items()), path
            disc = next(n for n in nodes if n.tag.endswith('circle') and n.get('r') == str(icon['radius']))
            assert disc.get('fill') == icon[theme] and disc.get('stroke') == 'none', path
            assert [float(disc.get(k)) for k in ['cx', 'cy']] == [12, 12], path
            checks.append(str(path.relative_to(ROOT)))
            expected = ImageColor.getrgb(icon[theme]) + (255,)
            for scale in [1, 2, 3]:
                png = ROOT / 'design/assets/png' / theme / f'{name}@{scale}x.png'
                with Image.open(png) as image:
                    rgba = image.convert('RGBA')
                    assert rgba.size == (24 * scale, 24 * scale), png
                    assert rgba.getpixel((0, 0))[3] == 0, png
                    # Four interior samples around the white mark must have identical paint.
                    for x, y in [(12, 4), (4, 12), (20, 12), (12, 20)]:
                        assert rgba.getpixel((x * scale, y * scale)) == expected, (png, x, y)
                checks.append(str(png.relative_to(ROOT)))
    result = dict(status='passed', checked=len(checks), files=checks,
                  method='SVG paint and geometry plus decoded PNG RGBA samples; no screenshot rescaling')
    output = ROOT / 'dist/flat-status-validation.json'
    output.parent.mkdir(exist_ok=True)
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(f'Flat status assets: {len(checks)} checks passed')


if __name__ == '__main__':
    main()
