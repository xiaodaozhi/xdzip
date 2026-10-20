# xdzip 高保真设计统一入口

[完整图片画廊](index.html)，[原稿清单与 SHA-256](catalog.json)，[统一素材库](../assets/index.html)，[颜色尺寸规范](../../docs/visual/tokens.json)

十张页面原稿与三张目录树 B 组件图板统一放在 `screens/`，按页面功能命名。主窗口、独立解压确认与各操作状态共同构成当前视觉依据；不再按交付批次拆成不同目录。图片是生成式设计稿，功能边界和精确几何仍以对应规范为准。

## 主界面与解压确认

| 原稿 | 状态 |
| --- | --- |
| [主窗口，浅色](screens/main-light.png) | 当前主界面视觉参考 |
| 主窗口，深色 | **完整原稿待补齐**；深色图标适配不等于完整页面设计 |
| [解压确认，macOS 独立窗口](screens/extract-macos.png) | 当前解压确认视觉参考 |

![当前浅色主界面](screens/main-light.png)

## 操作窗口与功能状态

| 原稿 | 状态 |
| --- | --- |
| [新建归档](screens/create.png) | C01 创建计划 |
| [重名冲突](screens/conflict.png) | E02 等待决策 |
| [任务进度](screens/progress.png) | T01 执行中 |
| [部分失败](screens/partial-result.png) | R01 部分未完成 |
| [设置](sources/settings.html) | S01 主题、文字大小与界面语言，当前修订 |
| [任务状态](screens/task-states.png) | 成功 / 正在停止 / 已取消 |
| [完整性检查进度](screens/integrity-progress.png) | 复用任务进度布局，只读对象、读取速度与校验阶段 |
| [创建覆盖确认](screens/create-overwrite.png) | C02 覆盖准入 |
| [Windows / Linux 平台对照](screens/platforms.png) | 平台窗口装饰示例 |

操作、关闭与恢复行为见 [功能状态契约](../../docs/ux/hifi-state-spec.md)。原稿中的静态进度、计数和示例路径不作为真实运行结果。

## 目录树 B 组件修订

| 原稿 | 定义范围 |
| --- | --- |
| [多级展开](screens/directory-tree.png) | 独立箭头槽、多级连接线、同层对齐 |
| [节点与交互状态](screens/directory-tree-states.png) | 折叠 / 展开、空 / 非空叶子、当前 / 悬停 / 焦点、长名称、大字号 |
| [空态与导航边界](screens/directory-tree-boundaries.png) | 读取、空归档、根文件、超限、失败、隐藏当前目录 |

[集中查看图板与切图](directory-tree.html)，[交互与接入规格](../../docs/ux/directory-tree.md)，[逻辑参数](../../docs/visual/directory-tree.json)，[实际交付检查（本地／CI 工件）](../../docs/qa/README.md)。B 为已确认的目录树修订，不覆盖旧主窗口原稿；图板按放大比例展示，HTML 示例与 Qt 只读目录树已接入核心交互，专用边界图标／骨架与完整深色图板仍待补齐；既有人工专项以 [WP-12 issue](https://github.com/xiaodaozhi/xdzip/issues/5)结论为准，不以图板交付替代实际验收。

## 原稿与素材的职责

`hifi/` 只保留完整原稿、清单与画廊。原来放在此处的图标、PNG、切片和素材预览全部合入 [design/assets](../assets/README.md)，从统一入口查看与维护。

- `assets/svg/`、`assets/png/`：HTML 和 Qt 共用的透明运行图标。
- `assets/reference/`：原图直接裁切的精美对照素材；保留原背景和原像素，用于评审透明重建的差异。
- `assets/demo/`：HTML 示例照片。
- `docs/visual/`：视觉规范与实现约束。
- `docs/qa/`：可复用验证方法与清单；实际证据放本地 build/ 或 CI 工件，结论由 issue 跟踪。

目录合并保持十张原稿及原像素切片的字节不变。历史 QA JSON 保留当时输入路径与哈希，结合 [迁移清单](../assets/migration.json) 或其记录的 Git 提交追溯，不回写历史测量。

新增原稿时更新 `catalog.json` 的用途、标题和 SHA-256，再运行 `python scripts/build-hifi-gallery.py`。CI 通过 `--check` 校验图片完整性及画廊同步，不重新生成图像。清单中的 schema 版本仅用于数据兼容，不是素材分组。

手机离线阅读可运行 `python scripts/build-hifi-gallery.py --standalone dist/xdzip-hifi-gallery.html`，导出内嵌全部原图的单文件 HTML；其他文档链接仍需仓库访问权限。


## 当前交互与尺寸修订

[操作窗口、按钮与任务状态修订](operation-revisions.html)覆盖 2026-10-15 的 491 逻辑像素统一宽度、动态结果高度、分段圆角、按下和禁用状态、重名正常密度及任务菜单。原始位图继续作为历史依据；当前行为及参数以该状态稿、docs/visual/layout-scale.json 和设计规范的修订段为准。

## W01 欢迎页候选

[欢迎页状态稿](sources/welcome.html)提供浅色、深色、832×544 最小窗口和拖入状态。对应原稿为 screens/welcome-*.png，布局与交互见[欢迎页设计](../../docs/ux/welcome.md)。设计已确认并接入正式 HTML，Qt 已接入真实打开与新建，桌面人工验收另行登记。sources/ 保存可编辑设计源，不是产品界面原型。

欢迎页标题栏修订见 screens/welcome-titlebar-*.png，原有 welcome-*.png 与 sources/welcome-initial.html 保留历史。修订只设计正文，标题栏与主窗口共用，不再另设品牌、标签或按钮几何。

S01 当前设置窗口修订见[主题、文字与语言设计源](sources/settings.html)，逻辑参数见[设置参数](../../docs/visual/settings-reference.json)，保存／取消与重启生效见[设置契约](../../docs/ux/settings-localization.md)。[历史 settings.png](screens/settings.png) 保留原图和哈希，不再定义当前设置功能范围。

P01 解压元数据预检修订：[设计源](sources/preflight.html)、[浅色](screens/preflight-body-note-light.png)、[深色](screens/preflight-body-note-dark.png)，参数见 [preflight-reference.json](../../docs/visual/preflight-reference.json)。客户区 491 逻辑像素宽，按内容自适应，最高 600，使用共享操作 footer。

预检的元数据边界说明位于可滚动正文末尾，footer 仅放返回按钮并保持共享操作栏高度；原 footer 含说明的浅深色稿保留在历史画廊。

## 中文文案修订

[当前术语状态稿](terminology.html)覆盖主窗口、欢迎页、解压、创建、进度、结果和设置的浅深色设计。容器名称为“压缩包”，范围为“全部内容”“所选文件和文件夹”，大小属性为“原始大小”“压缩大小”。历史 PNG、初版欢迎页和原始墨迹量测保留原文与哈希，其文字不再定义当前产品用词；布局、密度和控件尺寸保持既定规范。详见[中文术语规范](../../docs/ux/chinese-terminology.md)。

当前操作窗口的中性描边按 [2026-10-15 描边校正](../../docs/visual/design-standard.md#2026-10-15-操作控件描边校正) 统一使用浅色 #DCE2EC。上述动态状态稿与 HTML 共用生成参数，历史位图及其哈希保持不变。

欢迎页新增 [正在打开的浮动遮罩](sources/welcome.html?opening=1)，与 Qt、HTML 共用 [参数和交互契约](../../docs/ux/welcome-opening.md)，支持浅深色，保留原有标题栏。

当前新增设计：[A01 关于 xdzip](about.html)，统一版本信息、开发状态与复制操作。参数来自 [about-reference.json](../../docs/visual/about-reference.json)，不覆盖历史原稿。

[系统菜单与托盘，D01](sources/desktop-integration.html)：原生菜单结构、关闭驻留及恢复交互，实际装饰由系统绘制。

[退出与重启确认状态稿](sources/confirmations.html)覆盖退出、正在停止及重启确认，采用共享 491 宽、20 标题、13 正文和内容自适应高度。中英文、大字号、正文滚动与 footer 规则见[窗口规范](../../docs/visual/design-standard.md)。

[密码解压，P02](sources/password-extraction.html)冻结输入、错误重试与提交中三种状态。采用 491 宽、20 标题、13 正文、12 说明和固定共享 footer，密码字段在截图时保持为空。参数见 [password-reference.json](../../docs/visual/password-reference.json)，真实行为见[密码解压交互](../../docs/ux/password-extraction.md)。
