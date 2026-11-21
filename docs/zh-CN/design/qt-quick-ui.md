# Qt Quick 桌面界面

[English](../../design/qt-quick-ui.md) | 简体中文 | [繁體中文](../../zh-TW/design/qt-quick-ui.md) | [日本語](../../ja-JP/design/qt-quick-ui.md) | [한국어](../../ko-KR/design/qt-quick-ui.md) | [Français](../../fr-FR/design/qt-quick-ui.md) | [Deutsch](../../de-DE/design/qt-quick-ui.md) | [Español](../../es-ES/design/qt-quick-ui.md) | [Português (Brasil)](../../pt-BR/design/qt-quick-ui.md) | [Русский](../../ru-RU/design/qt-quick-ui.md)

桌面使用 Qt Quick 呈现界面与交互，C++ 控制器转换操作意图并管理视图状态。归档解释、任务规则和文件授权由 Rust 运行时持有。下文调用关系见[系统架构](system-architecture.md)和[桥接契约](qt-rust-bridge.md)。

## 窗口、会话与模型

| 组件 | 当前职责 |
| --- | --- |
| `Browser.qml` 与 `SessionPage.qml` | 承载欢迎页、归档标签、路径与搜索控件、目录导航、列表或网格、属性和操作入口。页面展示控制器当前状态并调用命令。 |
| `ApplicationSessions` | 将打开请求交给独立浏览会话，协调标签或窗口、共享任务入口与退出。打开方式设置对后续打开生效。关闭页面会释放浏览呈现，任务控制器仍可继续运行。 |
| `BrowserController` 与 `ArchiveTableModel` | 管理选区、列表状态和结果身份。小结果完整提交到模型，大结果保留完整逻辑行数，最多常驻 1,024 行并按需加载。未加载行无法提交条目操作。 |
| `TaskListModel` 与操作窗口 | 呈现有界任务快照、进度、结果和当前的待决定事项。任务操作通过桌面的组合身份发往所属控制器。 |
| 设置与平台适配层 | 管理外观、语言、字号和应用行为。由原生适配处理各系统的托盘、激活、文件对话框、关联及传输交接。 |

[桌面入口](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/browser_main.cpp)、[会话页面](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/ui/SessionPage.qml)、[控制器与模型](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/BrowserController.h)、[应用会话](../engineering/application-sessions.md)、[设置与本地化](../ux/settings-localization.md)

## 浏览与操作流程

打开后发布完整的元数据快照。新的搜索、排序或导航请求携带自己的结果身份。过期分页无法替换当前模型。控制器仍认定已提交结果有效时，界面可在等待新查询期间继续显示该结果。核心传输每批最多 128 行和 64 KiB 名称数据。[领域数据](domain-data.md)、[大型归档索引](../engineering/large-index.md)

打开目录会进入其内容。双击或对符合条件的普通条目执行打开操作，会启动临时打开任务。属性面板提供单独的预览操作。重名与密码提示保留当前操作身份。进度和结果可通过应用任务入口查看，浏览页关闭后也可访问。[浏览](../user-guide/browse.md)、[临时打开](../engineering/archive-opening.md)、[内容预览](../engineering/content-preview.md)、[任务窗口](../ux/task-windows.md)

拖入文件可根据目标打开归档或提供创建输入。符合条件的归档内移动与拖出先准备任务，再按平台交接。各系统的传输能力决定异步内容与目标写出的处理方式。[归档传输](../engineering/archive-transfers.md)

## 共享控件与原生边界

操作窗口复用 `OperationButton`、`OperationFooterButton`、`OperationFrame`、`OperationSelect` 和 `OperationSegmentedGroup`。共享控件负责视觉参数和交互。离散选择使用共享意图控件，将鼠标、键盘和辅助功能操作路由到同一状态模型。界面文字与文件数据遵循[视觉规范](../visual/design-standard.md)中的字体和本地化规则。归档名称作为数据安全显示。

独立操作窗口使用窗口登记与各窗口自己的模态和关闭行为。macOS 使用系统标题栏控件，Windows 和 Linux 使用对应的系统适配。[平台窗口契约](../ux/platform-windows.md)定义真实窗口的验收，[桌面集成契约](../ux/desktop-integration.md)说明托盘与激活行为。

## 设计与验证范围

产品行为以源码和测试为据。涉及原生焦点、菜单、输入法、辅助功能操作、显示缩放和系统文件管理器时，需要对应平台证据。扩大桌面平台配置的支持范围或新增操作窗口，需要相应实现与验收。[视觉流程](../visual/design-workflow.md)、[质量工程](quality-engineering.md)
