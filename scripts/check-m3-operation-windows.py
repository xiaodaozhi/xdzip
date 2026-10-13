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
reference = json.loads((ROOT / 'docs/qa/m3/html-operation-metrics.json').read_text())['scenes']
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
    capture = json.loads((args.input / (name + '.json')).read_text())
    model = reference[kind]
    window = next(n for n in model if n['selector'] == 'DIALOG.')
    heading = next(n for n in model if n['selector'] == 'H1.')
    check(name + '/logical width', capture['width'], round(window['w']))
    check(name + '/title size', text_node(capture, title)['fontPixelSize'], round(heading['font']))
    check(name + '/capture width divided by DPR', capture['imageWidth'] / capture['dpr'], capture['width'])
    check(name + '/capture height divided by DPR', capture['imageHeight'] / capture['dpr'], capture['height'])
    for node in capture['nodes']:
        if node['visible'] and node['class'].startswith('AppButton') and node['y'] > capture['height'] - 120:
            label = text_node(capture, node['text'])
            check(name + '/' + node['text'] + '/footer size', label['fontPixelSize'], 15)
            check(name + '/' + node['text'] + '/inside right edge', node['x'] + node['width'] <= capture['width'] + 1, True)
            check(name + '/' + node['text'] + '/inside bottom edge', node['y'] + node['height'] <= capture['height'] + 1, True)
            if kind == 'extract':
                check(name + '/' + node['text'] + '/action height', node['height'], 40)
                check(name + '/' + node['text'] + '/action minimum width', node['width'] >= 96, True)
            else:
                check(name + '/' + node['text'] + '/action height', node['height'], 38)
    if kind == 'progress':
        body = text_node(capture, '实际耗时取决于文件与磁盘状态。')
        footer = text_node(capture, '关闭此窗口后，任务仍会继续。')
        check(name + '/hint text left edges', body['x'], footer['x'], .5)
        check(name + '/body hint size', body['fontPixelSize'], 15)
        check(name + '/footer hint size', footer['fontPixelSize'], 13)
        for note in (body, footer):
            check(name + '/' + note['text'] + '/visible without scroll', 0 <= note['y'] and note['y'] + note['height'] <= capture['height'], True)

report = dict(passed=sum(c['passed'] for c in checks), failed=sum(not c['passed'] for c in checks),
              checks=checks, source='Frozen HTML logical metrics; actual Qt JSON and unedited captures',
              limits=['Qt captures use owned fixture data, different from the illustrated photo archive',
                      'System window decorations follow the target OS and are not included in grabWindow',
                      'This check does not certify identical glyph contours or desktop appearance'])
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
for c in checks:
    if not c['passed']:
        print(json.dumps(c, ensure_ascii=False))
print(f"Qt operation metrics: {report['passed']} passed, {report['failed']} failed")
raise SystemExit(bool(report['failed']))
