# xdzip 统一素材库

运行图标、原稿参考和演示照片统一维护在本目录；[素材画廊](index.html)按用途展示浅深色、固定尺寸及原稿并排对照。[完整高保真原稿](../hifi/README.md)只保存整幅设计图，不维护另一份运行资源。

| 路径 | 数量 / 规格 | 用途 |
| --- | --- | --- |
| `svg/light`、`svg/dark` | 每主题 54 类；全部 24×24 透明画布 | HTML 和 Qt 共用的矢量源 |
| `png/light`、`png/dark` | 每主题 162 张；24/48/72 px | 同一矢量的 1×/2×/3× 透明导出；Qt QRC 使用 3× |
| `reference` | 45 张 PNG；保留源区域物理尺寸 | 原像素参考，保留背景，不作为透明运行图标 |
| `demo` | 3 张照片；保留原始切片分辨率 | HTML 示例缩略图及预览；产品预览服务提供真实图像 |
| `preview` | 1 张素材联系表 | 本次生成的审核预览，不进入产品包 |
| [fonts](fonts/README.md) | Noto 派生设计字体与 OFL-1.1 | HTML 设计的稳定排版，不进入 Qt 产品字体 |
| [manifest.json](manifest.json) | 480 项资产 | 用途、来源、哈希、画布、倍率、光学边界和实际 Qt 引用 |
| [validation.json](validation.json) | 生成一致性与资产检查 | 检查结果和仍需人工 / 原生核对的范围 |

## 一份矢量源

Qt 的稳定资源别名保留在 `apps/desktop/ui.qrc`，实际 PNG 全部从本目录输出。HTML 将同一组 SVG 嵌入 `assets.js`，照片来自 `demo`；缺失共享图标直接报告打包错误，不绘制第二套替代图形。Qt 引用别名写入清单和画廊；其他图标可被设计页面和未来产品组件复用。

图标画布统一为 24×24，物理 PNG 固定 24/48/72 像素；48 像素文件不能直接当作 48 个逻辑像素。PNG 不按墨迹边界重新裁剪，各控件使用统一显示槽，不再按文件类型扭曲宽高或补丁缩放。

文件家族以 `(12,12)` 为中心，PDF、TXT、CODE、Markdown 和图像文件的可见高度统一为 22 单位。天然宽度分别为 PDF 19、TXT 16.5、CODE / Markdown 20、图像 17.6 单位；目录采用 22×约18.86 的横向轮廓，从 17.5×15 等比放大约25.7%，中心和画布不变。HTML 列表统一使用 48×48 原稿坐标槽，正常窗口按同一个比例缩放。不同轮廓不强求相同墨迹宽度。

`app-mark` 和 `archive` 是标题栏品牌 / 归档标识，保留独立的 20 单位视觉高度；它们不属于列表文档家族。操作窗口的插图使用明确的独立语义名称，折角、内侧高光、细边框和渐变在矢量源内定义。校准后的工具栏图标继续使用原几何形状。

## 原稿与参考的关系

所有 10 张整幅原稿位于 `design/hifi/screens`。45 张参考切片和 3 张演示照片保留迁移前的 PNG 文件字节；来源矩形、原稿 SHA-256 和切片 SHA-256 均在清单中。原稿区域的不同大小来自裁剪坐标，不能据此给图标控件设置不同尺寸。

原稿没有可恢复的 alpha 图层，参考切片的白底、淡蓝选中背景和阴影属于原像素。它们供独立对照，不进行全局去白，也不伪称为透明运行图层。只有独立重建的 SVG 与对应透明 PNG 用于运行图标。

照片是生成设计中的示例画面。预览切片只有 351×277 原像素，不是文案中的 6000×4000 源照片；不提供虚假的高清多倍率版本。复选框、单选框、控件背景、文字、进度条与系统窗口按钮由界面 / 平台绘制，参考切片不作为整块控件贴图。

## 生成与检查

在仓库根目录执行：

```sh
python3 scripts/build-design-assets.py
python3 scripts/render-design-assets.py
python3 scripts/check-design-assets.py
python3 scripts/check-asset-integration.py
# Embed the existing gallery images for offline / mobile review.
python3 scripts/render-design-assets.py --gallery-only --standalone dist/xdzip-assets.html
```

`asset_definitions.py` 是所有矢量形状、参考坐标、固定光学边界的唯一维护源。构建工具依赖 Pillow；透明导出和联系表依赖 Playwright / Chromium。使用 `XDZIP_CHROMIUM` 指定浏览器，使用 `XDZIP_FONTCONFIG_FILE` 指定画廊字体配置；这套工具不进入产品运行时。

`build-design-assets.py --check` 验证定义、SVG、清单和画廊同步，不改写文件。完整检查还核对原稿裁剪像素、全部画布及透明通道，并验证文件家族在浅深色和三个倍率的光学边界与中心。光学边界统一按 `alpha > 8` 测量；像素边界包含抗锯齿尾部与采样取整，采用固定 1.5 物理像素容差并按倍率收紧，中心容差另行验证，不改变矢量目标边界。[折角采样实测](../../docs/qa/asset-review/image-antialias-evidence.json)记录 SVG 几何及 1×/2×/3× 外沿；独立原稿对照的容差保持原值。

画廊是直接展示当前运行素材的复核入口。结构 / 坐标检查不代表与原稿逐像素一致；独立视觉对照、Qt 的真实渲染和目标系统窗口验收分别记录，不能相互替代。

项目原创素材与工具采用 [Apache-2.0](LICENSE)。字体单独按 `fonts/OFL.txt` 的 OFL-1.1 交付；不包含第三方图标集合。
