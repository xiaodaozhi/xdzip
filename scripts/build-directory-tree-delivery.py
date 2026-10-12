#!/usr/bin/env python3
"""Package the approved tree boards and assets without implementing a UI."""
from __future__ import annotations

import argparse
import base64
import hashlib
import html
import json
import mimetypes
from pathlib import Path
import re
from urllib.parse import quote
import zipfile

ROOT = Path(__file__).resolve().parents[1]
OUTPUT_ROOT = ROOT / 'design/hifi'
REPOSITORY_URL = 'https://github.com/xiaodaozhi/xdzip/blob/main/'


def image(path, label, width=None, height=None):
    source = path.relative_to(ROOT).as_posix()
    relative = '../../' + source
    dimensions = f' width="{width}" height="{height}"' if width and height else ''
    return f'<img src="{relative}" alt="{html.escape(label)}"{dimensions} loading="lazy">'


def generate():
    manifest = json.loads((ROOT / 'design/assets/manifest.json').read_text())
    spec = json.loads((ROOT / 'docs/visual/directory-tree.json').read_text())
    catalog = json.loads((OUTPUT_ROOT / 'catalog.json').read_text())
    records = {item['file']: item for item in manifest['assets']}
    files = {ROOT / 'docs/ux/directory-tree.md', ROOT / 'docs/visual/directory-tree.json',
             ROOT / 'docs/visual/tokens.json', ROOT / 'LICENSE'}
    boards = []
    for entry in catalog['screens']:
        if entry.get('component') != 'directory-tree':
            continue
        path = OUTPUT_ROOT / entry['file']
        assert hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256']
        files.add(path)
        boards.append(f'<figure>{image(path, entry["title"])}<figcaption>{html.escape(entry["title"])} · 1024×1536 原图</figcaption></figure>')
    cut_cards = []
    labels = {'shared-svg': '共享透明图标', 'native-component': '动态组件', 'native-geometry': '动态几何'}
    references = manifest['component_reference_coverage']
    for entry in references:
        record = records[entry['reference']]
        path = ROOT / 'design/assets' / entry['reference']
        assert hashlib.sha256(path.read_bytes()).hexdigest() == record['sha256']
        files.add(path)
        width, height = record['size']
        x, y, _, _ = record['rect']
        cut_cards.append(f'<figure class="cut"><div class="cut-image">{image(path, entry["title"], width, height)}</div><figcaption>{html.escape(entry["title"])}<small>{html.escape(path.name)} · {width}×{height} 原像素</small><small>原图位置 ({x}, {y}) · {labels[entry["implementation"]]}</small></figcaption></figure>')
    glyph_cards = []
    icons = sorted({name for item in references for name in item['icon_names']})
    for name in icons:
        previews = []
        for theme in ('light', 'dark'):
            path = ROOT / 'design/assets' / f'svg/{theme}/{name}.svg'
            files.add(path)
            previews.append(f'<div class="glyph {theme}">{image(path, name + " " + theme, 48, 48)}</div>')
            files.update(ROOT / 'design/assets' / f'png/{theme}/{name}@{scale}x.png' for scale in (1, 2, 3))
        glyph_cards.append(f'<article><b>{html.escape(name)}</b><div class="glyphs">{"".join(previews)}</div><small>统一 24×24 源画布；PNG 24 / 48 / 72</small></article>')
    geometry_labels = {
        'sidebar_default_width': '侧栏默认宽度', 'row_min_height': '最小行高', 'depth_indent': '逐层缩进',
        'disclosure_hit_width': '箭头命中区宽', 'disclosure_glyph_canvas': '箭头图形槽',
        'folder_canvas': '文件夹图形槽', 'icon_label_gap': '图标与文字间距',
        'body_font_size': '正文', 'count_font_size': '计数字号', 'count_min_width': '计数最小列宽',
        'selection_indicator_width': '当前目录左侧标记', 'selection_radius': '当前行圆角',
        'focus_stroke': '焦点描边', 'guide_stroke': '连接线描边',
        'branch_marker_visible_diameter': '隐藏当前目录圆点直径',
    }
    geometry_rows = ''.join(f'<tr><td>{label}</td><td>{spec["geometry"][key]} 逻辑 px</td></tr>' for key, label in geometry_labels.items())
    events = [
        ('点击箭头', '只展开 / 收起，不改变列表路径、文件选择或详情。'),
        ('点击名称 / Enter', '激活目录；成功替换查询后按既有规则重置分页、选择与详情。'),
        ('上 / 下', '移动可见目录行的焦点，不自动进入目录。'),
        ('右 / 左', '展开 / 收起；已经展开或无法收起时移到子节点 / 父节点。'),
        ('收起当前目录的祖先', '列表保持当前目录；最近可见折叠祖先显示圆点，焦点回到可见行。'),
        ('深层级 / 长名称', '层级区横向滚动，数字固定靠右；省略名称可查看完整名称与路径。'),
        ('200% 字号', '行高、图标槽与箭头命中区增长，文字和计数不裁切。'),
    ]
    event_rows = ''.join(f'<tr><td>{html.escape(label)}</td><td>{html.escape(note)}</td></tr>' for label, note in events)
    state_rows = ''.join(f'<tr><td>{html.escape(next(item["title"] for item in references if Path(item["reference"]).stem == state["cut"]))}</td><td><code>{html.escape(name)}</code></td></tr>' for name, state in spec['states'].items())
    page = '''<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>xdzip · 目录树 B 设计交付</title>
<style>*{box-sizing:border-box}html{scroll-behavior:smooth}body{margin:0;background:#f5f7fb;color:#20232a;font:15px/1.7 system-ui,-apple-system,sans-serif}main{max-width:1100px;margin:auto;padding:32px 24px}h1{font-size:28px;margin:0 0 12px}h2{font-size:22px;margin:36px 0 16px}a{color:#245ce9}nav{display:flex;gap:16px;flex-wrap:wrap}p{max-width:900px}figure,article{margin:0;padding:16px;border:1px solid #dce3ee;background:#fff;border-radius:12px;min-width:0}figure img{display:block;max-width:100%;height:auto;margin:auto}figcaption{margin-top:12px;font-weight:600;overflow-wrap:anywhere}small{display:block;font-size:12px;font-weight:400;color:#626874;overflow-wrap:anywhere}.boards{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}.cuts{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px}.cut-image{min-height:90px;display:grid;place-items:center}.cut-image img{max-height:360px;object-fit:contain}.icons{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}.glyphs{display:flex;gap:8px;margin:12px 0}.glyph{width:80px;height:80px;display:grid;place-items:center;border-radius:8px;background:repeating-conic-gradient(#eef1f6 0% 25%,white 0% 50%) 0/10px 10px}.glyph.dark{background:#202936}.glyph img{width:48px;height:48px}.note{padding:16px;background:#e9effb;border-radius:8px}.table-wrap{overflow:auto}table{border-collapse:collapse;width:100%;background:#fff}td,th{padding:10px 14px;border:1px solid #dce3ee;text-align:left;vertical-align:top}code{font-size:12px;overflow-wrap:anywhere}.geometry td:last-child{white-space:nowrap}@media(max-width:700px){main{padding:24px 14px}.boards,.cuts{grid-template-columns:1fr}.icons{grid-template-columns:repeat(2,minmax(0,1fr))}.boards figure{padding:8px}h1{font-size:24px}td,th{padding:8px}.geometry td:last-child{white-space:normal}}@media(max-width:360px){.icons{grid-template-columns:1fr}}</style></head><body><main>
<h1>xdzip · 目录树 B 设计交付</h1>
<p class="note">三张已确认图板、26 张原像素切片、共享透明图标与精确组件规格。设计和素材已交付，HTML / Qt 目录树改造待接入；本页仅展示原稿与素材，不模拟目录交互。</p>
<nav><a href="#boards">正式图板</a><a href="#geometry">尺寸</a><a href="#behavior">行为与状态</a><a href="#cuts">逐项切图</a><a href="#icons">共享透明图标</a></nav>
<p>图板是放大的视觉说明，局部卡片比例不同。下面的逻辑尺寸用于开发；切片保留物理像素和背景。原始主稿保持不变，此处只修订左侧目录组件。完整深色图板仍待补齐。</p>
<h2 id="boards">正式图板</h2><div class="boards">''' + ''.join(boards) + '''</div>
<h2 id="geometry">落地尺寸</h2><div class="table-wrap"><table class="geometry"><tbody>''' + geometry_rows + '''</tbody></table></div>
<p>Qt 直接使用上述逻辑值。现有 HTML 源画布的 CSS 值 = 逻辑值 ÷ (1280/1548)：40 行高对应 48.375 源 CSS px，14 字号对应 16.93125，20 图标槽对应 24.1875。预览缩放在换算后整体应用，DPR 不参与逻辑尺寸换算。</p>
<p>完整参数与大字号公式见 <a href="../../docs/visual/directory-tree.json">组件参数</a>；各主题共享形状，颜色遵循主题参数。文件夹保持自然比例，不按切片大小设置显示槽。</p>
<h2 id="behavior">行为与状态</h2><p>有无子目录决定箭头；条目数表示全部后代物理记录，包含重复记录，排除该节点自己的目录记录，不受搜索影响。只有文件或为空的叶子都没有箭头，同层级保留空槽。方向键焦点、展开、当前目录及文件复选框状态相互独立。</p>
<div class="table-wrap"><table><thead><tr><th>操作</th><th>结果</th></tr></thead><tbody>''' + event_rows + '''</tbody></table></div><p>读取时未知计数显示“—”，索引完整后发布树。空归档的文件列表为空；只有根文件时列表正常显示。导航索引超限仍可使用全部文件和搜索；读取失败提供重新打开，不假装空目录。</p>
<details><summary>16 种规定状态</summary><div class="table-wrap"><table><tbody>''' + state_rows + '''</tbody></table></div></details>
<p>接入与验收细则见 <a href="../../docs/ux/directory-tree.md">目录树规格</a>；本次资产检查见 <a href="../../docs/qa/directory-tree.md">交付复核</a>。</p>
<h2 id="cuts">26 张原像素切图</h2><p>裁剪没有缩放、去底色或补画；小图保持自然像素尺寸，宽图在手机上缩小以便浏览。背景、文字、连接线和状态由界面动态绘制，切片用于对照。</p><div class="cuts">''' + ''.join(cut_cards) + '''</div>
<h2 id="icons">共享透明图标</h2><p>下列浅深色图标统一显示在 48×48 审阅画布，实际组件按尺寸表等比显示。新增分支圆点、实心信息 / 警告和分段转轮尚无 Qt 引用，HTML 仅登记共享资源；文件夹与箭头复用当前素材库。</p><div class="icons">''' + ''.join(glyph_cards) + '''</div>
<footer><p>完整素材台账见 <a href="../assets/manifest.json">manifest.json</a>；项目原创设计采用 <a href="../../LICENSE">Apache-2.0</a>。离线版内嵌所有展示图片，辅助文档链接需要仓库访问权限。</p></footer>
</main></body></html>
'''
    return page, files


def standalone(page):
    def embed(match):
        path = (OUTPUT_ROOT / html.unescape(match[1])).resolve()
        assert path.is_relative_to(ROOT)
        data = base64.b64encode(path.read_bytes()).decode('ascii')
        return 'src="data:' + mimetypes.guess_type(path.name)[0] + ';base64,' + data + '"'

    def link(match):
        target = html.unescape(match[1])
        if target.startswith('#'):
            return match[0]
        path = (OUTPUT_ROOT / target).resolve().relative_to(ROOT)
        return 'href="' + REPOSITORY_URL + quote(path.as_posix()) + '"'

    return re.sub(r'href="([^"]+)"', link, re.sub(r'src="([^"]+)"', embed, page))


def package(path, page, files):
    files.add(OUTPUT_ROOT / 'directory-tree.html')
    members = {item.resolve() for item in files}
    ledger = []
    assets = json.loads((ROOT / 'design/assets/manifest.json').read_text())
    asset_records = {'design/assets/' + item['file']: item for item in assets['assets']}
    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, 'w', zipfile.ZIP_DEFLATED) as archive:
        for member in sorted(members):
            payload = member.read_bytes()
            relative = member.relative_to(ROOT).as_posix()
            if member.suffix == '.md':
                # Keep in-package links local; supporting repository pages are
                # online links so a partial handoff never contains dead paths.
                def supporting_link(match):
                    raw = match[2]
                    if raw.startswith('#') or '://' in raw:
                        return match[0]
                    target = (member.parent / raw.split('#', 1)[0]).resolve()
                    if target in members:
                        return match[0]
                    return match[1] + '(' + REPOSITORY_URL + quote(target.relative_to(ROOT).as_posix()) + ')'
                payload = re.sub(r'(!?\[[^\]]*\])\(([^)]+)\)', supporting_link, payload.decode('utf-8')).encode('utf-8')
            elif member.suffix == '.html':
                def supporting_html_link(match):
                    raw = html.unescape(match[1])
                    if raw.startswith('#') or '://' in raw:
                        return match[0]
                    target = (member.parent / raw.split('#', 1)[0]).resolve()
                    if target in members:
                        return match[0]
                    return 'href="' + REPOSITORY_URL + quote(target.relative_to(ROOT).as_posix()) + '"'
                payload = re.sub(r'href="([^"]+)"', supporting_html_link, payload.decode('utf-8')).encode('utf-8')
            archive.writestr(relative, payload)
            record = {'file': relative, 'sha256': hashlib.sha256(payload).hexdigest()}
            if relative in asset_records:
                asset = asset_records[relative]
                record.update({key: asset[key] for key in ('kind', 'size', 'logical_size', 'scale', 'theme', 'rect') if key in asset})
                if asset.get('source'):
                    record['source'] = (ROOT / 'design/assets' / asset['source']).resolve().relative_to(ROOT).as_posix()
                    record['source_sha256'] = asset['source_sha256']
            ledger.append(record)
        extras = {
            'xdzip-directory-tree.html': standalone(page),
            'README.txt': 'xdzip 目录树 B 设计与素材交付\n打开 xdzip-directory-tree.html 查看全部内嵌图片。\n正式图板与切片保留原像素；组件行、连接线和文字动态绘制。\n规格位于 docs/ux/directory-tree.md 与 docs/visual/directory-tree.json。\n包外辅助 Markdown 链接已转换为仓库链接，正文不变。\nHTML / Qt 目录树改造尚未接入。\n原创内容采用 Apache-2.0，见 LICENSE。\n',
        }
        for name, content in extras.items():
            payload = content.encode('utf-8')
            archive.writestr(name, payload)
            ledger.append({'file': name, 'sha256': hashlib.sha256(payload).hexdigest()})
        archive.writestr('handoff-manifest.json', json.dumps({'status': 'design-ready-awaiting-html-and-qt-integration', 'files': ledger}, ensure_ascii=False, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--standalone', type=Path)
    parser.add_argument('--package', type=Path)
    args = parser.parse_args()
    page, files = generate()
    target = OUTPUT_ROOT / 'directory-tree.html'
    if args.check:
        assert target.read_text() == page, 'Stale directory-tree design delivery'
    else:
        target.write_text(page)
    if args.standalone:
        args.standalone.parent.mkdir(parents=True, exist_ok=True)
        args.standalone.write_text(standalone(page))
    if args.package:
        package(args.package, page, files)
    print(f'Directory-tree handoff: three approved boards, 26 pixel cuts; {len(files)} supporting files')


if __name__ == '__main__':
    main()
