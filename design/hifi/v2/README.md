# xdzip 下一阶段高保真状态稿与素材 v2

状态：设计审阅候选。沿用 v1 高保真主窗口和独立解压窗口的视觉语言。本包使用生成式图像完成视觉稿，未以 HTML 页面渲染替代设计稿。HTML 仅为离线图片画廊。

[打开设计画廊](index.html) · [图板总览 PNG](preview/boards.png) · [新增素材总览 PNG](preview/assets.png) · [功能状态契约](../../../docs/ux/hifi-state-spec.md)

## 图板

| 图板 | 功能状态 | 入口 / 关键动作 |
| --- | --- | --- |
| [01 新建](screens/01-create.png) | C01 创建计划 | 添加 / 删除输入，选择 ZIP/TAR/TAR.GZ，确认目标后创建 |
| [02 重名冲突](screens/02-conflict.png) | E02 等待决策 | 跳过、自动改名、覆盖；默认焦点为跳过 |
| [03 任务进度](screens/03-progress.png) | T01 执行中 | 返回浏览不取消任务；取消单独提交 |
| [04 部分失败](screens/04-partial-result.png) | R01 部分未完成 | 保留成功项，仅重试未完成范围并重新规划 |
| [05 设置](screens/05-settings.png) | S01 配置草稿 | 主题、密度、最近记录和解压限额 |
| [06 任务状态](screens/06-task-states.png) | R02 成功 / T02 正在停止 / R03 已取消 | 三个独立窗口状态，不是一个窗口的三个面板 |
| [07 创建覆盖确认](screens/07-create-overwrite.png) | C02 覆盖准入 | 返回修改、使用新名称、替换并创建 |
| [08 平台对照](screens/08-platforms.png) | T01 Windows 浅色 / Linux 深色 | 同一任务语义，按平台更换标题栏和主题 |

共 8 张原始图板、11 个窗口画面、9 类功能状态。平台对照是共享窗口装饰规则示例，并非每个平台每个状态都有独立绘制。主窗口与解压确认继续使用 [v1 原稿](../../assets/v1/README.md)。

## 切图与资产

- `slices/`：24 张原像素切片，不缩放、不抠底；阴影和背景属于原稿像素，作为视觉参考。
- `svg/light/`、`svg/dark/`：新增 14 类状态/功能图标，每种主题 14 张，共 28 张透明 SVG。它们是矢量重建候选，不是原图图层。
- `png/light/`、`png/dark/`：上述矢量的 1×/2×/3× 导出，共 84 张透明 PNG。逻辑尺寸 24×24，物理尺寸 24/48/72。
- 共享文件图标、导航和按钮图形继续使用 [v1 素材](../../assets/v1/README.md)，不重复维护。
- [manifest.json](manifest.json)：图板与素材 SHA-256、来源坐标、尺寸和素材类型；[validation.json](validation.json)：实际资产检查结果。

新增图标：archive-add、document-download、document-warning、image-file、status-warning、status-success、status-skipped、status-stopped、spinner、info、remove、clock、theme-system、settings-tile。

文本、按钮、数字输入、状态背景、选择态、进度条、系统标题栏和交通灯不固化为产品图片。spinner 是单帧形状候选，动画由 Qt 控件执行并遵从减少动态效果设置。预览图标仅表达文件类型，未启用图像内容解码。

## 精度与实现边界

图片是生成式视觉提案，不保证每个像素、文字形状、进度长度与规范严格相符。例如 60% 应在 Qt 中依据数值绘制为 0.6 倍轨道宽度，不能从图片里切出进度条直接使用。原稿切片已校验与记录的源像素完全一致；矢量重建没有通过逐像素相似度或 Qt 渲染验收。

原稿中的吞吐、剩余时间、磁盘容量、文件内容和任务结果都是设计示例，不是软件性能或功能实测。操作窗口最终必须是真正的 Qt 顶层窗口；此交付不改变三平台原生验收要求。

本次补足主要操作窗口的设计输入，尚不宣称 WP-04 已通过。首次启动、空归档、损坏/不支持格式、多任务退出确认、200% 字号和完整读屏仍需按实施范围补充验收；其他状态通过共享组件与文字规则表达，未逐张生成高保真图。

## 复现与维护

依次运行：

```sh
python3 scripts/build-hifi-v2.py
python3 scripts/render-hifi-v2.py
python3 scripts/check-hifi-v2.py
```

切片导出依赖 ImageMagick；透明 PNG 和总览渲染依赖 Playwright/Chromium；验证依赖 Pillow。可用 `XDZIP_CHROMIUM` 指定 Chromium 路径，`XDZIP_FONTCONFIG_FILE` 指定预览字体配置。构建以固定图板为输入，不自动重新生成设计图。它们是设计工具，不进入产品运行时。

项目原创设计及工具按仓库 Apache-2.0 交付；下载包包含 LICENSE，不引入第三方图标集合或字体。
