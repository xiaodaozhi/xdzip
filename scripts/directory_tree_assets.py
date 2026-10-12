"""Source-pixel references for the approved directory-tree component.

Cuts deliberately retain their backgrounds. Rows and guides are specifications
for live components, not bitmaps to ship as controls. Existing folder/disclosure
artwork is reused; the filled status glyphs and hidden-branch marker are distinct.
"""

# source stem, physical rect (x, y, width, height), label, rendering, shared icons
TREE_CUTS = {
    'tree-folder': ('directory-tree', [140, 660, 82, 70], '目录图标', 'shared-svg', ['folder']),
    'tree-disclosure-expanded': ('directory-tree', [93, 681, 30, 28], '展开箭头', 'shared-svg', ['chevron-down']),
    'tree-disclosure-collapsed': ('directory-tree', [194, 842, 31, 39], '折叠箭头', 'shared-svg', ['chevron-right']),
    'tree-guides-nested': ('directory-tree', [100, 812, 126, 165], '多级连接线区域', 'native-geometry', []),
    'tree-current-indicator': ('directory-tree', [60, 557, 18, 91], '当前目录左侧标记', 'native-geometry', []),
    'tree-row-collapsed': ('directory-tree-states', [35, 290, 455, 110], '有子目录 · 已折叠', 'native-component', ['folder', 'chevron-right']),
    'tree-row-expanded': ('directory-tree-states', [540, 291, 452, 245], '有子目录 · 已展开', 'native-component', ['folder', 'chevron-down', 'chevron-right']),
    'tree-row-leaf': ('directory-tree-states', [35, 735, 455, 95], '非空叶子目录', 'native-component', ['folder']),
    'tree-row-empty': ('directory-tree-states', [35, 831, 455, 98], '显式空目录', 'native-component', ['folder']),
    'tree-row-current': ('directory-tree-states', [537, 729, 457, 76], '当前目录', 'native-component', ['folder', 'chevron-down']),
    'tree-row-hover': ('directory-tree-states', [538, 815, 456, 77], '悬停', 'native-component', ['folder']),
    'tree-row-focus': ('directory-tree-states', [538, 899, 456, 76], '键盘焦点', 'native-component', ['folder', 'chevron-right']),
    'tree-name-tooltip': ('directory-tree-states', [125, 1319, 361, 83], '名称与完整路径提示', 'native-geometry', []),
    'tree-deep-hierarchy': ('directory-tree-states', [35, 1171, 455, 265], '深层级与横向滚动', 'native-component', ['folder', 'chevron-down', 'chevron-right']),
    'tree-large-type': ('directory-tree-states', [540, 1170, 452, 156], '大字号', 'native-component', ['folder', 'chevron-down']),
    'tree-loading': ('directory-tree-boundaries', [58, 224, 420, 371], '正在读取完整索引', 'native-component', ['folder', 'tree-spinner']),
    'tree-empty-archive': ('directory-tree-boundaries', [547, 225, 423, 370], '空归档', 'native-component', ['folder']),
    'tree-root-files': ('directory-tree-boundaries', [58, 730, 420, 281], '只有根目录文件', 'native-component', ['folder']),
    'tree-nav-limit': ('directory-tree-boundaries', [547, 730, 423, 281], '目录索引超限', 'native-component', ['folder', 'tree-info']),
    'tree-read-failed': ('directory-tree-boundaries', [58, 1143, 420, 294], '读取失败', 'native-component', ['folder', 'tree-warning']),
    'tree-hidden-current': ('directory-tree-boundaries', [547, 1143, 423, 287], '收起当前目录的祖先', 'native-component', ['folder', 'chevron-right', 'tree-active-branch']),
    'tree-active-branch': ('directory-tree-boundaries', [873, 1293, 24, 24], '隐藏当前目录的分支圆点', 'shared-svg', ['tree-active-branch']),
    'tree-skeleton': ('directory-tree-boundaries', [92, 458, 274, 121], '索引读取骨架', 'native-geometry', []),
    'tree-loading-glyph': ('directory-tree-boundaries', [93, 411, 38, 38], '读取状态图标', 'shared-svg', ['tree-spinner']),
    'tree-info-glyph': ('directory-tree-boundaries', [577, 920, 37, 37], '目录超限提示图标', 'shared-svg', ['tree-info']),
    'tree-error-glyph': ('directory-tree-boundaries', [92, 1340, 40, 40], '读取失败提示图标', 'shared-svg', ['tree-warning']),
}

TREE_ICON_REFERENCES = {'tree-active-branch': 'tree-active-branch', 'tree-spinner': 'tree-loading-glyph',
                        'tree-info': 'tree-info-glyph', 'tree-warning': 'tree-error-glyph'}


def tree_shapes(theme):
    """Keep the approved solid info, amber triangle and segmented spinner.

    These have different silhouettes from the existing outlined status controls.
    Geometry is shared across themes; only neutral/accent colors change. Spinner
    rotation belongs to the live component and respects reduced-motion settings.
    """
    accent, neutral, spinner = (('#1674ff', '#8092b3', '#194e9b') if theme == 'light'
                               else ('#78a6ff', '#92a9ca', '#a6c7ff'))
    spokes = ''.join(
        f'<path d="M12 3v3" transform="rotate({index * 30} 12 12)" '
        f'stroke="{spinner}" stroke-width="2" opacity="{opacity}"/>'
        for index, opacity in enumerate((.5, .75, 1, .8, .6, .35, .25, .22, .2, .22, .28, .4)))
    return {
        'tree-active-branch': f'<circle cx="12" cy="12" r="3" fill="{accent}" stroke="none"/>',
        'tree-info': f'<circle cx="12" cy="12" r="10" fill="{neutral}" stroke="none"/>'
                     '<circle cx="12" cy="6.9" r="1" fill="white" stroke="none"/>'
                     '<path d="M12 10.2v6.9" stroke="white" stroke-width="1.8"/>',
        'tree-warning': '<defs><linearGradient id="tree-warning-fill" x2="0" y2="1">'
                        '<stop stop-color="#ffbd21"/><stop offset="1" stop-color="#f5a300"/></linearGradient></defs>'
                        '<path d="M10.3 3.5a2 2 0 0 1 3.4 0l8 14a2 2 0 0 1-1.7 3H4a2 2 0 0 1-1.7-3Z" '
                        'fill="url(#tree-warning-fill)" stroke="none"/>'
                        '<path d="M12 8.2v5.8" stroke="white" stroke-width="1.8"/>'
                        '<circle cx="12" cy="17.2" r=".9" fill="white" stroke="none"/>',
        'tree-spinner': spokes,
    }


def component_reference_coverage():
    """Give every cut a truthful rendering and integration contract."""
    return [{
        'reference': f'reference/{name}.png', 'component': 'directory-tree',
        'title': title, 'implementation': implementation, 'icon_names': icons,
        'spec': '../../docs/ux/directory-tree.md',
        'geometry_spec': '../../docs/visual/directory-tree.json',
        'integration_status': 'design-ready-awaiting-html-and-qt-integration',
        'note': ('原像素切片保留背景；对应共享矢量用于图标，尺寸由组件规格决定。'
                 if implementation == 'shared-svg' else
                 '原像素切片仅用于对照；文字、状态、连接线和背景由界面动态绘制。'),
    } for name, (_, _, title, implementation, icons) in TREE_CUTS.items()]
