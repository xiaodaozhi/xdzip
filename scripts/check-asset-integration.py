#!/usr/bin/env python3
"""Check that Qt and HTML consume the canonical, traceable icon library."""
from html.parser import HTMLParser
from pathlib import Path
import hashlib
import json
import re
import struct
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
ASSET_ROOT = ROOT / 'design/assets'
QUOTED = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def quoted_values(expression):
    """Icon IDs are ASCII; escaped prose must not be interpreted as an ID."""
    return [match[0][1:-1] for match in QUOTED.finditer(expression)]


def selected_values(expression):
    """Read selector results, excluding strings used in branch conditions."""
    if QUOTED.fullmatch(expression):
        return quoted_values(expression)
    return [value[1:-1] for value in re.findall(
        r'[?:]\s*("[^"\\]*"|\'[^\'\\]*\')', expression)]


def check_selector_results(expression):
    """Dynamic branches must resolve to a literal or the checked file selector."""
    if QUOTED.fullmatch(expression):
        return
    masked = QUOTED.sub('ASSET', expression)
    masked = re.sub(r'\bfileIcon\([^)]*\)', 'ASSET', masked)
    for branch in re.finditer(r'[?:]\s*([^?:]+)(?=([?:])|$)', masked):
        # A branch ending at another question mark is a nested condition.
        if branch[2] != '?':
            require(branch[1].strip() == 'ASSET',
                    f'Untracked dynamic HTML icon branch: {expression}')


def call_arguments(source, name):
    """Read named calls, preserving nested arguments and quoted commas.

    This covers the app's small icon-selection expressions without importing
    its DOM state machine or interpreting arbitrary JavaScript.
    """
    for call in re.finditer(r'(?<![\w.])' + re.escape(name) + r'\s*\(', source):
        if re.search(r'\bfunction\s*$', source[max(0, call.start() - 30):call.start()]):
            continue
        start = cursor = call.end()
        stack, quote, escaped, arguments = [], None, False, []
        while cursor < len(source):
            char = source[cursor]
            if quote:
                if escaped:
                    escaped = False
                elif char == '\\':
                    escaped = True
                elif char == quote:
                    quote = None
            elif char in ('"', "'", '`'):
                quote = char
            elif char in '([{':
                stack.append(char)
            elif char == ')' and not stack:
                arguments.append(source[start:cursor].strip())
                yield arguments
                break
            elif char in ')]}':
                require(bool(stack), f'Unbalanced {name}() expression')
                stack.pop()
            elif char == ',' and not stack:
                arguments.append(source[start:cursor].strip())
                start = cursor + 1
            cursor += 1


class IconAttributes(HTMLParser):
    def __init__(self):
        super().__init__()
        self.names = set()

    def handle_starttag(self, tag, attrs):
        for key, value in attrs:
            if key == 'data-icon' and value:
                self.names.add(value)


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
    """Admit exactly the declared high-resolution application icon.

    A platform icon does not weaken shared 24-unit glyph rules. Archive document
    icons and the original logo reference cannot use this runtime exception.
    Package ICO/ICNS payload validation belongs to the brand-export checker.
    """
    require(alias == 'assets/app-icon.png' and prefix.rstrip('/') == '/xdzip',
            'Application icon must have the dedicated /xdzip/assets/app-icon.png alias')
    require(path == (base / 'brand/xdzip.png').resolve(),
            'Application icon alias must reference brand/xdzip.png')
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
        require(scale in (1, 2, 3) and entry['size'] == [24 * scale, 24 * scale],
                f'Invalid icon raster scale: {node.text}')
        require(png_dimensions(path) == entry['size'],
                f'Runtime PNG dimensions differ from manifest: {node.text}')
        checked += 1
    require(applications == 1, 'Qt must register exactly one high-resolution application icon')
    return checked


def check_html(base, assets, html_root):
    source = (html_root / 'app.js').read_text()
    require(not re.search(r'\bconst\s+paths\s*=|\bpaths\s*\[', source),
            'HTML contains a second hand-drawn icon fallback')
    mapping = re.search(r'\bconst\s+assetNames\s*=\s*\{(.*?)\};', source, re.S)
    require(mapping is not None, 'HTML icon aliases must remain explicit')
    aliases = {name: value[1:-1] for name, value in re.findall(
        r'([\w$]+)\s*:\s*("(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\')', mapping[1])}

    attributes = IconAttributes()
    for page in html_root.glob('*.html'):
        attributes.feed(page.read_text())
    names = set(attributes.names) | set(aliases.values())
    dynamic_calls = []
    for args in call_arguments(source, 'icon'):
        argument = args[0]
        names.update(selected_values(argument))
        if not QUOTED.fullmatch(argument):
            dynamic_calls.append(argument)
            # Dataset calls reuse checked attributes or already-rendered assets;
            # fileIcon and ico are resolved from their actual selectors below.
            known = (
                argument in {'n.dataset.icon', 'el.dataset.asset', 'ico'}
                or re.fullmatch(r'fileIcon\(.+\)', argument, re.S)
                or (quoted_values(argument) and '?' in argument)
            )
            require(known, f'Untracked dynamic HTML icon selector: {argument}')
            if '?' in argument:
                check_selector_results(argument)

    selector = re.search(r'function\s+fileIcon\([^)]*\)\s*\{(.*?)\n\}', source, re.S)
    require(selector is not None, 'HTML file icon selector is missing')
    result = re.search(r'\breturn\s+(.*?);', selector[1], re.S)
    require(result is not None, 'HTML file selector must return explicit icon names')
    check_selector_results(result[1])
    # Only ternary results are asset names; JPEG/PDF and suffix tests are prose.
    names.update(selected_values(result[1]))
    require(not re.search(r'\bico\s*=\s*[^\s"\']', source),
            'HTML ico assignments must select explicit icon names')
    names.update(re.findall(r'\bico\s*=\s*["\']([\w-]+)["\']', source))
    for args in call_arguments(source, 'header'):
        if len(args) >= 3 and args[2]:
            selected = selected_values(args[2])
            require(selected, f'Untracked dynamic dialog icon selector: {args[2]}')
            check_selector_results(args[2])
            names.update(selected)

    canonical = {aliases.get(name, name) for name in names}
    for theme in ('light', 'dark'):
        for name in sorted(canonical):
            path = (base / 'svg' / theme / f'{name}.svg').resolve()
            entry = assets.get(path)
            require(entry is not None and entry['kind'] == 'vector-reconstruction',
                    f'HTML icon missing from canonical {theme} library: {name}')
            require(hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256'],
                    f'HTML icon source is stale: {theme}/{name}')
    return len(canonical), len(dynamic_calls)


def main():
    assets = load_assets(ASSET_ROOT)
    qt_count = check_qt(ASSET_ROOT, assets, ROOT / 'apps/desktop/ui.qrc')
    html_count, dynamic_count = check_html(ASSET_ROOT, assets, ROOT / 'design/html')
    print(f'Checked {qt_count} Qt raster aliases and {html_count} HTML icon names '
          f'({dynamic_count} dynamic calls) against one canonical library. '
          'The single 512×512 application icon has separate brand provenance. '
          'Visual fidelity requires separate review.')


if __name__ == '__main__':
    main()
