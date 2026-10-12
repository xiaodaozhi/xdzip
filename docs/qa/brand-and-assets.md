# 品牌图标与遗漏素材复核

日期：2026-10-12。品牌以四条交错短带和底部开口 U 为单一标记；主窗口布局、十张完整原稿与 48 张原像素切片保持不变。

[当前统一素材库](../../design/assets/index.html) · [实际图标与补图预览](brand-and-assets/index.html) · [逐项原稿对照](asset-review/index.html) · [完整清单](../../design/assets/manifest.json)

## 应用与关联文件图标

`brand/mark.svg` 保存标记几何，`brand/xdzip.svg` 是带透明安全边距的 1024 应用母版，`brand/xdzip-zip.svg` 是白色折角纸张、蓝色品牌标记及路径绘制 ZIP 字样的文件母版。图形维护源在 [brand_assets.py](../../scripts/brand_assets.py)，没有依赖系统字体绘制标记或 ZIP 字样。

| 导出 | 规格与实际用途 |
| --- | --- |
| PNG | 两家族各提供 16／24／32／48／64／128／256／512／1024 px；512 版本名为 `xdzip.png`、`xdzip-zip.png` |
| ICO | 每个文件内含 16／24／32／48／64／128／256 共七种表示 |
| ICNS | 每个文件内含十一种普通／Retina 表示，最高 1024 px |
| UI SVG／PNG | `app-mark`、`archive`、`archive-grey` 保持 24 单位画布，与所有其他共享素材统一导出 |

16／24 px 的文件图标放大内部标记；32 px 使用过渡比例；48 px 起使用母版照明与字样。每个尺寸直接从对应矢量布局渲染，不放大小位图。标记本身始终是同一五片路径，U 内孔和交错短带在小尺寸下保留辨识度。PNG 内记录母版与实际渲染 SVG 的 SHA-256，ICO／ICNS 逐项核验尺寸、容器和 PNG 载荷。

Qt 正式浏览应用及预览程序使用 512 应用图标；标题栏使用浅深色共享素材。macOS bundle 写入 ICNS 与 `CFBundleIconFile`，Windows 将 ICO 编入可执行资源，Linux 提供无 MIME 声明的桌面启动项与 hicolor 图标。[平台接入说明](../../apps/desktop/packaging/README.md)记录安装范围。关联文件的 PNG／ICO／ICNS 已交付，供后续安装器使用；当前不注册文件关联或修改默认打开应用。

新品牌是明确的设计修订，旧 `brand-titlebar`／`brand-extract` 切片在清单中标为历史视觉参考。生成的精美设计方向图是独立参考，生产图是可维护的矢量重绘，不宣称逐像素切自生成图。

## 补齐的十三类素材

| 名称 | 对应形态与实际消费 |
| --- | --- |
| theme-light、theme-dark | 完整微型窗口，进入设置页的原生主题 radio 卡片；与 theme-system 使用同一窗口结构 |
| existing-archive | 蓝色折角归档与拉片细节，进入覆盖确认的已有目标比较项 |
| windows-document、linux-document | 两平台进度窗口的下载文档；透明背景与各自原稿配色 |
| cancelled-large | 灰色圆盘＋白色圆角停止方块，进入取消终态 |
| check-selected | 勾选外观，保留真实 checkbox 的状态、键盘与可访问性 |
| radio-selected、radio-unselected | 选择环与空心环，保留真实单选行为 |
| success | 小绿色校验提示，进入解压／创建预检提示 |
| conflict-pdf、conflict-warning | 冲突比较 PDF 和圆形警告，不以列表小图或裸三角替代 |
| progress-image | 当前图片文件插图，进入进度内容行 |

另修正既有成功／部分成功／跳过／停止／警告圆章、停止中 spinner 和跟随系统主题图：恢复渐变、高光、内边缘和完整窗口内容。圆盘白色符号、停止方块与单选圆芯按实际着色轮廓量测，不把原切片外部留白算作圆径。可复验的源哈希、颜色阈值、白色墨迹、最终 SVG 参数及 3× PNG 结果见 [细节量测](asset-review/source-detail-metrology.json)。

HTML 同时接回已有的新建、冲突、设置与结果素材，取消状态使用停止方块，停止中使用 spinner，非活动归档使用独立灰色图标。设置页从下拉框改为三个真实 radio 卡片；点击、方向键和焦点仍由实际控件承担。新增素材与当前实际页面截图见 [HTML 消费记录](brand-and-assets/html/validation.json)。

## 一份素材清单与验证

共享库为 **68 类、136 SVG、408 PNG**，另有 45 张原参考、3 张照片和 26 项品牌导出，共 **618 项**。主清单内的 `reference_coverage` 逐项登记全部 45 张参考的角色、浅深色文件、消费方式与原生控件说明；不以近义小图冒充缺失的大图。原有未涉及的 90 张 SVG 字节保持不变。

本地生成与素材检查通过：618 项资产、45 项精确语义覆盖、42 项既有文件光学约束。Qt 42 个资源别名和 HTML 61 个图标名／18 处动态调用有效。512 应用图标是唯一独立规格，验证固定用途、路径、别名及完整母版哈希链，其余共享素材仍执行 24／48／72 规格。五种缺失、错误映射、别名及来源负向用例被拒绝。

HTML 顶部 27 项、正文 70 项、预览 17 项、原稿加载 27 项、尺寸 391 项及示例流程 20 条通过。最后状态图精修只改变白色符号墨迹，重新嵌入最新 SVG 后另跑实际消费专项 9 项、9 张截图，页面错误与外部请求均为零。586 个字体请求字符均覆盖，字体字节没有变化。

Qt 在 Linux、Qt 6.8.2、Rust 1.99、GCC 14.2 的独立构建通过，最终素材重生成 rcc 并重新链接，5 项 CTest 通过；此前已验证的 10 次品牌和 36 次文件图像加载对应 PNG 未改变，保留真实截图与 XML。Linux 暂存安装通过，安装图标与母版导出字节一致；并未在系统注册默认应用。详见 [Qt 消费记录](brand-and-assets/qt/validation.json)。

另对 C++ 的系统应用图标做实际探测：`QIcon` 必须使用 `:/xdzip/assets/app-icon.png`，QML 的 `qrc:/` 写法在此得到空图标。两入口已修正；实际 `QGuiApplication.windowIcon()` 非空，资源表示为 512×512，64×64 pixmap 可生成。两应用的既有冒烟路径检查图标与 pixmap，避免只凭 QML 图像加载推断系统图标有效。[路径对比记录](brand-and-assets/qt/qicon-path-validation.json)保存正反验证。

几何、哈希和加载证明各自约束，不证明原稿所有阴影、字形、纹理或整页逐像素一致。折角投影、拉链微字及 PDF 字形等仍是独立重绘；原生 Finder／Dock／Explorer 的显示还需目标系统核对。完整原稿及原像素参考始终保留，供后续逐项对照。
