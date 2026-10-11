#!/usr/bin/env python3
"""Check local Markdown targets, JSON and required design assets (stdlib only).

This checks file targets, not Markdown anchor rendering or remote URL availability.
The HTML builder separately checks generated page anchors. Code fences are ignored.
"""
from pathlib import Path
import json
import re
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]

def main():
    failures = []
    links = 0
    documents = sorted(ROOT.rglob('*.md'))
    for path in documents:
        if any(part in {'.git', 'dist', '.venv', 'target', 'build'} for part in path.relative_to(ROOT).parts):
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
    for name in ['xdzip-main-light.png', 'xdzip-main-dark.png', 'xdzip-extract-sheet.png']:
        path = ROOT / 'docs/visual' / name
        if not path.is_file() or path.read_bytes()[:8] != b'\x89PNG\r\n\x1a\n':
            failures.append(f'Invalid or missing visual: {name}')
    print(json.dumps({'markdown_documents': len(documents), 'local_file_links': links,
                      'json_files': len(json_files), 'failures': failures}, ensure_ascii=False, indent=2))
    raise SystemExit(bool(failures))

if __name__ == '__main__':
    main()
