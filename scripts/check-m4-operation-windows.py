#!/usr/bin/env python3
"""Verify actual creation/task captures in logical pixels, independent of DPR."""
import argparse
import hashlib
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--input', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
reference = json.loads((root/'docs/visual/create-reference.json').read_text(encoding='utf-8'))
if hashlib.sha256((root/reference['source']).read_bytes()).hexdigest() != reference['sha256']:
    raise ValueError('Creation hifi source changed without review')
checks = []

def check(name, actual, expected, tolerance=1):
    passed = abs(actual-expected) <= tolerance if isinstance(actual, (int, float)) else actual == expected
    checks.append(dict(name=name, actual=actual, expected=expected, passed=passed))

for name, width, height, title in [('create-selected', 491, 736, '创建压缩包'),
                                    ('create-completed', 491, 600, '压缩完成')]:
    capture = json.loads((args.input / (name+'.json')).read_text(encoding='utf-8'))
    nodes = [n for n in capture['nodes'] if n['visible']]
    named = {n['name']: n for n in nodes if n.get('name')}
    check(name+'/logical width', capture['width'], width)
    # Short desktops may constrain creation height while preserving footer.
    available = capture.get('availableScreenHeight', height+80)
    expected_height = min(height, available-(60 if name == 'create-selected' else 80))
    if name == 'create-selected':
        check(name+'/screen-constrained logical height', capture['height'], expected_height)
    else:
        check(name+'/content-driven bounded height', 0 < capture['height'] <= expected_height, True)
    check(name+'/DPR width', capture['imageWidth']/capture['dpr'], capture['width'])
    check(name+'/DPR height', capture['imageHeight']/capture['dpr'], capture['height'])
    heading = next(n for n in nodes if n.get('text') == title and n['class'].startswith('QQuickText'))
    check(name+'/title', heading['fontPixelSize'], reference['title_size'])
    if name == 'create-selected':
        for key in ['createName','createTarget','createVolume','createLevel']:
            field = named[key]
            check(key+'/body font', field['fontPixelSize'], reference['body_size'])
            check(key+'/field height', field['height'], reference['field_height'], 1.5)
            check(key+'/horizontal bounds', field['x'] >= 0 and field['x']+field['width'] <= width+1, True)
        target, change = named['createTarget'], named['createChangeTarget']
        check('target/change center', target['y']+target['height']/2, change['y']+change['height']/2, .5)
        mark, label = named['createOpenAfterIndicator'], named['createOpenAfterLabel']
        check('checkbox/label center', mark['y']+mark['height']/2, label['y']+label['height']/2, .5)
        action = named['startCreate']
        check('footer/action height', action['height'], 32)
        check('footer/action inside window', action['y']+action['height'] <= capture['height'], True)
        label = next(n for n in nodes if n.get('text') == '创建压缩包' and n['class'].startswith('QQuickText'))
        check('footer/action font', label['fontPixelSize'], 13)
    else:
        check('task/result panel present', 'taskResultPanel' in named, True)
        check('task/result counts present', 'resultCounts' in named, True)

report = dict(checks=checks, passed=sum(c['passed'] for c in checks), failed=sum(not c['passed'] for c in checks),
              scope='Actual Qt JSON and unedited captures; reviewed HTML logical dimensions',
              limits=['Native window decorations are outside grabWindow', 'Fixture paths and content differ from illustrative hifi/HTML data',
                      'Geometry checks do not certify font contours or desktop appearance'])
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
for c in checks:
    if not c['passed']: print(json.dumps(c))
print(f"Qt creation metrics: {report['passed']} passed, {report['failed']} failed")
raise SystemExit(bool(report['failed']))
