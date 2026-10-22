#!/usr/bin/env python3
"""Verify crop pixels, canonical canvases and independent optical contracts."""
from __future__ import annotations

import hashlib
import json
import re
import xml.etree.ElementTree as ET
import subprocess
import sys
from pathlib import Path

from PIL import Image
from directory_tree_assets import TREE_CUTS, component_reference_coverage, tree_shapes
from operation_reference_assets import OPERATION_CUTS, operation_reference_coverage
from asset_definitions import (
    CONTROL_PATHS, EXTRA_CONTROL_PATHS, FILE_OPTICAL_SPECS, MAIN_SHAPES,
    STATE_SHAPES, file_shapes, hero_shapes, revised_status_shapes, supplemental_shapes,
)

ROOT = Path(__file__).resolve().parents[1] / 'design/assets'
THEMES = ('light', 'dark')
BRAND_KINDS = {'brand-master', 'platform-icon', 'brand-reference'}
SUPPLEMENTAL_NAMES = {
    'theme-light', 'theme-dark', 'existing-archive', 'windows-document',
    'linux-document', 'cancelled-large', 'check-selected', 'success',
    'radio-selected', 'radio-unselected', 'conflict-pdf', 'conflict-warning',
    'progress-image',
}
# Semantic correspondence is checked independently of the generator's ledger.
# A generic skipped/stopped/file symbol cannot silently cover a distinct source.
REFERENCE_ICONS = {
    'brand-titlebar': 'app-mark', 'brand-extract': 'archive', 'folder': 'folder',
    'file-pdf': 'file-pdf', 'file-text': 'file-text', 'file-code': 'file-code',
    'file-markdown': 'file-markdown', 'open': 'open', 'new': 'new',
    'archive-test': 'archive-test', 'more': 'more', 'search': 'search',
    'settings': 'settings', 'view-list': 'view-list', 'view-grid': 'view-grid',
    'sort': 'sort', 'close': 'close', 'check-selected': 'check-selected',
    'success': 'success', 'radio-selected': 'radio-selected',
    'radio-unselected': 'radio-unselected', 'create-archive': 'archive-add',
    'image-blue': 'image-file', 'image-amber': 'image-amber', 'remove-row': 'remove',
    'conflict-document': 'document-conflict', 'conflict-pdf': 'conflict-pdf',
    'conflict-warning': 'conflict-warning', 'progress-document': 'document-download',
    'progress-image': 'progress-image', 'partial-document': 'document-warning',
    'partial-success': 'status-partial-success', 'partial-skipped': 'status-skipped',
    'partial-warning': 'status-warning', 'settings-gear': 'settings-tile',
    'theme-system': 'theme-system', 'theme-light': 'theme-light', 'theme-dark': 'theme-dark',
    'success-large': 'status-success', 'stopping-spinner': 'spinner',
    'cancelled-large': 'cancelled-large', 'overwrite-archive': 'archive-warning',
    'existing-archive': 'existing-archive', 'windows-document': 'windows-document',
    'linux-document': 'linux-document',
}
CONTROL_REFERENCES = {'check-selected', 'radio-selected', 'radio-unselected'}
HISTORICAL_BRAND_REFERENCES = {'brand-titlebar', 'brand-extract'}
NON_REFERENCE_NAMES = {
    'activity', 'archive-grey', 'arrow-right', 'check', 'chevron-down',
    'chevron-right', 'chevron-up', 'clock', 'completed', 'external-open',
    'extract', 'info', 'maximize', 'minimize', 'moon', 'panel', 'plus',
    'shield', 'sort-ascending', 'status-stopped', 'sun', 'task-list', 'warning',
}
NON_REFERENCE_NAMES.update({'file-document', 'file-spreadsheet', 'file-presentation',
                            'file-audio', 'file-video', 'file-archive', 'file-installer',
                            'file-font', 'file-disk'})

HIGH_RES_ICONS = set(json.loads((Path(__file__).resolve().parents[1] / 'docs/visual/operation-assets.json').read_text())['icons'])


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bounds(image: Image.Image, scale: int) -> list[float]:
    alpha = image.getchannel('A').point(lambda value: 255 if value > 8 else 0)
    bbox = alpha.getbbox()
    assert bbox is not None, 'Empty icon'
    return [round(edge / scale, 4) for edge in bbox]


def check_reference_coverage(manifest: dict, by_file: dict) -> list[dict]:
    """Require all preserved cuts to have an explicit, accurate usage contract.

    Native controls own state and interaction. Their optional reference SVGs
    document appearance; they never turn a PNG crop into a shipping control.
    Current brand drawings are a revision, not recovered old brand pixels.
    """
    coverage = manifest.get('reference_coverage', [])
    references = {f'reference/{name}.png' for name in REFERENCE_ICONS}
    assert len(coverage) == len(references) == 45, 'Incomplete reference coverage'
    assert {item['reference'] for item in coverage} == references, 'Untracked reference coverage'
    assert {entry['file'] for entry in manifest['assets']
            if entry['kind'] == 'original-pixel-crop' and entry['usage'] == 'visual-reference-only'} == (
                references | {f'reference/{name}.png' for name in TREE_CUTS} | {f'reference/{name}.png' for name in OPERATION_CUTS})
    assert manifest.get('operation_reference_coverage') == operation_reference_coverage()
    checked = []
    for item in coverage:
        reference = item['reference']
        name = Path(reference).stem
        entry = by_file.get(reference)
        assert entry and entry['kind'] == 'original-pixel-crop', reference
        assert item.get('role') and item.get('note'), f'Missing usage explanation: {reference}'
        implementation = item.get('implementation')
        if implementation == 'native-control':
            assert name in CONTROL_REFERENCES, f'Non-control cannot bypass SVG coverage: {reference}'
            assert item['role'] == 'control-state-reference' and item.get('native_control_note'), reference
            checked.append({'reference': reference, 'implementation': implementation,
                            'role': item['role'], 'production_icons': []})
            continue
        assert implementation == 'shared-svg', f'Unknown reference implementation: {reference}'
        names = item.get('icon_names', [])
        assert names and len(set(names)) == len(names), f'Missing or duplicate semantic icons: {reference}'
        assert names[0] == REFERENCE_ICONS[name], f'Inexact primary semantic mapping: {reference}'
        assert set(item.get('svg_files', {})) == set(THEMES), reference
        for theme in THEMES:
            expected = [f'svg/{theme}/{icon}.svg' for icon in names]
            assert item['svg_files'][theme] == expected, f'Incomplete SVG coverage: {reference}/{theme}'
            for file in expected:
                vector = by_file.get(file)
                assert vector and vector['kind'] == 'vector-reconstruction', file
                assert vector['theme'] == theme and vector['logical_size'] == [24, 24], file
                assert vector.get('source_reference') == reference, f'Unrelated SVG reference: {file}'
                assert all(f'png/{theme}/{Path(file).stem}@{scale}x.png' in by_file
                           for scale in (1, 2, 3)), f'Missing covered raster export: {file}'
        if name in CONTROL_REFERENCES:
            assert item['role'] == 'control-state-reference' and item.get('native_control_note'), reference
        if name in HISTORICAL_BRAND_REFERENCES:
            assert item['role'] == 'historical-design-reference', reference
            assert item.get('historical_reference') is True and item.get('source_revision'), reference
            for theme in THEMES:
                assert by_file[item['svg_files'][theme][0]].get('reference_relation') == 'historical-design-reference'
        else:
            assert item['role'] != 'historical-design-reference', f'Non-brand source marked historical: {reference}'
        checked.append({'reference': reference, 'implementation': implementation,
                        'role': item['role'], 'production_icons': names})
    return checked


def main() -> None:
    subprocess.run([sys.executable, str(Path(__file__).with_name('check-flat-status-assets.py'))], check=True)
    # A changed definition must be exported before manifest hashes can pass.
    subprocess.run([sys.executable, str(Path(__file__).with_name('build-design-assets.py')), '--check'],
                   check=True, stdout=subprocess.DEVNULL)
    # Platform package formats and their multi-size payloads have one validator.
    # Keep their exception separate from the strict shared 24-unit icon canvas.
    subprocess.run([sys.executable, str(Path(__file__).with_name('build-brand-assets.py')), '--check'],
                   check=True, stdout=subprocess.DEVNULL)
    from brand_assets import brand_export_specs, brand_shapes
    brand_specs = {spec['file']: spec for spec in brand_export_specs()}
    assert len(brand_specs) == len(brand_export_specs()), 'Duplicate brand export paths'
    definitions = {theme: set(CONTROL_PATHS) | set(EXTRA_CONTROL_PATHS) | set(MAIN_SHAPES)
                   | set(STATE_SHAPES) | set(file_shapes(theme)) | set(hero_shapes())
                   | set(revised_status_shapes(theme)) | set(supplemental_shapes(theme))
                   | set(brand_shapes(theme)) | set(tree_shapes(theme)) for theme in THEMES}
    assert definitions['light'] == definitions['dark'], 'Theme semantic names differ'
    names = definitions['light']
    assert SUPPLEMENTAL_NAMES | {'app-mark', 'archive', 'archive-grey'} <= names, 'Missing new semantic/brand icons'
    assert set(REFERENCE_ICONS.values()) | NON_REFERENCE_NAMES <= names, 'An established icon semantic disappeared'
    manifest = json.loads((ROOT / 'manifest.json').read_text())
    by_file = {entry['file']: entry for entry in manifest['assets']}
    assert len(by_file) == len(manifest['assets']), 'Duplicate asset paths'
    assert len({entry['id'] for entry in manifest['assets']}) == len(manifest['assets']), 'Duplicate asset IDs'
    optical_checks = []
    for entry in manifest['assets']:
        path = ROOT / entry['file']
        assert path.resolve().is_relative_to(ROOT.resolve()), f'Asset escapes library: {entry["file"]}'
        assert path.is_file(), entry['file']
        assert digest(path) == entry['sha256'], f'Stale hash: {entry["file"]}'
        kind = entry['kind']
        if kind == 'original-pixel-crop':
            source = ROOT / entry['source']
            assert digest(source) == entry['source_sha256'], entry['source']
            x, y, width, height = entry['rect']
            with Image.open(source) as original, Image.open(path) as crop:
                assert min(x, y) >= 0 and min(width, height) > 0
                assert x + width <= original.width and y + height <= original.height, entry['file']
                assert crop.size == (width, height)
                assert crop.convert('RGBA').tobytes() == original.convert('RGBA').crop((x, y, x + width, y + height)).tobytes()
            assert entry['usage'] in ('visual-reference-only', 'demo-image-only')
        elif kind == 'vector-reconstruction':
            svg = ET.parse(path).getroot()
            assert svg.attrib['viewBox'] == '0 0 24 24'
            assert svg.attrib['width'] == svg.attrib['height'] == '24'
            identifiers = [node.attrib['id'] for node in svg.iter() if 'id' in node.attrib]
            assert len(identifiers) == len(set(identifiers)), f'Duplicate SVG IDs: {entry["file"]}'
            for node in svg.iter():
                assert node.tag.rsplit('}', 1)[-1] not in ('script', 'image', 'text', 'filter')
                assert not any('href' in attr for attr in node.attrib)
                for value in node.attrib.values():
                    for reference in re.findall(r'url\(#([^)]+)\)', value):
                        assert reference in identifiers, f'Missing SVG paint reference: {entry["file"]}#{reference}'
            assert entry['canvas'] == entry['logical_size'] == [24, 24]
            assert entry['theme'] in ('light', 'dark')
            if path.stem in ('app-mark', 'archive', 'archive-grey'):
                source_file = 'brand/xdzip.svg' if path.stem == 'app-mark' else 'brand/xdzip-zip.svg'
                assert entry.get('brand_source') == source_file, f'Missing current brand source: {entry["file"]}'
                source = by_file.get(source_file)
                assert source and source['kind'] == 'brand-master', source_file
                assert entry.get('source_sha256') == source['sha256'], f'Stale brand SVG provenance: {entry["file"]}'
                assert entry.get('source_revision') == 'current-logo', entry['file']
        elif kind in ('tray-vector', 'tray-rasterization'):
            assert entry['logical_size'] == [22, 22]
            source = by_file[entry['source']]
            assert source['sha256'] == entry['source_sha256']
            if kind == 'tray-vector':
                assert entry['source'] == 'brand/mark.svg'
                svg = ET.parse(path).getroot()
                assert svg.attrib['viewBox'] == '0 0 22 22'
                assert all(node.tag.rsplit('}',1)[-1] in ('svg','path') for node in svg.iter())
            else:
                assert source['kind'] == 'tray-vector'
                with Image.open(path) as icon:
                    assert icon.size == tuple(entry['size'])
                    assert icon.mode == 'RGBA' and icon.getbbox()
        elif kind == 'vector-rasterization':
            scale = entry['scale']
            assert scale in (1, 2, 3, 9)
            if scale == 9:
                assert Path(entry['source']).stem in HIGH_RES_ICONS
            assert entry['canvas'] == entry['logical_size'] == [24, 24]
            assert entry['source'] in by_file
            assert by_file[entry['source']]['kind'] == 'vector-reconstruction'
            assert entry['source_sha256'] == by_file[entry['source']]['sha256'], 'Stale raster provenance'
            with Image.open(path) as export:
                image = export.convert('RGBA')
                assert image.size == tuple(entry['size']) == (24 * scale, 24 * scale)
                assert image.getchannel('A').getextrema()[0] == 0
                measured = bounds(image, scale)
                assert measured == entry['visible_bounds']
            name = Path(entry['source']).stem
            if name in FILE_OPTICAL_SPECS:
                expected = FILE_OPTICAL_SPECS[name]['bounds']
                # Pixel-edge measurements include a full antialias fringe plus
                # half-pixel phase rounding (e.g. an alpha=12 fold-tip pixel).
                # This fixed 1.5-physical-pixel allowance decreases with scale;
                # nominal shape bounds and independent source checks stay fixed.
                tolerance = 1.5 / scale
                errors = [round(abs(actual - target), 4) for actual, target in zip(measured, expected)]
                assert max(errors) <= tolerance, (entry['file'], measured, expected, errors)
                center = [(measured[0] + measured[2]) / 2, (measured[1] + measured[3]) / 2]
                assert max(abs(value - 12) for value in center) <= .5 / scale + .05, (entry['file'], center)
                optical_checks.append({'file': entry['file'], 'bounds': measured, 'expected': expected,
                                       'max_edge_error': max(errors), 'tolerance': tolerance, 'center': center})
        elif kind == 'menu-shadow-rasterization':
            assert entry['theme'] in ('light', 'dark'), entry['file']
            assert entry['font_scale'] in (1, 1.25, 1.5, 2), entry['file']
            assert entry['source'] == '../../docs/visual/controls.json', entry['file']
            assert entry['source_sha256'] == hashlib.sha256((ROOT / entry['source']).read_bytes()).hexdigest(), entry['file']
            with Image.open(ROOT / entry['file']) as image:
                assert list(image.size) == entry['size'] and image.mode == 'RGBA', entry['file']
                assert image.getchannel('A').getextrema()[1] > 0, entry['file']
        elif kind in BRAND_KINDS:
            spec = brand_specs.get(entry['file'])
            assert spec and all(entry.get(key) == value for key, value in spec.items()), entry['file']
            assert entry['category'] == 'brand', entry['file']
            if entry.get('source'):
                source = by_file.get(entry['source'])
                assert source and source['kind'] in BRAND_KINDS, f'Missing brand provenance: {entry["file"]}'
                assert entry['source_sha256'] == source['sha256'], f'Stale brand provenance: {entry["file"]}'
        else:
            raise AssertionError(f'Unknown asset kind: {kind}')
    counts = {kind: sum(item['kind'] == kind for item in manifest['assets'])
              for kind in ('original-pixel-crop', 'vector-reconstruction', 'vector-rasterization')}
    assert counts == {'original-pixel-crop': 48 + len(TREE_CUTS) + len(OPERATION_CUTS), 'vector-reconstruction': len(names) * 2,
                      'vector-rasterization': len(names) * 6 + len(HIGH_RES_ICONS) * 2}, counts
    assert {entry['file'] for entry in manifest['assets'] if entry['kind'] in BRAND_KINDS} == set(brand_specs)
    for theme in THEMES:
        vector_files = {f'svg/{theme}/{name}.svg' for name in names}
        raster_files = {f'png/{theme}/{name}@{scale}x.png' for name in names for scale in (1, 2, 3)}
        raster_files |= {f'png/{theme}/{name}@9x.png' for name in HIGH_RES_ICONS}
        assert {path.relative_to(ROOT).as_posix() for path in (ROOT / 'svg' / theme).glob('*.svg')} == vector_files
        assert {path.relative_to(ROOT).as_posix() for path in (ROOT / 'png' / theme).glob('*.png')} == raster_files
        assert {entry['file'] for entry in manifest['assets']
                if entry['kind'] == 'vector-reconstruction' and entry['theme'] == theme} == vector_files
        assert {entry['file'] for entry in manifest['assets']
                if entry['kind'] == 'vector-rasterization' and entry['theme'] == theme} == raster_files
    coverage = check_reference_coverage(manifest, by_file)
    components = manifest.get('component_reference_coverage', [])
    assert components == component_reference_coverage(), 'Incomplete component reference coverage'
    for component in components:
        assert component['reference'] in by_file
        assert component['integration_status'] in {'core-interactions-integrated', 'design-reference-only'}
        for icon in component['icon_names']:
            for theme in THEMES:
                assert f'svg/{theme}/{icon}.svg' in by_file
                assert all(f'png/{theme}/{icon}@{scale}x.png' in by_file for scale in (1, 2, 3))
        for key in ('spec', 'geometry_spec'):
            target = (ROOT / component[key]).resolve()
            assert target.is_relative_to(ROOT.parents[1]) and target.is_file(), component[key]
    subprocess.run([sys.executable, str(Path(__file__).with_name('check-directory-tree-assets.py'))], check=True)
    report = {
        'status': 'passed', 'asset_entries': len(manifest['assets']), 'source_crops': counts['original-pixel-crop'],
        'svg_files': counts['vector-reconstruction'], 'vector_png_files': counts['vector-rasterization'],
        'shared_icon_names': len(names), 'brand_exports': len(brand_specs),
        'reference_checks': len(coverage), 'optical_checks': optical_checks, 'reference_coverage': coverage,
        'component_reference_checks': len(components),
        'checks': ['Unique assets exist and match SHA-256', f'{counts["original-pixel-crop"]} source crops match recorded original pixels',
                   f'{counts["vector-reconstruction"]} safe SVGs share the 24×24 canvas',
                   f'{counts["vector-rasterization"]} transparent exports have 24/48/72-pixel canvases, plus {2 * len(HIGH_RES_ICONS)} high-resolution 216-pixel operation/file exports',
                   f'{len(coverage)} references map to exact semantic SVGs or explicitly native controls',
                   f'{len(components)} component cuts declare shared icons or dynamic native geometry',
                   f'{len(brand_specs)} brand exports pass their platform-format validator and source hashes',
                   'SVG paint references resolve to unique local definitions',
                   f'{len(optical_checks)} file/folder theme-and-scale exports satisfy fixed optical bounds and center',
                   'Generated SVGs, asset metadata and gallery match canonical definitions'],
        'limits': ['Independent visual comparison remains necessary', 'No native Qt renderer validation in this script',
                   'Reference crops contain baked backgrounds and cannot ship as transparent icons'],
    }
    report_file = ROOT.parents[1] / 'build/review/assets/validation.json'
    report_file.parent.mkdir(parents=True, exist_ok=True)
    report_file.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({key: value for key, value in report.items()
                      if key not in ('optical_checks', 'reference_coverage')}, ensure_ascii=False))


if __name__ == '__main__':
    main()
