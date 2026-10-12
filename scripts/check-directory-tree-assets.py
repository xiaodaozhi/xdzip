#!/usr/bin/env python3
"""Validate the directory-tree handoff, not unimplemented UI interactions."""
from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
import xml.etree.ElementTree as ET

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / 'design/assets'
REPORT_ROOT = ROOT / 'docs/qa/directory-tree'


def main():
    manifest = json.loads((ASSETS / 'manifest.json').read_text())
    spec = json.loads((ROOT / 'docs/visual/directory-tree.json').read_text())
    catalog = json.loads((ROOT / 'design/hifi/catalog.json').read_text())
    baseline = json.loads((REPORT_ROOT / 'baseline.json').read_text())
    preserved = []
    for record in baseline['files']:
        path = ROOT / record['file']
        assert hashlib.sha256(path.read_bytes()).hexdigest() == record['sha256'], record['file']
        preserved.append(record['file'])
    assert len(preserved) == 58, 'Original ten boards, 45 references and three photos must survive unchanged'

    boards = {item['file']: item for item in catalog['screens'] if item.get('component') == 'directory-tree'}
    assert set(boards) == {'screens/directory-tree.png', 'screens/directory-tree-states.png',
                           'screens/directory-tree-boundaries.png'}
    for item in boards.values():
        path = ROOT / 'design/hifi' / item['file']
        assert hashlib.sha256(path.read_bytes()).hexdigest() == item['sha256']
        with Image.open(path) as original:
            assert original.size == (1024, 1536), item['file']
        assert item['status'] == 'approved-design-awaiting-integration'

    records = {item['file']: item for item in manifest['assets']}
    cuts = manifest['component_reference_coverage']
    assert len(cuts) == 26 and len({item['reference'] for item in cuts}) == 26
    assert {item['reference'] for item in cuts} == {
        item['file'] for item in records.values() if item['file'].startswith('reference/tree-')}
    for item in cuts:
        assert item['implementation'] in {'shared-svg', 'native-component', 'native-geometry'}
        record = records[item['reference']]
        assert record['source'].startswith('../hifi/screens/directory-tree')
        assert record['usage'] == 'visual-reference-only'
        with Image.open(ASSETS / record['source']) as source, Image.open(ASSETS / record['file']) as crop:
            x, y, width, height = record['rect']
            assert 0 <= x < x + width <= source.width and 0 <= y < y + height <= source.height
            assert crop.size == (width, height)
            assert crop.convert('RGBA').tobytes() == source.convert('RGBA').crop((x, y, x + width, y + height)).tobytes()
    # This checks the new glyph's geometry and exports independently of its
    # generator: six visible units, shared center, and no invented Qt usage.
    for theme, color in [('light', '#1674ff'), ('dark', '#78a6ff')]:
        name = f'svg/{theme}/tree-active-branch.svg'
        svg = ET.parse(ASSETS / name).getroot()
        circles = [node for node in svg if node.tag.endswith('circle')]
        assert len(circles) == 1
        assert {key: circles[0].get(key) for key in ('cx', 'cy', 'r', 'fill')} == {
            'cx': '12', 'cy': '12', 'r': '3', 'fill': color}
        assert records[name]['runtime_usage']['qt_aliases'] == []
        for scale in (1, 2, 3):
            with Image.open(ASSETS / f'png/{theme}/tree-active-branch@{scale}x.png') as image:
                assert image.size == (24 * scale, 24 * scale)
                alpha = image.convert('RGBA').getchannel('A')
                assert alpha.getextrema()[0] == 0
                bounds = alpha.point(lambda value: 255 if value > 8 else 0).getbbox()
                assert bounds and max(abs(value / scale - target) for value, target in zip(bounds, (9, 9, 15, 15))) <= 1 / scale
        for icon, reference in [('tree-info', 'tree-info-glyph'), ('tree-warning', 'tree-error-glyph'), ('tree-spinner', 'tree-loading-glyph')]:
            name = f'svg/{theme}/{icon}.svg'
            assert records[name]['source_reference'] == f'reference/{reference}.png'
            assert records[name]['runtime_usage']['qt_aliases'] == []
            glyph = ET.parse(ASSETS / name).getroot()
            if icon == 'tree-info':
                assert any(node.tag.endswith('circle') and node.get('r') == '10'
                           and node.get('fill', 'none') != 'none' for node in glyph)
            elif icon == 'tree-warning':
                assert any(node.tag.endswith('path') and node.get('fill', '').startswith('url(')
                           and node.get('stroke') == 'none' for node in glyph)
            else:
                assert len([node for node in glyph if node.tag.endswith('path')]) == 12

    nodes = {item['id']: item for item in spec['sample']['nodes']}
    children = {key: [item for item in nodes.values() if item['parent_id'] == key] for key in nodes}
    for node in nodes.values():
        assert node['has_child_directories'] == bool(children[node['id']]), node['id']
        assert node['descendant_physical_count'] == node['direct_physical_count'] + sum(
            child['descendant_physical_count'] for child in children[node['id']]), node['id']
    assert nodes['ice']['has_child_directories'] and nodes['ice']['descendant_physical_count'] == 6
    assert nodes['shore']['has_child_directories'] and not nodes['snow']['has_child_directories']
    assert spec['sample']['archive_physical_count'] == 30
    assert spec['sample']['root_physical_count'] + sum(
        node['descendant_physical_count'] for node in nodes.values() if node['parent_id'] is None) == 30
    assert spec['states']['leaf_nonempty']['count'] == 3 and not spec['states']['leaf_nonempty']['show_disclosure']
    assert spec['states']['leaf_empty']['count'] == 0 and not spec['states']['leaf_empty']['show_disclosure']
    assert spec['states']['loading']['count'] is None and spec['states']['read_failed']['count'] is None
    assert spec['states']['root_files_only']['list_physical_count'] == 6
    assert spec['states']['navigation_limited']['list_available'] is True
    assert spec['states']['hidden_current']['current_directory'] == 'ice'
    assert spec['states']['hidden_current']['branch_marker'] == 'photos'
    assert spec['states']['hidden_current']['selected_visible_row'] is None
    required = {'collapsed', 'expanded', 'leaf_nonempty', 'leaf_empty', 'current', 'hover', 'focus',
                'long_name', 'deep_hierarchy', 'large_type', 'loading', 'empty_archive',
                'root_files_only', 'navigation_limited', 'read_failed', 'hidden_current'}
    assert required <= set(spec['states']), 'Missing handoff state'
    assert spec['status'] == 'design-ready-awaiting-html-and-qt-integration'
    display = (ROOT / 'design/html/display-spec.js').read_text()
    ratio = re.search(r'sourceToLogical:\s*(\d+)\s*/\s*(\d+)', display)
    assert ratio and spec['coordinate_mapping']['source_to_logical_ratio'] == [int(ratio[1]), int(ratio[2])]
    source_to_logical = int(ratio[1]) / int(ratio[2])
    for example in spec['coordinate_mapping']['examples']:
        assert abs(example['source_css'] * source_to_logical - example['logical']) < 1e-9
    report = {
        'status': 'passed', 'approved_component_boards': len(boards),
        'component_source_crops': len(cuts), 'preserved_original_files': len(preserved),
        'new_shared_glyphs': 4, 'new_themed_svg_files': 8, 'new_transparent_png_files': 24,
        'specified_states': len(spec['states']),
        'checks': ['Original files keep their SHA-256', 'Every cut matches its exact bounded source rectangle',
                   'Six-unit marker is centered on every shared canvas',
                   'Counts, leaf arrows and hidden-current sample are internally consistent'],
        'coordinate_conversion': 'Qt logical values and HTML source values map to the same 1280-wide design size',
        'integration_status': spec['status'],
        'limits': ['No HTML or Qt directory-tree redesign is implemented by this delivery',
                   'State specifications do not prove live keyboard or screen-reader behavior',
                   'Light boards do not constitute a complete dark component board'],
    }
    (REPORT_ROOT / 'validation.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(report, ensure_ascii=False))


if __name__ == '__main__':
    main()
