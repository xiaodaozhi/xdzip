#!/usr/bin/env python3
"""Publish actual normalized UI captures at their logical dimensions."""
import argparse
import base64
import html
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
QA = ROOT / 'docs/qa/ui-density'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'dist/xdzip-qt-fidelity.html')
    parser.add_argument('--qt-captures', type=Path, help='Actual current CI capture directory')
    args = parser.parse_args()
    platform = 'Linux x86_64，Qt 6.8.2，offscreen/software，非原生 macOS 窗口'
    if args.qt_captures:
        metadata = json.loads((args.qt_captures / 'main-light.json').read_text())
        if metadata['width'] != 1024 or metadata['height'] != 622:
            raise SystemExit('Capture uses an obsolete logical size')
        platform = '，'.join(str(metadata[key]) for key in ['os', 'architecture', 'qt', 'platform'])
    scenes = [
        ('校正前 HTML，1280 逻辑像素', QA / 'before-html-dropdown.png', 1280),
        ('当前 HTML，胶囊标签和正常菜单', QA / 'html-dropdown.png', 1024),
        ('当前 Qt，' + platform, args.qt_captures / 'main-dropdown.png' if args.qt_captures else QA / 'qt-dropdown.png', 1024),
        ('HTML 主窗口', QA / 'html-main.png', 1024),
        ('Qt 主窗口，' + platform, args.qt_captures / 'main-integrated-layout.png' if args.qt_captures else QA / 'qt-main.png', 1024),
        ('HTML 右键菜单', QA / 'html-context.png', 1024),
        ('Qt 右键菜单', args.qt_captures / 'main-context.png' if args.qt_captures else QA / 'qt-context.png', 1024),
        ('HTML 子菜单', QA / 'html-submenu.png', 1024),
        ('Qt 深色', args.qt_captures / 'main-dark.png' if args.qt_captures else QA / 'qt-dark.png', 1024),
    ]
    cards = []
    for title, path, width in scenes:
        data = base64.b64encode(path.read_bytes()).decode()
        cards.append(f'<section><h2>{html.escape(title)}</h2><img alt="{html.escape(title)}" width="{width}" src="data:image/png;base64,{data}"></section>')
    document = '''<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>xdzip — 正常密度与悬浮胶囊验收</title><style>
*{box-sizing:border-box}body{margin:0;padding:24px;background:#f2f4f8;color:#182033;font:15px/1.6 system-ui,sans-serif}header{max-width:1024px}h1{font-size:24px}h2{font-size:16px}section{margin:28px 0}img{display:block;height:auto;max-width:100%;border:1px solid #dce2ed;border-radius:12px}body.actual img{max-width:none}button{font:inherit;padding:8px 14px;border:1px solid #ccd3e0;background:white;border-radius:8px;cursor:pointer}
</style><header><h1>xdzip — 正常密度与悬浮胶囊验收</h1>
<p>主窗口 1024×622；标题栏 48，胶囊 32，上下各 8。列表字号 Qt 13 / HTML 12.57；菜单字号 13、宽 240、行高 32。截图按逻辑尺寸显示，手机默认适应屏幕。</p>
<p>Qt 读取真实 ZIP；图片预览、创建和安全解压尚未实现，禁用状态及数据差异如实保留。集成标题截图验证布局，不代表操作系统原生窗口装饰。原稿的底部吸附 Tab 已由悬浮胶囊修订替代。</p>
<button onclick="document.body.classList.toggle('actual');this.textContent=document.body.classList.contains('actual')?'切换适应屏幕':'切换 100% 逻辑尺寸'">切换 100% 逻辑尺寸</button></header>'''+''.join(cards)+'</html>'
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(document, encoding='utf-8')
    print(args.output)


if __name__ == '__main__':
    main()
