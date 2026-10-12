# xdzip 素材切图 v1

基于 [高保真统一入口](../../hifi/README.md)中的 v1 主窗口与解压确认两张原稿交付，所有文件本地可用。打开 [素材总览](index.html) 或 [总览 PNG](preview/asset-contact-sheet.png) 检查。素材是设计交付候选，尚未完成 Qt 实机、主题对比度及缩放验收。

## 内容与精度

| 目录 | 数量 | 含义 |
| --- | --- | --- |
| [原稿](../../hifi/README.md) | 2 张 | 统一存放于 design/hifi/v1/screens；主窗口 1586×992，解压窗口 1092×1440 |
| slices | 24 张 PNG | 原像素矩形裁切，不缩放、不去底；包含图标、照片、控件状态参考 |
| svg/light、svg/dark | 各 28 张 SVG | 根据原稿视觉语言重建的矢量候选；24×24 坐标，1.5 描边 |
| png/light、png/dark | 各 84 张 PNG | 矢量的 1×/2×/3× 透明导出，对应 24/48/72 像素 |
| preview | 1 张 PNG | 素材对照总览，不属于产品运行资源 |

[manifest.json](manifest.json) 记录每项素材的文件名、类型、原稿坐标或设计尺寸，以及 SHA-256。`original-pixel-crop` 为真正的原像素裁切；`vector-reconstruction` 是重新制作，不能宣称无损提取、自动图层还原或逐像素一致。未采用的生成式品牌已删除；标题栏品牌使用 app-mark，原稿对照见 slices/brand-titlebar.png。

## 使用规则

- 原像素切片带有原背景，特别是选中图标的淡蓝底，不能直接用于任意主题。原稿没有 alpha 图层，不做破坏抗锯齿的全局白色抠图。
- UI 图标优先采用 SVG。浅色和深色文件已固定颜色，Qt 加载时无需依赖浏览器 CSS 的 currentColor。SVG 不含字体、脚本、外链和滤镜。
- 多倍率 PNG 的逻辑尺寸均为 24×24；指定目标控件尺寸并选择合适倍率，不能把 48 像素直接当作 48 个逻辑像素。18/20 像素显示应由 SVG 对应渲染，避免低清位图放大。
- PNG 保留完整透明画布，不按不透明边缘随意改变对齐基准。不同轮廓的品牌和压缩文件图标分别使用 app-mark 与 archive。
- 照片切片是生成设计中的演示图。主预览仅有 350×277 原像素，不是原稿文案所示的 6000×4000 原始照片；缩略图不提供伪造高清的 2×/3×版本。正式产品由文件预览服务提供真实图像。
- 文字、按钮、输入框、复选框、单选框、进度条、选中背景和分割线由 Qt Quick 控件/矢量绘制，控件状态切片只供对照，不作为整块按钮贴图。
- macOS 交通灯不切成产品按钮素材，由系统绘制。Windows/Linux 的 close/minimize/maximize 矢量仅是自定义标题栏图形，交互仍须接入系统窗口行为。
- 这套深色 SVG 是素材颜色适配，不等同于整套深色高保真稿已经完成。

## 图标清单

品牌与文件：app-mark、archive、folder、file-pdf、file-text、file-code、file-markdown。

工具栏与导航：open、new、archive-test、search、more、settings、extract、external-open、view-list、view-grid、sort、sort-ascending、chevron-down、chevron-right、chevron-up、arrow-right。

通用状态与窗口：check、completed、close、minimize、maximize。

## 生成与检查

在仓库根目录执行 `python3 scripts/build-design-assets.py`，再执行 `python3 scripts/render-design-assets.py`。前者使用 ImageMagick 进行原像素裁切及 SVG 生成；后者使用 Playwright/Chromium 渲染矢量 PNG 及总览。生成过程不再依赖已删除的品牌候选。

生成器会重写本目录派生资源，修改矢量定义和坐标应同步维护生成脚本。执行 `python3 scripts/check-design-assets.py` 校验原像素一致性、尺寸、透明通道与清单哈希，依赖 Pillow。校验记录见 [validation.json](validation.json)。矢量重建未完成与原稿逐像素匹配验收，浏览器栅格化也不代替 Qt SVG 渲染验收。

项目原创设计与工具按仓库 Apache-2.0 交付；包内附完整 LICENSE。来源为本项目生成设计，不包含第三方图标包或字体。

## 素材形状复核（2026-10-12）

见[逐项对照与问题记录](../../../docs/qa/asset-review.md)。工具栏打开图标已恢复正面圆角文件夹；设置齿轮、列表、排序、文件轮廓等从矢量源修正。标题栏品牌 `app-mark` 与折角压缩文件 `archive` 分离。未采用的生成式品牌母图已删除，历史可从 Git 追溯。

[离线逐项对照](../../../docs/qa/asset-review/index.html) 将原稿切片与当前矢量并排显示。结构检查通过不代表视觉验收通过，未对应独立切片的图标在表中明确标记。
