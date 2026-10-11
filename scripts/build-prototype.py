#!/usr/bin/env python3
"""Build a self-contained, offline HTML design prototype from its source files."""
from pathlib import Path
import re

PROJECT = Path(__file__).resolve().parents[1]
SOURCE = PROJECT / 'design/prototype'
OUTPUT = PROJECT / 'dist'
OUTPUT.mkdir(exist_ok=True)
html = (SOURCE / 'index.html').read_text()
css = (SOURCE / 'styles.css').read_text()
js = (SOURCE / 'app.js').read_text()
html, css_count = re.subn(r'<link\s+rel="stylesheet"\s+href="styles.css"\s*/?>', lambda _: '<style>\n' + css + '\n</style>', html)
html, js_count = re.subn(r'<script\s+defer\s+src="app.js"></script>', '', html)
if css_count != 1 or js_count != 1:
    raise ValueError('Expected exactly one stylesheet and script entry')
# Inline script runs after DOM creation; source uses no modules or remote assets.
html = html.replace('</body>', '<script>\n' + js.replace('</script', '<\\/script') + '\n</script>\n</body>')
path = OUTPUT / 'xdzip-interactive-design.html'
path.write_text(html)
print(path)
