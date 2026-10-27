#!/usr/bin/env python3
"""Reject per-window footer presentation overrides and information delimiters."""
import re
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
failures = []
registry = json.loads((ROOT/'docs/visual/window-registry.json').read_text(encoding="utf-8"))
windows = [name for name, policy in registry['windows'].items() if policy['role'] in {'operation', 'confirmation'}] + ['OperationConfirmationWindow']
for name in windows:
    text = (ROOT/'apps/desktop/ui'/f'{name}.qml').read_text(encoding="utf-8")
    # Match balanced QML objects, including signal-handler JS bodies.
    def blocks(kind):
        for match in re.finditer(r'\b'+kind+r'\s*\{', text):
            start = match.end(); depth = 1; end = start
            while depth and end < len(text):
                depth += (text[end] == '{') - (text[end] == '}'); end += 1
            yield text[start:end-1]
    footers = list(blocks('OperationFooter'))
    inherited_footer = registry['windows'][name]['role'] == 'confirmation'
    if (not footers and not inherited_footer) or any(re.search(r'\bAppButton\s*\{', b) for b in footers):
        failures.append(f'{name}: footer must use shared operation role')
    actions = list(blocks('OperationFooterButton'))
    if not actions and name != 'OperationConfirmationWindow':
        failures.append(f'{name}: shared footer has no actions')
    forbidden = r'\b(cornerRadius|radius|density|labelSize|font|surfaceColor|outlineColor|baseHeight|horizontalInset|verticalInset|minimumWidth)\s*:'
    if any(re.search(forbidden, b) for b in actions):
        failures.append(f'{name}: footer presentation override')
for folder in ['apps', 'crates', 'design/html', 'scripts', 'tools']:
    for path in (ROOT/folder).rglob('*'):
        if path.suffix in {'.qml','.cpp','.h','.js','.html','.css','.py','.rs','.ps1'} and not any(p in {'build','target','dist','__pycache__'} for p in path.parts):
            if '\u00b7' in path.read_text(encoding="utf-8"):
                failures.append(f'{path.relative_to(ROOT)}: middle-dot delimiter in maintained source')
# Explicit user-approved normal-density policy, independent of captured pixels.
layout = json.loads((ROOT/'docs/visual/layout-scale.json').read_text(encoding="utf-8"))
widths = layout['operation']['logical_widths']
for role in ['extract', 'create', 'progress', 'result', 'conflict']:
    if widths[role] != 491:
        failures.append(f'{role}: normal operation width must be 491 logical pixels')
conflict = (ROOT/'apps/desktop/ui/ConflictWindow.qml').read_text(encoding="utf-8")
if 'width: Theme.operationConflictWidth' not in conflict or 'OperationHeader {' not in conflict:
    failures.append('ConflictWindow: width and typography must use normal-density operation policy')
for name, policy in registry['windows'].items():
    if policy.get('header_policy'):
        source = (ROOT/'apps/desktop/ui'/f'{name}.qml').read_text(encoding='utf-8')
        if source.count('OperationHeader {') != 1 or 'OperationHero {' in source:
            failures.append(f'{name}: icon/title/subtitle header must use the shared component exactly once')
        if 'Theme.operationHeaderBodyGap' not in source or 'objectName: "operationHeaderBody"' not in source:
            failures.append(f'{name}: header-to-body spacing must use the shared contract')
        if not (ROOT/policy['header_design']).is_file():
            failures.append(f'{name}: shared header design is missing')
header = (ROOT/'apps/desktop/ui/OperationHeader.qml').read_text(encoding='utf-8')
for binding in ['Theme.operationHeaderIconSize','Theme.operationHeaderNormalHeight',
                'Theme.operationHeaderTitleSize','Theme.operationHeaderSubtitleSize','Text.PlainText']:
    if binding not in header:
        failures.append('OperationHeader: missing shared geometry or text contract '+binding)
segments = (ROOT/'apps/desktop/ui/OperationSegmentedGroup.qml').read_text(encoding="utf-8")
if 'radius: segment.checked' in segments:
    failures.append('OperationSegmentedGroup: selection must not round inner corners')
radio = (ROOT/'apps/desktop/ui/IntentRadioButton.qml').read_text(encoding="utf-8")
for contract in ['Accessible.onPressAction:', 'Accessible.onToggleAction:', 'onClicked:',
                 'if (enabled) selectionRequested();']:
    if contract not in radio:
        failures.append(f'IntentRadioButton: missing selection intent contract {contract}')
for path in (ROOT/'apps/desktop/ui').glob('*.qml'):
    if path.name != 'IntentRadioButton.qml' and re.search(
            r'Accessible\.role\s*:\s*Accessible\.RadioButton', path.read_text(encoding="utf-8")):
        failures.append(f'{path.name}: custom radio must use IntentRadioButton')
checkbox = (ROOT/'apps/desktop/ui/IntentCheckBox.qml').read_text(encoding="utf-8")
for contract in ['Accessible.onPressAction:', 'Accessible.onToggleAction:',
                 'if (enabled) root.click();']:
    if contract not in checkbox:
        failures.append(f'IntentCheckBox: missing toggle intent contract {contract}')
for path in (ROOT/'apps/desktop/ui').glob('*.qml'):
    if path.name != 'IntentCheckBox.qml' and re.search(
            r'\bCheckBox\s*\{', path.read_text(encoding="utf-8")):
        failures.append(f'{path.name}: checkbox must use IntentCheckBox')
for failure in failures:
    print(failure)
print(f'Shared operation roles: {len(windows)} windows inspected, {len(failures)} failures')
raise SystemExit(bool(failures))
