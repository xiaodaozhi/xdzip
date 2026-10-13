# 主窗口正常密度与菜单比例验收

日期：2026-10-13。当前依据为[严格设计规范](../../visual/design-standard.md)、[设计流程](../../visual/design-workflow.md)与[单位／密度契约](../../visual/layout-scale.json)。本轮将此前 1280 宽主窗口按 125% 基准折算到 100%，执行 0.8 密度校正；菜单保持正常逻辑尺寸。

本页保留密度校正阶段的截图与量测；后续按钮、目录树、日期与无分页浏览修订见[主窗口交互验收](../qt-interaction-fixes.md)。

## 实际结果

| 项目 | 校正前 | 当前 HTML | 当前 Qt |
| --- | --- | --- | --- |
| 主窗口逻辑尺寸 | 1280×约777 | 1024×621.81 | 1024×622 |
| 列表正文 | HTML 15.71 / Qt 16 | 12.57 | 13 |
| 工具栏文字 | HTML 14.88 / Qt 15 | 11.91 | 12 |
| 菜单文字 | 13 | 13 | 13 |
| 菜单宽／正常行高 | 240／32 | 240／32 | 240／32 |
| 菜单／列表字号比例 | HTML 0.827 / Qt 0.813 | 1.034 | 1.000 |
| 侧栏／预览栏 | Qt 241／322 | 193.16／257.98 | 193／258 |
| 列表图标槽 | 约39.69 | 31.75 | 31.75 |
| 目录／文件行 | 约53.75／55.81 | 43.00／44.65 | 43.00／44.65 |

主窗口空间尺寸和字号同步校正，菜单通过源坐标反向换算保持正常尺寸。独立操作窗口保留其既有展示倍率。Qt 直接使用规范生成的逻辑布局和整数 pixelSize，没有给根 Item 加渲染缩放，也没有加入额外的 1.25 产品倍率。

结构边界按累计坐标取整，避免标题区、工具栏、状态栏分别取整后积累误差。同步校正名称列最小宽度，正常窗口不再因旧列宽下限出现水平滚动条；小数行高的图标采用精确中心对齐。原图字节、历史 HTML 基线和素材采样分辨率没有改变。

标题栏显式修订为 48 逻辑像素，胶囊标签为 32 高、16 圆角，上下各留 8；关闭、加号和设置按钮共用垂直中心。正文顶边随标题增高移动，底栏固定；预览属性区间距同步收紧，保证底部操作完整显示。面包屑文字宽度向上取整，避免 Cocoa 上因小数舍入出现多余省略号。历史原图保留，凹肩像素比较不再作为胶囊的准入依据。

## 证据与范围

- [校正前 HTML 下拉菜单](before-html-dropdown.png)来自 `7990e58` 的源文件；[当前 HTML 下拉菜单](html-dropdown.png)、[右键菜单](html-context.png)、[子菜单](html-submenu.png)、[完整主窗口](html-main.png)来自实际浏览器。
- [当前 Qt 下拉菜单](qt-dropdown.png)、[右键菜单](qt-context.png)、[完整主窗口](qt-main.png)、[深色](qt-dark.png)来自 Linux x86_64、Qt 6.8.2、offscreen/software、DPR 1；不作为 macOS 原生窗口装饰证据。
- [浏览器量测](html-measurements.json)：实际 Chromium 页面缩放 100%／125%，搭配基准 DPR 1／1.25／2，144 项检查通过。页面缩放由独立 Xvfb 中的原生 Ctrl+0／Ctrl+= 驱动，不用 CSS zoom 或 pinch zoom 冒充。基准 DPR 通过 Chromium 的 `--force-device-scale-factor` 提供。
- [Qt 量测](qt-measurements.json)：正常密度、下拉菜单与右键菜单的 729 项约束通过；[DPR 1.25](qt-dpr1.25-measurements.json)和[DPR 2](qt-dpr2-measurements.json)各自通过同样 729 项约束。共享组件另有 72 项检查通过。
- HTML 源图顶部 21 项、内容区域 70 项、菜单／组件交互 22 项、展示尺寸 391 个场景通过。数字是约束或场景数量，不是还原率。
- [来源与哈希](provenance.json)保留截图、原稿、规范和环境信息。浏览器页面缩放会进入有效 DPR；截图物理宽度不能直接当作逻辑宽度。例如 Qt DPR 1.25 的图像宽 1280，但客户区逻辑宽仍为 1024。

图片内容预览、多归档会话、安全解压和创建尚未实现。Qt 读取真实 ZIP，因此文件数量、字节数、日期可用性与 HTML 示例不同；保留显式不可用状态。macOS／Windows 原生窗口装饰仍需桌面视觉验收。此前 1280 校准记录保留在[历史验收](../qt-fidelity/README.md)，不再作为当前尺寸准入依据。

## 复验命令与准入

```sh
python scripts/generate-display-spec.py --check
python scripts/generate-control-tokens.py --check
python scripts/generate-qml-theme.py --check
python scripts/build-html-design.py
python scripts/check-html-density.py
python scripts/check-qt-fidelity.py --input build/desktop/visual-audit --output dist/fidelity.json
python scripts/export-qt-fidelity.py
```

浏览器实际页面缩放检查需要 Xvfb、X11 与 Xtst。HTML CI 已执行这组检查；三平台 Qt CI 同时执行正常 DPR 和 1.25／2 DPR 的量测。任何后续主界面、菜单或尺寸规范变更必须执行相应检查并保留实际截图，再按仓库约定推送 main。
