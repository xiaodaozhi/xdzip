#!/usr/bin/env python3
"""Read-only integrity checks for fixed design sources and derived asset files."""
from pathlib import Path
import hashlib
import json
import xml.etree.ElementTree as ET
from PIL import Image

ROOT = Path(__file__).resolve().parents[1] / 'design/hifi/v2'


def check_hash(entry):
    path = ROOT / entry['file']
    assert path.is_file(), entry['file']
    assert hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256'], entry['file']
    return path


def main():
    manifest = json.loads((ROOT / 'manifest.json').read_text())
    assert len(manifest['screens']) == 8
    for screen in manifest['screens']:
        path = check_hash(screen)
        with Image.open(path) as image:
            image.verify()
    crops = vectors = rasters = 0
    for entry in manifest['assets']:
        path = check_hash(entry)
        kind = entry['kind']
        if kind == 'original-pixel-crop':
            crops += 1
            x, y, width, height = entry['rect']
            with Image.open(ROOT / entry['source']) as original, Image.open(path) as sliced:
                assert 0 <= x < x + width <= original.width
                assert 0 <= y < y + height <= original.height
                assert sliced.size == (width, height)
                # Cropping here is read-only comparison; no image is written.
                assert sliced.convert('RGBA').tobytes() == original.convert('RGBA').crop((x, y, x + width, y + height)).tobytes(), entry['id']
        elif kind == 'vector-reconstruction':
            vectors += 1
            tree = ET.parse(path)
            assert tree.getroot().get('viewBox') == '0 0 24 24'
            for node in tree.iter():
                assert node.tag.rsplit('}', 1)[-1] not in {'script', 'image', 'text', 'foreignObject'}
                assert not any(key.endswith('href') for key in node.attrib)
        elif kind == 'vector-rasterization':
            rasters += 1
            assert (ROOT / entry['source']).exists()
            with Image.open(path) as image:
                assert image.size == tuple(entry['size'])
                assert image.size == (24 * entry['scale'], 24 * entry['scale'])
                alpha = image.convert('RGBA').getchannel('A')
                low, high = alpha.getextrema()
                assert low == 0 and high > 0, entry['id']
        else:
            raise AssertionError(f'Unknown asset kind: {kind}')
    assert (crops, vectors, rasters) == (24, 28, 84)
    assert (ROOT / manifest['shared_asset_base'] / 'manifest.json').exists()
    report = {
        'status': 'passed', 'boards': 8, 'source_crops': crops,
        'svg_files': vectors, 'transparent_pngs': rasters,
        'checks': ['Source and asset hashes match', 'Crops are in bounds and pixel-identical',
                   'SVG files parse without scripts, images, fonts or external references',
                   'Raster sizes and alpha channels are valid', 'Shared v1 asset base exists'],
        'not_validated': ['Native Qt rendering', 'Pixel-perfect vector reconstruction',
                          'Human visual approval', 'Complete platform/theme/state matrix'],
    }
    (ROOT / 'validation.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(report, ensure_ascii=False))


if __name__ == '__main__':
    main()
