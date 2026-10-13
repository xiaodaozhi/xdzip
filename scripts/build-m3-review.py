#!/usr/bin/env python3
"""Embed unchanged review images; keep evidence links pinned to a repository commit."""
import base64
import html
from pathlib import Path
import re
import subprocess
from urllib.parse import quote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'docs/qa/m3/index.html'
OUTPUT = ROOT / 'dist/xdzip-m3-review.html'


def main():
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    source = SOURCE.read_text()

    def replace(match):
        attribute, raw = match.groups()
        target = html.unescape(raw)
        url = urlsplit(target)
        if url.scheme or not url.path:
            return match.group(0)
        path = (SOURCE.parent / url.path).resolve()
        if not path.is_relative_to(ROOT) or not path.is_file():
            raise ValueError(f'Missing repository evidence: {raw}')
        if path.suffix == '.png' and attribute == 'src':
            # Keep original capture/reference bytes, including dimensions and alpha.
            value = 'data:image/png;base64,' + base64.b64encode(path.read_bytes()).decode()
        else:
            value = f'https://github.com/xiaodaozhi/xdzip/blob/{commit}/' + quote(str(path.relative_to(ROOT)))
            if url.fragment:
                value += '#' + url.fragment
        return attribute + '="' + html.escape(value, quote=True) + '"'

    output = re.sub(r'(src|href)="([^\"]+)"', replace, source)
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(output)
    print(f'{OUTPUT}: {OUTPUT.stat().st_size} bytes; original image bytes embedded')


if __name__ == '__main__':
    main()
