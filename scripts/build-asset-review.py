#!/usr/bin/env python3
"""Build source comparisons and fixed-frame checks for the shared asset library.

Reference crops preserve source backgrounds. Production SVGs are displayed
without per-icon CSS transforms so defects remain visible in this review.
Historical before files are evidence and are never regenerated here.
"""
import argparse
import base64
import hashlib
import html
import json
import mimetypes
import os
from pathlib import Path
import xml.etree.ElementTree as ET
from asset_definitions import REFERENCES as SOURCE_REFERENCES

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / 'design/assets'
OUT = ROOT / 'docs/qa/asset-review'
# A crop is appearance evidence, not a recovered alpha layer or a runtime asset.
REFERENCES = dict(SOURCE_REFERENCES)
FILE_ICONS = ('folder', 'file-pdf', 'file-text', 'file-code',
              'file-markdown', 'image-file')
HERO_ICONS = ('archive-add', 'document-download', 'document-warning',
              'document-conflict', 'archive-warning', 'image-file',
              'image-amber', 'settings-tile', 'existing-archive', 'windows-document',
              'linux-document', 'cancelled-large', 'theme-light', 'theme-dark',
              'theme-system', 'conflict-pdf', 'conflict-warning', 'progress-image')
NOTES = {
    'app-mark': '当前品牌使用四条交错短带和开口 U；左侧旧拉链原图是历史视觉，不是新标志的像素目标。',
    'archive': '关联文件采用白色折角纸张和新品牌蓝色标记；旧拉链原图保留作历史对照。',
    'archive-grey': '非活动归档使用独立灰色矢量，不通过 CSS 灰度或透明度改造活动图。',
    'cancelled-large': '取消终态为灰色圆盘与白色停止方块；方块尺寸按实际圆盘墨迹测量，不使用切片留白作分母。',
    'theme-light': '完整浅色微型窗口；标题栏、两按钮、双列内容、边线与轻影在同一透明源内。',
    'theme-dark': '完整深色微型窗口，与浅色和跟随系统预览使用同一结构。',
    'folder': '目录光学尺寸从 17.5 × 15 修订为 22 × 18.857，中心保持 (12, 12)，'
              '使其与文件类型图标的视觉重量协调。旧原稿中的较小目录保留作对照，'
              '不再作为当前目录墨迹大小的匹配目标。',
    'file-pdf': 'PDF 纸张圆角、白折角和红色边缘从素材源表达，不在界面中横向拉伸。',
    'file-text': '折角纸张、短首行和三条正文线在矢量源中定义。',
    'file-code': '纸张边框与代码符号在素材源中校准，界面不再缩放内部路径。',
    'file-markdown': 'Markdown 字形与下箭头在素材源中平衡，界面不再改变描边。',
    'image-file': '图片类型图标采用透明矢量；照片缩略图是另一类演示资源。',
    'open': '使用正面圆角文件夹与横向前缘。',
    'view-list': '列表视图是纯横线；任务列表的圆点使用独立 task-list 图标。',
    'document-warning': '圆形提示与三角冲突分别使用 document-warning 和 document-conflict。',
    'info': '信息图标使用次要色；没有独立原稿切片。',
}
STYLE = '''
*{box-sizing:border-box}body{font:15px system-ui;background:#f3f5f8;color:#242735;
margin:0;padding:24px;max-width:1040px;margin:auto}h1{font-size:25px}h2{font-size:19px}
p{line-height:1.65}a{color:#255be7}.notice{background:#e9efff;border-radius:10px;padding:12px 16px}
article,.inspection{background:white;border:1px solid #dce1ea;border-radius:12px;padding:18px;
margin:16px 0;break-inside:avoid}article h2{font-size:17px;margin:0 0 16px}
small{font-size:12px;color:#747c90;margin-left:10px}.comparison{display:grid;
grid-template-columns:repeat(4,1fr);gap:12px}.comparison>div{display:flex;align-items:center;
flex-direction:column;gap:12px}b{font-size:12px}.tile{display:grid;place-items:center;
width:88px;height:88px;background:repeating-conic-gradient(#e7eaf0 0% 25%,#fff 0% 50%) 0/12px 12px;
border:1px solid #e7eaf0;border-radius:8px}.tile.dark{background:
repeating-conic-gradient(#272d39 0% 25%,#20232a 0% 50%) 0/12px 12px}
.tile img{width:64px;height:64px;object-fit:contain}.missing{display:grid;place-items:center;
height:88px;color:#8992a5;font-size:12px}.frame-grid{display:grid;
grid-template-columns:repeat(6,minmax(94px,1fr));gap:10px}.frame-card{display:flex;
align-items:center;flex-direction:column;gap:10px;min-width:0}.frame-card strong{
font-size:11px;font-weight:500;overflow-wrap:anywhere;text-align:center}
.fixed-frame{display:grid;place-items:center;width:48px;height:48px;background:
repeating-conic-gradient(#e7eaf0 0% 25%,#fff 0% 50%) 0/8px 8px}
.fixed-frame img{display:block;width:48px;height:48px}.theme-dark{background:#20232a;color:#eceef3}
.theme-dark .fixed-frame,.theme-dark .size-frame{background:
repeating-conic-gradient(#272d39 0% 25%,#20232a 0% 50%) 0/8px 8px}
.size-grid{display:grid;grid-template-columns:repeat(6,minmax(94px,1fr));gap:10px}
.size-card{display:flex;align-items:center;flex-direction:column;gap:8px;min-width:0}
.size-card strong{font-size:11px;font-weight:500;overflow-wrap:anywhere;text-align:center}
.size-frame{width:80px;height:80px;display:grid;place-items:center;background:
repeating-conic-gradient(#e7eaf0 0% 25%,#fff 0% 50%) 0/8px 8px}
.size-frame img{display:block}.size-title{font-size:14px;margin:22px 0 14px}
.legend{font-size:13px;color:#647087}.theme-dark .legend{color:#a8b1c5}
.folder-review{max-width:430px;padding:12px}.folder-review h1{font-size:21px}
.folder-review .inspection{padding:12px;margin:12px 0}
.folder-review h2{font-size:16px;margin:0 0 12px}
.folder-review .frame-grid{grid-template-columns:repeat(6,48px);gap:8px;justify-content:space-between}
.folder-review .frame-card{gap:8px}.folder-review .frame-card strong{font-size:11px}
@media(max-width:760px){.frame-grid,.size-grid{grid-template-columns:repeat(4,minmax(72px,1fr))}}
@media(max-width:540px){body{padding:12px}.comparison{grid-template-columns:repeat(2,1fr)}
.frame-grid,.size-grid{grid-template-columns:repeat(3,minmax(72px,1fr))}h1{font-size:22px}}
'''
HEAD = ('<!doctype html><html lang="zh-CN"><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width,initial-scale=1">')


def embedded(path):
    """Embed local files for offline review without an asset server."""
    mime = mimetypes.guess_type(path.name)[0]
    return f'data:{mime};base64,' + base64.b64encode(path.read_bytes()).decode()


def picture(path, dark=False):
    if path is None:
        return '<span class="missing">无独立切片 / 未收录历史</span>'
    return (f'<span class="tile {"dark" if dark else ""}">'
            f'<img src="{embedded(path)}" alt="{html.escape(path.stem)}"></span>')


def page(title, content):
    return (HEAD + f'<title>{html.escape(title)}</title><style>{STYLE}</style>'
            + content + '</html>')


def file_inspection():
    """Use the same frame for every icon, without path-level style overrides."""
    sections = []
    for theme, label in [('light', '浅色'), ('dark', '深色')]:
        cards = []
        for name in FILE_ICONS:
            src = ASSETS / 'svg' / theme / f'{name}.svg'
            cards.append('<div class="frame-card"><span class="fixed-frame">'
                         f'<img src="{embedded(src)}" alt="{name}"></span>'
                         f'<strong>{name}</strong></div>')
        sections.append(f'<section class="inspection theme-{theme}" id="fixed-{theme}">'
                        f'<h2>{label} · 同一 48 × 48 显示槽</h2>'
                        '<p class="legend">透明画布大小一致；目录和纸张保留各自轮廓比例。'
                        '没有按类型设置缩放、位移或内部描边。</p>'
                        '<div class="frame-grid">' + ''.join(cards) + '</div></section>')
        sizes = []
        for size in [24, 48, 72]:
            cards = []
            for name in FILE_ICONS:
                src = ASSETS / 'svg' / theme / f'{name}.svg'
                cards.append('<div class="size-card"><span class="size-frame">'
                             f'<img width="{size}" height="{size}" '
                             f'src="{embedded(src)}" alt="{name}"></span>'
                             f'<strong>{name}</strong></div>')
            sizes.append(f'<h3 class="size-title">SVG 显示为 {size} × {size} CSS 像素</h3>'
                         '<div class="size-grid">' + ''.join(cards) + '</div>')
        sections.append(f'<section class="inspection theme-{theme}" id="sizes-{theme}">'
                        f'<h2>{label} · 24 / 48 / 72 尺寸检视</h2>' + ''.join(sizes)
                        + '</section>')
    intro = ('<h1>xdzip · 文件图标共同画布检视</h1><p><a href="index.html">返回逐项对照</a></p>'
             '<p class="notice">每个生产 SVG 的逻辑画布都是 24 × 24。'
             '本页把同一矢量直接显示在指定尺寸；72 像素视图用于检查曲线，'
             '不代表产品必须采用 72 逻辑像素图标。图片类型图标与真实照片缩略图分开。'
             '标题栏品牌和 Tab 归档标识按自己的使用角色，在逐项对照中展示。</p>')
    return page('xdzip 文件图标共同画布检视', intro + ''.join(sections))


def folder_comparison():
    """Show actual prior/current SVGs, not a CSS approximation of the revision."""
    labels = {'folder': '目录', 'file-pdf': 'PDF', 'file-text': 'TXT',
              'file-code': 'CODE', 'file-markdown': 'MD', 'image-file': '图片'}
    sections = []
    for theme, theme_label in [('light', '浅色'), ('dark', '深色')]:
        for previous, caption in [(True, '修订前 · 17.5 × 15'),
                                  (False, '当前 · 22 × 18.857')]:
            cards = []
            for name in FILE_ICONS:
                source = (OUT / 'before' / f'folder-list-{theme}.svg'
                          if previous and name == 'folder'
                          else ASSETS / 'svg' / theme / f'{name}.svg')
                cards.append('<div class="frame-card"><span class="fixed-frame">'
                             f'<img src="{embedded(source)}" alt="{name}"></span>'
                             f'<strong>{labels[name]}</strong></div>')
            sections.append(f'<section class="inspection theme-{theme}">'
                            f'<h2>{theme_label} · {caption}</h2>'
                            '<div class="frame-grid">' + ''.join(cards) + '</div></section>')
    content = ('<h1>xdzip · 目录图标大小修订</h1><p><a href="file-icons.html">完整尺寸检视</a></p>'
               '<p>所有图片使用同一 48 × 48 显示槽。只修订目录的源几何；'
               '另外五类素材完全相同，页面没有逐类型缩放。</p>' + ''.join(sections))
    return page('xdzip 目录图标大小修订', content).replace(
        '</style>', '</style><body class="folder-review">').replace('</html>', '</body></html>')


def file_geometry():
    """Record source raster bounds, without treating bounds as visual similarity.

    Alpha threshold 32 excludes almost invisible edges. All files share a
    canvas; legitimate silhouette differences remain visible in this report.
    """
    from PIL import Image
    records = []
    for theme in ['light', 'dark']:
        for name in FILE_ICONS:
            svg = ASSETS / 'svg' / theme / f'{name}.svg'
            tree = ET.parse(svg).getroot()
            assert tree.get('viewBox') == '0 0 24 24', svg
            assert float(tree.get('width')) == float(tree.get('height')) == 24, svg
            raster = ASSETS / 'png' / theme / f'{name}@3x.png'
            image = Image.open(raster).convert('RGBA')
            assert image.size == (72, 72), raster
            alpha = image.getchannel('A')
            bounds = alpha.point(lambda value: 255 if value > 32 else 0).getbbox()
            assert bounds is not None
            for point in [(0, 0), (71, 0), (0, 71), (71, 71)]:
                assert alpha.getpixel(point) == 0, raster
            x1, y1, x2, y2 = [value / 3 for value in bounds]
            center = [(x1 + x2) / 2, (y1 + y2) / 2]
            assert all(abs(value - 12) <= 1 / 3 for value in center), raster
            expected_height = 22 / 17.5 * 15 if name == 'folder' else 22
            # A transformed outline may cover an extra low-alpha raster edge.
            # Bounds at 3× quantize to thirds; two edge pixels are the limit.
            height_tolerance = 2 / 3 if name == 'folder' else 1 / 3
            assert abs(y2 - y1 - expected_height) <= height_tolerance + 0.001, raster
            records.append({'id': name, 'theme': theme, 'canvas': [24, 24],
                            'visible_bounds_alpha_gt32': [round(value, 3) for value in
                                                         [x1, y1, x2 - x1, y2 - y1]],
                            'visible_center': [round(value, 3) for value in center],
                            'accepted_visible_height': expected_height,
                            'svg_sha256': hashlib.sha256(svg.read_bytes()).hexdigest(),
                            'raster_sha256': hashlib.sha256(raster.read_bytes()).hexdigest()})
    return records


def render():
    """Capture current review pages, preserving historical SVG evidence."""
    from playwright.sync_api import sync_playwright
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(
            executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
            args=['--no-sandbox', '--disable-dev-shm-usage'])
        canvas = browser.new_page(viewport={'width': 1040, 'height': 900},
                                  device_scale_factor=1)
        canvas.set_content((OUT / 'file-icons.html').read_text(), wait_until='load')
        canvas.evaluate('document.fonts.ready')
        fixed = canvas.locator('.fixed-frame img').evaluate_all('elements => elements.map(image => ({'
            'name:image.alt, width:image.getBoundingClientRect().width,'
            'height:image.getBoundingClientRect().height, naturalWidth:image.naturalWidth,'
            'naturalHeight:image.naturalHeight, transform:getComputedStyle(image).transform}))')
        assert len(fixed) == len(FILE_ICONS) * 2
        assert all(item['width'] == item['height'] == 48
                   and item['naturalWidth'] == item['naturalHeight'] == 24
                   and item['transform'] == 'none' for item in fixed)
        sized = canvas.locator('.size-frame img').evaluate_all('elements => elements.map(image => ({'
            'name:image.alt, requested:Number(image.getAttribute("width")),'
            'width:image.getBoundingClientRect().width, height:image.getBoundingClientRect().height,'
            'transform:getComputedStyle(image).transform}))')
        assert len(sized) == len(FILE_ICONS) * 6
        assert all(item['width'] == item['height'] == item['requested']
                   and item['requested'] in [24, 48, 72]
                   and item['transform'] == 'none' for item in sized)
        canvas.screenshot(path=str(OUT / 'file-icons.png'), full_page=True)
        for theme in ['light', 'dark']:
            canvas.locator(f'#fixed-{theme}').screenshot(path=str(OUT / f'file-icons-{theme}.png'))
        canvas.set_content((OUT / 'focus.html').read_text(), wait_until='load')
        canvas.screenshot(path=str(OUT / 'comparison.png'), full_page=True)
        canvas.locator('#open').screenshot(path=str(OUT / 'open-comparison.png'))
        canvas.set_content((OUT / 'hero-icons.html').read_text(), wait_until='load')
        canvas.screenshot(path=str(OUT / 'hero-icons.png'), full_page=True)
        canvas.set_viewport_size({'width': 430, 'height': 600})
        canvas.set_content((OUT / 'folder-before-after.html').read_text(), wait_until='load')
        assert canvas.evaluate('document.documentElement.scrollWidth <= innerWidth')
        assert canvas.locator('.fixed-frame img').evaluate_all(
            'images => images.every(image => image.getBoundingClientRect().width === 48'
            ' && image.getBoundingClientRect().height === 48'
            ' && getComputedStyle(image).transform === "none")')
        folder_frame_count = canvas.locator('.fixed-frame img').count()
        assert folder_frame_count == len(FILE_ICONS) * 4
        canvas.screenshot(path=str(OUT / 'folder-before-after.png'), full_page=True)
        canvas.set_viewport_size({'width': 390, 'height': 844})
        assert canvas.evaluate('document.documentElement.scrollWidth <= innerWidth')
        canvas.set_content((OUT / 'file-icons.html').read_text(), wait_until='load')
        assert canvas.evaluate('document.documentElement.scrollWidth <= innerWidth')
        canvas.locator('#fixed-light').screenshot(path=str(OUT / 'file-icons-mobile.png'))
        (OUT / 'capture-validation.json').write_text(json.dumps({
            'status': 'passed', 'fixed_frame_instances': fixed, 'size_instances': sized,
            'folder_comparison_frames': folder_frame_count,
            'folder_comparison_mobile_horizontal_overflow': False,
            'mobile_viewport': [390, 844], 'mobile_horizontal_overflow': False,
            'renderer': 'Chromium; CSS pixel viewport; device scale factor 1',
            'limits': ['Frame and overflow checks do not prove resemblance to the reference',
                       'Qt and native platform rendering are outside this capture']},
            ensure_ascii=False, indent=2) + '\n')
        browser.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--render', action='store_true', help='also capture Chromium PNG evidence')
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    folder_baseline = json.loads((OUT / 'folder-baseline.json').read_text())
    for entry in folder_baseline['assets']:
        assert hashlib.sha256((ROOT / entry['file']).read_bytes()).hexdigest() == entry['sha256']
    resource_paths = {str((ROOT / 'apps/desktop' / node.text).resolve().relative_to(ROOT))
                      for node in ET.parse(ROOT / 'apps/desktop/ui.qrc').iter('file')}
    rows, records = [], []
    for src in sorted((ASSETS / 'svg/light').glob('*.svg')):
        name = src.stem
        crop = ASSETS / 'reference' / (REFERENCES[name] + '.png') if name in REFERENCES else None
        old = OUT / 'before' / src.name
        if not old.exists():
            old = None
        runtime = any(f'design/assets/png/{theme}/{name}@3x.png' in resource_paths
                      for theme in ['light', 'dark'])
        note = NOTES.get(name, '相邻原稿用于人工检查轮廓；透明重建不表示恢复了原图图层或完成逐像素验收。')
        records.append({'id': name, 'source_crop': str(crop.relative_to(ROOT)) if crop else None,
                        'source_kind': 'baked-background-reference' if crop else 'no-independent-crop',
                        'historical_vector': str(old.relative_to(ROOT)) if old else None,
                        'historical_sha256': hashlib.sha256(old.read_bytes()).hexdigest() if old else None,
                        'vector': str(src.relative_to(ROOT)), 'runtime': runtime,
                        'html_registered': True,
                        'reference_relation': 'historical-design-reference' if name in ('app-mark', 'archive', 'archive-grey') else 'appearance-reference',
                        'review': note})
        rows.append(f'<article id="{name}"><h2>{name} '
                    f'<small>{"Qt 资源已引用" if runtime else "Qt 尚未引用"} · HTML 共享库已登记</small></h2>'
                    '<div class="comparison">' + ''.join(f'<div><b>{label}</b>{pic}</div>' for label, pic in [
                        ('历史品牌原稿 · 含背景' if name in ('app-mark', 'archive', 'archive-grey') else '原稿参考 · 含背景', picture(crop)), ('历史重建 · 留存证据', picture(old)),
                        ('当前透明 SVG · 浅色', picture(src)),
                        ('当前透明 SVG · 深色', picture(ASSETS / 'svg/dark' / src.name, True))])
                    + f'</div><p>{html.escape(note)}</p></article>')
    intro = ('<h1>xdzip · 原稿与统一素材逐项对照</h1><p>'
             '<a href="../../../design/assets/index.html">统一素材目录</a> · '
             '<a href="file-icons.html">文件图标 24 / 48 / 72 尺寸检视</a> · '
             '<a href="folder-before-after.html">目录大小修订</a> · '
             '<a href="hero-icons.html">状态大图轮廓与配色</a></p>'
             '<p class="notice">原稿切片含背景，仅作视觉参考。浅深色生产 SVG 位于同一素材库；'
             '历史重建只用于说明变化。每栏采用共同显示画布，没有逐图缩放补偿。'
             '深色是配色适配，不表示存在独立的深色原稿。'
             '品牌采用新标志；主题、状态与遗漏插图补全详见 <a href="../brand-and-assets.md">本次复核</a>。</p>')
    (OUT / 'index.html').write_text(page('xdzip 素材逐项对照', intro + ''.join(rows)))
    focus_names = ('open', 'settings', 'view-list', 'folder', 'file-pdf', 'file-text',
                   'file-code', 'file-markdown', 'archive', 'archive-add', 'document-download',
                   'document-warning', 'document-conflict', 'archive-warning', 'image-file',
                   'image-amber', 'settings-tile')
    focus = [row for name in focus_names for row, record in zip(rows, records)
             if record['id'] == name]
    (OUT / 'focus.html').write_text(page('xdzip 素材重点对照', intro + ''.join(focus)))
    heroes = [row for name in HERO_ICONS for row, record in zip(rows, records)
              if record['id'] == name]
    (OUT / 'hero-icons.html').write_text(page('xdzip 状态大图对照', intro + ''.join(heroes)))
    (OUT / 'file-icons.html').write_text(file_inspection())
    (OUT / 'folder-before-after.html').write_text(folder_comparison())
    geometry = file_geometry()
    (OUT / 'review.json').write_text(json.dumps({
        'status': 'visual-review-with-known-differences', 'source_library': 'design/assets',
        'assets': records, 'file_icon_geometry': geometry,
        'folder_revision': {'previous': [17.5, 15], 'current': [22, 22 / 17.5 * 15],
                            'center': [12, 12], 'historical_evidence': folder_baseline,
                            'source_matching': 'original smaller folder retained for comparison'},
        'display_check': {'shared_svg_canvas': [24, 24], 'file_icon_frame': [48, 48],
                          'inspection_sizes': [24, 48, 72], 'per_icon_css_transform': False},
        'remaining_visual_differences': [
            'Source baked shadows and transparent layered lighting are independently reconstructed, not recovered alpha layers',
            'ZIP micro-lettering is reconstructed in the existing archive illustration; other zipper pull details remain simplified',
            'Image mountain motifs and gear corner curves are simplified compared with the source',
            'Source gradients, optical textures and antialiased curves are not exact recovered layers'],
        'limits': ['Baked references are not runtime alpha layers',
                   'Bounds checks do not prove pixel-perfect visual fidelity',
                   'Chromium captures do not replace native Qt renderer acceptance']},
        ensure_ascii=False, indent=2) + '\n')
    if args.render:
        render()
    print(f'Compared {len(records)} shared vector designs and checked {len(geometry)} file-icon geometries.')


if __name__ == '__main__':
    main()
