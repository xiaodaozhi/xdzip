# 单平台截图生成三平台宣传图方案

[English](../../engineering/cross-platform-marketing.md) | 简体中文

一次采集十种语言的真实应用界面，即可生成供官网等渠道使用的各平台风格宣传图。由 macOS 派生的 Windows 和 Ubuntu 成品在清单中标记为合成图。目标平台的功能、原生视觉验收和上架须分别验证。三平台采用 [150% 截图比例](marketing-screenshot-scale.md)，保持实际窗口尺寸并记录原生密度。

## 一次采集

合成入口接受[干净构建的完整 macOS 采集回执](platform-marketing.md)，其中应有完整的源码提交号。使用默认浅色外观、字号和经测量的 200% Retina 原图。主窗口应为 1024 × 622 逻辑像素，创建窗口应为 491 × 736，另带 32 逻辑像素高的 macOS 标题栏。保留完整窗口的 PNG 原图和采集回执。[Windows 原生商店采集流程](../market/windows/capture.md)仍按自己的候选版本与证据要求执行。

起始页使用一个标签，创建窗口使用 ZIP 和受控的演示路径，预览和工作区使用两个归档标签。采集时检查实际产品、语言、窗口范围、场景及画面。逐语言测量起始页的两个快捷键和创建窗口的保存路径，矩形仅覆盖原文字及替换所需留白。按完整 PNG 左上角为原点，在 `build/` 下记录 JSON 区域图，使用原图物理像素 `[x, y, 宽, 高]`。JSON 包含 `schema_version: 1`、`capture-session.json` 的 `source_receipt_sha256` 和以十个语言 ID 为键的 `regions` 对象。每种语言的 `shortcuts` 按打开、创建顺序各列一个矩形，`path` 为保存路径矩形。合成器检查区域边界、背景颜色和留白，发布前还应核对原文及替换后的画面。

## 标题区域参数

以下值为逻辑坐标，转换时统一乘以 1.5 并按最终边界取整；不改产品组件尺寸。

| 窗口 | 已确认的实现依据 | 转换要求 |
| --- | --- | --- |
| Windows 主窗口 | 标题高 48，右侧三个按钮各宽 46，图标 18；标签和品牌由 `ArchiveTabs.qml` 布局。 | 重建完整标题区或保留 Windows 原区，右侧依次为最小化、最大化、关闭。 |
| Ubuntu 主窗口 | 标题高 48，右侧间距 6，三个按钮各宽 36、圆形底直径 28；主窗圆角 10。 | 使用仓库的 `WindowCaptionButton.qml` 和现有浅／深色图标、颜色，重新布局设置按钮与标签。 |
| macOS 主窗口 | 标题高 48；品牌左坐标约 78.057，首个标签约 180.589。原生三按钮在左侧，中心由 `MacTitlebar.mm` 对齐到标题高度中线。 | 重建完整标题区并按原生测量放置按钮，不能仅把红黄绿覆盖在 Windows 品牌上。 |
| 创建等操作窗口 | `OperationWindow.qml` 使用原生窗口装饰，当前请求标题栏与关闭按钮，未默认请求最小化或最大化。 | 独立处理操作窗口，区分窗口边框和客户区，不给不具备的操作伪造可用按钮。 |

导出器按共享视觉参数分别重排品牌、标签及设置按钮，支持保留默认标签宽度的布局。Windows 与 Ubuntu 主窗口按钮使用仓库已有的应用图标与尺寸。创建窗口移除 macOS 窗框，并添加目标平台的本地化标题和关闭按钮。参数见 [cross-platform-frames.json](../../../specs/marketing/cross-platform-frames.json)。Ubuntu 操作窗框以浅色 Yaru 为参考，实际外观随窗口管理器主题变化。 [Yaru 主题](https://github.com/ubuntu/yaru)及 [GTK X11 窗口装饰说明](https://gnome.pages.gitlab.gnome.org/gtk/gtk3/class.Window.html)。

macOS 按钮坐标、尺寸及配色由 AppKit 和系统外观决定。可在 Mac 上编译并运行仓库的独立测量探针，测量无需读取桌面内容。

```sh
mkdir -p build/marketing-probe
clang -fobjc-arc -framework Cocoa scripts/measure-macos-marketing-titlebar.m -o build/marketing-probe/titlebar
build/marketing-probe/titlebar
```

一次 macOS 27.2.1 的本机探针中，三个按钮初始横坐标分别为 9、32、55，均为 14 × 14；操作窗口的后两个按钮不可用。该坐标以客户区为参照，操作窗口按钮位于客户区上方时纵坐标为负。探针记录 AppKit 初始按钮几何，主窗口最终纵向中心还须按 `MacTitlebar.mm` 的实际布局测量；它不证明按钮像素、系统主题或其他系统版本的外观。平台图形资源应依据项目自有控件或经核对的本机系统参考制作，记录采样平台、外观和版本；系统外观变化时重测，不把单次测量硬编码为所有 macOS 版本的原生事实。

## 导出与同步

从仓库根目录执行下列命令，沿用原生素材导出器的[图像渲染工具](platform-marketing.md#准备图片生成环境)，并使用新的输出目录。

```sh
node scripts/build-windows-store-assets.cjs --composite-from-macos build/macos-captures --composition-layout build/regions.json --output build/windows-marketing
node scripts/build-linux-store-assets.cjs --composite-from-macos build/macos-captures --composition-layout build/regions.json --output build/linux-marketing
```

清单以 `cross-platform-composites` 标记导出，按摘要、语言和场景追溯每张原图，并记录区域图、模板及来源回执的摘要。导出时替换窗框、起始页快捷键和演示路径。主窗口成品为 1536 × 933，含目标标题栏的创建窗口为 737 × 1152。融合宣传图使用共用的 1920 × 1080 版式。Mac 原图和回执随采集留存。

宣传图保留对应平台和窗口类型的实际截图轮廓。渲染器按摆放尺寸的轮廓生成阴影，再把处理好的截图图层放在干净底图上。

如果预览或工作区被面包屑悬停提示遮挡，可选用同一产品版本和构建号的另一份已核对 Mac 原图。在区域图中添加 `alternate_capture`，记录备选回执的 `source_receipt_sha256` 和 `scenes` 列表，条目格式为 `{ "language": "en-US", "scene": "04-workspace" }`。两条命令均追加 `--alternate-macos DIR`。每张成品保留所用来源回执与提交的标识。起始页和创建窗口使用主采集。

逐一检查四场景、十语言、登记区域之外的画面及输出摘要。窗框应对照视觉规格及对应平台已有原生参考图，注明参考不足的部分。用 `node scripts/test-platform-store-assets.cjs` 运行合成夹具回归。图片通过核对后，按[最新版同步](marketing-archive.md)原位更新 Drive 的现有平台目录，最后发布清单。实际交付证据记录在 [issue #409](https://github.com/xiaodaozhi/xdzip/issues/409)。
