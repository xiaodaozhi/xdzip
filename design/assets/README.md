# xdzip 统一素材库

运行图标、原稿参考和演示照片统一维护在本目录；[素材画廊](index.html)按用途展示浅深色、固定尺寸及原稿并排对照。[完整高保真原稿](../hifi/README.md)只保存整幅设计图，不维护另一份运行资源。

| 路径 | 数量 / 规格 | 用途 |
| --- | --- | --- |
| `svg/light`、`svg/dark` | 每主题 74 类；全部 24×24 透明画布 | HTML 和 Qt 共用的界面矢量源 |
| `png/light`、`png/dark` | 每主题 222 张；24/48/72 px | 同一矢量的 1×/2×/3× 透明导出；Qt QRC 使用 3× |
| `png/light/*@9x.png`、`png/dark/*@9x.png` | 每主题 14 张；216 px | 操作窗口大图标；语义清单维护在 `docs/visual/operation-assets.json` |
| `reference` | 71 张 PNG（45 既有图标参考 + 26 目录树 B）；保留源区域物理尺寸 | 原像素参考，保留背景，不作为透明运行图标 |
| `demo` | 3 张照片；保留原始切片分辨率 | HTML 示例缩略图及预览；产品预览服务提供真实图像 |
| `brand` | 26 项；应用与 ZIP 文档母版、平台格式及设计参考 | 系统应用图标、界面品牌和后续安装器的文件图标素材 |
| `preview` | 1 张素材联系表 | 本次生成的审核预览，不进入产品包 |
| [fonts](fonts/README.md) | Noto 派生界面字体与 OFL-1.1 | HTML WOFF2 与同源 Qt TTF；子集外字形使用系统回退 |
| [manifest.json](manifest.json) | 统一资产、45 张图标参考与 26 张组件切片的覆盖台账 | 用途、来源、哈希、画布、倍率、光学边界和实际 Qt 引用；总数以本清单为准 |
| [validation.json（本地／CI 工件）](../../docs/qa/README.md) | 生成一致性与资产检查 | 检查结果和仍需人工 / 原生核对的范围 |

## 目录树 B 设计交付

[专用 HTML 画廊](../hifi/directory-tree.html)与[交互规格](../../docs/ux/directory-tree.md)集中说明 26 张 `reference/tree-*.png` 的来源、逻辑尺寸和动态绘制职责。新增分支圆点、实心信息 / 警告及分段转轮四种素材使用统一 24 单位画布；八张浅深色 SVG 与 24 张透明 PNG 已导出，尚未接入目录树消费者。文件夹与箭头复用既有图形；连接线、行状态与文字由组件绘制。

主清单新增 `component_reference_coverage`，逐项记录共享图标 / 动态组件 / 动态几何及待接入状态；原有 `reference_coverage` 的 45 张图标映射保持独立语义。新增切片全部与确认稿原像素相等，旧十张图板、45 张参考和三张照片保持原字节。

## 一份矢量源

Qt 的稳定资源别名保留在 `apps/desktop/ui.qrc`，实际 PNG 全部从本目录输出。HTML 将同一组 SVG 嵌入 `assets.js`，照片来自 `demo`；缺失共享图标直接报告打包错误，不绘制第二套替代图形。Qt 引用别名写入清单和画廊；其他图标可被设计页面和未来产品组件复用。

图标画布统一为 24×24，物理 PNG 固定 24/48/72 像素；48 像素文件不能直接当作 48 个逻辑像素。PNG 不按墨迹边界重新裁剪，各控件使用统一显示槽，不再按文件类型扭曲宽高或补丁缩放。

文件家族以 `(12,12)` 为中心，PDF、TXT、CODE、Markdown 和图像文件的可见高度统一为 22 单位。天然宽度分别为 PDF 19、TXT 16.5、CODE / Markdown 20、图像 17.6 单位；目录采用 22×约18.86 的横向轮廓，从 17.5×15 等比放大约25.7%，中心和画布不变。HTML 列表统一使用 48×48 原稿坐标槽，正常窗口按同一个比例缩放。不同轮廓不强求相同墨迹宽度。

`app-mark`、`archive` 和 `archive-grey` 分别用于应用品牌、蓝色归档和灰色归档标识；它们采用下述新品牌几何，不属于列表文档家族。操作窗口的插图使用明确的独立语义名称，折角、内侧高光、细边框和渐变在矢量源内定义。校准后的工具栏图标继续使用原几何形状。

## 品牌与平台图标

品牌标记采用四条交错带与开放的 U 形下部，几何统一维护在 `scripts/brand_assets.py`。`brand/mark.svg` 是独立标记，`brand/xdzip.svg` 为应用图标母版，`brand/xdzip-zip.svg` 为 ZIP 文档图标母版。界面的 `app-mark`、`archive`、`archive-grey` 从同一几何定义生成；不在 Qt 或 HTML 中另外手绘。

当前标记使用更修长的 440:660 比例，缩放保留宽高比与光学高度。应用底板和文档徽章共用四角旋转对称、曲率连续的轮廓；表面光照使用完全相同的路径，取消单独描边和开口高光线。`scripts/check-brand-corners.py` 从实际 SVG 检查曲线和轮廓，`--render` 另外检查抗锯齿后的边界位移；测量记录见 [圆角复核（本地／CI 工件）](../../docs/qa/README.md)。

两种母版分别导出 16、24、32、48、64、128、256、512、1024 像素的透明 PNG，共 18 张；512 像素文件命名为 `xdzip.png` 和 `xdzip-zip.png`。16/24/32 像素采用简化的小尺寸表现，保持品牌几何，省去不可读微字和大图光影。Windows 的两份 ICO 各含七种尺寸；macOS 的两份 ICNS 各含十一种表示，包含相应 Retina 密度。所有导出纳入主素材清单，不另建品牌清单。

平台导出留白于 2026-10-16 显式修订：1024 母版的取景框由 `0 0 1024 1024` 改为 `32 32 960 960`，主体高度由 84.375% 增至 90%，单侧留白由约 7.8% 降至 5%。应用底板及 ZIP 纸张、折角、徽章、文字整体等比扩大，不拉伸窄文档去填满正方形。保留透明边缘和原有阴影；不把微弱阴影尾部当成主体边界。16／24／32 像素直接从简化矢量渲染，将约 90% 的主体高度对齐到偶数像素，避免半透明边缘使小图标再缩一圈。ICO、ICNS 和 PNG 共用这些导出，不在打包时二次加边；24 单位界面品牌、主窗口与菜单的密度保持原值。

参考 [Microsoft 图标构建规范](https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-construction)的多尺寸、透明背景和逐尺寸采样，以及 [Apple 应用图标指南](https://developer.apple.com/design/human-interface-guidelines/app-icons)的画布与平台呈现原则。90% 是本项目传统 ICO／ICNS 的光学校准值，不是平台统一强制比例；当前未迁移 Icon Composer 的分层图标。验证以 `alpha >= 128` 的主体边界为准，允许 1.25 物理像素的抗锯齿与取整误差，并验证透明角、每项来源凭据以及全部 ICO／ICNS 表示可解码。修订前后联系表与边界量测只输出到 `build/review/icon-padding/`，原始设计参考不改写。

`brand/identity-reference.png` 是新应用与文档标志的生成式视觉方向，用于审阅，不作为运行图标。此次品牌是明确的新设计修订，不能称为旧高保真 ZIP 拉链图标的无差别还原；十张完整原稿及 48 张原像素切片仍保留原始字节。

应用图标接入 Qt 窗口、macOS app bundle、Windows 可执行资源与 Linux 桌面启动项 / hicolor 目录，详情见 [平台接入说明](../../apps/desktop/packaging/README.md)。ZIP 的 PNG、ICO、ICNS 供后续安装器和文件关联工作使用；当前不注册默认应用或文件关联，不新增双击打开及已运行应用收件行为。

## 参考素材覆盖

主清单的 `reference_coverage` 为全部 45 张原像素参考记录对应语义、浅深色 SVG、用途及说明。旧品牌参考标明历史设计依据，当前品牌指向新母版；控件参考标明实际控件呈现，避免把静态切片称为可操作控件。

补齐主题浅 / 深、已有归档、Windows / Linux 文档、取消大图、选中复选框、成功、选中 / 未选中单选框、冲突 PDF、冲突警告和进度图片十三种语义。取消大图保留方形停止符号；主题系统图标展示完整窗口结构。成功、部分成功、跳过、停止、警告、停止中及系统主题七类既有图标调整渐变、高光或结构，实际表现与仍存在的差异见 [品牌与素材复核（本地／CI 工件）](../../docs/qa/README.md)。

## 原稿与参考的关系

十张页面原稿与三张目录树组件图板位于 `design/hifi/screens`。45 张参考切片和 3 张演示照片保留迁移前的 PNG 文件字节；来源矩形、原稿 SHA-256 和切片 SHA-256 均在清单中。原稿区域的不同大小来自裁剪坐标，不能据此给图标控件设置不同尺寸。

原稿没有可恢复的 alpha 图层，参考切片的白底、淡蓝选中背景和阴影属于原像素。它们供独立对照，不进行全局去白，也不伪称为透明运行图层。只有独立重建的 SVG 与对应透明 PNG 用于运行图标。

照片是生成设计中的示例画面。预览切片只有 351×277 原像素，不是文案中的 6000×4000 源照片；不提供虚假的高清多倍率版本。复选框、单选框、控件背景、文字、进度条与系统窗口按钮由界面 / 平台绘制，参考切片不作为整块控件贴图。

## 生成与检查

在仓库根目录执行：

```sh
python3 scripts/build-brand-assets.py
python3 scripts/build-design-assets.py
python3 scripts/build-design-assets.py --metadata-only
python3 scripts/render-design-assets.py
python3 scripts/check-design-assets.py
python3 scripts/check-asset-integration.py
python3 scripts/build-directory-tree-delivery.py --standalone dist/xdzip-directory-tree.html --package dist/xdzip-directory-tree.zip
# Embed the existing gallery images for offline / mobile review.
python3 scripts/render-design-assets.py --gallery-only --standalone dist/xdzip-assets.html
```

仅更新 QRC 使用映射时可单独执行 `build-design-assets.py --metadata-only`；它核对既有文件的 SHA-256，再刷新清单和画廊，不重新裁图或渲染素材。

`asset_definitions.py` 维护界面形状、参考坐标及固定光学边界，`brand_assets.py` 维护共用品牌几何与平台导出规格。构建工具依赖 Pillow；透明导出和联系表依赖 Playwright / Chromium。使用 `XDZIP_CHROMIUM` 指定浏览器，使用 `XDZIP_FONTCONFIG_FILE` 指定画廊字体配置；这套工具不进入产品运行时。`build-brand-assets.py --check` 只校验品牌母版、透明 PNG、来源凭据及 ICO / ICNS 容器，不写回文件。

`build-design-assets.py --check` 验证定义、SVG、清单和画廊同步，不改写文件。完整检查还核对原稿裁剪像素、全部画布及透明通道，并验证文件家族在浅深色和三个倍率的光学边界与中心。光学边界统一按 `alpha > 8` 测量；像素边界包含抗锯齿尾部与采样取整，采用固定 1.5 物理像素容差并按倍率收紧，中心容差另行验证，不改变矢量目标边界。[折角采样实测（本地／CI 工件）](../../docs/qa/README.md)记录 SVG 几何及 1×/2×/3× 外沿；独立原稿对照的容差保持原值。

画廊是直接展示当前运行素材的复核入口。结构 / 坐标检查不代表与原稿逐像素一致；独立视觉对照、Qt 的真实渲染和目标系统窗口验收分别记录，不能相互替代。

项目原创素材与工具采用 [Apache-2.0](LICENSE)。字体单独按 `fonts/OFL.txt` 的 OFL-1.1 交付；不包含第三方图标集合。

## 扁平任务状态圆盘

完成、部分完成、警告、跳过、停止、取消和校验通过改用纯色圆盘与白色标记；没有渐变、高光、外圈或阴影。色值与几何以 [status-icons.json](../../docs/visual/status-icons.json) 为准。浅深主题与三档透明 PNG 从同一 SVG 定义生成；画廊的原稿列保留历史外观，运行图标使用当前扁平导出。
