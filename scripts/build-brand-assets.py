#!/usr/bin/env python3
"""Build and validate xdzip's transparent PNG, Windows ICO and macOS ICNS icons.

Every size is rendered directly from vector geometry. Small exports use
24-unit optical profiles, without illegible lettering or large-icon shadows.
PNG provenance stays inside each file, including the PNGs packaged in ICO/ICNS.
The canonical asset manifest inventories these exports; no second manifest is
written here.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import os
from pathlib import Path
import shutil
import struct
import xml.etree.ElementTree as ET

from PIL import Image, PngImagePlugin

from brand_assets import (EXPORT_SIZES, ICO_SIZES, ICNS_CHUNKS, brand_shapes,
                          master_svg, mark_svg, png_name)

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'design/assets/brand'


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def source_svg(stem: str, size: int) -> str:
    if size > 32:
        return master_svg('application' if stem == 'xdzip' else 'archive')
    icon = 'app-mark' if stem == 'xdzip' else 'archive'
    # Enlarge the document's badge at 16/24 pixels so its U opening and four
    # staggered ribbons survive native rasterization. The signature is unchanged.
    profile = 'ui' if stem == 'xdzip' else 'tiny' if size <= 24 else 'small'
    # Platform canvases omit the UI slot's extra padding. Keep small-size
    # optical detail, without scaling a previously rasterized PNG.
    shape = brand_shapes('light', document_profile=profile)[icon]
    # Snap the 20-unit silhouette to an even number of physical pixels so
    # near-opaque edge rows do not disappear at 16/24/32 px.
    body_pixels = 2 * round(size * .90 / 2)
    extent = 20 * size / body_pixels
    inset = (24 - extent) / 2
    return ('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" '
            f'viewBox="{inset:.12g} {inset:.12g} {extent:.12g} {extent:.12g}" fill="none">{shape}</svg>\n')


def png_bytes(stem: str, size: int) -> bytes:
    return (OUTPUT / png_name(stem, size)).read_bytes()


def ico_bytes(stem: str) -> bytes:
    payloads = [png_bytes(stem, size) for size in ICO_SIZES]
    header = struct.pack('<HHH', 0, 1, len(payloads))
    offset = 6 + 16 * len(payloads)
    directory = []
    for size, payload in zip(ICO_SIZES, payloads):
        directory.append(struct.pack('<BBBBHHII', size if size < 256 else 0,
                                     size if size < 256 else 0, 0, 0, 1, 32,
                                     len(payload), offset))
        offset += len(payload)
    return header + b''.join(directory) + b''.join(payloads)


def icns_bytes(stem: str) -> bytes:
    chunks = []
    for tag, size in ICNS_CHUNKS.items():
        payload = png_bytes(stem, size)
        chunks.append(tag.encode('ascii') + struct.pack('>I', len(payload) + 8) + payload)
    data = b''.join(chunks)
    return b'icns' + struct.pack('>I', len(data) + 8) + data


def validate() -> None:
    """Check exact vector sources, PNG receipts, transparency and package records."""
    expected_svgs = {'mark.svg': mark_svg(), 'xdzip.svg': master_svg('application'),
                     'xdzip-zip.svg': master_svg('archive')}
    for name, expected in expected_svgs.items():
        path = OUTPUT / name
        if not path.exists() or path.read_text() != expected:
            raise ValueError(f'Stale brand vector: {name}')
        ET.fromstring(expected)
    with Image.open(OUTPUT / 'identity-reference.png') as reference:
        reference.verify()
    for stem in ['xdzip', 'xdzip-zip']:
        source_hash = digest((OUTPUT / f'{stem}.svg').read_bytes())
        for size in EXPORT_SIZES:
            path = OUTPUT / png_name(stem, size)
            with Image.open(path) as picture:
                if picture.size != (size, size) or picture.mode != 'RGBA':
                    raise ValueError(f'Wrong brand canvas: {path.name}')
                if picture.info.get('xdzip-source-sha256') != source_hash:
                    raise ValueError(f'Stale brand PNG source receipt: {path.name}')
                if picture.info.get('xdzip-render-sha256') != digest(source_svg(stem, size).encode()):
                    raise ValueError(f'Stale brand PNG rendering profile: {path.name}')
                alpha = picture.getchannel('A')
                if alpha.getextrema() != (0, 255) or not alpha.getbbox():
                    raise ValueError(f'Missing transparent margin or opaque body: {path.name}')
                # Measure the 50%-coverage silhouette, never the shadow's alpha
                # tails. An independent 90% target catches double padding in any
                # size; 1.25 px covers edge antialiasing and small-icon rounding.
                bounds = alpha.point(lambda value: 255 if value >= 128 else 0).getbbox()
                # Large document paper has a 1.7-unit outline beyond its path.
                expected_height = size * (.90 + (1.7 / 960 if stem == 'xdzip-zip' and size > 32 else 0))
                if not bounds or abs(bounds[3] - bounds[1] - expected_height) > 1.25:
                    raise ValueError(f'Wrong optical body height: {path.name}: {bounds}')
                expected_width = (size * .90 if stem == 'xdzip' else
                                  size * (641.7 / 960) if size > 32 else .76 * (2 * round(size * .90 / 2)))
                if abs(bounds[2] - bounds[0] - expected_width) > 1.25:
                    raise ValueError(f'Wrong optical body width: {path.name}: {bounds}')
                if any(abs((bounds[axis] + bounds[axis + 2]) / 2 - size / 2) > 1
                       for axis in (0, 1)):
                    raise ValueError(f'Off-centre optical body: {path.name}: {bounds}')
                if any(alpha.getpixel(point) for point in [(0, 0), (size - 1, 0),
                                                            (0, size - 1), (size - 1, size - 1)]):
                    raise ValueError(f'Clipped brand corner: {path.name}')
        ico = (OUTPUT / f'{stem}.ico').read_bytes()
        if ico != ico_bytes(stem):
            raise ValueError(f'Stale ICO directory or embedded images: {stem}')
        with Image.open(io.BytesIO(ico)) as image:
            if image.format != 'ICO' or image.ico.sizes() != {(s, s) for s in ICO_SIZES}:
                raise ValueError(f'Unreadable ICO representations: {stem}')
            for size in ICO_SIZES:
                if image.ico.getimage((size, size)).size != (size, size):
                    raise ValueError(f'Unreadable ICO size {size}: {stem}')
        icns = (OUTPUT / f'{stem}.icns').read_bytes()
        if icns != icns_bytes(stem):
            raise ValueError(f'Stale ICNS chunk directory or embedded images: {stem}')
        with Image.open(io.BytesIO(icns)) as image:
            if image.format != 'ICNS':
                raise ValueError(f'Unreadable ICNS: {stem}')
            for size in image.info['sizes']:
                # Decode each density representation through the ICNS reader;
                # generic Image.size is a two-tuple, while ICNS sizes add density.
                decoded = image.icns.getimage(size)
                if decoded.size != (size[0] * size[2], size[1] * size[2]):
                    raise ValueError(f'Unreadable ICNS size {size}: {stem}')
    print('2 brand families: 18 transparent PNGs, 14 ICO representations, 22 ICNS chunks; sources and receipts valid')


def build(reference: Path | None) -> None:
    from playwright.sync_api import sync_playwright

    OUTPUT.mkdir(parents=True, exist_ok=True)
    if reference:
        if reference.resolve() == (OUTPUT / 'identity-reference.png').resolve():
            raise ValueError('Reference source and destination must be different')
        shutil.copyfile(reference, OUTPUT / 'identity-reference.png')
    if not (OUTPUT / 'identity-reference.png').is_file():
        raise ValueError('Supply the generated visual direction with --reference on the first build')
    (OUTPUT / 'mark.svg').write_text(mark_svg())
    for stem, kind in [('xdzip', 'application'), ('xdzip-zip', 'archive')]:
        (OUTPUT / f'{stem}.svg').write_text(master_svg(kind))
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(
            executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
            args=['--no-sandbox', '--disable-dev-shm-usage'])
        try:
            page = browser.new_page(device_scale_factor=1)
            page.route('**/*', lambda route: route.abort())
            for stem in ['xdzip', 'xdzip-zip']:
                source_hash = digest((OUTPUT / f'{stem}.svg').read_bytes())
                for size in EXPORT_SIZES:
                    svg = source_svg(stem, size)
                    page.set_viewport_size({'width': size, 'height': size})
                    page.set_content('<style>html,body{margin:0;background:transparent}svg{display:block;width:100vw;height:100vh}</style>' + svg)
                    raw = page.screenshot(omit_background=True)
                    with Image.open(io.BytesIO(raw)) as image:
                        receipt = PngImagePlugin.PngInfo()
                        receipt.add_text('xdzip-source-sha256', source_hash)
                        receipt.add_text('xdzip-render-sha256', digest(svg.encode()))
                        image.convert('RGBA').save(OUTPUT / png_name(stem, size), pnginfo=receipt)
        finally:
            browser.close()
    for stem in ['xdzip', 'xdzip-zip']:
        (OUTPUT / f'{stem}.ico').write_bytes(ico_bytes(stem))
        (OUTPUT / f'{stem}.icns').write_bytes(icns_bytes(stem))
    validate()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, help='Generated brand design sheet; preserved byte-for-byte')
    parser.add_argument('--check', action='store_true', help='Validate tracked exports without rasterizing')
    args = parser.parse_args()
    if args.check and args.reference:
        parser.error('--reference cannot be combined with --check')
    if args.check:
        validate()
    else:
        build(args.reference)


if __name__ == '__main__':
    main()
