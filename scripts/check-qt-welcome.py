#!/usr/bin/env python3
"""Check actual W01 Qt geometry; captures remain build/CI artifacts."""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--input', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
spec = json.loads((ROOT / 'docs/visual/welcome-reference.json').read_text())
checks = []
for name, size in [('welcome-light', spec['window']), ('welcome-dark', spec['window']), ('welcome-minimum', spec['minimum_window']), ('welcome-dark-minimum', spec['minimum_window'])]:
    data = json.loads((args.input / (name + '.json')).read_text())
    nodes = {node['name']: node for node in data['nodes'] if node['name']}
    expected = {'window width': (data['width'], size[0]), 'window height': (data['height'], size[1]), 'titlebar height': (nodes['archiveTitlebar']['height'], spec['titlebar_height']), 'statusbar height': (nodes['browserStatus']['height'], spec['statusbar_height']), 'open width': (nodes['welcomeOpen']['width'], (spec['content_width'] - spec['action_gap']) / 2), 'open height': (nodes['welcomeOpen']['height'], spec['action_height']), 'create width': (nodes['welcomeCreate']['width'], nodes['welcomeOpen']['width']), 'create height': (nodes['welcomeCreate']['height'], nodes['welcomeOpen']['height']), 'drop height': (nodes['welcomeDropZone']['height'], spec['drop_height'])}
    for label, (actual, target) in expected.items():
        checks.append({'scene': name, 'check': label, 'actual': actual, 'expected': target, 'passed': abs(actual - target) <= 1})
    assert nodes['welcomePage']['visible'] and not nodes['archiveWorkspace']['visible']
    assert abs(data['imageWidth'] - data['width'] * data['dpr']) <= 1
    assert abs(data['imageHeight'] - data['height'] * data['dpr']) <= 1
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps({'checks': checks, 'passed': sum(c['passed'] for c in checks), 'failed': sum(not c['passed'] for c in checks), 'scope': 'Automated Qt capture, not desktop manual acceptance'}, ensure_ascii=False, indent=2) + '\n')
if any(not c['passed'] for c in checks):
    raise SystemExit(json.dumps([c for c in checks if not c['passed']], ensure_ascii=False))
print(f'Qt welcome: {len(checks)} geometry checks passed')
