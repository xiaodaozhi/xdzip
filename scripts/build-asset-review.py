#!/usr/bin/env python3
"""Build adjacent source/vector comparisons, with explicit unmatched references."""
from pathlib import Path
import base64
import html
import json
import mimetypes
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/qa/asset-review'
# A source crop is evidence of appearance, not an alpha layer or a shipping asset.
V1 = {
    'app-mark': 'brand-titlebar', 'archive': 'brand-extract',
    'folder': 'folder', 'file-pdf': 'file-pdf', 'file-text': 'file-text',
    'file-code': 'file-code', 'file-markdown': 'file-markdown',
    'open': 'open', 'new': 'new', 'archive-test': 'archive-test',
    'more': 'more', 'search': 'search', 'settings': 'settings',
    'view-list': 'view-list', 'view-grid': 'view-grid', 'sort': 'sort', 'close': 'close',
}
V2 = {
    'document-conflict': 'conflict-document', 'image-amber': 'image-amber',
    'status-partial-success': 'partial-success', 'archive-warning': 'overwrite-archive',
    'archive-add': 'create-archive', 'document-download': 'progress-document',
    'document-warning': 'partial-document', 'image-file': 'image-blue',
    'status-warning': 'partial-warning', 'status-success': 'success-large',
    'status-skipped': 'partial-skipped', 'status-stopped': 'cancelled-large',
    'spinner': 'stopping-spinner', 'remove': 'remove-row',
    'theme-system': 'theme-system', 'settings-tile': 'settings-gear',
}
NOTES = {
    'v1/open': '已改：斜口展开文件夹 → 正面圆角文件夹；横向前缘和底部轮廓恢复。',
    'v1/settings': '已改：六边形外框 → 带圆角齿槽的齿轮。',
    'v1/view-list': '已改：移除原稿没有的三个圆点，保留三条横线；选中底色由控件绘制。',
    'v1/sort': '已改：四条横线 → 原稿三条横线。',
    'v1/folder': '已改：硬直角和厚描边 → 圆角、分层蓝色文件夹。',
    'v1/archive': '已改：拉链改为细齿；保留文件折角。大图中的 ZIP 微字样未重建。',
    'v1/app-mark': '已分离：标题栏品牌为圆角方形，压缩文件为折角纸张，不再共用同一轮廓。',
    'v1/file-text': '已改：纸张轮廓与内文线分开闭合，降低描边并补圆角。',
    'v1/file-pdf': '已改：PDF 字母曲线、纸张圆角、折角与红色渐变。',
    'v1/file-code': '已改：边框变细、内符号收窄；保留可缩放路径。',
    'v1/file-markdown': '已改：边框变细，字形大小与下箭头重新平衡。',
    'v1/new': '已改：圆角增大，加号收窄。',
    'v1/archive-test': '已改：圆角增大，勾号收窄。',
    'v1/view-grid': '已改：四格收窄，恢复留白。',
    'v2/document-download': '已改：圆角纸张、独立折角、居中的短箭头。',
    'v2/document-warning': '已改：圆角纸张、独立折角、居中的圆形警告。三角警告使用独立 document-conflict。',
    'v2/archive-add': '已改：细拉链、圆角纸张、加号徽标保持透明边界内。',
    'v2/image-file': '已改：浅蓝纸张、山形与太阳；琥珀色图片改用独立 image-amber。',
    'v2/info': '已改：采用提示文字的次要色，避免旧图标过黑。无独立原稿切片。',
    'v2/remove': '已改：浅灰外圈与深色减号分开着色。',
    'v2/theme-system': '已改：右侧深蓝、浅色标题栏、两个窗口圆点与细边框。',
    'v2/status-success': '保留绿色完成状态；部分失败计数使用独立蓝色 status-partial-success。',
}


def embedded(path):
    mime = mimetypes.guess_type(path.name)[0]
    return f'data:{mime};base64,' + base64.b64encode(path.read_bytes()).decode()


def picture(path, dark=False):
    if path is None:
        return '<span class="missing">无独立切片</span>'
    return f'<span class="tile {"dark" if dark else ""}"><img src="{embedded(path)}" alt="{html.escape(path.stem)}"></span>'


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    resource_paths = {str((ROOT / 'apps/desktop' / node.text).resolve().relative_to(ROOT))
                      for node in ET.parse(ROOT / 'apps/desktop/ui.qrc').iter('file')}
    rows, records = [], []
    for version, folder, references in [('v1', 'design/assets/v1', V1), ('v2', 'design/hifi/v2', V2)]:
        base = ROOT / folder
        sources = sorted((base / 'svg/light').glob('*.svg'))
        for src in sources:
            key = f'{version}/{src.stem}'
            crop = base / 'slices' / (references[src.stem] + '.png') if src.stem in references else None
            old = OUT / 'before' / src.name if version == 'v1' else None
            if old is not None and not old.exists():
                old = None
            runtime = any(f'{folder}/png/{theme}/{src.stem}@3x.png' in resource_paths for theme in ('light', 'dark'))
            note = NOTES.get(key, '本次保留；相邻图用于人工核对，不表示逐像素验收通过。')
            records.append({'id': key, 'source_crop': str(crop.relative_to(ROOT)) if crop else None,
                            'vector': str(src.relative_to(ROOT)), 'runtime': runtime, 'review': note})
            rows.append(f'<article id="{version}-{src.stem}"><h2>{key} <small>{"Qt 已引用" if runtime else "尚未引用"}</small></h2>'
                        '<div class="comparison">' + ''.join(f'<div><b>{label}</b>{pic}</div>' for label, pic in [
                            ('原稿切片', picture(crop)), ('旧重建', picture(old)),
                            ('修正后 · 浅色', picture(src)), ('深色适配', picture(base / 'svg/dark' / src.name, True))])
                        + f'</div><p>{note}</p></article>')
    style = '''body{font:15px system-ui;background:#f3f5f8;color:#242735;margin:0;padding:24px;max-width:1000px;margin:auto}h1{font-size:25px}p{line-height:1.65}article{background:white;border:1px solid #dce1ea;border-radius:12px;padding:18px;margin:16px 0;break-inside:avoid}h2{font-size:17px;margin:0 0 16px}small{font-size:12px;color:#747c90;margin-left:10px}.comparison{display:grid;grid-template-columns:repeat(4,1fr);gap:12px}.comparison>div{display:flex;align-items:center;flex-direction:column;gap:12px}b{font-size:12px}.tile{display:grid;place-items:center;width:88px;height:88px;background:#fafbfc;border:1px solid #e7eaf0;border-radius:8px}.tile.dark{background:#20232a}.tile img{width:64px;height:64px;object-fit:contain}.missing{display:grid;place-items:center;height:88px;color:#8992a5;font-size:12px}@media(max-width:540px){body{padding:12px}.comparison{grid-template-columns:repeat(2,1fr)}}'''
    header = '<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>xdzip 素材逐项对照</title>'
    intro = '<h1>xdzip · 原稿与素材逐项对照</h1><p>同一行对照原稿、旧重建和当前矢量。放大视图用于检查轮廓；原稿切片含背景和留白，不据此计算相似度。深色为配色适配，不能视为存在对应深色原稿。未收录旧重建的栏位不表示原文件不存在。</p>'
    (OUT / 'index.html').write_text(header + '<style>' + style + '</style>' + intro + ''.join(rows) + '</html>')
    focus = [rows[i] for key in ('v1/open','v1/settings','v1/view-list','v1/folder','v1/archive') for i,r in enumerate(records) if r['id']==key]
    (OUT / 'focus.html').write_text(header + '<style>' + style + '</style>' + intro + ''.join(focus) + '</html>')
    (OUT / 'review.json').write_text(json.dumps({'status':'visual-review-with-known-differences','assets':records},ensure_ascii=False,indent=2)+'\n')
    print(f'Compared {len(records)} vector designs; reference gaps remain explicit.')


if __name__ == '__main__':
    main()
