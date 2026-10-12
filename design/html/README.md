# xdzip HTML 高保真设计

[打开审阅入口](index.html) · [源像素测量画布](canvas.html) · [设计目录职责](../README.md) · [已确认图片](../hifi/index.html)

本交付以主窗口、解压确认及操作状态原稿为视觉依据，完整原稿统一位于 `design/hifi/screens/`。主界面采用可编辑 HTML/CSS，操作和状态由示例数据驱动。图标从与 Qt 相同的 `design/assets/svg/` 源生成，照片来自 `design/assets/demo/`；原像素图标切片集中在 `design/assets/reference/`，用于外观对照。

## 如何查看

- 在浏览器打开 `index.html`，选择主窗口、解压确认、新建、任务进度、冲突、设置及异常状态。
- 桌面默认按设计尺寸展示：主窗口宽 1280 CSS 逻辑像素，高度按原稿比例为约 777.26。手机默认适应屏幕；“100%（设计尺寸）”保持这个大小并允许滚动，不改排为手机产品。
- “原稿像素 1:1”用于检查源图细节：主窗口为 1548 × 940，源画布为 1586 × 992。它与正常应用尺寸是两个明确区分的基准，不根据设备像素比自动除以 2。
- “对照原稿”与 HTML 使用相同画布、倍率和滚动范围，禁止原稿另行适应容器。深色主窗口是适配稿，目前没有同版本完整深色原图。
- “独立预览窗口”打开当前画面，并沿用当前倍率、平台与主题。浏览器可能需要允许弹窗；HTML 内的 macOS/Windows/Linux 装饰仍是设计说明，正式应用使用原生 Qt 顶层窗口。
- 文件选择、搜索、目录切换、解压范围、创建与任务恢复均为示例流程，不读取归档或写磁盘。固定进度场景暂停在 60%，速度和剩余时间都是设计示例。

## 离线单文件

执行 `python scripts/build-html-design.py`，生成 `dist/xdzip-hifi-design.html`。文件包含界面、SVG、示例照片、原稿对照图及随包字体，无需下载网络资源；字体的完整 OFL 许可包含在单文件中。目录说明等仓库文档链接不包含在单文件中。

## 源文件职责

| 文件 | 用途 |
| --- | --- |
| index.html、review.css、review.js | 外部审阅工具、等比缩放、原稿对照、独立预览窗口 |
| display-spec.js | 源画布尺寸、原稿映射及设计逻辑尺寸的统一定义；所有预览共用 |
| canvas.html | 桌面主窗口结构 |
| styles.css | 交互控件、状态和平台窗口基础样式 |
| hifi.css | 按当前原稿校准的画布、间距、字号、卡片和窗口样式 |
| header-reference.json、body-reference.json | 从原稿独立测量的顶部与内容区域锚点，不从 CSS 自动生成 |
| app.js | 内存示例数据和操作状态，不含压缩库 |
| assets.js | 自动生成的共享素材，修改 SVG 后重新生成，不手改 |
| screenshots/ | 本次 HTML 实际渲染结果，不是设计原图或 Qt 截图 |

## 视觉覆盖与剩余差异

- 主窗口恢复双标签、正面文件夹图标、工具栏、文件缩略图和图片预览结构；源窗口 1548×940，正常展示 1280×约777.26。
- 解压确认恢复目标、范围和重名策略的分组卡片；任务窗口恢复目标、进度、指标、当前文件和两行提示。
- 新建、设置及异常流程保留可操作的功能逻辑并使用同一素材与窗口规范；尚未逐像素匹配每张操作状态原稿，不能把这些页面称为全部完成视觉验收。
- HTML 采用固定的离线字体，抗锯齿仍因浏览器和平台不同。原稿的原始字体无法证明。大图标的高光、阴影、拉链微字样仍有重建差异。
- 原稿中只有部分输入、状态和数据；真实交互需要显示的搜索反馈、半选复选框、动态计数等遵循功能语义，不强行伪造静态状态。

![HTML 正常设计尺寸主界面](../../docs/qa/html-scale/main-normal.png)

![HTML 解压确认](screenshots/03-extract-confirm.png)

验证运行 `python scripts/check-html-design.py`，报告位于 `dist/html-design-review/validation.json`。它验证示例流程和交付可用性，不代表产品格式、安全或原生窗口验收。

原稿加载专项检查：`python scripts/check-html-references.py`，覆盖普通网页、断网单文件和隔离来源画布，验证六个原稿均可解码、两个无原稿状态正确隐藏图片，以及画布配置不依赖同源 DOM 访问。

预览栏布局专项检查：`python scripts/check-html-preview.py`。覆盖桌面与手机外层缩放、浅深色、经典滚动条空间、照片/PDF/目录/多选/归档属性以及长名称。正常照片预览完整显示操作按钮，无内部横纵滚动；长内容保留纵向滚动，避免截断属性和操作。

顶部复核采用 [独立锚点与同尺寸叠加流程](../../docs/visual/html-fidelity.md)，查看 [原稿／修复后／叠加](../../docs/qa/html-header/index.html)。离线对照文件为 `dist/xdzip-header-review.html`，不是整张图片覆盖在 HTML 界面上。

工具栏下方采用同样流程，查看 [内容区域说明](../../docs/qa/html-body-review.md)、[逐区域对照](../../docs/qa/html-body/index.html) 和 [测量证据](../../docs/qa/html-body/measurements.json)。执行 `python scripts/check-html-body.py`、`python scripts/build-html-body-review.py` 后生成 `dist/xdzip-body-review.html`。同尺寸裁切包括侧栏、列表、预览和状态栏；手机可逐区域检查。几何与隔离图标采用 2 px 容差，文字轮廓只记录，不据此宣称整页像素一致。

原稿选中一项但全选框为空、节省标注 26% 但示意条约填 56% 的两处静态矛盾，HTML 分别保留半选状态和实际 26% 比例。它们是有记录的功能语义例外。本轮正文校准仍属于 HTML 设计交付，不表示 Qt 产品同步完成。
