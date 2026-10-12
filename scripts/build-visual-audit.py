#!/usr/bin/env python3
"""Summarize opt-in Qt geometry captures; failed visual checks remain explicit findings.

This compares logical geometry to the repository tokens, not unrelated screenshot pixels.
Generated design art is a separate, approximate visual reference, never a golden image.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess


ROOT = Path(__file__).resolve().parents[1]


def luminance(color):
    values = [int(color[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    values = [v / 12.92 if v <= .04045 else ((v + .055) / 1.055) ** 2.4 for v in values]
    return sum(v * w for v, w in zip(values, [.2126, .7152, .0722]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, default=ROOT / '.cache/visual-audit')
    parser.add_argument('--output', type=Path, default=ROOT / 'docs/qa/visual-audit')
    parser.add_argument('--fail-on-differences', action='store_true', help='Fail on measured regressions; unresolved fonts remain explicitly unverified')
    parser.add_argument('--require-navigation', action='store_true', help='Require the real tree interaction capture')
    parser.add_argument('--product-commit', help='Revision that produced the captures')
    parser.add_argument('--print-checks', action='store_true', help='Emit ASCII JSON for native CI log review')
    args = parser.parse_args()
    captures = {p.stem: json.loads(p.read_text(encoding='utf-8')) for p in sorted(args.input.glob('*.json'))}
    tokens = json.loads((ROOT / 'docs/visual/tokens.json').read_text(encoding='utf-8'))
    checks = []

    def one(scene, predicate):
        nodes = [n for n in captures[scene]['nodes'] if n['visible'] and predicate(n)]
        if len(nodes) != 1:
            raise ValueError(f'{scene}: expected one node, found {len(nodes)}')
        return nodes[0]

    def label(scene, text):
        return one(scene, lambda n: n['class'].startswith('QQuickText') and n.get('text') == text)

    def check(title, actual, expected, source, tolerance=0):
        if '字号' in title and (actual <= 0 or expected <= 0):
            # QFontInfo may not resolve a font in a platform's offscreen backend. A -1
            # sentinel is missing evidence, not a negative font size or a visual failure.
            checks.append(dict(title=title, actual=None, expected=expected if expected > 0 else None,
                               raw_actual=actual, raw_expected=expected, passed=None, source=source,
                               note='QFontInfo 未解析；需要原生字体后端复核'))
            return
        passed = abs(actual - expected) <= tolerance if isinstance(actual, (float, int)) else actual == expected
        checks.append(dict(title=title, actual=actual, expected=expected, passed=passed,
                           source=source, tolerance=tolerance))

    light = captures['browser-light']
    for field, expected in zip(['width', 'height', 'minimumWidth', 'minimumHeight'],
                               tokens['size']['initialWindow'] + tokens['size']['minimumWindow']):
        check('主窗口 ' + field, light[field], expected, 'tokens.size')
    sidebar = one('browser-light', lambda n: n['name'] == 'archiveSidebar' or (n.get('color') == tokens['light']['sidebar'].lower() and n['height'] > 500))
    inspector = one('browser-light', lambda n: n['name'] == 'archiveInspector' or (n['class'] == 'QQuickRectangle' and n['x'] > 1000 and n['height'] > 500 and n['width'] > 200))
    check('侧栏宽度', sidebar['width'], tokens['size']['sidebar'], 'tokens.size.sidebar')
    check('详情栏宽度', inspector['width'], tokens['size']['inspector'], 'tokens.size.inspector')
    row = one('browser-light', lambda n: n['name'] == 'entry-0-0')
    check('列表行高', row['height'], tokens['size']['row'], 'tokens.size.row')
    button = one('browser-light', lambda n: n['name'] == 'archiveOpen' or (n['class'].startswith('AppButton') and n.get('text') == '打开归档'))
    check('普通按钮高度', button['height'], tokens['size']['button'], 'tokens.size.button')
    search = one('browser-light', lambda n: n['name'] == 'archiveSearch')
    check('搜索框高度', search['height'], tokens['size']['button'], 'visual/README 搜索框高 36')
    check('详情面板标题字号', label('browser-light', '条目详情')['resolvedPixelSize'], tokens['type']['panelTitle'], 'tokens.type.panelTitle')
    name = one('browser-light', lambda n: n['path'].startswith(row['path'] + '/') and n.get('text') == '旅行/')
    check('列表正文字号', name['resolvedPixelSize'], tokens['type']['table'], 'tokens.type.table')
    check('按钮正文字号', label('browser-light', button['text'])['resolvedPixelSize'], tokens['type']['body'], 'tokens.type.body')
    header = label('browser-light', '名称')
    check('名称表头与文件名起点差', name['x'] - header['x'] - header['leftPadding'], 0, 'same column alignment', .5)
    icon = one('browser-light', lambda n: n['path'].startswith(row['path'] + '/') and n['class'] == 'QQuickImage')
    check('列表图标容器网格', icon['width'], 18, 'visual/README 建议 18 px；不等于可见墨迹尺寸')
    dark_inspector = one('browser-dark', lambda n: n['path'] == inspector['path'])
    check('深色详情栏背景', dark_inspector['color'], tokens['dark']['inspector'].lower(), 'tokens.dark.inspector')
    for scene in ['task-light', 'task-dark', 'task-large-type']:
        hints = [one(scene, lambda n, text=text: n['class'].startswith('InfoRow') and n.get('text') == text)
                 for text in ['实际耗时取决于文件与磁盘状态。', '关闭此窗口后，任务仍会继续。']]
        images = [one(scene, lambda n, h=h: n['path'].startswith(h['path'] + '/') and n['class'] == 'QQuickImage') for h in hints]
        labels = [label(scene, h['text']) for h in hints]
        check(scene + ' 提示图标中心 X 差', (images[0]['x'] + images[0]['width'] / 2) - (images[1]['x'] + images[1]['width'] / 2), 0, '两行提示对齐约束', .5)
        check(scene + ' 提示文字起点 X 差', labels[0]['x'] - labels[1]['x'], 0, '两行提示对齐约束', .5)
    headings = [label('task-light', t) for t in ['速度', '预计剩余', '当前阶段']]
    check('任务三列标题 Y 极差', max(n['y'] for n in headings) - min(n['y'] for n in headings), 0, '同排标题顶边对齐', .5)
    values = [label('task-light', t) for t in ['6.2 MB/s', '约 5 秒', '写入文件']]
    baselines = [n['y'] + n['baselineOffset'] for n in values]
    check('任务三列数值基线极差', max(baselines) - min(baselines), 0, '同排数字基线对齐', .5)
    for text in [button['text'], '条目详情']:
        check('200% 字号：' + text, label('browser-large-type', text)['resolvedPixelSize'], label('browser-light', text)['resolvedPixelSize'] * 2, 'Theme.fontScale 传播')
    # Archive names also appear in tabs; select the sidebar by its horizontal region.
    archive_name = lambda scene: one(scene, lambda n: n['class'].startswith('QQuickText') and n.get('text') == 'normal.zip' and n['x'] < 100)
    check('200% 字号：侧栏归档名', archive_name('browser-large-type')['resolvedPixelSize'], archive_name('browser-light')['resolvedPixelSize'] * 2, 'Theme.fontScale 传播')
    search_large = one('browser-large-type', lambda n: n['name'] == 'archiveSearch')
    check('200% 字号：搜索输入', search_large['resolvedPixelSize'], search['resolvedPixelSize'] * 2, 'Theme.fontScale 传播')
    large_header = one('browser-large-type', lambda n: n.get('text') == '原始大小' and n.get('leftPadding') in [12, 14])
    check('200% 表头文字可容纳', large_header['contentWidth'] <= large_header['width'] - large_header['leftPadding'], True, 'contentWidth <= width - leftPadding')
    if any(n['name'] == 'archiveInspector' for n in captures['browser-light']['nodes']):
        for scene in ['browser-light', 'browser-large-type']:
            for column in [1, 2]:
                cell = one(scene, lambda n: n['name'] == f'entry-1-{column}')
                number = one(scene, lambda n: n['path'].startswith(cell['path'] + '/') and n['class'].startswith('QQuickText') and n.get('text') == ('12 B' if column == 1 else '15 B'))
                check(f'{scene} 数字列 {column} 右对齐', number['horizontalAlignment'], 'AlignRight', 'Text.AlignRight')
                check(f'{scene} 数字列 {column} 文字可容纳', number['contentWidth'] <= number['width'], True, 'not elided')
        narrow = captures['browser-request-1040']
        narrow_inspector = next(n for n in narrow['nodes'] if n['name'] == 'archiveInspector')
        check('1040 窄窗口详情栏收起', narrow_inspector['visible'], False, 'inspectorCollapseWidth')
        check('最小窗口宽度可达', narrow['width'], 1040, 'minimumWindow')
        for scene in ['browser-light', 'browser-large-type']:
            opened = one(scene, lambda n: n['name'] == 'archiveOpen')
            searched = one(scene, lambda n: n['name'] == 'archiveSearch')
            check(scene + ' 工具栏控件中心 Y 差', opened['y'] + opened['height'] / 2 - searched['y'] - searched['height'] / 2, 0, 'shared controlHeight', .5)
    if args.require_navigation and 'navigation-tree' not in captures:
        raise ValueError('Missing required navigation-tree capture')
    if 'navigation-tree' in captures:
        scene = 'navigation-tree'
        parent = label(scene, 'photos')
        child = label(scene, '2026')
        sibling = label(scene, 'documents')
        check('目录树父子缩进', child['x'] - parent['x'], 16, 'tree indentation', .5)
        check('目录树同层文字起点差', sibling['x'] - parent['x'], 0, 'same-depth alignment', .5)
        node = one(scene, lambda n: n['name'] == 'directory-d2')
        check('目录树行高', node['height'], tokens['size']['row'], 'tokens.size.row')
        icon = one(scene, lambda n: n['path'].startswith(node['path'] + '/') and n['class'] == 'QQuickImage')
        check('目录树图标网格', icon['width'], 18, '18 px icon container')
        check('目录树图文间距', child['x'] - icon['x'] - icon['width'], 8, 'tree spacing', .5)
        check('目录树文字与行中心 Y 差', child['y'] + child['height'] / 2 - node['y'] - node['height'] / 2, 0, 'row content centered', .5)
        check('目录树图标与行中心 Y 差', icon['y'] + icon['height'] / 2 - node['y'] - node['height'] / 2, 0, 'row content centered', .5)
        tree = one(scene, lambda n: n['name'] == 'directoryTree')
        count = one(scene, lambda n: n['path'].startswith(node['path'] + '/') and n.get('text') == '2')
        check('目录树计数完整显示', count['x'] + count['width'] <= tree['x'] + tree['width'], True, 'count fits tree viewport')
    for theme in ['light', 'dark']:
        for foreground, background in [('text', 'surface'), ('textSecondary', 'surface'), ('accentText', 'accent')]:
            a, b = sorted([luminance(tokens[theme][foreground]), luminance(tokens[theme][background])])
            ratio = (b + .05) / (a + .05)
            checks.append(dict(title=f'{theme} {foreground}/{background} 对比度', actual=round(ratio, 2), expected='>= 4.5', passed=ratio >= 4.5, source='WCAG sRGB token 色；不代表所有实际状态组合'))

    output = args.output
    output.mkdir(parents=True, exist_ok=True)
    files = {}
    for name in captures:
        for suffix in ['.json', '.png']:
            source = args.input / (name + suffix)
            destination = output / source.name
            shutil.copyfile(source, destination)
            files[source.name] = hashlib.sha256(source.read_bytes()).hexdigest()
    inputs = ['apps/desktop/ui/' + p.name for p in (ROOT / 'apps/desktop/ui').glob('*.qml')]
    inputs += ['design/hifi/v1/screens/main-light.png', 'design/hifi/v2/screens/03-progress.png', 'docs/visual/tokens.json']
    environment = {key: light[key] for key in ['os', 'architecture', 'qt', 'platform', 'requestedQuickBackend', 'dpr']}
    report = dict(status='differences_found' if any(c['passed'] is False for c in checks) else 'incomplete' if any(c['passed'] is None for c in checks) else 'measured_checks_pass', environment=environment,
                  product_commit=args.product_commit or subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True, encoding='utf-8').strip(), checks=checks,
                  failed=sum(c['passed'] is False for c in checks), passed=sum(c['passed'] is True for c in checks),
                  unverified=sum(c['passed'] is None for c in checks),
                  evidence_sha256=files, input_sha256={p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in sorted(inputs)})
    # Manually identified raster boundaries, +/- 2 source pixels. Width-only normalization
    # preserves aspect ratio; it is not an instruction to stretch the art to fit the app.
    report['art_reference'] = {
        'source': 'design/hifi/v2/screens/03-progress.png',
        'content_rect_xyxy': [85, 114, 1361, 1036],
        'boundary_uncertainty_source_px': 2,
        'scale_to_760_content_width': 760 / 1276,
        'target_card_xyxy': [123, 279, 1323, 429],
        'progress_card_xyxy': [123, 447, 1323, 712],
        'file_card_xyxy': [123, 730, 1323, 876],
        'footer_divider_y': 943,
        'note': '参考估计；不推导生成式图片的精确字号，也不作为规范 token'
    }
    (output / 'measurements.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    lines = ['| 检查项 | 期望 | 实测 | 结果 |', '| --- | --- | --- | --- |']
    lines += [f"| {c['title']} | {c['expected']} | {c['actual']} | {'待验证' if c['passed'] is None else '通过' if c['passed'] else '差异'} |" for c in checks]
    (output / 'measurements.md').write_text('# 自动采集检查表\n\n原始坐标与来源见 [measurements.json](measurements.json)。几何容差 0.5 逻辑像素；尺寸 token 按精确值比较。以下计数只描述本表，不是视觉还原率。\n\n' + '\n'.join(lines) + '\n', encoding='utf-8')
    print(json.dumps({k: report[k] for k in ['status', 'passed', 'failed', 'unverified']}, ensure_ascii=False))
    if args.print_checks:
        print('VISUAL_AUDIT_CHECKS=' + json.dumps(dict(environment=environment, checks=checks), ensure_ascii=True))

    if args.fail_on_differences and report['failed']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
