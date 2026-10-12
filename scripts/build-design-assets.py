#!/usr/bin/env python3
"""Export design assets; these tools do not form part of the product runtime."""
from pathlib import Path
import subprocess, json, hashlib, html
root = Path(__file__).resolve().parents[1] / 'design/assets/v1'
assets = []

def run(*args):
    subprocess.run(args, check=True, capture_output=True)
crops = {'brand-titlebar': ('main-light', 136, 44, 31, 37), 'brand-extract': ('extract-macos', 125, 136, 104, 122), 'folder': ('main-light', 397, 302, 42, 41), 'file-pdf': ('main-light', 397, 626, 44, 50), 'file-text': ('main-light', 398, 694, 43, 49), 'file-code': ('main-light', 397, 760, 47, 48), 'file-markdown': ('main-light', 397, 829, 47, 51), 'photo-lake-preview': ('main-light', 1198, 241, 350, 277), 'photo-lake-thumbnail': ('main-light', 397, 494, 45, 45), 'photo-sunrise-thumbnail': ('main-light', 397, 560, 45, 46), 'open': ('main-light', 48, 111, 32, 32), 'new': ('main-light', 174, 111, 28, 32), 'archive-test': ('main-light', 317, 110, 34, 34), 'more': ('main-light', 448, 111, 31, 31), 'search': ('main-light', 920, 111, 32, 32), 'settings': ('main-light', 1515, 43, 36, 34), 'view-list': ('main-light', 1000, 182, 46, 40), 'view-grid': ('main-light', 1065, 185, 28, 30), 'sort': ('main-light', 1114, 185, 36, 30), 'close': ('main-light', 1524, 192, 28, 27), 'check-selected': ('main-light', 337, 501, 30, 31), 'success': ('extract-macos', 141, 501, 43, 43), 'radio-selected': ('extract-macos', 142, 654, 42, 42), 'radio-unselected': ('extract-macos', 142, 934, 42, 43)}
for name, (source, x, y, w, h) in crops.items():
    target = root / 'slices' / f'{name}.png'
    run('magick', str(root / '../../hifi/v1/screens' / f'{source}.png'), '-crop', f'{w}x{h}+{x}+{y}', '+repage', str(target))
    assets.append(dict(id='slice/' + name, file=str(target.relative_to(root)), kind='original-pixel-crop', source='../../hifi/v1/screens/' + source + '.png', rect=[x, y, w, h], size=[w, h], background='baked source pixels; not transparent', usage='visual-reference' if not name.startswith('photo') else 'demo-image-only'))
paths = {'open': 'M3 8V5.5A2 2 0 0 1 5 3.5h4.2L12 6h7a2 2 0 0 1 2 2v10.5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8Z M3 9h18',
 'new': 'M6 3h12a3 3 0 0 1 3 3v12a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3Z M12 8v8 M8 12h8',
 'archive-test': 'M6 3h12a3 3 0 0 1 3 3v12a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3Z M7.5 12l3 3 6-6',
 'search': 'M17 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0 M15 15l6 6',
 'close': 'M6 6l12 12 M18 6L6 18',
 'chevron-down': 'M6 9l6 6 6-6',
 'chevron-right': 'M9 6l6 6-6 6',
 'chevron-up': 'M6 15l6-6 6 6',
 'check': 'M5 12l4 4L19 6',
 'arrow-right': 'M4 12h16 M14 6l6 6-6 6',
 'extract': 'M12 3v12 M7 10l5 5 5-5 M4 15v6h16v-6',
 'view-list': 'M5 6h14 M5 12h14 M5 18h14',
 'view-grid': 'M4 4h6v6H4Z M14 4h6v6h-6Z M4 14h6v6H4Z M14 14h6v6h-6Z',
 'sort': 'M5 5v14 M2 16l3 3 3-3 M12 6h9 M12 12h6 M12 18h3',
 'sort-ascending': 'M12 20V4 M6 10l6-6 6 6',
 'completed': 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0 M7 12l3 3 7-7',
 'external-open': 'M14 3h7v7 M21 3L10 14 M10 5H4v16h16v-6',
 'minimize': 'M5 12h14',
 'maximize': 'M5 5h14v14H5z',
 'settings': 'M10 3.2c0-.7.5-1.2 1.2-1.2h1.6c.7 0 1.2.5 1.2 1.2v1.3l1.9 1.1 1.1-.6c.6-.4 1.3-.2 1.7.4l.8 '
             '1.4c.4.6.2 1.3-.4 1.7l-1.1.6v2.2l1.1.6c.6.4.8 1.1.4 1.7l-.8 1.4c-.4.6-1.1.8-1.7.4l-1.1-.6-1.9 '
             '1.1v1.3c0 .7-.5 1.2-1.2 1.2h-1.6c-.7 '
             '0-1.2-.5-1.2-1.2v-1.3l-1.9-1.1-1.1.6c-.6.4-1.3.2-1.7-.4l-.8-1.4c-.4-.6-.2-1.3.4-1.7l1.1-.6V9.1L4.9 '
             '8.5c-.6-.4-.8-1.1-.4-1.7l.8-1.4c.4-.6 1.1-.8 1.7-.4l1.1.6L10 4.5Z M15 10.2a3 3 0 1 1-6 0 3 3 0 '
             '0 1 6 0'}
colored = {'folder': '<defs><linearGradient id="f" x1="0" y1="0" x2="0" y2="1"><stop stop-color="#64b8ff"/><stop '
           'offset="1" stop-color="#429cff"/></linearGradient></defs><path fill="#1761ff" stroke="#1761ff" '
           'stroke-width="1" d="M3.5 4.5h5.3l2.8 2.6h8.9a1 1 0 0 1 1 1v11.4a1 1 0 0 1-1 1h-17a1 1 0 0 '
           '1-1-1v-14a1 1 0 0 1 1-1Z"/><rect x="2.5" y="8" width="19" height="12.5" rx="1.2" fill="url(#f)" '
           'stroke="#267dff" stroke-width="1"/>',
 'archive': '<defs><linearGradient id="a" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#409cff"/><stop '
            'offset="1" stop-color="#164fff"/></linearGradient></defs><path fill="url(#a)" stroke="none" '
            'd="M6 2h8l6 6v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2Z"/><path fill="#1650da" '
            'stroke="none" d="M14 2v4a2 2 0 0 0 2 2h4Z"/><path stroke="white" stroke-width=".8" d="M11 '
            '3.5v11 M10 4h2 M11 6h2 M10 8h2 M11 10h2 M10 12h2"/><rect x="10.1" y="14.5" width="3.2" '
            'height="4.5" rx=".8" fill="none" stroke="white" stroke-width="1"/>',
 'file-text': '<path fill="{paper}" stroke="{muted}" stroke-width="1.2" d="M6 2h8l5 5v13.5a1.5 1.5 0 0 1-1.5 '
              '1.5H6a1.5 1.5 0 0 1-1.5-1.5v-17A1.5 1.5 0 0 1 6 2Z"/><path stroke="{muted}" '
              'stroke-width="1.2" d="M14 2v5h5 M7.5 10h8 M7.5 13.5h8 M7.5 17h8"/>',
 'file-code': '<rect x="3" y="2" width="18" height="20" rx="2.5" fill="{paper}" stroke="{muted}" '
              'stroke-width="1"/><path d="m8.5 9-3 3 3 3m7-6 3 3-3 3m-2.2-7-2.6 8"/>',
 'file-markdown': '<rect x="3" y="2" width="18" height="20" rx="2.5" fill="{paper}" stroke="{muted}" '
                  'stroke-width="1"/><path d="M6 15V9l3 4 3-4v6m5-6v6m-2-2 2 2 2-2"/>',
 'file-pdf': '<defs><linearGradient id="p" x2="0" y2="1"><stop stop-color="#ff393e"/><stop offset="1" '
             'stop-color="#ed121c"/></linearGradient></defs><path fill="url(#p)" stroke="none" d="M6 2h8l5 '
             '5v13.5a1.5 1.5 0 0 1-1.5 1.5H6a1.5 1.5 0 0 1-1.5-1.5v-17A1.5 1.5 0 0 1 6 2Z"/><path '
             'fill="white" stroke="none" d="M14 2v5h5Z"/><g stroke="white" stroke-width=".9" '
             'stroke-linejoin="round"><path d="M7 18v-5h1.3a1.3 1.3 0 0 1 0 2.6H7 M11 13h.9a1.6 1.6 0 0 1 '
             '1.6 1.6v1.8a1.6 1.6 0 0 1-1.6 1.6H11Z M16 18v-5h2 M16 15.4h1.6"/></g>',
 'more': '<circle cx="5" cy="12" r="1.5" fill="{ink}" stroke="none"/><circle cx="12" cy="12" r="1.5" '
         'fill="{ink}" stroke="none"/><circle cx="19" cy="12" r="1.5" fill="{ink}" stroke="none"/>',
 'app-mark': '<defs><linearGradient id="a" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#409cff"/><stop '
             'offset="1" stop-color="#164fff"/></linearGradient></defs><rect x="3" y="2" width="18" '
             'height="20" rx="3" fill="url(#a)" stroke="none"/><path stroke="white" stroke-width=".8" d="M11 '
             '3.5v11 M10 4h2 M11 6h2 M10 8h2 M11 10h2 M10 12h2"/><rect x="10.1" y="14.5" width="3.2" '
             'height="4.5" rx=".8" fill="none" stroke="white" stroke-width="1"/>'}
for theme, ink, paper, muted in [('light', '#20232a', '#ffffff', '#a4acbc'), ('dark', '#eceef3', '#20232a', '#8590a6')]:
    for name in list(paths) + list(colored):
        content = f'<path d="{paths[name]}" transform="translate(0 1.8)"/>' if name == 'settings' else f'<path d="{paths[name]}"/>' if name in paths else colored[name].replace('{paper}', paper).replace('{muted}', muted).replace('{ink}', ink)
        svg = f'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{ink}" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">{content}</svg>\n'
        target = root / 'svg' / theme / f'{name}.svg'
        target.write_text(svg)
        assets.append(dict(id=f'{theme}/{name}', file=str(target.relative_to(root)), kind='vector-reconstruction', logical_size=[24, 24], usage='candidate-ui-asset', theme=theme))
        for scale in [1, 2, 3]:
            dest = root / 'png' / theme / f'{name}@{scale}x.png'
for a in assets:
    a['sha256'] = hashlib.sha256((root / a['file']).read_bytes()).hexdigest()
(root / 'manifest.json').write_text(json.dumps(dict(version=1, source_coordinate_system='physical source pixels, top-left origin; rect = x,y,width,height', assets=assets), ensure_ascii=False, indent=2) + '\n')
cards = ''.join((f'''<div class="card"><img src="{html.escape(a['file'])}"><b>{html.escape(a['id'])}</b><small>{a['kind']}</small></div>''' for a in assets if a['kind'] == 'original-pixel-crop' or a['id'].startswith('light/')))
(root / 'index.html').write_text('<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>xdzip 素材切图</title><style>body{margin:0;padding:32px;background:#f4f5f7;color:#20232a;font:14px system-ui}h1{font-size:28px}p{line-height:1.8;max-width:850px}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(190px,1fr));gap:16px}.card{background:white;border:1px solid #dde1e9;border-radius:12px;padding:18px;display:flex;align-items:center;flex-direction:column;gap:14px;overflow:hidden}.card img{width:80px;height:80px;object-fit:contain;background:repeating-conic-gradient(#e7e9ed 0% 25%,#fff 0% 50%) 0/12px 12px}.card b{font-size:12px;overflow-wrap:anywhere}.card small{color:#6a7180;font-size:10px}a{color:#315fea}</style><h1>xdzip · 素材切图 v1</h1><p><a href="../../hifi/index.html">查看完整高保真原稿</a> · <a href="../../../docs/qa/asset-review/index.html">逐项对照原稿与修正素材</a></p><p>原像素切片保留背景，用于对照；矢量为重建素材，并非原图图层。PNG 提供 1× / 2× / 3×，SVG 提供浅色与深色版本。系统窗口按钮由平台绘制。<a href="manifest.json">素材清单与坐标</a></p><div class="grid">' + cards + '</div></html>')
print(json.dumps({'original_crops': len(crops), 'vector_designs': len(paths) + len(colored), 'svg_files': len(list((root / 'svg').rglob('*.svg'))), 'png_files': len(list(root.rglob('*.png')))}))
