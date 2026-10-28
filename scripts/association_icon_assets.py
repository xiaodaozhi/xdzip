"""File-association artwork built from the existing xdzip document and mark.

The format lettering is original path geometry, with no runtime font dependency.
This module describes artwork only; it never registers file handlers.
"""
from __future__ import annotations

import json
from pathlib import Path
import xml.etree.ElementTree as ET

from brand_assets import brand_shapes, master_svg
from association_types import WINDOWS_ICONS, MACOS_ICONS

ROOT = Path(__file__).resolve().parents[1]
SPEC_PATH = ROOT / 'docs/visual/file-association-icons.json'
SPEC = json.loads(SPEC_PATH.read_text(encoding='utf-8'))
ASSETS = ROOT / 'design/assets/file-associations'
NS = 'http://www.w3.org/2000/svg'
ET.register_namespace('', NS)

# The legacy ZIP fixes a 61-unit cap height, 14-unit stroke and square endpoints.
# Other letters extend those metrics, not a different font.
GLYPHS = {
    'A': (56, 'M0 61L28 0L56 61M12 39H44'),
    'B': (58, 'M0 61V0H29C63 0 63 29 29 29H0M29 29C68 29 68 61 29 61H0'),
    'C': (56, 'M56 6C40 -5 0 -4 0 30.5S40 66 56 55'),
    'D': (56, 'M0 0V61H24C67 61 67 0 24 0Z'),
    'F': (52, 'M0 61V0H52M0 29H43'),
    'G': (58, 'M56 6C40 -5 0 -4 0 30.5S40 66 58 55V33H34'),
    'I': (0, 'M0 0V61'),
    'O': (58, 'M29 0C-9.666667 0 -9.666667 61 29 61S67.666667 0 29 0Z'),
    'P': (63, 'M0 61V0H33C73 0 73 35 33 35H0'),
    'R': (63, 'M0 61V0H33C73 0 73 35 33 35H0M30 35L63 61'),
    'S': (54, 'M53 5C28 -7 0 0 0 16C0 33 54 25 54 45C54 64 24 66 0 56'),
    'T': (56, 'M0 0H56M28 0V61'),
    'U': (58, 'M0 0V37C0 69 58 69 58 37V0'),
    'Z': (56, 'M0 0H56L0 61H56'),
    '7': (56, 'M0 0H56L16 61'),
    '.': (0, 'M0 60V61'),
}


def lettering(label: str, legacy_zip: ET.Element) -> ET.Element:
    policy = SPEC['label']
    if label == 'ZIP':
        return legacy_zip
    group = ET.Element(f'{{{NS}}}g', {
        'fill': 'none', 'stroke': policy['color'],
        'stroke-width': str(policy['stroke_width']),
        'stroke-linejoin': 'round', 'stroke-linecap': policy['linecap'],
        'data-role': 'format-label',
    })
    if not label:
        # A neutral rule denotes a generic document, not an unsupported extension.
        ET.SubElement(group, f'{{{NS}}}path', {'d': 'M420 847.5H604', 'stroke': '#8996ac'})
        return group
    gaps = [policy['narrow_letter_gap'] if ch == 'I' else policy['letter_gap'] for ch in label[:-1]]
    width = sum(GLYPHS[ch][0] for ch in label) + sum(gaps)
    scale = min(policy['height'] / 61, policy['maximum_width'] / (width + policy['stroke_width']))
    x = policy['x_center'] - width * scale / 2
    y = policy['y'] + (policy['height'] - 61 * scale) / 2
    group.set('transform', f'translate({x:.6f} {y:.6f}) scale({scale:.6f})')
    offset = 0
    for index, char in enumerate(label):
        advance, path = GLYPHS[char]
        ET.SubElement(group, f'{{{NS}}}path', {'d': path, 'transform': f'translate({offset} 0)'})
        offset += advance + (gaps[index] if index < len(gaps) else 0)
    return group


def icon_svg(icon: dict, size: int = 1024, platform: str = 'default') -> str:
    if platform not in SPEC['platforms'] or size not in SPEC['png_sizes']:
        raise ValueError('Unsupported platform or icon size')
    if size > 32:
        root = ET.fromstring(master_svg('archive', platform))
        root.set('viewBox', ' '.join(map(str, SPEC['platforms'][platform]['viewbox'])))
        legacy_zip = list(root)[-1]
        root.remove(legacy_zip)
        root.append(lettering(icon['label'], legacy_zip))
        gradient_id = 'xdzip-master-blue'
    else:
        profile = 'tiny' if size <= 24 else 'small'
        ratio = SPEC['platforms'][platform]['body_height_ratio']
        body_pixels = 2 * round(size * ratio / 2)
        extent = 20 * size / body_pixels
        inset = (24 - extent) / 2
        shape = brand_shapes('light', document_profile=profile)['archive']
        root = ET.fromstring(f'<svg xmlns="{NS}" width="24" height="24" '
                             f'viewBox="{inset:.12g} {inset:.12g} {extent:.12g} {extent:.12g}">{shape}</svg>')
        gradient_id = 'archive-brand-blue'
    for gradient in root.iter(f'{{{NS}}}linearGradient'):
        if gradient.get('id') == gradient_id:
            for stop, color in zip(gradient, SPEC['families'][icon['family']]):
                stop.set('stop-color', color)
    root.set('role', 'img')
    root.set('aria-label', icon['name'])
    title = ET.Element(f'{{{NS}}}title')
    title.text = icon['name']
    root.insert(0, title)
    return ET.tostring(root, encoding='unicode') + '\n'


def source_specs():
    """Inventory all reusable sources for the canonical asset manifest."""
    for icon in SPEC['icons']:
        for profile, size in [('master', 1024), ('tiny', 16), ('small', 32)]:
            yield icon, profile, size, f"xdzip-{icon['id']}-{profile}.svg"


def asset_entries() -> list[dict]:
    import hashlib
    return [{
        'id': f"file-association/{icon['id']}/{profile}",
        'file': f'file-associations/{name}', 'kind': 'association-icon-source',
        'purpose': 'file-association', 'format_id': icon['id'],
        'extensions': icon['extensions'], 'profile': profile,
        'source': '../../scripts/association_icon_assets.py',
        'brand_source': 'brand/mark.svg', 'specification': '../../docs/visual/file-association-icons.json',
        'usage': 'file-association-source',
        'sha256': hashlib.sha256((ASSETS / name).read_bytes()).hexdigest(),
    } for icon, profile, _, name in source_specs()] + [{
        'id': f'file-association/{icon_id}/{suffix}',
        'file': f'file-associations/xdzip-{icon_id}.{suffix}',
        'kind': 'association-icon-platform', 'purpose': 'file-association',
        'format_id': icon_id, 'format': suffix,
        'source': f'file-associations/xdzip-{icon_id}-master.svg',
        'sha256': hashlib.sha256((ASSETS / f'xdzip-{icon_id}.{suffix}').read_bytes()).hexdigest(),
    } for suffix, ids in [('ico', WINDOWS_ICONS.values()), ('icns', MACOS_ICONS)]
      for icon_id in ids]
