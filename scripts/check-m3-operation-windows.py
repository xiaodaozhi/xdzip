#!/usr/bin/env python3
"""Check actual Qt operation captures against the frozen HTML logical metrics."""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--input', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
layout = json.loads((ROOT / 'docs/visual/layout-scale.json').read_text(encoding='utf-8'))['operation']
checks = []


def check(name, actual, expected, tolerance=1):
    passed = abs(actual - expected) <= tolerance if isinstance(actual, (int, float)) else actual == expected
    checks.append(dict(name=name, actual=actual, expected=expected, tolerance=tolerance, passed=passed))


def text_node(scene, text):
    nodes = [n for n in scene['nodes'] if n['visible'] and n['class'].startswith('QQuickText') and n.get('text') == text]
    if len(nodes) != 1:
        raise ValueError(f'Expected one visible text node: {text}')
    return nodes[0]


for name, kind, title in [('extract-options', 'extract', '解压文件'),
                          ('task-waiting', 'progress', '等待处理重名文件'),
                          ('conflict', 'conflict', '目标位置已有同名文件')]:
    capture = json.loads((args.input / (name + '.json')).read_text(encoding='utf-8'))
    check(name + '/logical width', capture['width'], layout['logical_widths'].get(kind, layout['logical_widths']['default']))
    density = layout.get(kind + '_density', 1)
    check(name + '/title size', text_node(capture, title)['fontPixelSize'], round(layout['components']['titleSize'] * density))
    check(name + '/capture width divided by DPR', capture['imageWidth'] / capture['dpr'], capture['width'])
    check(name + '/capture height divided by DPR', capture['imageHeight'] / capture['dpr'], capture['height'])
    footer_buttons = [node for node in capture['nodes'] if node['visible'] and node['class'].startswith('OperationFooterButton')]
    check(name + '/shared footer actions present', bool(footer_buttons), True)
    for node in footer_buttons:
        if node['visible']:
            label = text_node(capture, node['text'])
            check(name + '/' + node['text'] + '/footer size', label['fontPixelSize'], layout['footer']['fontSize'])
            check(name + '/' + node['text'] + '/inside right edge', node['x'] + node['width'] <= capture['width'] + 1, True)
            check(name + '/' + node['text'] + '/inside bottom edge', node['y'] + node['height'] <= capture['height'] + 1, True)
            check(name + '/' + node['text'] + '/action height', node['height'], layout['controls']['footerButtonHeight'])
            check(name + '/' + node['text'] + '/action minimum width', node['width'] >= 77, True)
            background = next(n for n in capture['nodes'] if 'radius' in n and n['path'].startswith(node['path']+'/'))
            check(name + '/' + node['text'] + '/action radius', background['radius'], layout['controls']['radius'], .01)
    if kind == 'progress':
        body = text_node(capture, '实际耗时取决于文件与磁盘状态。')
        footer = text_node(capture, '关闭此窗口后，任务仍会继续。')
        check(name + '/hint text left edges', body['x'], footer['x'], .5)
        check(name + '/body hint size', body['fontPixelSize'], 12)
        check(name + '/footer hint size', footer['fontPixelSize'], layout['footer']['noteSize'])
        for note in ((body, footer) if capture['height'] >= 490 else (footer,)):
            check(name + '/' + note['text'] + '/visible without scroll', 0 <= note['y'] and note['y'] + note['height'] <= capture['height'], True)

# Destination geometry comes from the independent source measurements, rather
# than copying the QML constants into the expected values.
source = json.loads((ROOT / 'docs/visual/extract-reference.json').read_text(encoding='utf-8'))
ratio = source['source_to_logical'][0] / source['source_to_logical'][1]
capture = json.loads((args.input / 'extract-selected.json').read_text(encoding='utf-8'))
named = {n['name']: n for n in capture['nodes'] if n.get('name') and n['visible']}
field = named['extractDestinationField']
change = named['extractChangeTarget']
check('extract/destination field height', field['height'], source['source_field_height'] * ratio, 1.5)
check('extract/change vertical center', change['y'] + change['height'] / 2, field['y'] + field['height'] / 2, .5)
check('extract/change inside field', change['x'] >= field['x'] and change['x'] + change['width'] <= field['x'] + field['width'], True)
right_gap = field['x'] + field['width'] - change['x'] - change['width']
top_gap = change['y'] - field['y']
bottom_gap = field['y'] + field['height'] - change['y'] - change['height']
check('extract/change right equals top inset', right_gap, top_gap, .5)
check('extract/change right equals bottom inset', right_gap, bottom_gap, .5)
checkbox_mark = named['extractSameFolderIndicator']
status_mark = named['extractTargetStatusIndicator']
checkbox_text = named['extractSameFolderLabel']
status_text = named['extractTargetStatusText']
check('extract/destination marker columns', checkbox_mark['x'] + checkbox_mark['width']/2, status_mark['x'] + status_mark['width']/2, .5)
check('extract/destination text columns', checkbox_text['x'] + checkbox_text['leftPadding'], status_text['x'], .5)
for row, mark, text in [('checkbox', checkbox_mark, checkbox_text), ('status', status_mark, status_text)]:
    check('extract/' + row + ' vertical center', mark['y'] + mark['height']/2, text['y'] + text['height']/2, .5)
check('extract/destination card fill', named['extractDestinationCard']['color'].upper(), source['colors']['card'].upper())
check('extract/conflict card fill', named['extractConflictCard']['color'].upper(), source['colors']['card'].upper())
check('extract/scope body fill', named['extractScopeCard']['color'], '#ffffff')
check('extract/path body size', named['extractTarget']['fontPixelSize'], round(source['source_body_font'] * ratio))

report = dict(passed=sum(c['passed'] for c in checks), failed=sum(not c['passed'] for c in checks),
              checks=checks, source='Authored layout-scale.json and hifi source contracts; current Qt captures',
              limits=['Qt captures use owned fixture data, different from the illustrated photo archive',
                      'System window decorations follow the target OS and are not included in grabWindow',
                      'This check does not certify identical glyph contours or desktop appearance'])
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
for c in checks:
    if not c['passed']:
        print(json.dumps(c, ensure_ascii=False))
print(f"Qt operation metrics: {report['passed']} passed, {report['failed']} failed")
raise SystemExit(bool(report['failed']))
