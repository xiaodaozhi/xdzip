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

def markdown_documents(root):
    # Ask Git for tracked and non-ignored new documents rather than traversing
    # generated build trees before filtering them out.
    names = subprocess.check_output(
        ['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z', '--', '*.md'], cwd=root
    ).decode('utf-8').split('\0')
    excluded = {'.git', '.cache', 'dist', '.venv', 'target', 'build'}
    return sorted(root / name for name in set(names) if name and
                  not excluded.intersection(Path(name).parts) and (root / name).is_file())

def bilingual_documents(root, documents):
    """Check the matching page and first language selector in each document tree."""
    paths = {path.relative_to(root) for path in documents}
    failures = []
    pairs = 0
    for tree in ('docs', 'developers'):
        english = {Path(*path.parts[1:]) for path in paths
                   if path.parts[0] == tree and len(path.parts) > 1 and path.parts[1] != 'zh-CN'}
        chinese = {Path(*path.parts[2:]) for path in paths
                   if path.parts[:2] == (tree, 'zh-CN')}
        pairs += len(english & chinese)
        for name in sorted(english | chinese):
            en, zh = Path(tree) / name, Path(tree) / 'zh-CN' / name
            if name not in english or name not in chinese:
                missing = en if name not in english else zh
                failures.append(f'{missing.as_posix()}: missing language counterpart')
                continue
            for path, counterpart, pattern in (
                (en, zh, r'English \| \[简体中文\]\(([^)]+)\)'),
                (zh, en, r'\[English\]\(([^)]+)\) \| 简体中文'),
            ):
                lines = iter((root / path).read_text(encoding='utf-8').splitlines())
                heading = next((line for line in lines if line.startswith('# ')), None)
                selector = next((line for line in lines if line.strip()), '')
                match = re.fullmatch(pattern, selector) if heading else None
                if not match or ((root / path.parent / unquote(match.group(1))).resolve()
                                 != (root / counterpart).resolve()):
                    failures.append(f'{path.as_posix()}: expected a language selector linking to {counterpart.as_posix()}')
    return pairs, failures

def main():
    failures = violations(ROOT)
    links = 0
    # Read the index so this also rejects files staged with `git add --force`.
    tracked = subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).decode('utf-8')
    for name in tracked.split('\0'):
        if name and 'dist' in Path(name).parts[:-1]:
            failures.append(f'{name}: generated dist/ files must not be tracked by Git')
        if name.startswith(('design/html/screenshots/', 'design/assets/preview/')) or name in {
                'design/assets/validation.json', 'design/assets/fonts/validation.json',
                'docs/html/VALIDATION.json', 'docs/html/BROWSER-VALIDATION.json'}:
            failures.append(f'{name}: generated review evidence must not be tracked by Git')
    documents = markdown_documents(ROOT)
    pairs, language_failures = bilingual_documents(ROOT, documents)
    failures.extend(language_failures)
    for path in documents:
        source = re.sub(r'^```[^\n]*\n.*?^```\s*$', '', path.read_text(encoding='utf-8'), flags=re.M | re.S)
        for raw in re.findall(r'!?\[[^\]]*\]\(([^\n)]+)\)', source):
            target = raw.strip().split(' "', 1)[0].strip('<>')
            parsed = urlsplit(target)
            if parsed.scheme or parsed.netloc or not parsed.path:
                continue
            resolved = (path.parent / unquote(parsed.path)).resolve()
            if not resolved.is_relative_to(ROOT) or not resolved.exists():
                failures.append(f'{path.relative_to(ROOT)}: missing or external local target {target}')
            links += 1
    json_files = list((ROOT / 'specs').rglob('*.json'))
    for path in json_files:
        try:
            json.loads(path.read_text(encoding='utf-8'))
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
    print(json.dumps({'markdown_documents': len(documents), 'bilingual_pairs': pairs, 'local_file_links': links,
                      'json_files': len(json_files), 'failures': failures}, ensure_ascii=False, indent=2))
    raise SystemExit(bool(failures))

if __name__ == '__main__':
    main()
