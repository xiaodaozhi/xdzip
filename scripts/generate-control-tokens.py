#!/usr/bin/env python3
"""Generate browser control tokens from the shared interaction specification."""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'docs/visual/controls.json'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    tokens = json.loads(SOURCE.read_text())
    tree = json.loads((ROOT / 'docs/visual/directory-tree.json').read_text())
    palette = json.loads((ROOT / 'docs/visual/tokens.json').read_text())
    layout = json.loads((ROOT / 'docs/visual/layout-scale.json').read_text())
    ratio = layout['main']['source_to_logical']
    source_factor = ratio[1] / ratio[0]
    javascript = '// Generated from docs/visual/controls.json; edit the specification.\n'
    javascript += 'window.XDZIP_CONTROL_TOKENS = Object.freeze(' + json.dumps(tokens, ensure_ascii=False) + ');\n'
    declarations = ['/* Generated logical tokens; artboards convert them to source coordinates. */', ':root {',
                    f'  --ui-source-factor: {source_factor};',
                    f'  --main-density: {layout["main"]["density"]};',
                    '  --tree-source-factor: calc(var(--ui-source-factor) * var(--main-density));']
    operation_ratio = layout['operation']['source_to_logical']
    declarations.append(f'  --operation-source-factor: {operation_ratio[1] / operation_ratio[0]};')
    for kind, density in layout['operation']['oversized_source_density'].items():
        declarations.append(f'  --operation-{kind}-density: {density};')
    for kind, ratio in layout['operation'].get('source_to_logical_by_view', {}).items():
        declarations.append(f'  --operation-{kind}-density: calc({ratio[0] / ratio[1]} * var(--ui-source-factor));')
    for kind, value in layout['operation']['logical_widths'].items():
        declarations.append(f'  --operation-{kind}-width: calc({value}px * var(--ui-source-factor));')
    for name, value in layout['operation']['footer'].items():
        key = ''.join('-' + char.lower() if char.isupper() else char for char in name)
        declarations.append(f'  --operation-footer-{key}: calc({value}px * var(--ui-source-factor));')
    for name, value in layout['operation']['controls'].items():
        key = ''.join('-' + char.lower() if char.isupper() else char for char in name)
        declarations.append(f'  --operation-control-{key}: calc({value}px * var(--ui-source-factor));')
    for name, value in layout['operation']['components'].items():
        if name.startswith('destinationIndicator') or name == 'footerHeight':
            key = ''.join('-' + char.lower() if char.isupper() else char for char in name)
            declarations.append(f'  --operation-{key}: calc({value}px * var(--ui-source-factor));')
    for name, value in tokens['size'].items():
        key = ''.join('-' + char.lower() if char.isupper() else char for char in name)
        declarations.append(f'  --ui-{key}: calc({value}px * var(--ui-source-factor));')
    for name, value in layout['main']['interaction'].items():
        key = ''.join('-' + char.lower() if char.isupper() else char for char in name)
        declarations.append(f'  --main-{key}: calc({value}px * var(--ui-source-factor));')
    for name, value in tokens['shadow'].items():
        key = ''.join('-' + char.lower() if char.isupper() else char for char in name)
        css_value = str(value) if name == 'layers' else f'calc({value}px * var(--ui-source-factor))'
        declarations.append(f'  --ui-menu-shadow-{key}: {css_value};')
    for name, value in tree['geometry'].items():
        declarations.append(f'  --tree-{name.replace("_", "-")}: calc({value}px * var(--ui-source-factor));')
    for theme in ('light', 'dark'):
        if theme == 'dark':
            declarations += ['}', ':root[data-theme="dark"] {']
        for name, value in tokens[theme].items():
            key = ''.join('-' + char.lower() if char.isupper() else char for char in name)
            declarations.append(f'  --ui-{key}: {value};')
        for name, value in palette[theme].items():
            if name.startswith('operation') or name in {'accentHover', 'accentPressed', 'accentDisabled', 'accentDisabledText'}:
                key = ''.join('-' + char.lower() if char.isupper() else char for char in name)
                declarations.append(f'  --{key}: {value};')
        for name, value in tree['colors'][theme].items():
            if not value.startswith('#'):
                source_theme, key = value.split('.')
                value = palette[source_theme][key]
            declarations.append(f'  --tree-{name.replace("_", "-")}-color: {value};')
    declarations += ['}', ':root[data-ui-scale="logical"] { --ui-source-factor: 1; }', '']
    outputs = {ROOT / 'design/html/controls-tokens.js': javascript,
               ROOT / 'design/html/controls-tokens.css': '\n'.join(declarations)}
    for path, output in outputs.items():
        if args.check:
            if path.read_text() != output:
                raise SystemExit(f'Stale shared control tokens: {path.relative_to(ROOT)}')
        else:
            path.write_text(output)
    print('Shared control tokens: browser scripts and palette match the specification')


if __name__ == '__main__':
    main()
