# xdzip 设计交付目录

**HTML 界面从 [html/index.html](html/index.html) 打开；已确认图片从 [hifi/index.html](hifi/index.html) 查看。** 二者各有职责，不是两套任选的视觉标准。HTML 按原稿实现，差异记录在 [HTML 交付说明](html/README.md)。

| 目录 | 内容 | 当前使用者 | 是否保留 |
| --- | --- | --- | --- |
| `html/` | 可操作的 HTML 高保真界面、审阅入口、截图 | 浏览器审阅、离线交付、后续 Qt 对照 | 保留；唯一 HTML 界面源 |
| `hifi/v1/screens/` | 已确认主窗口和解压确认原图 | HTML、Qt、素材复核 | 保留；仍是当前原稿 |
| `hifi/v2/screens/` | 新建、冲突、任务、结果、设置等八张补充状态稿 | HTML、Qt、功能状态规范 | 保留；补充 v1，不替代 v1 |
| `assets/fonts/` | HTML 离线字体子集、OFL许可及来源清单 | 固定浏览器复核的字形与文字度量 | 保留；仅用于 HTML |
| `assets/v1/svg/` | 主窗口与通用图标的矢量源 | HTML 直接嵌入；PNG 导出源 | 保留 |
| `assets/v1/png/` | 浅深色、1×/2×/3× PNG 导出 | Qt 当前引用 3×；其他倍率属于素材交付 | 保留 |
| `assets/v1/slices/` | 原稿图标对照切片与示例照片 | 视觉核对；HTML 示例图片 | 保留；带背景图标不可进入 Qt |
| `assets/v1/preview/` | 自动生成的素材总览 | 素材审阅 | 保留；不是额外设计标准 |
| `hifi/v2/svg/`、`png/`、`slices/`、`preview/` | 操作窗口补充素材及其对照 | Qt 信息/文件图标、HTML 状态展示、素材审阅 | 保留；目录位置沿用 v2 包，跨包用途见清单 |

`hifi/` 的 catalog 和画廊统一索引当前十张原稿；素材 manifest 记录来源、尺寸、类型与哈希。目录中的 `v1/v2` 表示交付来源，不表示“v1 已作废”。

## 已删除或替换

- `hifi/archive/early/`：未采用的三张早期提案已删除。
- `assets/v1/brand/`：未采用的生成式品牌母图及五种导出已删除，当前品牌使用共享 `app-mark` 矢量。
- `assets/v1/sources/`：原稿迁移后留下的空目录已删除。
- `prototype/`：功能逻辑迁入 `html/` 并重做视觉；旧目录不再保留。
- 旧原型截图：被新的 HTML 实际渲染截图替换，避免作为当前效果误用。

历史版本通过 Git 追溯，不在设计目录另存一套废弃交付。删除不影响已确认原稿和 Qt 运行素材。

## 更新顺序

1. 视觉依据更新 `hifi` 原稿清单；不能修改原稿来掩盖实现差异。
2. 图标修改 SVG 生成器，导出两种主题和多倍率 PNG，并查看相邻原稿对照。
3. 运行 `python scripts/build-html-design.py`，同步 HTML 共用素材并生成 `dist/xdzip-hifi-design.html`。
4. 运行 `python scripts/check-html-design.py` 验证浏览器交互；另做原稿对照，不把测试通过当作视觉相似度。

Qt 实际截图在 `docs/qa/`，HTML 截图在 `html/screenshots/`；两者都不是原稿。
