#!/usr/bin/env python3
"""Check that Qt consumes the canonical, traceable icon library."""
from pathlib import Path
import hashlib
import json
import re
import struct
import subprocess
import sys
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
ASSET_ROOT = ROOT / 'design/assets'
QUOTED = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'')

HIGH_RES_ICONS = set(json.loads((ROOT / 'specs/visual/operation-assets.json').read_text())['icons'])


def require(condition, message):
    if not condition:
        raise ValueError(message)


def load_assets(base):
    manifest = json.loads((base / 'manifest.json').read_text())
    assets = {}
    for entry in manifest['assets']:
        path = (base / entry['file']).resolve()
        require(path.is_relative_to(base.resolve()), f'Asset escapes library: {entry["file"]}')
        require(path not in assets, f'Duplicate asset path: {entry["file"]}')
        assets[path] = entry
    return assets


def png_dimensions(path):
    """Read only the PNG header; this checker stays usable in documentation CI."""
    header = path.read_bytes()[:24]
    require(header[:8] == b'\x89PNG\r\n\x1a\n' and header[12:16] == b'IHDR',
            f'Invalid runtime PNG: {path.name}')
    return list(struct.unpack('>II', header[16:24]))


def check_application_icon(base, assets, path, entry, alias, prefix):
    """Admit only the two declared high-resolution application icon profiles.

    A platform icon does not weaken shared 24-unit glyph rules. Archive document
    icons and the original logo reference cannot use this runtime exception.
    Package ICO/ICNS payload validation belongs to the brand-export checker.
    """
    icon_paths = {'assets/app-icon.png': 'brand/xdzip.png',
                  'assets/app-icon-macos.png': 'brand/macos/xdzip.png'}
    require(alias in icon_paths and prefix.rstrip('/') == '/xdzip',
            'Application icon must have a declared /xdzip application alias')
    require(path == (base / icon_paths[alias]).resolve(),
            'Application icon alias must reference its declared platform PNG')
    require(entry['kind'] == 'platform-icon', 'Application icon must be a platform export')
    require(entry.get('purpose') == 'application-icon' and entry.get('format') == 'png',
            'Application icon must have the application PNG purpose')
    require(entry.get('size') == [512, 512] and png_dimensions(path) == [512, 512],
            'Application icon runtime export must be 512×512')
    require(entry.get('source') == 'brand/xdzip.svg', 'Application icon has the wrong brand master')
    source = assets.get((base / 'brand/xdzip.svg').resolve())
    require(source is not None and source['kind'] == 'brand-master',
            'Application icon lacks its tracked brand master')
    require(source.get('purpose') == 'application-icon' and source.get('source') == 'brand/mark.svg',
            'Application master must derive from the current signature')
    mark = assets.get((base / 'brand/mark.svg').resolve())
    require(mark is not None and mark['kind'] == 'brand-master'
            and mark.get('purpose') == 'signature-source', 'Current signature source is untracked')
    require(hashlib.sha256((base / 'brand/mark.svg').read_bytes()).hexdigest() == mark['sha256']
            and source.get('source_sha256') == mark['sha256'], 'Application master has stale signature provenance')
    require(hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256'],
            'Application icon differs from its manifest hash')
    require(hashlib.sha256((base / 'brand/xdzip.svg').read_bytes()).hexdigest() == source['sha256']
            and entry.get('source_sha256') == source['sha256'],
            'Application icon has stale brand source provenance')


def check_qt(base, assets, qrc):
    checked, applications, aliases = 0, 0, set()
    shadow_kinds = {'menu': 'menu-shadow-rasterization', 'frame': 'window-shadow-rasterization'}
    resources = [(resource.attrib.get('prefix', ''), node)
                 for resource in ET.parse(qrc).iter('qresource') for node in resource.findall('file')]
    for prefix, node in resources:
        path = (qrc.parent / node.text).resolve()
        if path.suffix != '.png':
            continue
        alias = node.attrib.get('alias', node.text)
        alias_path = prefix.rstrip('/') + '/' + alias.lstrip('/')
        require(alias_path not in aliases, f'Duplicate runtime image alias: {alias_path}')
        aliases.add(alias_path)
        entry = assets.get(path)
        require(entry is not None, f'Untracked runtime image: {node.text}')
        if path.parent.parent == base and path.parent.name in shadow_kinds:
            require(entry['kind'] == shadow_kinds[path.parent.name], 'Invalid shadow raster')
            require(png_dimensions(path) == entry['size'], 'Invalid shadow dimensions')
            require(entry['border'] * 2 < entry['size'][0], 'Invalid shadow center')
            require(hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256'], 'Stale shadow')
            require(hashlib.sha256((base / entry['source']).read_bytes()).hexdigest() == entry['source_sha256'], 'Stale shadow specification')
            checked += 1
            continue
        if path.is_relative_to((base / 'tray').resolve()):
            require(entry['kind'] == 'tray-rasterization', 'Invalid native tray raster')
            require(entry['logical_size'] == [22, 22], 'Invalid native tray canvas')
            require(png_dimensions(path) == entry['size'], 'Invalid native tray raster size')
            require(hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256'], 'Stale tray raster')
            source = (base / entry['source']).resolve()
            vector = assets.get(source)
            require(vector is not None and vector['kind'] == 'tray-vector', 'Missing tray source')
            require(hashlib.sha256(source.read_bytes()).hexdigest() == vector['sha256'] == entry['source_sha256'], 'Stale tray vector')
            require(hashlib.sha256((base / 'brand/mark.svg').read_bytes()).hexdigest() == vector['source_sha256'], 'Stale tray brand provenance')
            checked += 1
            continue
        if alias == 'assets/app-icon.png' or path.is_relative_to((base / 'brand').resolve()):
            check_application_icon(base, assets, path, entry, alias, prefix)
            applications += 1
            checked += 1
            continue
        require(path.is_relative_to((base / 'png').resolve()),
                f'Only canonical transparent icons can ship: {node.text}')
        require(entry['kind'] == 'vector-rasterization',
                f'Reference/demo images cannot ship as icons: {node.text}')
        require(hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256'], node.text)
        source = (base / entry['source']).resolve()
        vector = assets.get(source)
        require(vector is not None and vector['kind'] == 'vector-reconstruction',
                f'Missing vector provenance: {node.text}')
        require(source.is_relative_to((base / 'svg').resolve()),
                f'Icon source is outside canonical SVGs: {node.text}')
        require(hashlib.sha256(source.read_bytes()).hexdigest() == vector['sha256'],
                f'Stale vector provenance: {node.text}')
        logical_size = entry.get('logical_size')
        scale = entry.get('scale')
        require(logical_size == [24, 24] and vector.get('logical_size') == logical_size,
                f'Runtime icon logical size must be 24×24: {node.text}')
        large = scale == 9 and Path(entry['source']).stem in HIGH_RES_ICONS
        require((scale in (1, 2, 3) or large) and entry['size'] == [24 * scale, 24 * scale],
                f'Invalid icon raster scale: {node.text}')
        require(png_dimensions(path) == entry['size'],
                f'Runtime PNG dimensions differ from manifest: {node.text}')
        checked += 1
    require(applications == 2, 'Qt must register the default and macOS high-resolution application icons')
    return checked


def main():
    subprocess.run([sys.executable, str(ROOT / "scripts/generate-file-icons.py"), "--check"], check=True)
    assets = load_assets(ASSET_ROOT)
    qt_count = check_qt(ASSET_ROOT, assets, ROOT / 'apps/desktop/ui.qrc')
    print(f'Checked {qt_count} Qt raster aliases against the canonical library. '
          'The default and macOS 512×512 application icons have separate platform profiles. '
          'Visual fidelity requires separate review.')


if __name__ == '__main__':
    main()
