#!/usr/bin/env python3
"""Generate file-association SVG designs and optional local platform exports.

Default writes reviewable design sources. --export writes PNG/ICO/ICNS only
under build/ or dist/. --check checks sources and any requested export without
changing them. No operating-system association or product resource is changed.
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import html
import io
import json
import os
from pathlib import Path
import struct
import xml.etree.ElementTree as ET

from association_icon_assets import ASSETS, ROOT, SPEC, icon_svg, source_specs
from brand_assets import ICO_SIZES, ICNS_CHUNKS, png_name

GALLERY = ROOT / 'design/hifi/sources/file-association-icons.html'


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def image_tag(icon: dict, size: int, platform: str = 'default') -> str:
    encoded = base64.b64encode(icon_svg(icon, size, platform).encode()).decode()
    return (f'<img src="data:image/svg+xml;base64,{encoded}" width="{size}" height="{size}" '
            f'alt="{html.escape(icon["name"])}" draggable="false">')


def gallery() -> str:
    cards, rows = [], []
    for icon in SPEC['icons']:
        caption = icon['label'] or '通用归档'
        suffix = ' / '.join(icon['extensions']) or '无专属格式时使用'
        cards.append(f'<article class="card"><div class="art">{image_tag(icon, 128)}</div>'
                     f'<h2>{caption}</h2><p>{suffix}</p></article>')
        samples = ''.join(f'<div class="sample">{image_tag(icon, size)}</div>'
                          for size in [16, 24, 32, 48, 64, 128])
        rows.append(f'<div class="size-row"><strong>{caption}</strong>{samples}</div>')
    cards.append('<article class="card identity"><div class="art"><img width="112" height="112" '
                 'src="../../assets/brand/xdzip-128.png" alt="xdzip 应用图标"></div>'
                 '<h2>同一品牌</h2><p>交错带与开放 U 形</p></article>')
    head = '''<!doctype html>
<html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>xdzip 文件关联图标</title><style>
*{box-sizing:border-box} :root{color-scheme:light;--page:#f5f5f3;--card:#fff;--ink:#20232a;--muted:#657084;--line:#dce2ec;--blue:#2455cd;--selection:#e6eeff}
html[data-theme=dark]{color-scheme:dark;--page:#20242d;--card:#292e39;--ink:#f0f2f6;--muted:#adb8cc;--line:#434e62;--blue:#9cbbff;--selection:#35445e}
body{margin:0;background:var(--page);color:var(--ink);font:15px/1.6 "Segoe UI","Microsoft YaHei",sans-serif}
main{max-width:1200px;margin:auto;padding:48px 48px 64px}header{display:flex;justify-content:space-between;align-items:flex-start;gap:24px;margin-bottom:32px}
.eyebrow{font-size:12px;font-weight:700;letter-spacing:2px;color:var(--blue);margin:0 0 10px}h1{font-size:34px;line-height:1.3;letter-spacing:-1px;margin:0 0 12px}header p{color:var(--muted);margin:0;max-width:720px}
.toolbar{display:flex;gap:8px;flex-shrink:0;padding-top:6px}button,a{color:var(--blue)}button{font:inherit;border:1px solid var(--line);border-radius:8px;background:var(--card);padding:8px 14px;cursor:pointer}button:focus-visible,a:focus-visible{outline:2px solid var(--blue);outline-offset:3px}
.grid{display:grid;grid-template-columns:repeat(4,1fr);gap:16px}.card{background:var(--card);border:1px solid var(--line);border-radius:16px;padding:14px 12px 18px;text-align:center;min-width:0}.art{height:154px;display:flex;align-items:center;justify-content:center}.art img{display:block}.card h2{font-size:16px;margin:0;font-weight:650;letter-spacing:.4px}.card p{font-size:12px;color:var(--muted);margin:3px 0 0}.identity{background:transparent;border-style:dashed}
.principles{display:grid;grid-template-columns:repeat(3,1fr);gap:28px;margin:32px 0 0}.principles strong{font-size:14px}.principles p{font-size:13px;color:var(--muted);margin:5px 0 0}
.sizes{margin-top:48px}.sizes h2{font-size:23px;margin:0 0 4px}.hint{font-size:13px;color:var(--muted);margin:0 0 22px}.table{border:1px solid var(--line);border-radius:12px;overflow:auto;background:var(--card)}
.size-row{display:grid;grid-template-columns:100px 64px 64px 64px 80px 96px 156px;align-items:center;justify-content:space-between;gap:12px;min-width:770px;padding:12px 24px;border-bottom:1px solid var(--line)}.size-row:last-child{border:0}.size-row strong{font-size:13px}.sample{display:flex;align-items:center;justify-content:center;min-height:40px}.sample img{display:block}.size-head{font-size:12px;text-align:center;color:var(--muted)}
html[data-selected=true] .card,html[data-selected=true] .size-row:not(.size-head){background:var(--selection)}
footer{margin-top:28px;border-top:1px solid var(--line);padding-top:18px;font-size:12px;color:var(--muted)}footer p{margin:4px 0}footer a{margin-right:18px}
@media(max-width:850px){main{padding:28px 20px}header{display:block}.toolbar{margin-top:16px}.grid{grid-template-columns:repeat(3,1fr)}.principles{gap:16px}}
@media(max-width:560px){.grid{grid-template-columns:repeat(2,1fr)}h1{font-size:27px}.principles{grid-template-columns:1fr}.art{height:144px}}
</style></head><body><main>
<section id="overview"><header><div><p class="eyebrow">XDZIP / FILE ASSOCIATIONS</p><h1>一眼识别，每一种归档。</h1>
<p>沿用折角文档与编织标记，以格式名称为主、辅助色为辅。<br>10 个格式图标与 1 个通用后备图标，共用一套形状与留白。</p></div>
<div class="toolbar"><button id="theme" aria-pressed="false">深色背景</button><button id="selected" aria-pressed="false">选中背景</button></div></header>
<div class="grid">'''
    foot = '''</div><div class="principles"><div><strong>品牌一致</strong><p>纸张、折角与品牌标记复用现有几何，格式标签全部使用矢量轮廓。</p></div><div><strong>小尺寸做减法</strong><p>16 / 24 / 32 px 省去微字，扩大标记。文件名承担精确的格式识别。</p></div><div><strong>同族有联系</strong><p>TAR.GZ、TGZ、GZ 共用青绿色。GZ 图标不代表支持独立 GZIP。</p></div></div></section>
<section class="sizes"><h2>按实际尺寸检查</h2><p class="hint">浏览器 100% 下按 CSS 像素显示，保持透明边缘，不把预览倍率写进图标几何。48 px 起保留标签，长标签优先在 64 px 以上阅读。</p><div class="table"><div class="size-row size-head"><strong>格式</strong><span>16</span><span>24</span><span>32</span><span>48</span><span>64</span><span>128</span></div>'''
    end = '''</div></section><footer><p>设计素材，尚未接入新的逐格式系统映射。原生 Explorer、Finder 与 Linux 文件管理器的显示需在接入阶段分别验收。</p>
<p><a href="../../../docs/visual/file-association-icons.md">设计与映射规范</a><a href="../../assets/index.html">统一素材库</a><a href="../index.html">高保真总览</a></p></footer></main>
<script>
const params=new URLSearchParams(location.search);if(params.get('theme')==='dark')document.documentElement.dataset.theme='dark';
const theme=document.getElementById('theme'),selected=document.getElementById('selected');
function sync(){const dark=document.documentElement.dataset.theme==='dark';theme.textContent=dark?'浅色背景':'深色背景';theme.setAttribute('aria-pressed',String(dark));selected.setAttribute('aria-pressed',String(document.documentElement.dataset.selected==='true'))}sync();
theme.addEventListener('click',()=>{document.documentElement.dataset.theme=document.documentElement.dataset.theme==='dark'?'light':'dark';sync()});
selected.addEventListener('click',()=>{document.documentElement.dataset.selected=String(document.documentElement.dataset.selected!=='true');sync()});
</script></body></html>
'''
    return head + ''.join(cards) + foot + ''.join(rows) + end


def write_sources(check: bool) -> None:
    sources = {ASSETS / name: icon_svg(icon, size) for icon, _, size, name in source_specs()}
    sources[GALLERY] = gallery()
    for path, data in sources.items():
        if check:
            if not path.exists() or path.read_text(encoding='utf-8') != data:
                raise ValueError(f'Stale design source: {path.relative_to(ROOT)}')
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(data, encoding='utf-8', newline='\n')
    print(f'{len(sources) - 1} SVG sources and design gallery are consistent')


def containers(output: Path, stem: str) -> dict[str, bytes]:
    payloads = [(output / png_name(stem, size)).read_bytes() for size in ICO_SIZES]
    offset, directory = 6 + 16 * len(payloads), []
    for size, payload in zip(ICO_SIZES, payloads):
        directory.append(struct.pack('<BBBBHHII', size % 256, size % 256, 0, 0, 1, 32, len(payload), offset))
        offset += len(payload)
    ico = struct.pack('<HHH', 0, 1, len(payloads)) + b''.join(directory) + b''.join(payloads)
    chunks = []
    for tag, size in ICNS_CHUNKS.items():
        png = (output / 'macos' / png_name(stem, size)).read_bytes()
        chunks.append(tag.encode() + struct.pack('>I', len(png) + 8) + png)
    content = b''.join(chunks)
    return {'ico': ico, 'icns': b'icns' + struct.pack('>I', len(content) + 8) + content}


def export(output: Path) -> str:
    from PIL import Image, PngImagePlugin
    from playwright.sync_api import sync_playwright

    output.mkdir(parents=True, exist_ok=True)
    (output / 'macos').mkdir(exist_ok=True)
    executable = os.environ.get('XDZIP_CHROMIUM')
    with sync_playwright() as driver:
        browser = driver.chromium.launch(**({'executable_path': executable} if executable else {}))
        version = browser.version
        try:
            page = browser.new_page(device_scale_factor=1)
            page.route('**/*', lambda route: route.abort())
            for icon in SPEC['icons']:
                stem = f"xdzip-{icon['id']}"
                for platform in SPEC['platforms']:
                    folder = output / ('macos' if platform == 'macos' else '')
                    for size in SPEC['png_sizes']:
                        svg = icon_svg(icon, size, platform)
                        page.set_viewport_size({'width': size, 'height': size})
                        page.set_content('<style>html,body{margin:0;background:transparent}svg{display:block;width:100vw;height:100vh}</style>' + svg)
                        raw = page.screenshot(omit_background=True)
                        receipt = PngImagePlugin.PngInfo()
                        receipt.add_text('xdzip-render-sha256', digest(svg.encode()))
                        receipt.add_text('xdzip-format-id', icon['id'])
                        receipt.add_text('xdzip-platform', platform)
                        with Image.open(io.BytesIO(raw)) as image:
                            image.convert('RGBA').save(folder / png_name(stem, size), pnginfo=receipt)
                for suffix, data in containers(output, stem).items():
                    (output / f'{stem}.{suffix}').write_bytes(data)
        finally:
            browser.close()
    return version


def validate_exports(output: Path) -> dict:
    from PIL import Image

    records = []
    for icon in SPEC['icons']:
        stem = f"xdzip-{icon['id']}"
        for platform in SPEC['platforms']:
            for size in SPEC['png_sizes']:
                path = output / ('macos' if platform == 'macos' else '') / png_name(stem, size)
                with Image.open(path) as image:
                    if image.mode != 'RGBA' or image.size != (size, size):
                        raise ValueError(f'Wrong canvas: {path.name}')
                    if image.info.get('xdzip-render-sha256') != digest(icon_svg(icon, size, platform).encode()):
                        raise ValueError(f'Stale raster source: {path.name}')
                    alpha = image.getchannel('A')
                    bounds = alpha.point(lambda value: 255 if value >= 128 else 0).getbbox()
                    canvas = SPEC['platforms'][platform]['viewbox'][2]
                    target = size * (SPEC['platforms'][platform]['body_height_ratio'] + (1.7 / canvas if size > 32 else 0))
                    if alpha.getextrema() != (0, 255) or not bounds or abs(bounds[3] - bounds[1] - target) > 1.25:
                        raise ValueError(f'Wrong optical height or transparency: {path.name}: {bounds}')
                    if any(abs((bounds[i] + bounds[i + 2]) / 2 - size / 2) > 1 for i in [0, 1]):
                        raise ValueError(f'Off-centre body: {path.name}: {bounds}')
                    ratio = SPEC['platforms'][platform]['body_height_ratio']
                    width = size * 641.7 / canvas if size > 32 else .76 * (2 * round(size * ratio / 2))
                    if abs(bounds[2] - bounds[0] - width) > 1.25:
                        raise ValueError(f'Wrong optical width: {path.name}: {bounds}')
                    if any(alpha.getpixel(point) for point in [(0, 0), (size - 1, 0), (0, size - 1), (size - 1, size - 1)]):
                        raise ValueError(f'Nontransparent corner: {path.name}')
                    records.append({'file': path.relative_to(output).as_posix(), 'sha256': digest(path.read_bytes()),
                                    'size': size, 'body_bounds': bounds, 'platform': platform})
        for suffix, expected in containers(output, stem).items():
            path = output / f'{stem}.{suffix}'
            if path.read_bytes() != expected:
                raise ValueError(f'Stale container: {path.name}')
            with Image.open(path) as image:
                if suffix == 'ico':
                    if image.ico.sizes() != {(s, s) for s in ICO_SIZES}:
                        raise ValueError(f'Missing ICO representation: {path.name}')
                    for size in ICO_SIZES:
                        decoded = image.ico.getimage((size, size)).convert('RGBA')
                        with Image.open(output / png_name(stem, size)) as png:
                            if decoded.tobytes() != png.tobytes():
                                raise ValueError(f'ICO decode mismatch: {path.name}')
                else:
                    for representation in image.info['sizes']:
                        decoded = image.icns.getimage(representation).convert('RGBA')
                        size = representation[0] * representation[2]
                        with Image.open(output / 'macos' / png_name(stem, size)) as png:
                            if decoded.tobytes() != png.tobytes():
                                raise ValueError(f'ICNS decode mismatch: {path.name}')
    return {'png_count': len(records), 'ico_count': len(SPEC['icons']), 'icns_count': len(SPEC['icons']),
            'records': records, 'native_shell_acceptance': 'not performed',
            'scope': 'design exports only; no associations installed'}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--export', type=Path, help='Optional build/ or dist/ platform output folder')
    args = parser.parse_args()
    output = args.export.resolve() if args.export else None
    if output and not any(output.is_relative_to(ROOT / folder) and output != ROOT / folder for folder in ['build', 'dist']):
        parser.error('--export must be a child of repository build/ or dist/')
    for icon, _, size, _ in source_specs():
        root = ET.fromstring(icon_svg(icon, size))
        if any(node.tag.rsplit('}', 1)[-1] in ['text', 'image', 'script', 'foreignObject'] for node in root.iter()):
            raise ValueError('Icon must contain self-contained vectors, without fonts or embedded images')
    legacy_zip = ET.parse(ROOT / 'design/assets/brand/xdzip-zip.svg').getroot()
    current_zip = ET.fromstring(icon_svg(next(icon for icon in SPEC['icons'] if icon['id'] == 'zip')))
    if ET.tostring(list(legacy_zip)[-1]) != ET.tostring(list(current_zip)[-1]):
        raise ValueError('ZIP lettering must preserve the original document master path exactly')
    write_sources(args.check)
    if output:
        version = None if args.check else export(output)
        result = validate_exports(output)
        if not args.check:
            result['renderer'] = {'name': 'Chromium', 'version': version, 'dpr': 1}
            (output / 'validation.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
        print(f"Validated {result['png_count']} PNGs, {result['ico_count']} ICOs and {result['icns_count']} ICNS containers")


if __name__ == '__main__':
    main()
