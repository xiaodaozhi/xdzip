#!/usr/bin/env python3
"""Generate file-association SVG designs and optional local platform exports.

Default writes reviewable design sources. --export writes PNG/ICO/ICNS only
under build/ or dist/. --check validates sources; --check-runtime validates
the tracked product containers. No operating-system association is changed.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import struct
import xml.etree.ElementTree as ET

from association_icon_assets import ASSETS, ROOT, SPEC, icon_svg, source_specs
from brand_assets import ICO_SIZES, ICNS_CHUNKS, master_svg, png_name
from association_types import WINDOWS_ICONS, MACOS_ICONS


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_sources(check: bool) -> None:
    sources = {ASSETS / name: icon_svg(icon, size) for icon, _, size, name in source_specs()}
    for path, data in sources.items():
        if check:
            if not path.exists() or path.read_text(encoding='utf-8') != data:
                raise ValueError(f'Stale design source: {path.relative_to(ROOT)}')
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(data, encoding='utf-8', newline='\n')
    print(f'{len(sources)} SVG sources are consistent')


def containers(output: Path, stem: str) -> dict[str, bytes]:
    payloads = [(output / png_name(stem, size)).read_bytes() for size in ICO_SIZES]
    offset, directory = 6 + 16 * len(payloads), []
    for size, payload in zip(ICO_SIZES, payloads):
        directory.append(struct.pack('<BBBBHHII', size % 256, size % 256, 0, 0, 1, 32, len(payload), offset))
        offset += len(payload)
    ico = struct.pack('<HHH', 0, 1, len(payloads)) + b''.join(directory) + b''.join(payloads)
    chunks = []
    for tag, size in ICNS_CHUNKS.items():
        png = (output / 'macos' / png_name(stem, size)).read_bytes()
        chunks.append(tag.encode() + struct.pack('>I', len(png) + 8) + png)
    content = b''.join(chunks)
    return {'ico': ico, 'icns': b'icns' + struct.pack('>I', len(content) + 8) + content}


def export(output: Path) -> str:
    from PIL import Image, PngImagePlugin
    from playwright.sync_api import sync_playwright

    output.mkdir(parents=True, exist_ok=True)
    (output / 'macos').mkdir(exist_ok=True)
    executable = os.environ.get('XDZIP_CHROMIUM')
    with sync_playwright() as driver:
        browser = driver.chromium.launch(**({'executable_path': executable} if executable else {}))
        version = browser.version
        try:
            page = browser.new_page(device_scale_factor=1)
            page.route('**/*', lambda route: route.abort())
            for icon in SPEC['icons']:
                stem = f"xdzip-{icon['id']}"
                for platform in SPEC['platforms']:
                    folder = output / ('macos' if platform == 'macos' else '')
                    for size in SPEC['png_sizes']:
                        svg = icon_svg(icon, size, platform)
                        page.set_viewport_size({'width': size, 'height': size})
                        page.set_content('<style>html,body{margin:0;background:transparent}svg{display:block;width:100vw;height:100vh}</style>' + svg)
                        raw = page.screenshot(omit_background=True)
                        receipt = PngImagePlugin.PngInfo()
                        receipt.add_text('xdzip-render-sha256', digest(svg.encode()))
                        receipt.add_text('xdzip-format-id', icon['id'])
                        receipt.add_text('xdzip-platform', platform)
                        with Image.open(io.BytesIO(raw)) as image:
                            image.convert('RGBA').save(folder / png_name(stem, size), pnginfo=receipt)
                for suffix, data in containers(output, stem).items():
                    (output / f'{stem}.{suffix}').write_bytes(data)
        finally:
            browser.close()
    return version


def validate_exports(output: Path) -> dict:
    from PIL import Image

    records = []
    for icon in SPEC['icons']:
        stem = f"xdzip-{icon['id']}"
        for platform in SPEC['platforms']:
            for size in SPEC['png_sizes']:
                path = output / ('macos' if platform == 'macos' else '') / png_name(stem, size)
                with Image.open(path) as image:
                    if image.mode != 'RGBA' or image.size != (size, size):
                        raise ValueError(f'Wrong canvas: {path.name}')
                    if image.info.get('xdzip-render-sha256') != digest(icon_svg(icon, size, platform).encode()):
                        raise ValueError(f'Stale raster source: {path.name}')
                    alpha = image.getchannel('A')
                    bounds = alpha.point(lambda value: 255 if value >= 128 else 0).getbbox()
                    canvas = SPEC['platforms'][platform]['viewbox'][2]
                    target = size * (SPEC['platforms'][platform]['body_height_ratio'] + (1.7 / canvas if size > 32 else 0))
                    if alpha.getextrema() != (0, 255) or not bounds or abs(bounds[3] - bounds[1] - target) > 1.25:
                        raise ValueError(f'Wrong optical height or transparency: {path.name}: {bounds}')
                    if any(abs((bounds[i] + bounds[i + 2]) / 2 - size / 2) > 1 for i in [0, 1]):
                        raise ValueError(f'Off-centre body: {path.name}: {bounds}')
                    ratio = SPEC['platforms'][platform]['body_height_ratio']
                    width = size * 641.7 / canvas if size > 32 else .76 * (2 * round(size * ratio / 2))
                    if abs(bounds[2] - bounds[0] - width) > 1.25:
                        raise ValueError(f'Wrong optical width: {path.name}: {bounds}')
                    if any(alpha.getpixel(point) for point in [(0, 0), (size - 1, 0), (0, size - 1), (size - 1, size - 1)]):
                        raise ValueError(f'Nontransparent corner: {path.name}')
                    records.append({'file': path.relative_to(output).as_posix(), 'sha256': digest(path.read_bytes()),
                                    'size': size, 'body_bounds': bounds, 'platform': platform})
        for suffix, expected in containers(output, stem).items():
            path = output / f'{stem}.{suffix}'
            if path.read_bytes() != expected:
                raise ValueError(f'Stale container: {path.name}')
            with Image.open(path) as image:
                if suffix == 'ico':
                    if image.ico.sizes() != {(s, s) for s in ICO_SIZES}:
                        raise ValueError(f'Missing ICO representation: {path.name}')
                    for size in ICO_SIZES:
                        decoded = image.ico.getimage((size, size)).convert('RGBA')
                        with Image.open(output / png_name(stem, size)) as png:
                            if decoded.tobytes() != png.tobytes():
                                raise ValueError(f'ICO decode mismatch: {path.name}')
                else:
                    for representation in image.info['sizes']:
                        decoded = image.icns.getimage(representation).convert('RGBA')
                        size = representation[0] * representation[2]
                        with Image.open(output / 'macos' / png_name(stem, size)) as png:
                            if decoded.tobytes() != png.tobytes():
                                raise ValueError(f'ICNS decode mismatch: {path.name}')
    return {'png_count': len(records), 'ico_count': len(SPEC['icons']), 'icns_count': len(SPEC['icons']),
            'records': records, 'native_shell_acceptance': 'not performed',
            'scope': 'design exports only; no associations installed'}


def validate_runtime_assets() -> None:
    """Validate each tracked platform representation and its vector receipt."""
    import importlib.util
    from PIL import Image
    legacy_spec = importlib.util.spec_from_file_location('brand_export', ROOT / 'scripts/build-brand-assets.py')
    legacy = importlib.util.module_from_spec(legacy_spec)
    legacy_spec.loader.exec_module(legacy)
    icons = {icon['id']: icon for icon in SPEC['icons']}
    total = 0
    for suffix, ids, platform in [('ico', WINDOWS_ICONS.values(), 'default'), ('icns', MACOS_ICONS, 'macos')]:
        for icon_id in dict.fromkeys(ids):
            path = ASSETS / f'xdzip-{icon_id}.{suffix}'
            with Image.open(path) as container:
                if suffix == 'ico':
                    if container.ico.sizes() != {(s, s) for s in ICO_SIZES}:
                        raise ValueError(f'Missing ICO sizes: {path.name}')
                    representations = [(s, container.ico.getimage((s, s))) for s in ICO_SIZES]
                else:
                    expected = {(s, s, 1) for s in (16, 32, 64, 128, 256, 512)} | {(s, s, 2) for s in (16, 32, 128, 256, 512)}
                    if set(container.info['sizes']) != expected:
                        raise ValueError(f'Missing ICNS sizes: {path.name}')
                    representations = [(r[0] * r[2], container.icns.getimage(r)) for r in container.info['sizes']]
                for size, image in representations:
                    svg = legacy.source_svg('xdzip-zip', size, platform) if icon_id == 'zip' else icon_svg(icons[icon_id], size, platform)
                    if image.info.get('xdzip-render-sha256') != digest(svg.encode()):
                        raise ValueError(f'Stale icon source: {path.name} {size}')
                    if icon_id == 'zip':
                        if image.info.get('xdzip-source-sha256') != digest(master_svg('archive').encode()):
                            raise ValueError(f'Stale legacy ZIP master: {path.name}')
                    elif image.info.get('xdzip-format-id') != icon_id or image.info.get('xdzip-platform') != platform:
                        raise ValueError(f'Wrong icon identity: {path.name}')
                    rgba = image.convert('RGBA')
                    alpha = rgba.getchannel('A')
                    if rgba.size != (size, size) or alpha.getextrema() != (0, 255) or not alpha.getbbox():
                        raise ValueError(f'Invalid icon image: {path.name} {size}')
                    if any(alpha.getpixel(point) for point in [(0, 0), (size-1, 0), (0, size-1), (size-1, size-1)]):
                        raise ValueError(f'Clipped icon corner: {path.name} {size}')
                    bounds = alpha.point(lambda value: 255 if value >= 128 else 0).getbbox()
                    ratio = SPEC['platforms'][platform]['body_height_ratio']
                    canvas = SPEC['platforms'][platform]['viewbox'][2]
                    expected_height = size * (ratio + (1.7 / canvas if size > 32 else 0))
                    if not bounds or abs(bounds[3] - bounds[1] - expected_height) > 1.25:
                        raise ValueError(f'Wrong optical height: {path.name} {size}')
                    total += 1
    print(f'{total} tracked ICO/ICNS representations and source receipts verified')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--check-runtime', action='store_true', help='Validate tracked product ICO and ICNS representations')
    parser.add_argument('--export', type=Path, help='Optional build/ or dist/ platform output folder')
    args = parser.parse_args()
    output = args.export.resolve() if args.export else None
    if output and not any(output.is_relative_to(ROOT / folder) and output != ROOT / folder for folder in ['build', 'dist']):
        parser.error('--export must be a child of repository build/ or dist/')
    for icon, _, size, _ in source_specs():
        root = ET.fromstring(icon_svg(icon, size))
        if any(node.tag.rsplit('}', 1)[-1] in ['text', 'image', 'script', 'foreignObject'] for node in root.iter()):
            raise ValueError('Icon must contain self-contained vectors, without fonts or embedded images')
    legacy_zip = ET.fromstring(master_svg('archive'))
    current_zip = ET.fromstring(icon_svg(next(icon for icon in SPEC['icons'] if icon['id'] == 'zip')))
    if ET.tostring(list(legacy_zip)[-1]) != ET.tostring(list(current_zip)[-1]):
        raise ValueError('ZIP lettering must preserve the original document master path exactly')
    write_sources(args.check)
    if args.check_runtime:
        validate_runtime_assets()
    if output:
        version = None if args.check else export(output)
        result = validate_exports(output)
        if not args.check:
            result['renderer'] = {'name': 'Chromium', 'version': version, 'dpr': 1}
            (output / 'validation.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
        print(f"Validated {result['png_count']} PNGs, {result['ico_count']} ICOs and {result['icns_count']} ICNS containers")


if __name__ == '__main__':
    main()
