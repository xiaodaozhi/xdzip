#!/usr/bin/env python3
"""Bundle measured body comparisons and regional mobile views without external URLs."""
from pathlib import Path
import argparse
import base64
import json

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'docs/qa/html-body'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=SOURCE)
    parser.add_argument('--output', type=Path, default=ROOT / 'dist/xdzip-body-review.html')
    args = parser.parse_args()
    report = json.loads((args.source / 'measurements.json').read_text())
    images = {name: 'data:image/png;base64,' + base64.b64encode((args.source / f'{name}.png').read_bytes()).decode()
              for name in ['reference', 'after', 'before', 'difference']}
    config = 'window.XDZIP_BODY_REPORT=' + json.dumps(report, ensure_ascii=False) + ';'
    config += 'window.XDZIP_BODY_IMAGES=' + json.dumps(images) + ';'
    html = (args.source / 'index.html').read_text().replace('<script src="measurements.js"></script>', '')
    html = html.replace('<script>', '<script>' + config, 1)
    output = args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(html)
    print(f'{output}: {output.stat().st_size} bytes; source measurements and four image layers embedded')


if __name__ == '__main__':
    main()
