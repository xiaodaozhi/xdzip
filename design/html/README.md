本交付遵守[界面规范](../../docs/visual/design-standard.md)与[设计流程](../../docs/visual/design-workflow.md)。主窗口已按 0.8 校正，菜单保持正常密度；最新[比例与缩放复核（本地工件）](../../docs/qa/README.md)为当前验收依据。

# xdzip HTML 高保真设计

[打开审阅入口](index.html)，[组件与交互](components.html)，[源像素测量画布](canvas.html)，[设计目录职责](../README.md)，[已确认图片](../hifi/index.html)

本交付以主窗口、解压确认及操作状态原稿为视觉依据，完整原稿统一位于 `design/hifi/screens/`。主界面采用可编辑 HTML/CSS，操作和状态由示例数据驱动。图标从与 Qt 相同的 `design/assets/svg/` 源生成，照片来自 `design/assets/demo/`；原像素图标切片集中在 `design/assets/reference/`，用于外观对照。

## 如何查看

- 在浏览器打开 `index.html`，选择主窗口、解压确认、新建、任务进度、冲突、设置及异常状态。
- 桌面默认按设计尺寸展示：主窗口宽 1024 CSS 逻辑像素，高度按原稿比例为约 621.81。手机默认适应屏幕；“100%（设计尺寸）”保持这个大小并允许滚动，不改排为手机产品。
- “原稿像素 1:1”用于检查源图细节：主窗口为 1548 × 940，源画布为 1586 × 992。它与正常应用尺寸是两个明确区分的基准，不根据设备像素比自动除以 2。
- “对照原稿”与 HTML 使用相同画布、倍率和滚动范围，禁止原稿另行适应容器。主窗口浅深色对照已同步当前胶囊标签设计导出，冻结量测仍使用历史原图，不以同源导出代替独立核验。
- 内嵌图板是固定画布，根文档不显示滚动条。页面负责上下查看；屏幕窄于选定倍率时，画布外层允许横向平移。文件列表、长属性和操作窗口仍按内容独立滚动。直接打开 `canvas.html` 的非嵌入页面保留浏览器滚动。
- “独立预览窗口”打开当前画面，并沿用当前倍率、平台与主题。浏览器可能需要允许弹窗；HTML 内的 macOS/Windows/Linux 装饰仍是设计说明，正式应用使用原生 Qt 顶层窗口。
- 文件选择、搜索、目录切换、解压范围、创建与任务恢复均为示例流程，不读取压缩包或写磁盘。固定进度场景暂停在 60%，速度和剩余时间都是设计示例。

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
| controls.js、controls.css | 共享菜单、键盘、定位、焦点、提示及按钮状态，不持有压缩包数据 |
| main-controls.js | 主窗口命令、上下文和独立示例会话适配器 |
| components.html、components.js、components.css | 使用共享实现的可操作组件审阅页 |
| controls-tokens.js、controls-tokens.css | 从共享控件及目录树规范生成，不手工维护第二份尺寸 |
| assets.js | 自动生成的共享素材，修改 SVG 后重新生成，不手改 |
| `build/`、`dist/` | 本地和 CI 生成的截图、量测与离线交付，不纳入 Git |

## 视觉覆盖与剩余差异

- 主窗口恢复双标签、正面文件夹图标、工具栏、文件缩略图和图片预览结构；源窗口 1548×940，正常展示 1024×约621.81。
- 解压确认恢复目标、范围和重名策略的分组卡片；任务窗口恢复目标、进度、指标、当前文件和两行提示。
- 新建、设置及异常流程保留可操作的功能逻辑并使用同一素材与窗口规范；尚未逐像素匹配每张操作状态原稿，不能把这些页面称为全部完成视觉验收。
- HTML 采用固定的离线字体，抗锯齿仍因浏览器和平台不同。原稿的原始字体无法证明。大图标的高光、阴影、拉链微字样仍有重建差异。
- 原稿中只有部分输入、状态和数据；真实交互需要显示的搜索反馈、半选复选框、动态计数等遵循功能语义，不强行伪造静态状态。

[HTML 正常设计尺寸主界面（本地工件）](../../docs/qa/README.md)

[HTML 解压确认（本地工件）](../../docs/qa/README.md)

验证运行 `python scripts/check-html-design.py`，报告位于 `dist/html-design-review/validation.json`。它验证示例流程和交付可用性，不代表产品格式、安全或原生窗口验收。

原稿加载专项检查：`python scripts/check-html-references.py`，覆盖普通网页、断网单文件和隔离来源画布，验证六个原稿均可解码、两个无原稿状态正确隐藏图片，以及画布配置不依赖同源 DOM 访问。

预览栏布局专项检查：`python scripts/check-html-preview.py`。启用真实经典滚动条，覆盖桌面与手机外层缩放、浅深色、自动与预留滚动条空间、照片/PDF/目录/多选/压缩包属性以及长名称。正常照片预览完整显示操作按钮，无内部横纵滚动；长内容须确实可以向下滚动，且操作按钮可达。

画布滚动专项检查并入 `python scripts/check-html-scale.py`：关闭 Playwright 默认的 `--hide-scrollbars`，检查根文档无滚动条占位、主窗口四边完整可见、手机横向平移与原稿同步，并加入 CSS 缩放和画布尺寸减少 1 px 的容错用例。移除嵌入规则的负向用例须重现经典滚动条占位，防止截图环境再次掩盖问题。查看[画布滚动复核（本地工件）](../../docs/qa/README.md)。

顶部复核采用 [独立锚点与同尺寸叠加流程](../../docs/visual/html-fidelity.md)，查看 [原稿／修复后／叠加（本地工件）](../../docs/qa/README.md)。离线对照文件为 `dist/xdzip-header-review.html`，不是整张图片覆盖在 HTML 界面上。

工具栏下方采用同样流程，查看 [内容区域说明（本地工件）](../../docs/qa/README.md)、[逐区域对照（本地工件）](../../docs/qa/README.md) 和 [测量证据（本地工件）](../../docs/qa/README.md)。执行 `python scripts/check-html-body.py`、`python scripts/build-html-body-review.py` 后生成 `dist/xdzip-body-review.html`。同尺寸裁切包括侧栏、列表、预览和状态栏；手机可逐区域检查。几何与隔离图标采用 2 px 容差，文字轮廓只记录，不据此宣称整页像素一致。

原稿选中一项但全选框为空、节省标注 26% 但示意条约填 56% 的两处静态矛盾，HTML 分别保留半选状态和实际 26% 比例。它们是有记录的功能语义例外。本轮正文校准仍属于 HTML 设计交付，不表示 Qt 产品同步完成。

主窗口[组件与交互契约](../../docs/ux/main-window-components.md)同时覆盖外观与行为。多压缩包示例、真实网格、各字段排序、递归目录、条目 / 空白 / 表头 / 标签右键、解压范围、更多 / 视图子菜单、快捷键、提示和焦点恢复已接入。按钮使用普通、悬停、按下、焦点、禁用及忙碌状态；完整性检查明确为演示任务，没有执行 CRC。压缩 / 解压仍使用示例进度。

运行 `python3 scripts/check-html-controls.py` 验证主窗口交互并生成浅深色、三种平台外观和手机组件截图。单文件 `dist/xdzip-components.html` 可离线打开，[复核文档（本地工件）](../../docs/qa/README.md)区分 HTML 演示与 Qt 只读功能。

## HTML／Qt 交互同步

当前主窗口已同步固定方形查看按钮、6 图文间距、28 高面包屑按钮、4 菜单间隔、向上展开的任务菜单及无计数目录树。类型／日期排序、当前目录全量展示、网格隐藏表头继续沿用既有交互。尺寸由 `layout-scale.json` 生成，原稿历史量测保留。逐项状态和缩放实测见[同步验收（本地工件）](../../docs/qa/README.md)。

操作窗口已校正解压／进度的偏大参数，并统一 footer 的 13／12 逻辑字号与 16 图标槽。任务状态图片从统一素材库读取扁平圆盘。参见[最新窗口与素材验收（本地工件）](../../docs/qa/README.md)。

## W01 欢迎页

任务入口图标按 [layout-scale.json](../../docs/visual/layout-scale.json) 的 16×16 逻辑像素量测。正文原稿检查保留历史图标墨迹与容差作为对照，并独立检查起始页和压缩文件页的实际图标尺寸、垂直中心、素材身份及按钮内位置，覆盖浅深色、三档字号与 DPR 1／1.25／2。不能把图标缩小后的合理墨迹差异当作回归，也不能仅跳过旧图标检查而缺少替代量测。

English: The body checker retains the archived task-icon ink and tolerance for review, and separately measures the shared 16×16 logical icon on both welcome and archive pages. It checks size, vertical center, asset identity and placement across both themes, all three text sizes and DPR 1/1.25/2; the obsolete ink comparison is replaced by strict live geometry checks.

[直接审阅欢迎页](index.html?view=welcome&zoom=1)，也可在画面菜单选择“欢迎页，最小窗口”。已接入浅深色、打开示例、新建、取消后焦点恢复、关闭最后标签返回、快捷键和拖入检查；拖入不读取文件内容，而是进入示例选择器。正式 Qt 已接入真实打开与新建，桌面人工验收另行登记。

welcome.css 承担欢迎页样式，逻辑参数来自 [welcome-reference.json](../../docs/visual/welcome-reference.json)。欢迎页客户区沿用主窗口 1024×约622 或最小 832×544，标题栏复用同一 DOM、样式与历史源坐标换算，不增加欢迎页专属标题栏。浅色和深色对照分别显示对应原稿，最小窗口深色没有对应位图时明确隐藏对照。离线单文件包含相同页面和原稿。`python scripts/build-html-design.py` 另生成默认展示欢迎页的 `dist/xdzip-welcome-design.html`。

运行 `python scripts/check-html-welcome.py`，验证布局、DPR 1／1.25／2、打开和新建、取消与焦点、最后标签关闭、拖入拒绝、菜单比例和断网离线流程。截图与报告只保存在 build/review/html-welcome/。

加载取消回归在页面加载前安装 Playwright 时钟，在选择示例前暂停计时，再通过真实按钮点击取消；取消后推进 650 毫秒，超过示例的 500 毫秒完成期限，核对遮罩移除、正文恢复交互、焦点返回及没有迟到的页面跳转。切换审阅视图也须推进时钟核对旧回调失效，正常打开流程恢复自然计时。不能通过增加示例延时、强制点击或跳过取消步骤消除竞争。

English: Cancellation checks use the Playwright clock to keep the 500 ms demo completion from racing a real button click. Advancing 650 ms after cancellation or a review-preset change verifies stale callbacks are suppressed. The normal opening flow resumes real time; no longer demo delay or forced click is used.

[系统菜单与托盘交互稿](desktop-integration.html)展示 macOS 顶部菜单与三平台托盘、驻留及退出状态；原生尺寸由系统决定。离线导出同步生成 `dist/xdzip-desktop-integration.html`。

退出与重启确认可从审阅入口选择 quit/restart，或打开[独立状态稿](../hifi/sources/confirmations.html)。共享确认布局与 Qt 同源参数，覆盖停止禁用、长文案滚动及英文大字号按钮内换行。执行 `scripts/check-html-confirmations.py` 验证实际布局；浏览器图不代替原生装饰验收。

密码解压从审阅入口选择 password，支持 `?view=password&retry=1` 或 `pending=1` 检查重试与提交中状态，[独立 P02 状态稿](../hifi/sources/password-extraction.html)为冻结设计依据。空密码可提交，提交与取消会清空字段，演示不执行解密、不保留密码。执行 `scripts/check-html-password.py` 验证中英文、浅深色、多字号、字节边界与固定 footer。
