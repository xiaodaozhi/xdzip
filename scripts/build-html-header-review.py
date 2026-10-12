#!/usr/bin/env python3
"""Package the measured header review as a standalone HTML file."""
from pathlib import Path
import base64
import json

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'docs/qa/html-header'
report = json.loads((SOURCE / 'measurements.json').read_text())
(SOURCE / 'measurements.js').write_text('window.XDZIP_HEADER_REPORT='+json.dumps(report,ensure_ascii=False)+';\n')
html = (SOURCE / 'index.html').read_text().replace('<script src="measurements.js"></script>', '')
for image in ['reference', 'after', 'before', 'difference', 'comparison']:
    url = 'data:image/png;base64,' + base64.b64encode((SOURCE / f'{image}.png').read_bytes()).decode()
    # Also replace filenames in the review controls' data mapping.
    html = html.replace(f'src="{image}.png"', f'src="{url}"')
    html = html.replace(f"'{image}.png'", json.dumps(url))
html = html.replace("base.src=mode+'.png'", 'base.src=window.XDZIP_HEADER_IMAGES[mode]')
images = {name:'data:image/png;base64,'+base64.b64encode((SOURCE/f'{name}.png').read_bytes()).decode()
          for name in ['reference','after','before','difference']}
config = 'window.XDZIP_HEADER_REPORT='+json.dumps(report,ensure_ascii=False)+';window.XDZIP_HEADER_IMAGES='+json.dumps(images)+';'
html = html.replace('<script>', '<script>'+config, 1)
output = ROOT / 'dist/xdzip-header-review.html'
output.write_text(html)
print(f'{output}: {output.stat().st_size} bytes')
