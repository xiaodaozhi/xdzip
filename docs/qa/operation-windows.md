# 操作窗口尺寸、footer 与扁平任务状态素材

日期：2026-10-13。尺寸依据 [layout-scale.json](../visual/layout-scale.json)，任务图标依据 [status-icons.json](../visual/status-icons.json)。

解压窗口的专用宽度、字号、控件、图标和卡片参数缩小 20%；通查发现进度窗口也有偏大的专用参数，同步校正。普通创建、设置、冲突等窗口保持现有正常密度。独立审阅宽度为解压约 614、进度约 846、其他普通窗口约 612 逻辑像素；长表单保留滚动和可达的操作按钮。

旧 `.modal-footer > .ready-label` 与 `.modal-footer .button` 把 footer 覆盖成 10／12 源像素。现在所有对话框由生成参数统一使用按钮文字 15、辅助说明 13、图标槽 16 逻辑像素。主画布内交互和独立窗口审阅使用各自的坐标换算，最终字号一致；进度窗口两行提示保持相同图标槽及文字起点。

任务素材包括 `status-success`、`status-partial-success`、`status-warning`、`status-skipped`、`status-stopped`、`cancelled-large`、`success`。删除径向渐变、高光弧线和细边，改为纯色圆盘和白色标记。画布统一为 24×24、可见直径 20；浅深主题各有 SVG 与 24／48／72 透明 PNG。HTML 共用素材、Qt 资源别名、素材清单和画廊一并更新。原稿与带背景参考切片保持原始字节，画廊已明确标为历史对照。

## 实际检查

- `check-html-operation-ui.py`：648 项通过。真实渲染解压、创建、设置、进度、冲突、结果，以及覆盖确认、归档选择、完整性演示对话框；检查三平台示意、浅深主题、DPR 1／1.25／2 的窗口宽度、footer 字号、横向溢出与操作可达性。
- `check-html-density.py`：504 项通过。真实 Chromium 100%／125% 页面缩放、DPR 1／1.25／2，包含新增解压／进度／设置窗口及 footer 测量；没有 CSS zoom 或截图缩小代替浏览器页面缩放。
- `check-flat-status-assets.py`：56 项通过；检查 14 个 SVG 没有渐变、透明高光或滤镜，42 个 PNG 的实际四个圆盘采样点为同一纯色，背景透明且画布正确。
- 素材清单及 74 个 Qt 别名／64 个 HTML 图标名称引用通过；22 组共享交互、20 条完整演示流程、391 项展示尺寸、27 项原稿引用和 17 项预览布局通过。
- 新素材重建后，本地 Qt 5 组 CTest 通过。该项为 Linux software／offscreen 证据，不替代原生系统窗口验收。

[窗口与素材截图](operation-windows/index.html)、[实际窗口测量](operation-windows/dialogs-validation.json)、[真实缩放数据](operation-windows/zoom-validation.json)、[扁平素材检查](operation-windows/status-validation.json)、[输入哈希](operation-windows/provenance.json)。

HTML 操作仍使用示例数据。原稿留存、生产素材验证、浏览器组件验证与原生平台行为各自保留证据边界。
