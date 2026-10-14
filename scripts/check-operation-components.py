#!/usr/bin/env python3
"""Reject per-window footer presentation overrides and information delimiters."""
import re
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
for failure in failures:
    print(failure)
print(f'Shared operation roles: {len(windows)} windows inspected, {len(failures)} failures')
raise SystemExit(bool(failures))
