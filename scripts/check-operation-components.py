#!/usr/bin/env python3
"""Reject per-window footer presentation overrides and information delimiters."""
import re
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
failures = []
windows = ['CreateWindow', 'ExtractWindow', 'TaskWindow', 'ConflictWindow', 'QuitWindow', 'PreflightWindow']
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
    if not footers or any(re.search(r'\bAppButton\s*\{', b) for b in footers):
        failures.append(f'{name}: footer must use shared operation role')
    actions = list(blocks('OperationFooterButton'))
    if not actions:
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
layout = json.loads((ROOT/'docs/visual/layout-scale.json').read_text())
widths = layout['operation']['logical_widths']
for role in ['extract', 'create', 'progress', 'result', 'conflict']:
    if widths[role] != 491:
        failures.append(f'{role}: normal operation width must be 491 logical pixels')
conflict = (ROOT/'apps/desktop/ui/ConflictWindow.qml').read_text()
if 'width: Theme.operationConflictWidth' not in conflict or 'Math.round(root.unit(Theme.operationTitleSize))' not in conflict:
    failures.append('ConflictWindow: width and typography must use normal-density operation policy')
segments = (ROOT/'apps/desktop/ui/OperationSegmentedGroup.qml').read_text()
if 'radius: segment.checked' in segments:
    failures.append('OperationSegmentedGroup: selection must not round inner corners')
for failure in failures:
    print(failure)
print(f'Shared operation roles: {len(windows)} windows inspected, {len(failures)} failures')
raise SystemExit(bool(failures))
