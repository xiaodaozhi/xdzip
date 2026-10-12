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
from asset_definitions import FILE_OPTICAL_SPECS

ROOT = Path(__file__).resolve().parents[1] / 'design/assets'


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bounds(image: Image.Image, scale: int) -> list[float]:
    alpha = image.getchannel('A').point(lambda value: 255 if value > 8 else 0)
    bbox = alpha.getbbox()
    assert bbox is not None, 'Empty icon'
    return [round(edge / scale, 4) for edge in bbox]


def main() -> None:
    # A changed definition must be exported before manifest hashes can pass.
    subprocess.run([sys.executable, str(Path(__file__).with_name('build-design-assets.py')), '--check'],
                   check=True, stdout=subprocess.DEVNULL)
    manifest = json.loads((ROOT / 'manifest.json').read_text())
    by_file = {entry['file']: entry for entry in manifest['assets']}
    assert len(by_file) == len(manifest['assets']), 'Duplicate asset paths'
    optical_checks = []
    for entry in manifest['assets']:
        path = ROOT / entry['file']
        assert path.is_file(), entry['file']
        assert digest(path) == entry['sha256'], f'Stale hash: {entry["file"]}'
        kind = entry['kind']
        if kind == 'original-pixel-crop':
            source = ROOT / entry['source']
            assert digest(source) == entry['source_sha256'], entry['source']
            x, y, width, height = entry['rect']
            with Image.open(source) as original, Image.open(path) as crop:
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
        elif kind == 'vector-rasterization':
            scale = entry['scale']
            assert scale in (1, 2, 3)
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
        else:
            raise AssertionError(f'Unknown asset kind: {kind}')
    counts = {kind: sum(item['kind'] == kind for item in manifest['assets'])
              for kind in ('original-pixel-crop', 'vector-reconstruction', 'vector-rasterization')}
    assert counts == {'original-pixel-crop': 48, 'vector-reconstruction': 108, 'vector-rasterization': 324}, counts
    for theme in ('light', 'dark'):
        assert len(list((ROOT / 'svg' / theme).glob('*.svg'))) == 54
        assert len(list((ROOT / 'png' / theme).glob('*.png'))) == 162
    report = {
        'status': 'passed', 'asset_entries': len(manifest['assets']), 'source_crops': 48,
        'svg_files': 108, 'vector_png_files': 324, 'optical_checks': optical_checks,
        'checks': ['Unique assets exist and match SHA-256', '48 source crops match recorded original pixels',
                   '108 safe SVGs share the 24×24 canvas', '324 transparent exports have 24/48/72-pixel canvases',
                   'SVG paint references resolve to unique local definitions',
                   f'{len(optical_checks)} file/folder theme-and-scale exports satisfy fixed optical bounds and center',
                   'Generated SVGs, asset metadata and gallery match canonical definitions'],
        'limits': ['Independent visual comparison remains necessary', 'No native Qt renderer validation in this script',
                   'Reference crops contain baked backgrounds and cannot ship as transparent icons'],
    }
    (ROOT / 'validation.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({key: value for key, value in report.items() if key != 'optical_checks'}, ensure_ascii=False))


if __name__ == '__main__':
    main()
