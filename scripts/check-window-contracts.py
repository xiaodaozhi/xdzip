#!/usr/bin/env python3
"""Inventory every QML window and reject unreviewed geometry or confirmation forks."""
import argparse
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
ROOT_OBJECT = re.compile(r'^([A-Za-z][\w.]*)\s*\{', re.M)


def inspect(sources, registry):
    errors = []
    parents = {name: match[1] for name, text in sources.items()
               if (match := ROOT_OBJECT.search(text))}
    def is_window(name, seen=()):
        name = name.split('.')[-1]
        return name in {'Window', 'ApplicationWindow', 'Dialog', 'MessageDialog'} or (name not in seen and name in parents and is_window(parents[name], (*seen, name)))
    discovered = {name for name in parents if is_window(name)}
    recorded = set(registry['windows'])
    for name in sorted(discovered - recorded):
        errors.append(f'{name}: window is missing from the reviewed registry')
    for name in sorted(recorded - discovered):
        errors.append(f'{name}: registered window is missing or no longer a window')
    for name in sorted(recorded & discovered):
        policy = registry['windows'][name]
        text = sources[name]
        if parents[name] != policy['parent']:
            errors.append(f'{name}: parent must remain {policy["parent"]}')
        if 'width' in policy and not re.search(r'^    width:\s*' + re.escape(policy['width']) + r'\s*$', text, re.M):
            errors.append(f'{name}: width must use reviewed expression {policy["width"]}')
        if policy['role'] == 'confirmation':
            if re.search(r'\b(?:width|height|requestedHeight|font\.pixelSize)\s*:', text):
                errors.append(f'{name}: confirmation geometry and typography belong to the shared component')
            if 'OperationFooterButton {' not in text:
                errors.append(f'{name}: confirmation must use shared footer actions')
        if policy['role'] == 'operation':
            if policy.get('width') == 'Theme.operationDefaultWidth':
                errors.append(f'{name}: generic legacy width is not an approved operation role')
            if not re.search(r'^    requestedHeight:\s*Math\.min\(', text, re.M) or 'Screen.desktopAvailableHeight' not in text:
                errors.append(f'{name}: operation height must respect available screen geometry')
            if 'ScrollView {' not in text or 'OperationFooter {' not in text:
                errors.append(f'{name}: operation needs a scrolling body and fixed shared footer')
        if policy.get('design') and not (ROOT / policy['design'].split('?')[0]).is_file():
            errors.append(f'{name}: independent design source is missing')
    # Inline windows cannot silently escape a file-based inventory.
    found_inline = {}
    def without_literals(text):
        pattern = r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\''
        return re.sub(pattern, lambda m: ''.join('\n' if c == '\n' else ' ' for c in m[0]), text)
    for name, text in sources.items():
        clean = without_literals(text)
        root = ROOT_OBJECT.search(clean)
        for match in re.finditer(r'\b(?:Window|ApplicationWindow|Dialog|MessageDialog)\s*\{', clean):
            if root and match.start() == root.start():
                continue
            start = match.end(); end = start; depth = 1
            while end < len(clean) and depth:
                depth += (clean[end] == '{') - (clean[end] == '}'); end += 1
            block = clean[start:end-1]
            identifier = re.search(r'\bid:\s*(\w+)', block)
            key = name + '.' + (identifier[1] if identifier else '<anonymous>')
            found_inline[key] = block
    if set(found_inline) != set(registry['inline_windows']):
        errors.append('Inline window inventory differs: ' + ', '.join(sorted(set(found_inline) ^ set(registry['inline_windows']))))
    for name, policy in registry['inline_windows'].items():
        text = found_inline.get(name, '')
        for key in ['width', 'height']:
            if not re.search(r'\b' + key + r':\s*' + str(policy[key]) + r'\b', text):
                errors.append(f'{name}: inline {key} differs from reviewed exception')
        if not policy.get('reason'):
            errors.append(f'{name}: inline exception requires a reason')
    shared = sources.get('OperationConfirmationWindow', '')
    for required in ['ScrollView {', 'OperationFooter {', 'Theme.confirmationTitleSize', 'Theme.confirmationBodySize',
                     'Theme.confirmationContentInset', 'Theme.confirmationMaximumHeight', 'availableClientHeight', 'Qt.ApplicationModal']:
        if required not in shared:
            errors.append('Shared confirmation missing contract: ' + required)
    return errors


def self_test(sources, registry):
    mutations = []
    unknown = dict(sources, ForgottenDialog='import QtQuick\nWindow { width: 612; height: 270 }')
    mutations.append(('unregistered standalone window', unknown, registry))
    old = dict(sources); old['QuitWindow'] = old['QuitWindow'].replace('OperationConfirmationWindow {', 'OperationWindow {').replace('    id: root', '    id: root\n    width: Theme.operationDefaultWidth\n    requestedHeight: 270')
    mutations.append(('historical oversized quit dialog', old, registry))
    fixed = dict(sources); fixed['RestartWindow'] = fixed['RestartWindow'].replace('    id: root', '    id: root\n    requestedHeight: 540')
    mutations.append(('fixed-height confirmation', fixed, registry))
    scroll = dict(sources); scroll['OperationConfirmationWindow'] = scroll['OperationConfirmationWindow'].replace('ScrollView {', 'Item {')
    mutations.append(('missing body scrolling', scroll, registry))
    inline = dict(sources); inline['Browser'] += '\n    Window {\n        id: forgotten\n    }\n'
    mutations.append(('unregistered inline window', inline, registry))
    screen = dict(sources); screen['TaskWindow'] = screen['TaskWindow'].replace('Screen.desktopAvailableHeight', '720')
    mutations.append(('operation ignores screen height', screen, registry))
    for label, changed, spec in mutations:
        if not inspect(changed, spec):
            raise SystemExit('Regression gate failed to reject: ' + label)
    return len(mutations)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--self-test', action='store_true')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    sources = {p.stem: p.read_text(encoding="utf-8") for p in (ROOT/'apps/desktop/ui').glob('*.qml')}
    registry = json.loads((ROOT/'docs/visual/window-registry.json').read_text(encoding="utf-8"))
    errors = inspect(sources, registry)
    confirmation = json.loads((ROOT/'docs/visual/layout-scale.json').read_text(encoding="utf-8"))['operation']['confirmation']
    for key, expected in {'logicalWidth':491, 'titleSize':20, 'bodySize':13, 'contentInset':24}.items():
        if confirmation.get(key) != expected:
            errors.append(f'Confirmation {key}: differs from approved normal-density baseline {expected}')
    mutations = self_test(sources, registry) if args.self_test else 0
    report = {'passed':not errors,'windows':list(registry['windows']),'inlineWindows':list(registry['inline_windows']),
              'regressionMutations':mutations,'errors':errors,'limits':'Source inventory is not a visual acceptance result; runtime geometry and screenshots are checked separately.'}
    if args.output:
        path = args.output.resolve()
        if not any(path.is_relative_to(ROOT/folder) for folder in ['build','dist']):
            raise SystemExit('Reports belong in build/ or dist/')
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print('\n'.join(errors) if errors else f'Window contracts: {len(registry["windows"])} types, {len(registry["inline_windows"])} inline window, {mutations} regression mutations passed')
    raise SystemExit(bool(errors))


if __name__ == '__main__':
    main()
