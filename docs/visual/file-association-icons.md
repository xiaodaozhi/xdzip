# 文件关联图标

系统文件图标沿用 xdzip 的折角文档、连续曲率徽章、四条交错带与开放 U 形标记。设计入口为[高保真审阅页](../../design/hifi/sources/file-association-icons.html)，参数来源为 [file-association-icons.json](file-association-icons.json)，文档图标源和产品所需平台容器统一位于 [file-associations](../../design/assets/file-associations)。默认应用仍由用户选择。

## 形状与识别

纸张与品牌几何复用 [brand_assets.py](../../scripts/brand_assets.py)。格式之间保持相同的纸张、折角、徽章位置和光学中心，通过标签识别格式，辅助色帮助扫视。TAR.GZ、TGZ、GZ 属于同一视觉家族，使用青绿色。通用后备采用中性灰蓝和文档横线，不虚构 ARC 等未支持格式。

标签严格沿用原 ZIP 的字形风格：高度 61、笔画 14、方形端点、圆角连接，顶边位于母版 Y=817，颜色为 #39465D。ZIP 直接复用原母版的完整文字路径，其他字母在相同度量与笔画规则下补齐；不使用 SVG text、系统字体、第三方图标或嵌入位图。最大宽度 504，保留完整的 TAR.GZ，不拉伸字形，只有超过宽度预算才等比缩小并垂直居中。颜色不承担唯一识别职责。

母版为 1024 单位。通用 PNG／ICO 沿用既有 `32 32 960 960` 取景框，纸张主体高度约 90%；macOS PNG／ICNS 沿用既有 `-20 -20 1064 1064` 取景框，主体高度约 81.2%。这是项目现有光学校准的延续，不宣称为系统强制标准。图标使用透明画布，不按桌面主题替换纸张，不带桌面底色或额外圆角遮罩。

16、24 像素采用放大的徽章和简化纸张，32 像素采用稍小的徽章；这三档省去标签与大图阴影，保留品牌标记的原始宽高比。此时无法仅靠图标可靠地区分全部格式，尤其是同色的 GZIP 家族，文件名／类型列承担精确识别。48 像素起使用带标签母版，长标签在 64 像素以上更易读。各尺寸直接从矢量渲染，不把 1024 PNG 连续缩小。

界面中的 24 单位图标、主窗口和菜单密度继续遵循 [layout-scale.json](layout-scale.json)。本素材不新增 QML 窗口，也不改变主窗口或菜单的逻辑尺寸。平台 PNG 尺寸是图像采样尺寸，不能写入 QML 组件逻辑几何。

## 格式与平台映射

| 图标标识 | 标签 | 后缀／用途 | 辅助色 |
| --- | --- | --- | --- |
| `zip` | ZIP | `.zip` | 品牌蓝 |
| `7z` | 7Z | `.7z` | 靛蓝 |
| `rar` | RAR | `.rar` | 紫色 |
| `tar` | TAR | `.tar` | 琥珀色 |
| `tar-gz` | TAR.GZ | 已确认的 `.tar.gz` 复合后缀 | 青绿 |
| `tgz` | TGZ | `.tgz` | 青绿 |
| `gz` | GZ | `.gz` 系统候选 | 青绿 |
| `cab` | CAB | `.cab` | 钢蓝 |
| `iso` | ISO | `.iso` | 湖蓝 |
| `udf` | UDF | `.udf` | 青蓝 |
| `archive` | 无 | 已识别归档无专属图标时的后备 | 中性灰蓝 |

此表是设计意图，不是跨平台统一注册表。文件名映射在可用时优先匹配完整 `.tar.gz`，再匹配 `.gz`；按最终扩展名识别的系统可能只能为两者显示 GZ 图标。不要承诺所有平台都能自动区别复合后缀。GZ 图标只表示扩展名，项目仍只支持其中的 TAR.GZ 内容，不扩大为独立 GZIP 解压能力。

加密与分卷是归档状态，不由扩展名图标猜测，不绘制锁或成功标记。已识别 ZIP、7z、RAR 分卷可在应用内复用对应家族，不注册通用 `.001`、`.rNN`、`.zNN` 或 `.img`。ISO／UDF 图标不表示挂载或执行能力。不能仅因后缀匹配就声明归档内容有效。

Windows 使用逐扩展名 ProgID 和 ICO，保留旧共享入口；macOS 以文档类型映射 ICNS，压缩 TAR 类型共用 TAR.GZ 图标。Linux 保留系统 MIME 图标，不覆盖全局文件图标。原生显示受默认应用和图标缓存影响，关联与回退见[安装与关联契约](../engineering/installers-associations.md)。

Windows 默认应用与“打开方式”里的应用入口使用 [品牌应用 ICO](../../design/assets/brand/xdzip.ico)，主程序和关联启动器均嵌入此资源；不使用逐格式文档图标代替应用身份。系统应只展示一个名称为 xdzip、带品牌图标的可选入口，旧共享命令仅用于兼容既有选择。该调整复用已有素材，不改变主窗口、菜单或 QML 逻辑几何。原生应用图标、文档图标及候选数量分别验收，不能将操作系统的通用应用方框判作格式文档图标问题。

## 生成与交付

在仓库根目录执行：

```sh
python scripts/build-association-icons.py
python scripts/build-design-assets.py --metadata-only
python scripts/build-association-icons.py --check --check-runtime
# 导出到本地工件，使用已有 Pillow、Playwright 环境与 Chromium。
python scripts/build-association-icons.py --export build/file-association-icons
python scripts/build-association-icons.py --check --export build/file-association-icons
```

`XDZIP_CHROMIUM` 可指定已有 Chromium／Edge 路径。SVG 母版、小尺寸源、规范和高保真审阅页纳入版本管理；素材源在主 [manifest.json](../../design/assets/manifest.json) 中登记。生成入口是 [build-association-icons.py](../../scripts/build-association-icons.py)，组合定义见 [association_icon_assets.py](../../scripts/association_icon_assets.py)。

导出工件每格式包含 16 至 1024 像素的通用与 macOS PNG，以及 ICO 和 ICNS。产品使用的九份 ICO、九份 ICNS 与矢量源同放 `file-associations/`；ZIP 容器保留迁移前原字节，其他容器来自相应导出。普通构建直接读取这些容器，不依赖浏览器。临时 PNG、完整导出、联系表和回执保留在 `build/` 或 `dist/`。

## 验证与验收

生成检查验证 SVG 与规格一致、无字体／外部图像依赖。平台检查包括画布与 RGBA、透明角、可见主体高度和居中、矢量摘要以及每种 ICO／ICNS 表示解码后的像素一致性。主体边界按 alpha ≥ 128 测量，容差沿用品牌的 1.25 物理像素。

视觉审阅同时覆盖浅色、深色与选中背景，16 至 256 像素，以及普通和高 DPR 的实际渲染。审阅页按 CSS 像素呈现，不将浏览器截图标记为 Explorer、Finder 或 Linux 原生验收。后续接入须检查真实文件名、复合后缀、图标缓存、默认选择前后和安装／卸载后的显示。阶段状态和实际执行范围在 GitHub issue 中记录，不在本规范累积测试报告。

English: Windows uses per-extension ICOs and macOS uses per-type ICNS; Linux retains system MIME icons. Windows main and association executables embed the existing brand application ICO, with one visible xdzip application candidate; application identity and document icons require separate native checks. ZIP's original platform containers are preserved. Source vectors and required platform containers share one directory, and ordinary builds require no browser. The 16/24/32 px profiles omit lettering; GZ does not imply standalone GZIP support. Compound suffix recognition is platform dependent, and no generic volume or IMG association is introduced. Browser rendering and container validation do not replace native shell acceptance.
