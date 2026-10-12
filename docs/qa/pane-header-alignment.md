# 分栏标题对齐

日期：2026-10-12。文件列表面包屑与右侧预览／详情标题采用相同的栏高、文字大小和垂直中心线。

HTML 共用 69 px 栏高、20 px 字号、28 px 行高和相同的文字光学偏移，数值使用原稿画布坐标。预览标题固定在内容上方；文件名和属性滚动时仍与面包屑对齐。Qt 共用 [PaneHeader.qml](../../apps/desktop/ui/PaneHeader.qml) 和 `Theme.bodySize`：常规栏高 68、字号 14；两倍字体栏高 89、字号 28。两种实现各自使用统一参数，避免分别修改后产生偏差。

[HTML 实际截图](pane-header-alignment/html-main.png) · [标题局部](pane-header-alignment/html-band.png) · [HTML 量测](pane-header-alignment/html.json) · [Qt 量测](pane-header-alignment/qt.json) · [Qt 常规窗口](pane-header-alignment/browser-1280.png) · [Qt 大字号窗口](pane-header-alignment/browser-large-type.png)

HTML 浅深色、照片／文档／未选择状态和长属性滚动共七个实际场景通过：两栏顶边、栏高、字号、行高与文字中心均相同。Qt 浅深色、1280 窗口及两倍字体的四张真实窗口记录中，栏高、文字中心、字号和文字基线差均为零。Qt 五项 CTest、HTML 顶部 27 项、正文 70 项及预览 17 项检查通过。

原生 CI 的既有视觉验收同步采用 `tokens.type.body` 检查详情标题，文件名继续使用 `panelTitle`。浅色、深色及两倍字体均检查两栏顶边、栏高、文字中心、基线和字号属性；字体后端无法解析实际字号时保留“待验证”标记。首轮 CI 中的旧 `panelTitle` 断言已据此修正，未放宽几何容差。

原稿与原像素切片保持原字节。正文基准保留原关闭按钮中心和墨迹，另记录分栏对齐设计修订：关闭按钮中心从 y206 改为 y201.5；预览照片仍从 y241 开始。正文 70 项包含 63 个原稿检查和七个明确的修订检查，不把新标题样式描述成原稿的逐像素还原。Qt 证据来自 Linux 软件渲染，原生系统显示按目标平台验收。
