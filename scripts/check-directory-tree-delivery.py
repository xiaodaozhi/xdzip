#!/usr/bin/env python3
"""Check offline design images, mobile overflow and the handoff ZIP ledger."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import zipfile

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / 'dist'
REPORT_ROOT = ROOT / 'build/review/directory-tree'


def main():
    with zipfile.ZipFile(DIST / 'xdzip-directory-tree.zip') as package:
        assert package.testzip() is None, 'Corrupt ZIP member'
        ledger = json.loads(package.read('handoff-manifest.json'))
        names = set(package.namelist())
        assert len(names) == len(package.namelist()), 'Duplicate ZIP paths'
        assert names == {item['file'] for item in ledger['files']} | {'handoff-manifest.json'}
        for item in ledger['files']:
            path = Path(item['file'])
            assert not path.is_absolute() and '..' not in path.parts
            assert hashlib.sha256(package.read(item['file'])).hexdigest() == item['sha256']
            if item.get('source'):
                assert item['source'] in names
                assert hashlib.sha256(package.read(item['source'])).hexdigest() == item['source_sha256']
        assert len([name for name in names if name.startswith('design/assets/reference/tree-')]) == 26
        assert len([name for name in names if name.startswith('design/hifi/screens/')]) == 3

    reports = []
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(
            executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
            args=['--no-sandbox', '--disable-dev-shm-usage'])
        for width, height in [(1280, 1000), (390, 844), (320, 740)]:
            page = browser.new_page(viewport={'width': width, 'height': height}, device_scale_factor=1)
            requests = []

            def reject_network(route):
                requests.append(route.request.url)
                route.abort()

            page.route('**/*', reject_network)
            page.set_content((DIST / 'xdzip-directory-tree.html').read_text())
            # Lazy images below the fold must also decode successfully offline.
            page.evaluate("Promise.all([...document.images].map(image => {image.loading='eager'; return image.decode();}))")
            result = page.evaluate("""() => ({
                width: innerWidth, content: document.documentElement.scrollWidth,
                images: document.images.length,
                broken: [...document.images].filter(image => !image.complete || !image.naturalWidth).map(image => image.alt),
                boards: document.querySelectorAll('.boards figure').length,
                cuts: document.querySelectorAll('.cuts figure').length
            })""")
            assert result['content'] <= width, result
            assert not result['broken'] and result['images'] == 43, result
            assert result['boards'] == 3 and result['cuts'] == 26, result
            assert not requests, 'Offline gallery requested external images'
            result['external_requests'] = len(requests)
            reports.append(result)
            page.close()
        browser.close()
    report = {
        'status': 'passed', 'offline_browser_cases': reports,
        'zip_members': len(names), 'zip_hash_checks': len(ledger['files']),
        'scope': 'Static design and asset delivery; no live directory-tree behavior tested',
    }
    (REPORT_ROOT / 'delivery-review.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(report, ensure_ascii=False))


if __name__ == '__main__':
    main()
