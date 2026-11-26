# scripts/check-add-dialog.py

"""Measure the C01-derived addition dialog from Qt captures."""
import argparse
import hashlib
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = json.loads((ROOT / 'specs/visual/add-reference.json').read_text(encoding='utf-8'))
PALETTE = json.loads((ROOT / 'specs/visual/tokens.json').read_text(encoding='utf-8'))


def qt_cases(directory):
    cases = []
    for language in ['zh-CN', 'en-US']:
        for factor in [1, 1.25, 1.5]:
            for dark in [False, True]:
                name = f'{language}{factor:g}{int(dark)}'
                capture = json.loads((directory / (name + '.json')).read_text(encoding='utf-8'))
                nodes = {node['name']: node for node in capture['nodes'] if node['name']}
                assert capture['width'] == REFERENCE['logical_width'], name
                assert capture['height'] <= min(736, capture['availableScreenHeight'] - 60), name
                for key in ['updateInputPanel', 'updateDestinationCard', 'updateOptionsCard']:
                    assert abs(nodes[key]['x'] - REFERENCE['content_inset']) <= 1, (name, key)
                    assert abs(nodes[key]['width'] - (491 - 2 * REFERENCE['content_inset'])) <= 1, (name, key)
                assert nodes['updateHeading']['fontPixelSize'] == math.floor(20 * factor + .5), name
                note = nodes['updateConflictNote']
                assert note['visible'] and note['text'], name
                assert note['fontPixelSize'] == math.floor(REFERENCE['note_size'] * factor + .5), name
                assert note['contentWidth'] <= note['width'] + .5, name
                assert note['contentHeight'] <= note['height'] + .5, name
                assert nodes['updateInputPanel']['color'].upper() == PALETTE['dark' if dark else 'light']['operationCard'].upper(), name
                assert nodes['updateInputSize-0']['text'] == '11 B', name
                assert nodes['updateRemoveInput-0']['width'] == 28, name
                assert nodes['updateRemoveInput-0Icon']['width'] == 24, name
                assert nodes['updateRemoveInput-0Icon']['source'].endswith('/remove.png'), name
                for key in ['updateCancel', 'startUpdate']:
                    node = nodes[key]
                    assert node['y'] >= nodes['updateFooter']['y'], (name, key)
                    assert node['y'] + node['height'] <= capture['height'] + 1, (name, key)
                assert abs(nodes['updateFooter']['y'] + nodes['updateFooter']['height'] - capture['height']) <= 1, name
                if factor == 1:
                    assert abs(nodes['updateInputRow-0']['height'] - REFERENCE['input_row_height']) <= 1, name
                    assert abs(nodes['startUpdate']['height'] - REFERENCE['footer_action_height']) <= 1, name
                cases.append({'name': name, 'width': capture['width'], 'height': capture['height'], 'dpr': capture['dpr'], 'platform': capture['platform'], 'renderer': capture.get('requestedQuickBackend')})
    return cases


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-input', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert hashlib.sha256((ROOT / REFERENCE['source']).read_bytes()).hexdigest() == REFERENCE['sha256']
    args.output.mkdir(parents=True, exist_ok=True)
    qt = qt_cases(args.qt_input)
    (args.output / 'report.json').write_text(json.dumps({'passed': True, 'qt': qt,
        'source': REFERENCE['source'], 'source_sha256': REFERENCE['sha256'],
        'limits': ['Qt offscreen captures do not approve native OS decorations or manual desktop interaction.']},
        ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'Addition dialog: {len(qt)} Qt captures passed')


if __name__ == '__main__':
    main()
