"""Supplemental operation component cuts; runtime controls reuse canonical icons."""
OPERATION_CUTS = {
    'integrity-progress-hero': ('integrity-progress', [138, 144, 93, 112], '完整性检查蓝色文档插图', ['document-check']),
    'integrity-progress-panel': ('integrity-progress', [122, 446, 1202, 267], '统一进度与三列指标卡片', []),
    'result-details-header': ('result-details', [70, 936, 916, 68], '处理明细展开栏', ['chevron-down']),
    'result-details-row-completed': ('result-details', [90, 1005, 876, 102], '已完成文件行', ['image-file', 'status-partial-success']),
    'result-details-row-unfinished': ('result-details', [90, 1206, 876, 102], '未完成文件行', ['file-pdf', 'status-warning']),
    'create-volume-control': ('create-options', [268, 1188, 622, 48], '自定义分卷与单位', ['chevron-down']),
    'create-password-control': ('create-options', [130, 1282, 758, 144], '密码启用与输入', ['check-selected']),
}

def operation_reference_coverage():
    return [dict(reference=f'reference/{name}.png', label=label, implementation='native-component',
                 shared_icons=icons, note='原稿切片只作组件参考；动态文字、布局和交互由控件绘制，状态使用统一扁平素材。')
            for name, (_, _, label, icons) in OPERATION_CUTS.items()]
