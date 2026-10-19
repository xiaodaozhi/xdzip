#!/usr/bin/env python3
"""Check repository hygiene, Markdown targets, JSON and design assets (stdlib only).

This checks file targets, not Markdown anchor rendering or remote URL availability.
The HTML builder separately checks generated page anchors. Code fences are ignored.
"""
from pathlib import Path
import json
import re
import subprocess
import sys
from urllib.parse import unquote, urlsplit
from repository_contents import violations

ROOT = Path(__file__).resolve().parents[1]

def main():
    failures = violations(ROOT)
    links = 0
    # Read the index so this also rejects files staged with `git add --force`.
    tracked = subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).decode('utf-8')
    for name in tracked.split('\0'):
        if name and 'dist' in Path(name).parts[:-1]:
            failures.append(f'{name}: generated dist/ files must not be tracked by Git')
        if name.startswith('docs/qa/') and Path(name).suffix != '.md':
            failures.append(f'{name}: QA captures/reports must not be tracked by Git')
        if name.startswith(('design/html/screenshots/', 'design/assets/preview/')) or name in {
                'design/assets/validation.json', 'design/assets/fonts/validation.json',
                'docs/html/VALIDATION.json', 'docs/html/BROWSER-VALIDATION.json'}:
            failures.append(f'{name}: generated review evidence must not be tracked by Git')
    documents = sorted(ROOT.rglob('*.md'))
    for path in documents:
        if any(part in {'.git', '.cache', 'dist', '.venv', 'target', 'build'} for part in path.relative_to(ROOT).parts):
            continue
        source = re.sub(r'^```[^\n]*\n.*?^```\s*$', '', path.read_text(), flags=re.M | re.S)
        for raw in re.findall(r'!?\[[^\]]*\]\(([^\n)]+)\)', source):
            target = raw.strip().split(' "', 1)[0].strip('<>')
            parsed = urlsplit(target)
            if parsed.scheme or parsed.netloc or not parsed.path:
                continue
            resolved = (path.parent / unquote(parsed.path)).resolve()
            if not resolved.is_relative_to(ROOT) or not resolved.exists():
                failures.append(f'{path.relative_to(ROOT)}: missing or external local target {target}')
            links += 1
    json_files = list((ROOT / 'docs').rglob('*.json'))
    for path in json_files:
        try:
            json.loads(path.read_text())
        except (ValueError, UnicodeError) as error:
            failures.append(f'{path.relative_to(ROOT)}: {error}')
    result = subprocess.run([sys.executable, str(ROOT / 'scripts/build-hifi-gallery.py'), '--check'],
                            capture_output=True, text=True)
    if result.returncode:
        failures.append('High-fidelity catalog/gallery: ' + (result.stderr or result.stdout).strip())
    result = subprocess.run([sys.executable, str(ROOT / 'scripts/build-directory-tree-delivery.py'), '--check'],
                            capture_output=True, text=True)
    if result.returncode:
        failures.append('Directory-tree design delivery: ' + (result.stderr or result.stdout).strip())
    print(json.dumps({'markdown_documents': len(documents), 'local_file_links': links,
                      'json_files': len(json_files), 'failures': failures}, ensure_ascii=False, indent=2))
    raise SystemExit(bool(failures))

if __name__ == '__main__':
    main()
