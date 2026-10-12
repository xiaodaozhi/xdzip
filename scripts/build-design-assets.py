#!/usr/bin/env python3
"""Build the one canonical asset library; never regenerate approved artwork."""
from __future__ import annotations

import hashlib
import argparse
import html
import json
import shutil
import xml.etree.ElementTree as ET
from pathlib import Path

from PIL import Image
from asset_definitions import (
    CONTROL_PATHS, CROPS, EXTRA_CONTROL_PATHS, FILE_OPTICAL_SPECS, MAIN_SHAPES,
    REFERENCES, STATE_SHAPES, file_shapes, hero_shapes,
)

REPO = Path(__file__).resolve().parents[1]
ROOT = REPO / 'design/assets'
CANVAS = [24, 24]
THEMES = ('light', 'dark')
GRADIENTS = ('<defs><linearGradient id="sky" x2="0" y2="1"><stop stop-color="#c1edff"/>'
             '<stop offset="1" stop-color="#85d3fb"/></linearGradient>'
             '<linearGradient id="blue" x2="0" y2="1"><stop stop-color="#5bb5ff"/>'
             '<stop offset=".6" stop-color="#327dff"/><stop offset="1" stop-color="#1758f2"/>'
             '</linearGradient><linearGradient id="amber-warning" x2="0" y2="1">'
             '<stop stop-color="#ffc838"/><stop offset="1" stop-color="#f88618"/></linearGradient>'
             '<linearGradient id="amber-conflict" x2="0" y2="1"><stop stop-color="#fff0c8"/>'
             '<stop offset="1" stop-color="#ffd178"/></linearGradient>'
             '<linearGradient id="amber-archive" x2="0" y2="1"><stop stop-color="#ffd353"/>'
             '<stop offset="1" stop-color="#ffaf30"/></linearGradient>'
             '<linearGradient id="amber-image" x2="0" y2="1"><stop stop-color="#ffe5b4"/>'
             '<stop offset="1" stop-color="#ffd188"/></linearGradient></defs>')
GROUPS = {
    'file': {'folder', 'file-pdf', 'file-text', 'file-code', 'file-markdown', 'image-file', 'image-amber'},
    'brand': {'app-mark', 'archive'},
    'illustration': {'archive-add', 'document-download', 'document-warning', 'document-conflict',
                     'archive-warning', 'settings-tile'},
    'status': {'status-warning', 'status-success', 'status-skipped', 'status-stopped',
               'status-partial-success', 'spinner', 'info', 'remove', 'clock', 'theme-system'},
}
GROUP_TITLES = {'brand': '品牌与归档', 'file': '文件与目录', 'tool': '工具栏与导航',
                'status': '任务状态', 'illustration': '操作窗口插图'}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def category(name: str) -> str:
    return next((key for key, names in GROUPS.items() if name in names), 'tool')


def aliases() -> dict[tuple[str, str], list[str]]:
    """Read actual Qt aliases; stable names survive the directory migration."""
    result: dict[tuple[str, str], list[str]] = {}
    for node in ET.parse(REPO / 'apps/desktop/ui.qrc').iter('file'):
        if node.text and node.text.endswith('.png'):
            name = Path(node.text).stem.split('@')[0]
            theme = Path(node.text).parent.name
            assert theme in THEMES, f'Unexpected runtime icon directory: {node.text}'
            result.setdefault((theme, name), []).append(node.attrib.get('alias', node.text))
    return result


def crop_entries(check: bool = False) -> list[dict]:
    entries = []
    for name, (source, rect) in CROPS.items():
        folder = 'demo' if name.startswith('photo-') else 'reference'
        target = ROOT / folder / f'{name}.png'
        source_path = REPO / 'design/hifi/screens' / f'{source}.png'
        x, y, width, height = rect
        with Image.open(source_path) as source_image:
            pixels = source_image.convert('RGBA').crop((x, y, x + width, y + height))
        # Preserve the encoding and hashes of existing cuts. A clean build uses
        # Pillow; integrity depends on pixels, never on a PNG encoder's bytes.
        if target.exists():
            with Image.open(target) as existing:
                if existing.convert('RGBA').tobytes() != pixels.tobytes() or existing.size != pixels.size:
                    raise ValueError(f'Recorded source crop changed: {target.relative_to(ROOT)}')
        else:
            if check:
                raise ValueError(f'Missing source crop: {target.relative_to(ROOT)}')
            pixels.save(target)
        entries.append({
            'id': f'{folder}/{name}', 'file': str(target.relative_to(ROOT)),
            'kind': 'original-pixel-crop', 'source': f'../hifi/screens/{source}.png',
            'source_sha256': digest(source_path), 'rect': rect, 'size': [width, height],
            'usage': 'demo-image-only' if folder == 'demo' else 'visual-reference-only',
            'background': 'baked original pixels; no recovered transparency',
            'sha256': digest(target),
        })
    return entries


def svg_entries(check: bool = False) -> list[dict]:
    entries = []
    qt_aliases = aliases()
    for theme in THEMES:
        ink, paper, muted = (('#20232a', '#ffffff', '#a4acbc') if theme == 'light'
                            else ('#eceef3', '#20232a', '#8590a6'))
        state_muted, track = (('#788399', '#e2e6ee') if theme == 'light' else ('#9aa8c2', '#404958'))
        main_shapes = MAIN_SHAPES | file_shapes(theme)
        state_shapes = STATE_SHAPES | hero_shapes()
        definitions = CONTROL_PATHS | EXTRA_CONTROL_PATHS
        for name in sorted(set(definitions) | set(main_shapes) | set(state_shapes)):
            if name in definitions:
                transform = ' transform="translate(0 1.8)"' if name == 'settings' else ''
                content = f'<path d="{definitions[name]}"{transform}/>'
            elif name in main_shapes:
                content = main_shapes[name].replace('{paper}', paper).replace('{muted}', muted).replace('{ink}', ink)
            else:
                content = GRADIENTS + state_shapes[name].replace('{paper}', paper).replace('{muted}', state_muted).replace('{track}', track)
            svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" '
                   f'fill="none" stroke="{ink}" stroke-width="1.5" stroke-linecap="round" '
                   f'stroke-linejoin="round">{content}</svg>\n')
            target = ROOT / 'svg' / theme / f'{name}.svg'
            if check:
                if not target.is_file() or target.read_text() != svg:
                    raise ValueError(f'Stale SVG definition: {target.relative_to(ROOT)}')
            else:
                target.write_text(svg)
            entry = {
                'id': f'{theme}/{name}', 'file': str(target.relative_to(ROOT)),
                'kind': 'vector-reconstruction', 'logical_size': CANVAS, 'canvas': CANVAS,
                'theme': theme, 'category': category(name), 'usage': 'runtime-and-design',
                'runtime_usage': {'html': 'registered in the shared design library',
                                  'qt_aliases': qt_aliases.get((theme, name), [])},
                'sha256': digest(target),
            }
            if name in REFERENCES:
                entry['source_reference'] = f'reference/{REFERENCES[name]}.png'
            if name in FILE_OPTICAL_SPECS:
                entry['optical_spec'] = FILE_OPTICAL_SPECS[name] | {'bounds_include_stroke': True}
            entries.append(entry)
    return entries


def gallery(entries: list[dict]) -> str:
    """Show production geometry at fixed sizes beside independently cut source pixels."""
    vectors = [item for item in entries if item['kind'] == 'vector-reconstruction' and item['theme'] == 'light']
    sections = []
    for group in ('brand', 'file', 'tool', 'status', 'illustration'):
        cards = []
        for item in vectors:
            if item['category'] != group:
                continue
            name = item['id'].split('/')[1]
            label = html.escape(name)
            crop = (f'<img width="48" height="48" src="{item["source_reference"]}" alt="原稿：{label}">'
                    if 'source_reference' in item else '<span class="missing">无独立切片</span>')
            qt_count = len(item['runtime_usage']['qt_aliases'])
            cards.append(f'''<article class="card" data-name="{label}"><header><b>{label}</b><span>{f'浅色 Qt {qt_count} 引用' if qt_count else 'HTML 素材库 / 可复用'}</span></header><div class="comparison"><div class="source"><small>原稿参考</small><div class="sample">{crop}</div></div><div><small>浅色 · 48 px</small><div class="sample"><img width="48" height="48" src="svg/light/{label}.svg" alt="{label} 浅色"></div></div><div><small>深色 · 48 px</small><div class="sample dark"><img width="48" height="48" src="svg/dark/{label}.svg" alt="{label} 深色"></div></div></div><div class="exports"><span>透明导出</span>{''.join(f'<div><img width="{24 * scale}" height="{24 * scale}" src="png/light/{label}@{scale}x.png" alt="{label} {scale} 倍"><small>{24 * scale} × {24 * scale}</small></div>' for scale in (1, 2, 3))}</div></article>''')
        sections.append(f'<section id="{group}"><h2>{GROUP_TITLES[group]}</h2><div class="grid">{"".join(cards)}</div></section>')
    cut_cards = ''.join(f'<figure><div><img src="{item["file"]}" alt="{html.escape(item["id"])}"></div><figcaption>{html.escape(Path(item["file"]).stem)}<small>{item["size"][0]} × {item["size"][1]} 原像素</small></figcaption></figure>' for item in entries if item['kind'] == 'original-pixel-crop')
    styles = '''*{box-sizing:border-box}html{scroll-behavior:smooth}body{margin:0;background:#f3f5f9;color:#222a3b;font:14px -apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}main{max-width:1240px;margin:auto;padding:40px 24px}h1{font-size:32px;letter-spacing:-.6px;margin:0 0 12px}h2{font-size:21px;margin:32px 0 16px}p{line-height:1.85;color:#647089;max-width:920px}a{color:#245ce9}nav{display:flex;gap:18px;flex-wrap:wrap;padding:18px 0}.grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}.card{background:#fff;border:1px solid #e0e5ef;border-radius:14px;padding:18px;box-shadow:0 2px 8px #22335304}.card header{display:flex;gap:8px;justify-content:space-between;align-items:center;margin-bottom:18px}.card header b{font-size:13px}.card header span{font-size:10px;white-space:nowrap;color:#68758d}.comparison{display:grid;grid-template-columns:repeat(3,1fr);gap:8px;text-align:center}small{display:block;font-size:10px;color:#78849a;line-height:1.5}.sample{height:72px;display:flex;align-items:center;justify-content:center;background:#f8fafc;border:1px solid #e7ebf3;border-radius:9px;margin-top:8px}.sample img{display:block;object-fit:contain}.sample.dark{background:#222a37;border-color:#222a37}.source .sample{background:#f1f3f7}.missing{font-size:9px;color:#9aa4b7}.exports{display:flex;align-items:center;justify-content:space-between;gap:8px;border-top:1px solid #edf0f5;padding-top:16px;margin-top:16px;min-height:102px}.exports>span{font-size:10px;line-height:1.6;color:#8993a6;max-width:30px}.exports>div{display:flex;flex-direction:column;align-items:center;gap:7px}.exports img{object-fit:contain;display:block;background:repeating-conic-gradient(#e9edf4 0% 25%,#fff 0% 50%) 0/8px 8px}.references{display:grid;grid-template-columns:repeat(5,minmax(0,1fr));gap:12px}.references figure{margin:0;background:white;border:1px solid #e0e5ef;border-radius:10px;padding:14px;text-align:center}.references figure>div{height:110px;display:grid;place-items:center}.references img{max-width:100%;max-height:100px;object-fit:contain}.references figcaption{font-size:11px;overflow-wrap:anywhere;margin-top:14px}.banner{border-left:3px solid #2d6bfa;padding:3px 16px;background:#eef3fd;border-radius:0 8px 8px 0}.banner p{margin:8px 0}@media(max-width:850px){.grid{grid-template-columns:repeat(2,minmax(0,1fr))}.references{grid-template-columns:repeat(3,minmax(0,1fr))}}@media(max-width:540px){main{padding:26px 14px}.grid{grid-template-columns:1fr}.references{grid-template-columns:repeat(2,minmax(0,1fr))}h1{font-size:26px}}'''
    return ('<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">'
            '<title>xdzip · 统一素材库</title><style>' + styles + '</style><main><h1>xdzip · 统一素材库</h1>'
            '<p>一套命名、一份清单、一组透明画布。54 类共享图标提供浅色 / 深色 SVG 与 1× / 2× / 3× PNG；原稿参考和演示照片集中保存。</p>'
            '<p><a href="../hifi/index.html">完整高保真原稿</a> · <a href="../html/index.html">HTML 高保真界面</a> · '
            '<a href="../../docs/qa/asset-review/index.html">逐项质量复核</a> · <a href="manifest.json">资产清单</a></p>'
            '<div class="banner"><p>每个运行图标均使用 24 × 24 逻辑画布；下列浅深色对照固定显示为 48 × 48。文档可见高度一致，目录保留自然横向比例。原稿栏包含背景，只用于核对造型；透明运行素材来自独立矢量重建。</p></div>'
            '<nav>' + ''.join(f'<a href="#{group}">{title}</a>' for group, title in GROUP_TITLES.items()) +
            '<a href="#references">原像素参考</a></nav>' + ''.join(sections) +
            '<section id="references"><h2>原像素参考与演示照片</h2><p>45 张原稿参考保留原始背景和不同裁剪矩形；3 张演示照片保留源分辨率。它们的物理尺寸由原稿区域决定，不能作为图标控件的尺寸。</p><div class="references">' + cut_cards + '</div></section></main></html>\n')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Verify generated SVGs and gallery without rewriting files')
    args = parser.parse_args()
    for folder in ('reference', 'demo', 'svg/light', 'svg/dark', 'png/light', 'png/dark', 'preview'):
        (ROOT / folder).mkdir(parents=True, exist_ok=True)
    entries = crop_entries(args.check) + svg_entries(args.check)
    manifest = {
        'schema_version': 2, 'status': 'canonical-runtime-and-design-library',
        'coordinate_system': 'source physical pixels, top-left origin; rect = x,y,width,height',
        'icon_geometry': {'canvas': CANVAS, 'raster_scales': [1, 2, 3],
                          'visible_bounds_threshold': 'alpha > 8; pixel edges divided by scale',
                          'file_optical_specs': FILE_OPTICAL_SPECS},
        'license': 'Apache-2.0; fonts/ is independently licensed under OFL-1.1',
        'assets': entries,
    }
    if args.check:
        recorded = {item['file']: item for item in json.loads((ROOT / 'manifest.json').read_text())['assets']}
        for item in entries:
            assert all(recorded[item['file']].get(key) == value for key, value in item.items()), item['file']
        assert (ROOT / 'index.html').read_text() == gallery(entries), 'Stale asset gallery'
        assert (ROOT / 'LICENSE').read_bytes() == (REPO / 'LICENSE').read_bytes()
    else:
        (ROOT / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
        (ROOT / 'index.html').write_text(gallery(entries))
        shutil.copyfile(REPO / 'LICENSE', ROOT / 'LICENSE')
    print(json.dumps({'source_crops': len(CROPS), 'icon_designs': len(entries[48:]) // 2,
                      'svg_files': len(entries[48:]), 'root': str(ROOT.relative_to(REPO))}))


if __name__ == '__main__':
    main()
