# xdzip 高保真设计统一入口

[打开完整图片画廊](index.html) · [图片清单与 SHA-256](catalog.json) · [颜色尺寸规范](../../docs/visual/tokens.json)

所有完整高保真原稿统一存放在本目录。当前视觉参考由 **v1 主窗口与独立解压确认 + v2 操作状态补充**共同构成；v2 不是替换全部页面的新版本。图片是生成式设计稿，功能边界和精确几何仍以对应规范为准。

## 主界面与解压确认

| 原稿 | 状态 | 对应素材 |
| --- | --- | --- |
| [主窗口 · 浅色](v1/screens/main-light.png) | 当前主界面视觉参考 | [v1 切图](../assets/v1/README.md) |
| 主窗口 · 同版本深色 | **待补齐**；不得用早期深色图冒充对应版本 | v1 深色图标仅为素材适配 |
| [解压确认 · macOS 独立窗口](v1/screens/extract-macos.png) | 当前解压确认视觉参考 | [v1 切图](../assets/v1/README.md) |

![当前浅色主界面](v1/screens/main-light.png)

## 操作窗口与功能状态

| 原稿 | 状态 |
| --- | --- |
| [新建归档](v2/screens/01-create.png) | C01 创建计划 |
| [重名冲突](v2/screens/02-conflict.png) | E02 等待决策 |
| [任务进度](v2/screens/03-progress.png) | T01 执行中 |
| [部分失败](v2/screens/04-partial-result.png) | R01 部分未完成 |
| [设置](v2/screens/05-settings.png) | S01 配置草稿 |
| [任务状态](v2/screens/06-task-states.png) | 成功 / 正在停止 / 已取消 |
| [创建覆盖确认](v2/screens/07-create-overwrite.png) | C02 覆盖准入 |
| [Windows / Linux 平台对照](v2/screens/08-platforms.png) | 平台窗口装饰示例 |

[v2 交付说明与素材](v2/README.md) · [功能状态契约](../../docs/ux/hifi-state-spec.md)

## 已清理的内容

不再采用的早期三张提案已从工作目录删除，通过 Git 历史追溯。当前原稿没有删除。

## 目录职责与维护

- `design/hifi/`：完整高保真原稿、统一画廊及版本索引。
- `design/assets/v1/`：切图、图标、品牌素材及裁切清单，原稿通过相对路径引用本目录。
- `docs/visual/`：视觉说明与颜色尺寸规范，不再单独存放设计原图。
- `docs/qa/`：真实 Qt 截图及验收证据，不能当作设计原稿。

2026-10-12 完成归档统一，图片字节保持不变。`catalog.json` 的 `previous_path` 记录迁移位置；历史验收 JSON 中的输入路径和哈希保持原样，用该映射或其记录的 Git 提交追溯。

新增原稿时更新 `catalog.json`，注明版本、用途与哈希，再运行 `python3 scripts/build-hifi-gallery.py`。CI 通过 `--check` 校验图片完整性与画廊同步；不重新生成图像。迁移不代表缺失状态已补齐或产品功能已实现。

手机离线阅读可运行 `python3 scripts/build-hifi-gallery.py --standalone dist/xdzip-hifi-gallery.html`，导出内嵌全部原图的单文件 HTML；其他文档链接仍需仓库访问权限。
