#!/usr/bin/env python3
"""Export reference slices and new vector candidates from the v2 design boards."""
from pathlib import Path
import hashlib
import html
import json
import subprocess

ROOT = Path(__file__).resolve().parents[1] / 'design/hifi/v2'
# Source coordinates are physical pixels: x, y, width, height. No resizing or matting.
SLICES = {
    'create-archive': ('01-create', [110, 130, 115, 125]),
    'image-blue': ('01-create', [144, 415, 54, 59]),
    'image-amber': ('01-create', [144, 485, 54, 60]),
    'remove-row': ('01-create', [886, 422, 49, 49]),
    'conflict-document': ('02-conflict', [130, 193, 116, 142]),
    'conflict-pdf': ('02-conflict', [156, 458, 109, 119]),
    'conflict-warning': ('02-conflict', [140, 681, 53, 53]),
    'progress-document': ('03-progress', [133, 140, 104, 119]),
    'progress-image': ('03-progress', [169, 794, 51, 61]),
    'partial-document': ('04-partial-result', [115, 143, 113, 130]),
    'partial-success': ('04-partial-result', [242, 306, 57, 57]),
    'partial-skipped': ('04-partial-result', [645, 307, 56, 56]),
    'partial-warning': ('04-partial-result', [1035, 306, 59, 58]),
    'settings-gear': ('05-settings', [115, 139, 119, 120]),
    'theme-system': ('05-settings', [337, 378, 109, 86]),
    'theme-light': ('05-settings', [568, 378, 109, 86]),
    'theme-dark': ('05-settings', [798, 378, 108, 86]),
    'success-large': ('06-task-states', [122, 107, 101, 99]),
    'stopping-spinner': ('06-task-states', [122, 634, 101, 100]),
    'cancelled-large': ('06-task-states', [122, 1077, 101, 98]),
    'overwrite-archive': ('07-create-overwrite', [122, 214, 138, 163]),
    'existing-archive': ('07-create-overwrite', [165, 504, 117, 138]),
    'windows-document': ('08-platforms', [139, 123, 77, 88]),
    'linux-document': ('08-platforms', [136, 852, 80, 89]),
}
# Reconstructed assets are explicit candidates, not recovered source layers.
SHAPES = {
    'archive-add': '<path fill="url(#blue)" stroke="none" d="M5 2h9l5 5v14H5a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z"/><path fill="#1750d8" stroke="none" d="M14 2v5h5z"/><path stroke="white" d="M8 4h2m-1 3h2m-3 3h2m-1 3h2m-3 3h3v3H8z"/><circle cx="18" cy="18" r="5" fill="#2858ff" stroke="white"/><path stroke="white" d="M18 15v6m-3-3h6"/>',
    'document-download': '<path fill="url(#blue)" stroke="none" d="M5 2h9l5 5v15H5a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z"/><path fill="#1750d8" stroke="none" d="M14 2v5h5z"/><path stroke="white" stroke-width="2" d="M11 9v8m-4-4 4 4 4-4"/>',
    'document-warning': '<path fill="url(#amber)" stroke="none" d="M5 2h9l5 5v15H5a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z"/><path fill="#df9200" stroke="none" d="M14 2v5h5z"/><circle cx="11" cy="14" r="5" fill="#fff1cc" stroke="none"/><path stroke="#b96a00" d="M11 11v3m0 3h.01"/>',
    'image-file': '<path fill="url(#blue)" stroke="none" d="M5 2h9l5 5v15H5a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z"/><path fill="#b9e1ff" stroke="none" d="M14 2v5h5"/><circle cx="8" cy="9" r="2" fill="white" stroke="none"/><path fill="#bee5ff" stroke="none" d="m4 18 5-5 3 3 3-4 4 5v5H4z"/>',
    'status-warning': '<circle cx="12" cy="12" r="10" fill="#eda100" stroke="none"/><path stroke="white" stroke-width="2" d="M12 6v7m0 4h.01"/>',
    'status-success': '<circle cx="12" cy="12" r="10" fill="#348c44" stroke="none"/><path stroke="white" stroke-width="2" d="m6 12 4 4 8-8"/>',
    'status-skipped': '<circle cx="12" cy="12" r="10" fill="{muted}" stroke="none"/><path stroke="white" stroke-width="2" d="M7 12h10"/>',
    'status-stopped': '<circle cx="12" cy="12" r="10" fill="{muted}" stroke="none"/><rect x="8" y="8" width="8" height="8" rx="1.5" fill="white" stroke="none"/>',
    'spinner': '<circle cx="12" cy="12" r="9" stroke="{track}" stroke-width="2.5"/><path d="M12 3a9 9 0 0 1 9 9" stroke="{muted}" stroke-width="2.5"/>',
    'info': '<circle cx="12" cy="12" r="9"/><path d="M12 11v6m0-10h.01"/>',
    'remove': '<circle cx="12" cy="12" r="9"/><path d="M7 12h10"/>',
    'clock': '<circle cx="12" cy="12" r="9"/><path d="M12 6v7l4 2"/>',
    'theme-system': '<rect x="2" y="4" width="20" height="16" rx="2" fill="{paper}" stroke="{muted}"/><path d="M12 4h8a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-8z" fill="#2858ff" stroke="none"/><path d="M2 8h20" stroke="{muted}"/>',
    'settings-tile': '<rect x="1" y="1" width="22" height="22" rx="5" fill="url(#blue)" stroke="none"/><path fill="#dceaff" stroke="none" d="m10 4 4 0 .5 2 2 1 2-.5 2 3-1.5 1.5v2l1.5 1.5-2 3-2-.5-2 1-.5 2h-4l-.5-2-2-1-2 .5-2-3L5 13v-2L3.5 9.5l2-3 2 .5 2-1z"/><circle cx="12" cy="12" r="3" fill="#2858ff" stroke="none"/>',
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    entries = []
    for name, (source, rect) in SLICES.items():
        x, y, width, height = rect
        target = ROOT / 'slices' / f'{name}.png'
        subprocess.run(['magick', str(ROOT / 'screens' / f'{source}.png'), '-crop',
                        f'{width}x{height}+{x}+{y}', '+repage', str(target)], check=True)
        entries.append({'id': name, 'file': str(target.relative_to(ROOT)),
                        'kind': 'original-pixel-crop', 'source': f'screens/{source}.png',
                        'rect': rect, 'size': [width, height], 'usage': 'reference-only; baked background'})
    for theme, ink, muted, paper, track in [
        ('light', '#20232a', '#788399', '#ffffff', '#e2e6ee'),
        ('dark', '#eceef3', '#9aa8c2', '#20232a', '#404958'),
    ]:
        for name, content in SHAPES.items():
            content = content.replace('{muted}', muted).replace('{paper}', paper).replace('{track}', track)
            gradient = '<defs><linearGradient id="blue" x2="0" y2="1"><stop stop-color="#51adff"/><stop offset="1" stop-color="#1454ff"/></linearGradient><linearGradient id="amber" x2="0" y2="1"><stop stop-color="#ffdc6d"/><stop offset="1" stop-color="#f4a000"/></linearGradient></defs>'
            svg = f'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{ink}" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">{gradient}{content}</svg>\n'
            target = ROOT / 'svg' / theme / f'{name}.svg'
            target.write_text(svg)
            entries.append({'id': f'{theme}/{name}', 'file': str(target.relative_to(ROOT)),
                            'kind': 'vector-reconstruction', 'logical_size': [24, 24], 'theme': theme})
    for entry in entries:
        entry['sha256'] = digest(ROOT / entry['file'])
    screens = [{'file': str(p.relative_to(ROOT)), 'sha256': digest(p)} for p in sorted((ROOT / 'screens').glob('*.png'))]
    manifest = {'schema_version': 1, 'status': 'design-review-candidate', 'screens': screens,
                'shared_asset_base': '../../assets/v1', 'assets': entries,
                'coordinate_system': 'physical pixels, origin top-left; rect = x,y,width,height'}
    (ROOT / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    titles = ['新建压缩包', '重名冲突', '任务进度', '部分失败', '设置', '成功 / 停止 / 取消', '创建覆盖确认', 'Windows / Linux']
    cards = ''.join(f'<figure><a href="{s["file"]}"><img src="{s["file"]}" alt="{title}"></a><figcaption>{title}</figcaption></figure>' for s, title in zip(screens, titles))
    assets = ''.join(f'<figure class="asset"><img src="{a["file"]}" alt="{html.escape(a["id"])}"><figcaption>{html.escape(a["id"])}</figcaption></figure>' for a in entries if a['kind'] == 'original-pixel-crop' or a['id'].startswith('light/'))
    style = '<style>body{margin:0;padding:30px;background:#f4f5f7;color:#20232a;font:14px system-ui}h1{font-size:28px}p{line-height:1.8}.screens{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:18px}figure{margin:0;background:white;border:1px solid #dde1e9;border-radius:12px;padding:12px}figure img{display:block;width:100%;height:400px;object-fit:contain}figcaption{padding-top:12px;text-align:center}.assets{display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:12px}.asset img{height:72px;width:72px;margin:auto;background:repeating-conic-gradient(#e4e7ee 0% 25%,#fff 0% 50%) 0/12px 12px}.asset figcaption{font-size:11px;overflow-wrap:anywhere}@media(max-width:650px){body{padding:14px}.screens{grid-template-columns:1fr}}</style>'
    head = '<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">'
    (ROOT / 'index.html').write_text(head + '<title>xdzip 下一阶段高保真稿</title>' + style + '<h1>xdzip · 功能状态高保真稿</h1><p><a href="../index.html">返回全部高保真稿（含主界面）</a></p><p>8 张图板 · 11 个窗口画面 · 9 类功能状态。点击图片查看原图。生成式视觉审阅稿，交互以规格为准。</p><div class="screens">' + cards + '</div><h2>对应切图与新增图标</h2><p>前 24 项为保留背景的原像素切片；后 14 项为透明矢量重建候选。共享素材沿用 v1。</p><div class="assets">' + assets + '</div></html>')
    (ROOT / 'assets.html').write_text(head + '<title>xdzip 状态素材</title>' + style + '<h1>xdzip · 新增状态素材</h1><p>原稿切片保留背景；SVG / 透明 PNG 是明确标记的重建候选。</p><div class="assets">' + assets + '</div></html>')
    (ROOT / 'boards.html').write_text(head + '<title>xdzip 设计图板总览</title>' + style + '<h1>xdzip · 下阶段功能状态稿</h1><p><a href="../index.html">返回全部高保真稿（含主界面）</a></p><div class="screens">' + cards + '</div></html>')
    print(f'Exported {len(SLICES)} source crops and {len(SHAPES) * 2} SVGs')


if __name__ == '__main__':
    main()
