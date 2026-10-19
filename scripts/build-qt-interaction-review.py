#!/usr/bin/env python3
"""Package real Qt captures for the main-window interaction review."""
import argparse
import base64
import hashlib
import html
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, default=ROOT / 'dist/qt-interaction-fixes')
    parser.add_argument('--output', type=Path, default=ROOT / 'dist/xdzip-qt-interaction-review.html')
    parser.add_argument('--archive-evidence', action='store_true')
    args = parser.parse_args()
    data = json.loads((args.input / 'main-integrated-layout.json').read_text())
    environment = '，'.join(str(data[k]) for k in ['os', 'architecture', 'qt', 'platform', 'dpr'])
    scenes = [('main-integrated-layout', '主界面：固定间距、方形控件、真实修改日期'),
              ('review-grid', '网格：隐藏列表表头'),
              ('browserSortMenuButton', '排序菜单：在按钮下方展开'),
              ('browserTasks', '任务菜单：在按钮上方展开'),
              ('navigation-tree', '目录树：无计数、全高竖线、无四边框'),
              ('review-directory-menu', '目录菜单：搜索项上方分隔线')]
    prefix = '''<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>xdzip — Qt 交互修订</title><style>*{box-sizing:border-box}body{margin:24px;background:#f4f6fa;color:#182033;font:15px/1.6 system-ui,sans-serif}h1{font-size:24px}h2{font-size:17px}section{margin:28px 0}img{display:block;max-width:100%;height:auto;border:1px solid #dce2ec;border-radius:12px}.actual img{max-width:none}button{font:inherit;padding:8px 14px;background:white;border:1px solid #ccd3df;border-radius:8px}</style><h1>xdzip — Qt 交互修订</h1>'''
    prefix += '<p>' + html.escape(environment) + '，1024×622 逻辑尺寸</p>'
    prefix += '<p>主界面、菜单和树均来自真实 Qt。离屏截图不包含 macOS 原生红绿灯；红绿灯另由 Cocoa 启动检查测量。解压及图片内容预览仍未实现，禁用状态保留。</p><button onclick="document.body.classList.toggle(\'actual\')">切换适应屏幕／100% 逻辑尺寸</button>'
    embedded, linked, hashes = [], [], {}
    evidence = ROOT / 'build/review/qt-interaction-fixes'
    if args.archive_evidence:
        evidence.mkdir(parents=True, exist_ok=True)
    for stem, label in scenes:
        picture = args.input / (stem + '.png')
        content = picture.read_bytes()
        hashes[picture.name] = hashlib.sha256(content).hexdigest()
        tag = '<section><h2>' + html.escape(label) + '</h2><img width="1024" alt="' + html.escape(label) + '" src="{}"></section>'
        embedded.append(tag.format('data:image/png;base64,' + base64.b64encode(content).decode()))
        linked.append(tag.format(picture.name))
        if args.archive_evidence:
            shutil.copyfile(picture, evidence / picture.name)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(prefix + ''.join(embedded) + '</html>')
    if args.archive_evidence:
        (evidence / 'index.html').write_text(prefix + ''.join(linked) + '</html>')
        sources = sorted([*ROOT.glob('apps/desktop/**/*.qml'), *ROOT.glob('apps/desktop/*.cpp'), *ROOT.glob('apps/desktop/*.mm'), *ROOT.glob('crates/*/src/*.rs')])
        provenance = dict(environment=environment, images=hashes,
                          source_sha256={str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources})
        (evidence / 'provenance.json').write_text(json.dumps(provenance, ensure_ascii=False, indent=2) + '\n')
    print(args.output)


if __name__ == '__main__':
    main()
