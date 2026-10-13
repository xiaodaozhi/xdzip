#!/usr/bin/env python3
"""Compare real Qt captures against frozen HTML/art measurements, never Qt theme tokens.

The gate measures geometry, declared font metrics, alignment and asset provenance.
It does not turn different archive contents or OS rasterization into a likeness score.
"""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def audit(directory):
    reference = json.loads((ROOT / 'docs/visual/qt-main-reference.json').read_text(encoding='utf-8'))
    scale = reference['logical_width'] / reference['source_window'][2]
    geometry = {key: value * scale for key, value in reference['source_geometry'].items()}
    sizes = {key: round(value * scale) for key, value in reference['source_type'].items()}
    checks = []

    def check(name, actual, expected, tolerance=1):
        numeric = isinstance(actual, (int, float)) and not isinstance(actual, bool)
        passed = abs(actual - expected) <= tolerance if numeric else actual == expected
        checks.append(dict(name=name, actual=actual, expected=expected,
                           tolerance=tolerance if numeric else None, passed=passed))

    for path_key, hash_key in [('source', 'source_sha256'), ('html_capture', 'html_sha256')]:
        digest = hashlib.sha256((ROOT / reference[path_key]).read_bytes()).hexdigest()
        check('frozen/' + path_key, digest, reference[hash_key])

    environments = {}
    for scene in ['main-light', 'main-dark', 'main-integrated-layout']:
        data = json.loads((directory / f'{scene}.json').read_text(encoding='utf-8'))
        environments[scene] = {key: data[key] for key in ['os', 'architecture', 'qt', 'platform', 'dpr']}
        nodes = [n for n in data['nodes'] if n['visible']]

        def record(name, actual, expected, tolerance=1):
            check(scene + '/' + name, actual, expected, tolerance)

        def one(predicate):
            matches = [n for n in nodes if predicate(n)]
            if len(matches) != 1:
                raise ValueError(f'{scene}: expected one node, found {len(matches)}')
            return matches[0]

        def named(name):
            return one(lambda n: n['name'] == name)

        def within(parent, predicate):
            return one(lambda n: n['path'].startswith(parent['path'] + '/') and predicate(n))

        def center(n):
            return n['y'] + n['height'] / 2

        record('window.width', data['width'], reference['logical_width'], 0)
        record('window.height', data['height'], reference['source_window'][3] * scale)
        for name, rectangle in reference['anchors'].items():
            node = named(name)
            for field, expected in zip(['x', 'y', 'width', 'height'], rectangle):
                record(f'{name}.{field}', node[field], expected * scale)

        left, right = named('filePaneHeader'), named('inspectorPaneHeader')
        for field in ['y', 'height']:
            record('paneHeaders.' + field, left[field], right[field], .01)
        left_title, right_title = named('filePaneTitle'), named('inspectorPaneTitle')
        for node in [left_title, right_title]:
            record(node['name'] + '.font', node['fontPixelSize'], sizes['paneTitle'], 0)
            record(node['name'] + '.family', node['fontFamily'], 'Xdzip Design Sans SC')
        record('paneTitles.centerY', center(left_title), center(right_title), .5)
        record('paneTitles.baseline', left_title['y'] + left_title['baselineOffset'],
               right_title['y'] + right_title['baselineOffset'], .5)
        preview = named('previewContent')
        record('preview.top', preview['y'], named('fileTableHeader')['y'], .01)
        record('preview.height', preview['height'], geometry['previewHeight'])
        record('preview.left', preview['x'] - named('archiveInspector')['x'], geometry['inspectorLeftInset'])
        extract = named('inspectorExtract')
        record('preview.actionFits', extract['y'] + extract['height'] <= named('browserStatus')['y'] - 16, True)

        for row_index in range(9):
            row = named(f'entry-{row_index}-0')
            record(f'row{row_index}.height', row['height'], geometry['folderRowHeight' if row_index < 3 else 'fileRowHeight'], .05)
            glyph = within(row, lambda n: n['class'] == 'QQuickImage' and n['width'] > 30)
            label = within(row, lambda n: n['class'].startswith('QQuickText') and bool(n.get('text')))
            record(f'row{row_index}.iconWidth', glyph['width'], geometry['fileIconSize'], .05)
            record(f'row{row_index}.iconHeight', glyph['height'], geometry['fileIconSize'], .05)
            record(f'row{row_index}.iconInset', glyph['x'] - row['x'], geometry['nameIconInset'], .05)
            record(f'row{row_index}.nameInset', label['x'] - row['x'], geometry['nameTextInset'], .05)
            record(f'row{row_index}.iconCenterY', center(glyph), center(row), .5)
            record(f'row{row_index}.textCenterY', center(label), center(row), .5)
            record(f'row{row_index}.font', label['fontPixelSize'], sizes['table'], 0)
            record(f'row{row_index}.family', label['fontFamily'], 'Xdzip Design Sans SC')
            record(f'row{row_index}.canonicalIcon', glyph['source'].startswith('qrc:/xdzip/assets/'), True)
            for column in [1, 2, 3, 4]:
                cell = named(f'entry-{row_index}-{column}')
                text = within(cell, lambda n: n['class'].startswith('QQuickText') and bool(n.get('text')))
                header = named(f'sort-column-{column}')
                align_right = column in [1, 2]
                expected_align = 'AlignRight' if align_right else 'AlignLeft'
                record(f'row{row_index}.col{column}.alignment', text['horizontalAlignment'], expected_align)
                header_edge = header['x'] + header['width'] - header['rightPadding'] if align_right else header['x'] + header['leftPadding']
                text_edge = text['x'] + text['width'] if align_right else text['x']
                record(f'row{row_index}.col{column}.edge', text_edge, header_edge, .5)
        for column in range(5):
            header = named(f'sort-column-{column}')
            record(f'header{column}.font', header['fontPixelSize'], sizes['header'], 0)
            record(f'header{column}.alignment', header['horizontalAlignment'], 'AlignRight' if column in [1, 2] else 'AlignLeft')

        opened, search = named('archiveOpen'), named('archiveSearch')
        record('toolbar.controlHeight', opened['height'], geometry['toolbarControlHeight'], .05)
        record('toolbar.searchHeight', search['height'], geometry['toolbarControlHeight'], .05)
        record('toolbar.searchWidth', search['width'], geometry['searchWidth'])
        record('toolbar.searchX', search['x'], 886 * scale)
        record('toolbar.centerY', center(opened), center(search), .5)
        record('toolbar.font', search['fontPixelSize'], sizes['toolbar'], 0)
        icon = within(opened, lambda n: n['name'] == 'actionIcon')
        record('toolbar.iconSize', icon['width'], geometry['toolbarIconSize'], .05)
        selected_view = within(named('browserViewMenuButton'), lambda n: n['name'] == 'actionIcon')
        record('view.selectedAsset', selected_view['source'].split('/')[-1], 'view-list-selected.png')
        for name, source_x in reference['toolbar_icon_source_x'].items():
            action = named(name)
            glyph = within(action, lambda n: n['name'] == 'actionIcon')
            record(name + '.iconX', glyph['x'], source_x * scale, 1)
        close, plus = named('archiveTabClose'), named('archiveNewTab')
        record('tab.closePlusCenterY', center(close), center(plus), .5)
        tab = named('activeArchiveTab')
        record('tab.width', tab['width'], geometry['tabWidth'], .05)
        record('tab.top', tab['y'], geometry['tabTopInset'], .05)
        tab_x = geometry['tabFirstMacX' if scene == 'main-integrated-layout' else 'tabFirstOtherX']
        record('tab.left', tab['x'], tab_x, .05)
        settings = named('archiveSettings')
        record('settings.right', data['width'] - settings['x'] - settings['width'], geometry['settingsRightInset'], .05)

    return dict(reference=reference['source_commit'], scale=scale,
                method='Frozen HTML/art geometry at 1280 CSS/Qt logical pixels; integer structural boundaries; QFont integer pixel sizes.',
                environments=environments, checks=checks,
                passed=sum(c['passed'] for c in checks), failed=sum(not c['passed'] for c in checks),
                exceptions=reference['intentional_exceptions'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = audit(args.input)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    for failure in (c for c in result['checks'] if not c['passed']):
        print(json.dumps(failure, ensure_ascii=True))
    print(f"Independent Qt fidelity: {result['passed']} passed, {result['failed']} failed")
    if result['failed']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
