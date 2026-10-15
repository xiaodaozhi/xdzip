#!/usr/bin/env python3
"""Check actual Qt result-window metrics against the authored R01 contract."""
import argparse
import hashlib
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--input', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
reference = json.loads((ROOT/'docs/visual/result-reference.json').read_text(encoding='utf-8'))
assert hashlib.sha256((ROOT/reference['source']).read_bytes()).hexdigest() == reference['sha256']
checks = []

def check(name, actual, expected, tolerance=1):
    passed = abs(actual-expected) <= tolerance if isinstance(actual, (float,int)) and isinstance(expected, (float,int)) else actual == expected
    checks.append(dict(name=name,actual=actual,expected=expected,passed=passed))

for scene in ['result-partial', 'result-dark', 'result-details']:
    capture = json.loads((args.input/(scene+'.json')).read_text(encoding='utf-8'))
    visible = [n for n in capture['nodes'] if n['visible']]
    named = {n['name']:n for n in visible if n.get('name')}
    check(scene+'/width',capture['width'],reference['logical_width'])
    maximum_height = min(reference['client_height'], capture.get('availableScreenHeight', 680)-80)
    natural_height = math.ceil(named['taskBodyContent']['implicitHeight'] + named['taskFooter']['implicitHeight'])
    check(scene+'/content-driven height', capture['height'], min(natural_height, maximum_height))
    check(scene+'/maximum height', 0 < capture['height'] <= maximum_height, True)
    check(scene+'/DPR width',capture['imageWidth']/capture['dpr'],capture['width'])
    check(scene+'/single heading',len([n for n in visible if n.get('text')=='部分文件未完成']),1)
    for text,key in [('部分文件未完成','title_size'),('未完成的文件','section_size'),('目标位置','section_size'),('查看处理明细','body_size')]:
        node = next(n for n in visible if n.get('text')==text and n['class'].startswith('QQuickText'))
        check(scene+'/'+text,node['fontPixelSize'],reference[key])
    card = named['resultCounts']
    check(scene+'/summary radius',card['radius'],reference['card_radius'],.01)
    check(scene+'/summary normal height',card['height'],80,1.5)
    for node in visible:
        if node['path'].startswith(card['path']+'/') and node.get('text') in ['2','0','1']:
            check(scene+'/counter '+node['text'],node['fontPixelSize'],reference['count_size'])
        if node['class'].startswith('OperationFooterButton'):
            check(scene+'/footer height',node['height'],32)
            check(scene+'/footer fully inside',0 <= node['y'] and node['y']+node['height']<=capture['height'],True)
    check(scene+'/disclosure present','resultDetailsToggle' in named,True)

args.output.parent.mkdir(parents=True,exist_ok=True)
args.output.write_text(json.dumps(dict(checks=checks,passed=sum(c['passed'] for c in checks),failed=sum(not c['passed'] for c in checks)),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
for c in checks:
    if not c['passed']: print(json.dumps(c,ensure_ascii=False))
print(f"Result UI: {sum(c['passed'] for c in checks)} passed, {sum(not c['passed'] for c in checks)} failed")
raise SystemExit(any(not c['passed'] for c in checks))
