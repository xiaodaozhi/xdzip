#!/usr/bin/env python3
"""Build the unified image gallery from a hash-verified catalog (stdlib only)."""
import argparse
import base64
import re
from urllib.parse import quote
import hashlib
import html
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / 'design/hifi'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Verify catalog and generated gallery without writing')
    parser.add_argument('--standalone', type=Path, help='Also export a mobile-friendly HTML with embedded original images')
    args = parser.parse_args()
    catalog = json.loads((ROOT / 'catalog.json').read_text(encoding='utf-8'))
    groups = {'current': [], 'historical': []}
    seen = set()
    for screen in catalog['screens']:
        path = (ROOT / screen['file']).resolve()
        if not path.is_relative_to(ROOT) or path in seen:
            raise ValueError('Invalid or duplicate screen path')
        seen.add(path)
        data = path.read_bytes()
        if data[:8] != b'\x89PNG\r\n\x1a\n' or hashlib.sha256(data).hexdigest() != screen['sha256']:
            raise ValueError(f"Image integrity mismatch: {screen['file']}")
        title = html.escape(screen['title'])
        source = html.escape(screen['file'], quote=True)
        groups[screen['group']].append(
            f'<figure><figcaption>{title}</figcaption>'
            f'<a href="{source}"><img src="{source}" alt="{title}" loading="lazy"></a></figure>')
    page = '''<!doctype html><html lang="zh-CN"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>xdzip · 高保真设计总览</title>
<style>*{box-sizing:border-box}body{margin:auto;max-width:1200px;padding:24px;background:#f5f5f3;color:#20232a;font:16px/1.7 system-ui,sans-serif}a{color:#2455cd}nav{display:flex;flex-wrap:wrap;gap:16px}figure{margin:28px 0;padding:16px;background:white;border:1px solid #dcdfe3;border-radius:12px}figcaption{font-weight:600;margin-bottom:12px}img{display:block;width:100%;height:auto}summary{cursor:pointer;font-size:22px;font-weight:600}.note{padding:16px;background:#e9eefb;border-radius:8px}@media(max-width:600px){body{padding:16px}figure{padding:8px}h1{font-size:26px}}</style></head><body>
<h1>xdzip 高保真设计总览</h1><p>当前主界面与操作窗口的原稿入口。下方展示原始设计图片，HTML 仅用于浏览，不是界面原型。</p>
<nav><a href="#current">当前视觉参考</a><a href="../html/index.html">HTML 高保真界面</a><a href="README.md">目录与用途说明</a><a href="../../docs/visual/tokens.json">颜色尺寸规范</a><a href="../assets/index.html">统一素材库</a></nav>
<p class="note">主窗口、独立解压确认及各操作状态共用一套视觉基线。完整原稿在此展示，图标、示例照片与原像素对照切片统一从素材库查看。当前主窗口的完整深色原稿尚缺；生成式图像不等于逐像素规范或已实现功能。</p>
<h2 id="current">当前视觉参考</h2>''' + ''.join(groups['current']) + '''
<footer><p>图片 SHA-256 与迁移来源见 <a href="catalog.json">统一清单</a>。实际实现截图保存在 <a href="../../docs/qa/navigation-sorting.md">验收文档</a>。</p></footer></body></html>
'''
    target = ROOT / 'index.html'
    if args.check:
        if not target.exists() or target.read_text(encoding='utf-8') != page:
            raise SystemExit('Gallery is stale; run scripts/build-hifi-gallery.py')
    else:
        target.write_text(page, encoding='utf-8')
    if args.standalone:
        standalone = page
        for screen in catalog['screens']:
            image = base64.b64encode((ROOT / screen['file']).read_bytes()).decode('ascii')
            standalone = standalone.replace('src="' + html.escape(screen['file'], quote=True) + '"',
                                            'src="data:image/png;base64,' + image + '"')
        def repository_link(match):
            href = html.unescape(match.group(1))
            if href.startswith('#'):
                return match.group(0)
            path = (ROOT / href).resolve().relative_to(ROOT.parents[1])
            return 'href="https://github.com/xiaodaozhi/xdzip/blob/main/' + quote(path.as_posix()) + '"'
        standalone = re.sub(r'href="([^"]+)"', repository_link, standalone)
        args.standalone.parent.mkdir(parents=True, exist_ok=True)
        args.standalone.write_text(standalone, encoding='utf-8')
    print(f'High-fidelity catalog: {len(seen)} original images verified; gallery up to date')


if __name__ == '__main__':
    main()
