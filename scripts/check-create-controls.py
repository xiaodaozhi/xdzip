#!/usr/bin/env python3
"""Compare unedited Qt captures with the frozen C01 control review contract."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--input', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
args = p.parse_args()
reference = json.loads((ROOT/'docs/visual/create-reference.json').read_text(encoding="utf-8"))
assert hashlib.sha256((ROOT/reference['source']).read_bytes()).hexdigest() == reference['sha256']
contract = reference['reviewed_control_contract']
checks = []
def check(name, actual, expected, tolerance=.5):
    passed = abs(actual-expected) <= tolerance if isinstance(actual,(int,float)) and isinstance(expected,(int,float)) else actual == expected
    checks.append(dict(name=name,actual=actual,expected=expected,tolerance=tolerance,passed=passed))
for scene in ['create-light','create-dark','create-remove-hover','create-field-focus','create-dropdown','create-removed']:
    capture = json.loads((args.input/(scene+'.json')).read_text(encoding="utf-8"))
    nodes = {n['name']:n for n in capture['nodes'] if n['visible'] and n.get('name')}
    check(scene+'/width', capture['width'], reference['logical_width'])
    check(scene+'/DPR width', capture['imageWidth']/capture['dpr'], capture['width'],1)
    check(scene+'/DPR height', capture['imageHeight']/capture['dpr'], capture['height'],1)
    check(scene+'/actual summary',nodes['createInputSummary']['text'], '2 个文件，19.0 MB' if scene=='create-removed' else reference['summary_fixture']['display'])
    dark = scene=='create-dark'
    for name in ['createName','createTarget','createVolume','createLevel']:
        field,bg = nodes[name],nodes[name+'Background']
        check(scene+'/'+name+'/height',field['height'],reference['field_height'],1.5)
        check(scene+'/'+name+'/radius',bg['radius'],contract['field_radius'],.01)
        check(scene+'/'+name+'/body font',field['fontPixelSize'],reference['body_size'],.01)
        # Keyboard focus intentionally has a distinct accent border.
        if scene in ['create-light','create-dark']:
            check(scene+'/'+name+'/border',bg['border.color'].upper(),'#485469' if dark else contract['field_border'])
        if name in ['createVolume','createLevel']:
            icon = nodes[name+'Indicator']
            check(scene+'/'+name+'/chevron right inset',field['x']+field['width']-icon['x']-icon['width'],contract['select_icon_right_inset'],.01)
            check(scene+'/'+name+'/chevron center',field['y']+field['height']/2,icon['y']+icon['height']/2,.01)
    for name in ['createCancel','startCreate','createAddFiles','createAddDirectories','createChangeTarget']:
        button,bg = nodes[name],nodes[name+'Background']
        check(scene+'/'+name+'/radius',bg['radius'],contract['button_radius'],.01)
        footer = name in ['createCancel','startCreate']
        check(scene+'/'+name+'/height',button['height'],32 if footer else 28,1.5)
        if footer:
            check(scene+'/'+name+'/fully visible',0<=button['y'] and button['y']+button['height']<=capture['height'],True)
        if scene in ['create-light','create-dark']:
            check(scene+'/'+name+'/border',bg['border.color'].upper(),'#485469' if dark else contract['neutral_button_border'])
    group,track = nodes['createFormat'],nodes['createFormatTrack']
    check(scene+'/segmented group height',group['height'],contract['format_group_height'],1)
    check(scene+'/segmented track radius',track['radius'],contract['format_group_radius'],.01)
    slots = [nodes['createFormat-'+str(i)] for i in range(3)]
    check(scene+'/segmented widths',max(n['width'] for n in slots)-min(n['width'] for n in slots),0,1)
    for a,b in zip(slots,slots[1:]):
        check(scene+'/segment seam',a['x']+a['width'],b['x'],.01)
    for i in range(2 if scene=='create-removed' else 3):
        button,icon = nodes[f'createRemoveInput-{i}'],nodes[f'createRemoveInput-{i}Icon']
        check(scene+f'/remove{i}/hit width',button['width'],contract['remove_hit_size'],.01)
        check(scene+f'/remove{i}/artwork',icon['width'],contract['remove_artwork_size'],.01)
        check(scene+f'/remove{i}/x center',button['x']+button['width']/2,icon['x']+icon['width']/2,.01)
        check(scene+f'/remove{i}/y center',button['y']+button['height']/2,icon['y']+icon['height']/2,.01)
        check(scene+f'/remove{i}/canonical glyph',icon['source'].endswith('/remove.png'),True)
report = dict(passed=sum(c['passed'] for c in checks),failed=sum(not c['passed'] for c in checks),checks=checks,
              source=reference['source'],source_sha256=reference['sha256'],
              limits=['Offscreen client captures exclude OS decorations; native desktop appearance needs manual review.',
                      'Measured controls do not imply pixel-identical fonts or identical file icons to historical artwork.'])
args.output.parent.mkdir(parents=True,exist_ok=True)
args.output.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n', encoding='utf-8')
for check in checks:
    if not check['passed']: print(json.dumps(check,ensure_ascii=False))
print(f"C01 Qt control review: {report['passed']} passed, {report['failed']} failed")
raise SystemExit(bool(report['failed']))
