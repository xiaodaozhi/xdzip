# HTML 顶部修正复核

日期：2026-10-12。范围是 HTML 主窗口标题栏、Tab、关闭／新增／设置按钮，以及工具栏图标和搜索／解压边界。

已按 [原稿独立测量](../../design/html/header-reference.json) 重建顶部槽位布局。字体随 HTML 离线提供；名称宽度不再取决于设备是否安装 Noto 或系统使用什么回退字体。图标继续使用共享 SVG 源，没有以整张原稿覆盖 HTML。

- [可切换的同尺寸对照](html-header/index.html)：原稿／修复前／修复后／透明叠加／差异。
- [原稿与修复后并排证据](html-header/comparison.png)：上为原稿，下为 HTML。
- [测量结果](html-header/measurements.json)：独立锚点、实际 DOM 和实际图形墨迹。
- [字体来源及许可](../../design/assets/fonts/README.md)：官方 Noto Sans SC 衍生子集，OFL-1.1。
- [后续复核方法](../visual/html-fidelity.md)：固定条件、容差与自动化工作流。

顶部边界／中心和图标墨迹检查已通过。预览布局、原稿加载和示例流程检查结果分别记录于 [预览报告](html-preview-validation.json)、[原稿报告](html-reference-validation.json)、[交互报告](html-design-validation.json)。这些不代表 Qt 原生渲染或压缩解压验收。

仍有生成原稿的柔化阴影、拉链细节和文字轮廓差异。原稿字体不可证明，固定字体减少新的跨平台漂移，但不宣称是同一个原始字体。新建、设置等其他画面的逐项视觉校准仍未全部完成。

云端验证：[HTML design review 38205107742](https://github.com/xiaodaozhi/xdzip/actions/runs/38205107742) 已通过，验证提交 `f27d32a84fba485154fa1bdcd09d49cde18edcf2`。在干净 Ubuntu 环境中安装固定工具与 Chromium，完成字体检查、离线构建、原稿锚点与实际图形检查、预览布局、原稿加载和完整示例流程，并上传 `xdzip-html-design-evidence`。Qt 原生产品不属于本次 HTML 验证范围。
