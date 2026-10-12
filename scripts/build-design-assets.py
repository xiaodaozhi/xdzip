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
    REFERENCES, STATE_SHAPES, SUPPLEMENTAL_ROLES, file_shapes, hero_shapes, revised_status_shapes,
    supplemental_shapes,
)
from brand_assets import brand_export_specs, brand_shapes
from directory_tree_assets import TREE_CUTS, TREE_ICON_REFERENCES, component_reference_coverage, tree_shapes

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
    'brand': {'app-mark', 'archive', 'archive-grey'},
    'illustration': {'archive-add', 'document-download', 'document-warning', 'document-conflict',
                     'archive-warning', 'settings-tile', 'existing-archive', 'windows-document',
                     'linux-document', 'conflict-pdf', 'progress-image'},
    'status': {'status-warning', 'status-success', 'status-skipped', 'status-stopped',
               'status-partial-success', 'spinner', 'info', 'remove', 'clock', 'theme-system',
               'theme-light', 'theme-dark', 'cancelled-large', 'check-selected', 'success',
               'radio-selected', 'radio-unselected', 'conflict-warning', 'tree-spinner', 'tree-info', 'tree-warning'},
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
            if (node.attrib.get('alias') == 'assets/app-icon.png'
                    and node.text == '../../design/assets/brand/xdzip.png'):
                # This one 512px application export is inventoried as a platform
                # icon below, never as a themed 24-unit interface glyph.
                continue
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
        main_shapes = MAIN_SHAPES | file_shapes(theme) | brand_shapes(theme) | tree_shapes(theme)
        state_shapes = STATE_SHAPES | hero_shapes() | revised_status_shapes(theme)
        supplemental = supplemental_shapes(theme)
        definitions = CONTROL_PATHS | EXTRA_CONTROL_PATHS
        for name in sorted(set(definitions) | set(main_shapes) | set(state_shapes) | set(supplemental)):
            if name in definitions:
                transform = ' transform="translate(0 1.8)"' if name == 'settings' else ''
                content = f'<path d="{definitions[name]}"{transform}/>'
            elif name in main_shapes:
                content = main_shapes[name].replace('{paper}', paper).replace('{muted}', muted).replace('{ink}', ink)
            elif name in supplemental:
                content = supplemental[name]
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
                if name in ('app-mark', 'archive', 'archive-grey'):
                    entry['reference_relation'] = 'historical-design-reference'
            if name in TREE_ICON_REFERENCES:
                entry['source_reference'] = f'reference/{TREE_ICON_REFERENCES[name]}.png'
                entry['component'] = 'directory-tree'
                entry['integration_status'] = 'design-ready-awaiting-html-and-qt-integration'
            if name in ('app-mark', 'archive', 'archive-grey'):
                brand_source = 'brand/xdzip.svg' if name == 'app-mark' else 'brand/xdzip-zip.svg'
                entry['brand_source'] = brand_source
                entry['source_sha256'] = digest(ROOT / brand_source)
                entry['source_revision'] = 'current-logo'
            if name in SUPPLEMENTAL_ROLES:
                entry['role'] = SUPPLEMENTAL_ROLES[name]
            if name in FILE_OPTICAL_SPECS:
                entry['optical_spec'] = FILE_OPTICAL_SPECS[name] | {'bounds_include_stroke': True}
            entries.append(entry)
    return entries


def brand_entries() -> list[dict]:
    """Admit the separately generated platform exports into the same manifest."""
    entries = []
    for spec in brand_export_specs():
        path = ROOT / spec['file']
        if not path.is_file():
            raise FileNotFoundError(f'Missing brand export; run build-brand-assets.py: {spec["file"]}')
        entry = {'id': 'brand/' + Path(spec['file']).name, 'category': 'brand',
                 'usage': 'application-brand' if spec['kind'] != 'brand-reference' else 'brand-source-reference',
                 **spec, 'sha256': digest(path)}
        if 'source' in spec:
            entry['source_sha256'] = digest(ROOT / spec['source'])
        if 'size' not in entry and path.suffix in ('.png', '.jpg', '.jpeg'):
            with Image.open(path) as picture:
                entry['size'] = list(picture.size)
        entry['runtime_usage'] = {'qt_aliases': [], 'platform_resources': []}
        if spec['file'] == 'brand/xdzip.png':
            entry['runtime_usage']['qt_aliases'] = ['assets/app-icon.png']
            entry['runtime_usage']['platform_resources'] = ['Linux application icon']
        elif spec['file'] == 'brand/xdzip.ico':
            entry['runtime_usage']['platform_resources'] = ['Windows application icon resource']
        elif spec['file'] == 'brand/xdzip.icns':
            entry['runtime_usage']['platform_resources'] = ['macOS application bundle icon']
        entries.append(entry)
    return entries


def reference_coverage() -> list[dict]:
    """Account for every preserved reference with an explicit semantic mapping."""
    entries = []
    for reference in sorted(name for name in CROPS if not name.startswith('photo-') and name not in TREE_CUTS):
        icons = sorted((name for name, source in REFERENCES.items() if source == reference),
                       key=lambda name: (name != reference, name))
        if not icons:
            raise ValueError(f'Unaccounted original reference: {reference}')
        primary = icons[0]
        historical = reference in ('brand-titlebar', 'brand-extract')
        role = ('historical-design-reference' if historical else
                SUPPLEMENTAL_ROLES.get(primary, 'shared-icon-source-reference'))
        entry = {
            'reference': f'reference/{reference}.png', 'role': role, 'icon_names': icons,
            'svg_files': {theme: [f'svg/{theme}/{name}.svg' for name in icons] for theme in THEMES},
            'implementation': 'shared-svg',
            'note': ('Original approved artwork is preserved; the current application brand follows its separate approved revision.'
                     if historical else 'Named reconstruction preserves this reference\'s semantic role; it is not a recovered alpha layer.'),
        }
        if role == 'control-state-reference':
            entry['native_control_note'] = 'The SVG documents appearance; Qt/HTML controls own checked state, focus and interaction.'
        if historical:
            entry['historical_reference'] = True
            entry['source_revision'] = 'current-logo'
        entries.append(entry)
    return entries


def gallery(entries: list[dict]) -> str:
    """Show production geometry at fixed sizes beside independently cut source pixels."""
    vectors = [item for item in entries if item['kind'] == 'vector-reconstruction' and item['theme'] == 'light']
    coverage = {item['reference']: item for item in reference_coverage()}
    sections = []
    for group in ('brand', 'file', 'tool', 'status', 'illustration'):
        cards = []
        for item in vectors:
            if item['category'] != group:
                continue
            name = item['id'].split('/')[1]
            label = html.escape(name)
            crop = (f'<img width="48" height="48" src="{item["source_reference"]}" alt="原稿：{label}">'
                    if 'source_reference' in item else
                    f'<img width="48" height="48" src="{item["brand_source"]}" alt="品牌母版：{label}">'
                    if 'brand_source' in item else '<span class="missing">无独立切片</span>')
            qt_count = len(item['runtime_usage']['qt_aliases'])
            reference_caption = ('历史设计参考' if item.get('reference_relation') == 'historical-design-reference' else
                                 '当前品牌母版' if 'brand_source' in item else '原稿参考')
            cards.append(f'''<article class="card" id="icon-{label}" data-name="{label}"><header><b>{label}</b><span>{f'浅色 Qt {qt_count} 引用' if qt_count else 'HTML 素材库 / 可复用'}</span></header><div class="comparison"><div class="source"><small>{reference_caption}</small><div class="sample">{crop}</div></div><div><small>浅色 · 48 px</small><div class="sample"><img width="48" height="48" src="svg/light/{label}.svg" alt="{label} 浅色"></div></div><div><small>深色 · 48 px</small><div class="sample dark"><img width="48" height="48" src="svg/dark/{label}.svg" alt="{label} 深色"></div></div></div><div class="exports"><span>透明导出</span>{''.join(f'<div><img width="{24 * scale}" height="{24 * scale}" src="png/light/{label}@{scale}x.png" alt="{label} {scale} 倍"><small>{24 * scale} × {24 * scale}</small></div>' for scale in (1, 2, 3))}</div></article>''')
        sections.append(f'<section id="{group}"><h2>{GROUP_TITLES[group]}</h2><div class="grid">{"".join(cards)}</div></section>')
    cut_cards = []
    component_cards = []
    components = {item['reference']: item for item in component_reference_coverage()}
    for item in entries:
        if item['kind'] != 'original-pixel-crop':
            continue
        component = components.get(item['file'])
        if component:
            labels = {'shared-svg': '共享透明图标', 'native-component': '动态组件', 'native-geometry': '动态几何'}
            links = ''.join(f'<a href="#icon-{html.escape(name)}">{html.escape(name)}</a> ' for name in component['icon_names'])
            component_cards.append(f'<figure><a href="{item["file"]}"><img src="{item["file"]}" alt="{html.escape(component["title"])}"></a><figcaption>{html.escape(component["title"])}<small>{html.escape(Path(item["file"]).stem)} · {item["size"][0]} × {item["size"][1]} 原像素</small><small>{labels[component["implementation"]]} · 待接入 HTML / Qt</small><small>{links}</small></figcaption></figure>')
            continue
        record = coverage.get(item['file'])
        links = (''.join(f'<a href="#icon-{html.escape(name)}">{html.escape(name)}</a> '
                         for name in record['icon_names']) if record else '<span>演示照片</span>')
        cut_cards.append(f'<figure><div><img src="{item["file"]}" alt="{html.escape(item["id"])}"></div><figcaption>{html.escape(Path(item["file"]).stem)}<small>{item["size"][0]} × {item["size"][1]} 原像素</small><small>{links}</small></figcaption></figure>')
    platform_cards = []
    for item in entries:
        if item['kind'] not in ('brand-master', 'platform-icon', 'brand-reference'):
            continue
        name = html.escape(Path(item['file']).name)
        # A 16px/24px platform PNG is shown at its real resolution, not enlarged
        # into a misleading, blurry 160px sample. Vector masters remain scalable.
        preview_size = min(160, max(item.get('size', [160]))) if Path(item['file']).suffix == '.png' else 160
        preview = (f'<img width="{preview_size}" height="{preview_size}" src="{item["file"]}" alt="{name}">'
                   if Path(item['file']).suffix in ('.svg', '.png', '.jpg', '.jpeg') else '<span class="format">' + name.rsplit('.', 1)[-1].upper() + '</span>')
        dimensions = f'{item["size"][0]} × {item["size"][1]} px' if 'size' in item else item.get('format', '').upper()
        platform_cards.append(f'<figure class="platform-card"><div>{preview}</div><figcaption><a href="{item["file"]}">{name}</a><small>{html.escape(item["kind"])} · {dimensions}</small></figcaption></figure>')
    platform_section = ('<section id="platform-brand"><h2>应用图标与平台导出</h2><p>品牌母版、已确认参考及各平台图标统一纳入这份清单。平台图标使用对应格式和物理分辨率，界面小图标继续使用 24 × 24 共享画布。</p><div class="platform-grid">'
                        + ''.join(platform_cards) + '</div></section>')
    styles = '''*{box-sizing:border-box}html{scroll-behavior:smooth}body{margin:0;background:#f3f5f9;color:#222a3b;font:14px -apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}main{max-width:1240px;margin:auto;padding:40px 24px}h1{font-size:32px;letter-spacing:-.6px;margin:0 0 12px}h2{font-size:21px;margin:32px 0 16px}p{line-height:1.85;color:#647089;max-width:920px}a{color:#245ce9}nav{display:flex;gap:18px;flex-wrap:wrap;padding:18px 0}.grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}.card{background:#fff;border:1px solid #e0e5ef;border-radius:14px;padding:18px;box-shadow:0 2px 8px #22335304}.card header{display:flex;gap:8px;justify-content:space-between;align-items:center;margin-bottom:18px}.card header b{font-size:13px}.card header span{font-size:10px;white-space:nowrap;color:#68758d}.comparison{display:grid;grid-template-columns:repeat(3,1fr);gap:8px;text-align:center}small{display:block;font-size:10px;color:#78849a;line-height:1.5}.sample{height:72px;display:flex;align-items:center;justify-content:center;background:#f8fafc;border:1px solid #e7ebf3;border-radius:9px;margin-top:8px}.sample img{display:block;object-fit:contain}.sample.dark{background:#222a37;border-color:#222a37}.source .sample{background:#f1f3f7}.missing{font-size:9px;color:#9aa4b7}.exports{display:flex;align-items:center;justify-content:space-between;gap:8px;border-top:1px solid #edf0f5;padding-top:16px;margin-top:16px;min-height:102px}.exports>span{font-size:10px;line-height:1.6;color:#8993a6;max-width:30px}.exports>div{display:flex;flex-direction:column;align-items:center;gap:7px}.exports img{object-fit:contain;display:block;background:repeating-conic-gradient(#e9edf4 0% 25%,#fff 0% 50%) 0/8px 8px}.platform-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(190px,1fr));gap:16px}.platform-card{margin:0;padding:20px;background:white;border:1px solid #e0e5ef;border-radius:14px;text-align:center}.platform-card>div{height:180px;display:grid;place-items:center}.platform-card img{object-fit:contain}.platform-card figcaption{font-size:12px;margin-top:16px}.format{font-size:26px;font-weight:650;color:#3975f0}.references{display:grid;grid-template-columns:repeat(5,minmax(0,1fr));gap:12px}.references figure{margin:0;background:white;border:1px solid #e0e5ef;border-radius:10px;padding:14px;text-align:center}.references figure>div{height:110px;display:grid;place-items:center}.references img{max-width:100%;max-height:100px;object-fit:contain}.references figcaption{font-size:11px;overflow-wrap:anywhere;margin-top:14px}.component-cuts{grid-template-columns:repeat(2,minmax(0,1fr))}.component-cuts img{max-height:320px;max-width:100%;object-fit:contain}.component-cuts figure>a{display:grid;place-items:center;min-height:90px}.banner{border-left:3px solid #2d6bfa;padding:3px 16px;background:#eef3fd;border-radius:0 8px 8px 0}.banner p{margin:8px 0}@media(max-width:850px){.grid{grid-template-columns:repeat(2,minmax(0,1fr))}.references{grid-template-columns:repeat(3,minmax(0,1fr))}}@media(max-width:540px){main{padding:26px 14px}.grid{grid-template-columns:1fr}.references{grid-template-columns:repeat(2,minmax(0,1fr))}.component-cuts{grid-template-columns:1fr}h1{font-size:26px}}'''
    return ('<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">'
            '<title>xdzip · 统一素材库</title><style>' + styles + '</style><main><h1>xdzip · 统一素材库</h1>'
            f'<p>一套命名、一份清单、一组透明画布。{len(vectors)} 类共享图标提供浅色 / 深色 SVG 与 1× / 2× / 3× PNG；原稿参考和演示照片集中保存。</p>'
            '<p><a href="../hifi/index.html">完整高保真原稿</a> · <a href="../html/index.html">HTML 高保真界面</a> · '
            '<a href="../../docs/qa/asset-review/index.html">逐项质量复核</a> · <a href="manifest.json">资产清单</a></p>'
            '<div class="banner"><p>每个运行图标均使用 24 × 24 逻辑画布；下列浅深色对照固定显示为 48 × 48。文档可见高度一致，目录保留自然横向比例。原稿栏包含背景，只用于核对造型；透明运行素材来自独立矢量重建。</p></div>'
            '<nav>' + ''.join(f'<a href="#{group}">{title}</a>' for group, title in GROUP_TITLES.items()) +
            '<a href="#platform-brand">平台品牌导出</a><a href="#directory-tree">目录树 B 切图</a><a href="#references">原像素参考</a></nav>' + ''.join(sections) + platform_section +
            f'<section id="directory-tree"><h2>目录树 B · {len(component_cards)} 张原稿切图</h2><p><a href="../hifi/index.html#components">三张正式图板</a> · <a href="../../docs/ux/directory-tree.md">交互与接入规范</a> · <a href="../../docs/visual/directory-tree.json">精确逻辑尺寸</a>。切片保留原始背景与裁剪尺寸，用于评审；动态行、连接线、提示和状态不作为整块控件贴图。文件夹与箭头复用统一资源；新增分支圆点、实心信息 / 警告和分段转轮已导出浅深色与三种倍率，当前尚未接入目录树界面。</p><div class="component-cuts references">' + ''.join(component_cards) + '</div></section>' +
            '<section id="references"><h2>原像素参考与演示照片</h2><p>45 张原稿参考保留原始背景和不同裁剪矩形，每项都有对应的独立命名 SVG；3 张演示照片保留源分辨率。下方链接可直接查看对应素材。两张历史品牌参考保留原像素，当前标识采用独立修订的品牌母版。</p><div class="references">' + ''.join(cut_cards) + '</div></section></main></html>\n')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Verify generated SVGs and gallery without rewriting files')
    args = parser.parse_args()
    for folder in ('reference', 'demo', 'svg/light', 'svg/dark', 'png/light', 'png/dark', 'preview'):
        (ROOT / folder).mkdir(parents=True, exist_ok=True)
    entries = crop_entries(args.check) + svg_entries(args.check) + brand_entries()
    manifest = {
        'schema_version': 2, 'status': 'canonical-runtime-and-design-library',
        'coordinate_system': 'source physical pixels, top-left origin; rect = x,y,width,height',
        'icon_geometry': {'canvas': CANVAS, 'raster_scales': [1, 2, 3],
                          'visible_bounds_threshold': 'alpha > 8; pixel edges divided by scale',
                          'file_optical_specs': FILE_OPTICAL_SPECS},
        'license': 'Apache-2.0; fonts/ is independently licensed under OFL-1.1',
        'reference_coverage': reference_coverage(),
        'component_reference_coverage': component_reference_coverage(),
        'assets': entries,
    }
    if args.check:
        existing = json.loads((ROOT / 'manifest.json').read_text())
        recorded = {item['file']: item for item in existing['assets']}
        assert existing['reference_coverage'] == reference_coverage(), 'Stale reference coverage ledger'
        assert existing['component_reference_coverage'] == component_reference_coverage(), 'Stale component reference coverage'
        for item in entries:
            assert all(recorded[item['file']].get(key) == value for key, value in item.items()), item['file']
        assert (ROOT / 'index.html').read_text() == gallery(entries), 'Stale asset gallery'
        assert (ROOT / 'LICENSE').read_bytes() == (REPO / 'LICENSE').read_bytes()
    else:
        (ROOT / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
        (ROOT / 'index.html').write_text(gallery(entries))
        shutil.copyfile(REPO / 'LICENSE', ROOT / 'LICENSE')
    svg_count = sum(item['kind'] == 'vector-reconstruction' for item in entries)
    print(json.dumps({'source_crops': len(CROPS), 'icon_designs': svg_count // 2,
                      'svg_files': svg_count, 'brand_exports': len(brand_export_specs()),
                      'reference_coverage': len(reference_coverage()), 'root': str(ROOT.relative_to(REPO))}))


if __name__ == '__main__':
    main()
