#!/usr/bin/env python3
"""Check bilingual navigation, local Markdown links, JSON and design assets.

Use only the standard library. External URLs and generated HTML anchors are
outside this check. The HTML builder validates its output separately.
"""
from html import unescape
from html.parser import HTMLParser
from pathlib import Path
import json
import re
import subprocess
import sys
import unicodedata
from urllib.parse import unquote, urlsplit
from repository_contents import violations

ROOT = Path(__file__).resolve().parents[1]
ROOT_LANGUAGES = ('README', 'CHANGELOG', 'CODE_OF_CONDUCT', 'CONTRIBUTING',
                  'GOVERNANCE', 'PRIVACY', 'SECURITY')
ZH_SUFFIX = '.zh-CN.md'
ATX_HEADING = re.compile(r'^ {0,3}(#{1,6})(?:[ \t]+(.*)|$)')
GITHUB_PATH = re.compile(r'/xiaodaozhi/xdzip/(?:blob|tree)/(?:HEAD|main|refs/heads/main)/(.+)')
DESTINATION = r'<[^>\n]*>|(?:\\[()]|[^\s()\\]|\([^()\s]*\))*'
INLINE_LINK = re.compile(r'(?<!\\)\[([^\]\n]*)\]\([ \t]*(' + DESTINATION
                         + r''')(?:[ \t]+(?:"[^"\n]*"|'[^'\n]*'|\([^)\n]*\)))?[ \t]*\)''')
REFERENCE_LINK = re.compile(r'^ {0,3}\[[^\]\n]+\]:[ \t]*(' + DESTINATION + r')(?=\s|$)', re.M)
INLINE_CODE = re.compile(r'(?<![\\\x60])(\x60+)(?!\x60)([^\n]*?)(?<!\x60)\1(?!\x60)')
MARKDOWN_TOKEN = re.compile(INLINE_CODE.pattern
                            + r'|^ {0,3}(?:(?P<backtick>\x60{3,})[^\x60\n]*|(?P<tilde>~{3,})[^\n]*)$'
                            + r'|<!--[\s\S]*?(?:-->|\Z)', re.M)


def markdown_documents(root):
    # Ask Git for tracked and non-ignored new documents rather than traversing
    # generated build trees before filtering them out.
    names = subprocess.check_output(
        ['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z', '--', '*.md'], cwd=root
    ).decode('utf-8').split('\0')
    excluded = {'.git', '.cache', 'dist', '.venv', 'target', 'build'}
    return sorted(root / name for name in set(names) if name and
                  not excluded.intersection(Path(name).parts) and (root / name).is_file())

def markdown_content(source):
    """Remove fenced examples and comments while keeping line numbers."""
    source = source.replace('\0', '\ufffd')
    parts = []
    position = 0
    while match := MARKDOWN_TOKEN.search(source, position):
        parts.append(source[position:match.start()])
        position = match.end()
        if match[1]:
            parts.append(match[0])
            continue
        marker = match['backtick'] or match['tilde']
        if marker:
            end = re.compile(r'^ {0,3}' + marker[0] + f'{{{len(marker)},}}[ \t]*$', re.M)
            closing = end.search(source, position)
            position = closing.end() if closing else len(source)
        parts.append('\n' * source.count('\n', match.start(), position))
    parts.append(source[position:])
    return ''.join(parts)


def language_selector(source, root_readme):
    if root_readme:
        heading = re.search(r'<h1\b[^>]*>.*?</h1>', source, re.I | re.S)
        if heading:
            intro = re.split(r'^ {0,3}#{1,6}[ \t]', source[heading.end():], maxsplit=1, flags=re.M)[0]
            for paragraph in re.findall(r'<p\b[^>]*>(.*?)</p>', intro, re.I | re.S):
                if 'English' in paragraph and '简体中文' in paragraph and '|' in paragraph:
                    return re.sub(r'''<a\s+href=(["'])([^"']+)\1>(English|简体中文)</a>''',
                                  r'[\3](\2)', paragraph, flags=re.I).strip()
            return ''
    lines = iter(source.splitlines())
    if any(line.startswith('# ') for line in lines):
        return next((line.strip() for line in lines if line.strip()), '')
    return ''


def local_path(root, parent, url_path):
    name = unquote(url_path)
    if name.startswith('/'):
        return (root / name.lstrip('/')).resolve()
    return (parent / name).resolve()


def bilingual_documents(root, documents):
    """Check mandatory root pairs and the reciprocal selectors in each language tree."""
    paths = {path.relative_to(root) for path in documents}
    root_topics = set(ROOT_LANGUAGES)
    root_topics.update(path.name[:-len(ZH_SUFFIX)] for path in paths
                       if len(path.parts) == 1 and path.name.endswith(ZH_SUFFIX))
    counterparts = [(Path(f'{stem}.md'), Path(f'{stem}{ZH_SUFFIX}')) for stem in sorted(root_topics)]
    for tree in ('docs', 'developers'):
        tree_paths = set()
        for path in paths:
            if len(path.parts) > 1 and path.parts[0] == tree:
                offset = 2 if path.parts[1] == 'zh-CN' else 1
                tree_paths.add(Path(*path.parts[offset:]))
        counterparts.extend((Path(tree) / name, Path(tree) / 'zh-CN' / name)
                            for name in sorted(tree_paths))

    failures = []
    pairs = 0
    for en, zh in counterparts:
        missing = [path.as_posix() for path in (en, zh) if path not in paths]
        if missing:
            failures.append(f'{", ".join(missing)}: missing language counterpart')
            continue
        pairs += 1
        for path, counterpart, pattern in (
            (en, zh, r'English \| \[简体中文\]\(([^)#?]+)\)'),
            (zh, en, r'\[English\]\(([^)#?]+)\) \| 简体中文'),
        ):
            source = markdown_content((root / path).read_text(encoding='utf-8'))
            selector = language_selector(source, en == Path('README.md'))
            match = re.fullmatch(pattern, selector)
            if not match or (local_path(root, root / path.parent, match[1])
                             != (root / counterpart).resolve()):
                failures.append(f'{path.as_posix()}: expected a language selector linking to '
                                f'{counterpart.as_posix()} with the current language in plain text')
    return pairs, failures


class MarkdownHtml(HTMLParser):
    def __init__(self, source):
        super().__init__()
        self.anchors = set()
        self.links = []
        self.feed(source)
        self.close()

    def handle_starttag(self, tag, attrs):
        for key, value in attrs:
            if not value:
                continue
            if key == 'id' or (key == 'name' and tag == 'a'):
                self.anchors.add(value)
            if key in ('href', 'src'):
                self.links.append((self.getpos()[0], value))


def heading_text(source):
    paragraph = []
    for line in source.splitlines():
        atx = ATX_HEADING.fullmatch(line)
        if atx:
            yield re.sub(r'(?:^|[ \t]+)#+[ \t]*$', '', atx[2] or '')
            paragraph.clear()
        elif paragraph and re.fullmatch(r' {0,3}(?:=+|-+)[ \t]*', line):
            yield ' '.join(paragraph)
            paragraph.clear()
        elif re.match(r'^ {0,3}\S', line) and not re.match(
                r'^ {0,3}(?:[<>|]|[-+*][ \t]|\d+[.)][ \t]|\[[^\]]+\]:)', line):
            paragraph.append(line.strip())
        else:
            paragraph.clear()


def heading_slug(heading):
    # Formatting is removed from prose, but text inside a code span is literal.
    code = []

    def take_code(match):
        code.append(match[2])
        return '\0'

    text = INLINE_CODE.sub(take_code, heading)
    text = INLINE_LINK.sub(lambda match: match[1], text)
    text = re.sub(r'\[([^\]\n]+)\]\[[^\]\n]*\]', r'\1', text)
    text = re.sub(r'<[^>\n]+>', '', text)
    text = unescape(re.sub(r'(?<![\w\\])(_{1,3})(?=\S)(.+?)(?<=\S)\1(?!\w)', r'\2', text))
    code_iter = iter(code)
    text = ''.join(next(code_iter) if char == '\0' else char for char in text).strip().lower()
    return ''.join(char for char in text if char in ' -_'
                   or unicodedata.category(char)[0] in 'LMN').replace(' ', '-')


def markdown_anchors(source, html):
    # HTML anchors do not affect the suffixes assigned to generated headings.
    headings = set()
    suffixes = {}
    for title in heading_text(source):
        base = slug = heading_slug(title)
        while slug in headings:
            suffixes[base] = suffixes.get(base, 0) + 1
            slug = f'{base}-{suffixes[base]}'
        headings.add(slug)
    return headings | html.anchors


def markdown_links(root, documents):
    sources = {}
    for path in documents:
        source = markdown_content(path.read_text(encoding='utf-8'))
        # Mask inline code so that literal Markdown and HTML examples do not count as links or IDs.
        prose = INLINE_CODE.sub(lambda match: ' ' * len(match[0]), source)
        sources[path] = (source, prose, MarkdownHtml(prose))

    links = fragments = 0
    failures = []
    anchors_by_path = {}
    for path, (_, prose, html) in sources.items():
        targets = list(html.links)
        for pattern, group in ((INLINE_LINK, 2), (REFERENCE_LINK, 1)):
            targets.extend((prose.count('\n', 0, match.start()) + 1, match[group].strip('<>'))
                           for match in pattern.finditer(prose))
        for line, target in sorted(targets):
            location = f'{path.relative_to(root).as_posix()}:{line}'
            try:
                url = urlsplit(unescape(re.sub(r'\\([()])', r'\1', target)))
            except ValueError as error:
                failures.append(f'{location}: invalid link {target}: {error}')
                continue
            fragment = unquote(url.fragment)
            github = GITHUB_PATH.fullmatch(url.path) if url.netloc.lower() == 'github.com' else None
            if github and url.scheme in ('https', 'http'):
                resolved = (root / unquote(github[1])).resolve()
            elif url.scheme or url.netloc or not (url.path or fragment):
                continue
            elif url.path:
                resolved = local_path(root, path.parent, url.path)
            else:
                resolved = path
            links += 1
            if not resolved.is_relative_to(root) or not resolved.exists():
                failures.append(f'{location}: missing or external local target {target}')
                continue
            if (not fragment or resolved.suffix.lower() != '.md' or not resolved.is_file()
                    or (github and re.fullmatch(r'L\d+(?:-L\d+)?', fragment))):
                continue
            fragments += 1
            if resolved not in anchors_by_path:
                if resolved in sources:
                    source, _, target_html = sources[resolved]
                else:
                    source = markdown_content(resolved.read_text(encoding='utf-8'))
                    target_html = MarkdownHtml(INLINE_CODE.sub('', source))
                anchors_by_path[resolved] = markdown_anchors(source, target_html)
            if fragment not in anchors_by_path[resolved]:
                failures.append(f'{location}: missing Markdown anchor {target}')
    return links, fragments, failures


def main():
    failures = violations(ROOT)
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
    links, fragments, link_failures = markdown_links(ROOT, documents)
    failures.extend(link_failures)
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
                      'markdown_anchor_links': fragments,
                      'json_files': len(json_files), 'failures': failures}, ensure_ascii=False, indent=2))
    raise SystemExit(bool(failures))

if __name__ == '__main__':
    main()
