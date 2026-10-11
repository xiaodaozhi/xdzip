#!/usr/bin/env python3
"""Export a self-contained review, retaining original image bytes. Requires pandoc and lxml."""
import argparse
import base64
from pathlib import Path
import subprocess
from urllib.parse import quote, urlsplit

from lxml import html


ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--report', type=Path, default=ROOT / 'docs/qa/visual-audit-review.md')
parser.add_argument('--evidence', default='visual-audit')
parser.add_argument('--caption', default='64bf2fe 实际 Qt 渲染')
parser.add_argument('--output', type=Path, default=ROOT / 'dist/xdzip-visual-audit.html')
args = parser.parse_args()
REPORT = args.report.resolve()
body = subprocess.run(['pandoc', '--from=gfm', '--to=html5', str(REPORT)],
                      check=True, capture_output=True, text=True, encoding='utf-8').stdout
comparisons = '''<section class="comparisons"><h2>原始设计图 / 实际 Qt 截图</h2>
<p>图片分别等比适应列宽，保留原始外框与背景；此处用于观察结构，精确差值以正文的坐标测量为准。</p>
<h3>主窗口</h3><div class="pair">
<figure><figcaption>v1 高保真原稿</figcaption><img src="../../design/assets/v1/sources/main-light.png" alt="主窗口设计原稿"></figure>
<figure><figcaption>64bf2fe 实际 Qt 渲染</figcaption><img src="visual-audit/browser-light.png" alt="主窗口实际截图"></figure></div>
<h3>任务窗口</h3><div class="pair">
<figure><figcaption>v2 高保真原稿</figcaption><img src="../../design/hifi/v2/screens/03-progress.png" alt="任务窗口设计原稿"></figure>
<figure><figcaption>实际示例组件，非真实解压任务</figcaption><img src="visual-audit/task-light.png" alt="任务窗口实际截图"></figure></div></section>'''
comparisons = comparisons.replace('visual-audit/', args.evidence + '/').replace('64bf2fe 实际 Qt 渲染', args.caption)
body = body.replace('<h2', comparisons + '<h2', 1)
page = html.document_fromstring('''<!doctype html><html lang="zh-CN"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>xdzip 视觉严格复核</title>
<style>
:root{color-scheme:light dark}*{box-sizing:border-box}body{margin:auto;max-width:1120px;padding:24px;font:16px/1.75 system-ui,sans-serif;color:#20232a;background:#fafbfc}
h1{font-size:30px;line-height:1.3}h2{margin-top:2.2em}h3{margin-top:1.6em}a{color:#2455cd}img{max-width:100%;height:auto;border:1px solid #dcdfe3;border-radius:8px}
table{display:block;overflow:auto;border-collapse:collapse;width:100%;font-size:14px}th,td{padding:10px 14px;border:1px solid #dcdfe3;text-align:left;min-width:95px}th{background:#eef2fa}pre{padding:16px;background:#eef2fa;overflow:auto}
code{overflow-wrap:anywhere}.pair{display:grid;grid-template-columns:1fr 1fr;gap:20px}figure{margin:0}figcaption{font-size:14px;margin-bottom:8px;color:#626874}.comparisons{padding:20px;border:1px solid #dcdfe3;border-radius:12px;background:white}
@media(max-width:720px){body{padding:16px}.pair{grid-template-columns:1fr}h1{font-size:26px}}
@media(prefers-color-scheme:dark){body{background:#17191e;color:#eceef3}a{color:#9db4ff}.comparisons{background:#1d2026}th,pre{background:#202630}figcaption{color:#a2a8b4}}
</style></head><body>''' + body + '</body></html>')
for img in page.xpath('//img'):
    path = (REPORT.parent / img.get('src')).resolve()
    if not path.is_relative_to(ROOT):
        raise ValueError('Image outside repository')
    img.set('src', 'data:image/png;base64,' + base64.b64encode(path.read_bytes()).decode())
for anchor in page.xpath('//a[@href]'):
    href = anchor.get('href')
    url = urlsplit(href)
    if url.scheme or href.startswith('#'):
        continue
    path = (REPORT.parent / url.path).resolve().relative_to(ROOT)
    anchor.set('href', 'https://github.com/xiaodaozhi/xdzip/blob/main/' + quote(str(path)) +
               ('#' + url.fragment if url.fragment else ''))
destination = args.output
destination.parent.mkdir(exist_ok=True)
destination.write_bytes(html.tostring(page, encoding='utf-8', doctype='<!doctype html>'))
print(f'{destination}: {destination.stat().st_size} bytes, images embedded; evidence links require repository access')
