# Qt 开发应用与组件预览

`xdzip-browser` 已接通真实 ZIP 目录浏览：打开文件、保留重复条目、分页、取消读取、本页多选、显式/隐式目录树、名称搜索及全结果排序；不解压或创建归档。运行 `./build/ui/apps/desktop/xdzip-browser`，也可追加 ZIP 路径。详细边界见 [只读浏览契约](../../docs/engineering/readonly-browser.md)。

`xdzip-ui-preview` 提供独立任务详情窗、共用提示行、按钮和主题 token；不是可用的压缩工具。入口始终标识“组件开发预览”，任务数据固定来自本地示例，不访问文件或连接诊断桥接。不用动画或计时器伪装真实压缩进度。

## 构建运行

准备根目录锁定的 Rust、CMake、Ninja、Qt 6.8+ Core/Gui/Qml/Quick/QuickControls2/QuickTest/Widgets。将 Qt 的前缀加入 `CMAKE_PREFIX_PATH`。macOS 同时设置 `MACOSX_DEPLOYMENT_TARGET=14.0`，该值仅为当前工具链候选。

```sh
cmake -S . -B build/ui -G Ninja -DCMAKE_BUILD_TYPE=Debug -DXDZIP_BUILD_DESKTOP_PREVIEW=ON
cmake --build build/ui
./build/ui/apps/desktop/xdzip-ui-preview
ctest --test-dir build/ui --output-on-failure
```

Linux 开发环境还需 OpenGL/EGL 和 xkbcommon 开发包；Ubuntu 可安装 `libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev`。即使使用软件离屏渲染，Qt Gui 的构建配置仍需要这些开发文件。

Windows 可执行文件带 `.exe`。macOS 的实际浏览应用生成 `build/ui/apps/desktop/xdzip.app`，组件预览仍是原生命令行开发构建。三平台 CI 提供开发验收产物，见 [M1 验收入口](../../docs/qa/m1-acceptance.md)；尚未完成 Developer ID 签名或公证。无显示服务的 Linux 设置 `QT_QPA_PLATFORM=offscreen` 与 `QT_QUICK_BACKEND=software` 后运行测试，普通桌面运行不设置这些变量。

主窗口可打开/重开任务详情、切换主题、放大文字、检查未知总量和重置示例。关闭详情不发送取消；“取消任务”仅请求示例进入“正在停止”，不显示虚假的取消完成。状态由预览主窗口持有，正式任务状态仍将由 Rust runtime 持有。

## 实现边界

- `OperationWindow` 是带 transient owner 的真实顶层非模态 Window，保留系统装饰。macOS 不绘制交通灯；Windows/Linux 暂用原生标题栏。当前只实例化任务窗，其余操作窗口的模态/关闭策略尚未实现。
- 两行提示使用同一个 `InfoRow`：相同外侧 24 逻辑像素、20 像素图标列、12 像素图文间距；正文与底栏都在窗口坐标下校验。
- 文字采用系统字体，200% 文字使用可滚动布局，按钮仍可到达；不是强制缩小字体或裁剪文字。
- `Theme.qml` 由视觉 token 生成；运行 `python3 scripts/generate-qml-theme.py` 更新，`--check` 校验同步。主题选择目前只作用于预览，不持久化。
- 图标从 [统一素材库](../../design/assets/index.html) 引用透明 3× PNG，避免额外 SVG 运行时依赖。品牌与归档小图标共用 24 单位源画布，按浅深主题选择资源；文字、百分比和进度轨道仍由 QML 绘制，未知总量不显示百分比或虚构已完成长度。
- 窗口默认图标与各平台应用图标来自同一品牌源；macOS bundle 包含 ICNS，Windows 可执行文件编入 ICO，Linux 提供桌面启动项与 hicolor 应用图标安装规则。ZIP 关联文件图标已交付，但尚未注册系统文件关联，详见 [系统图标打包说明](packaging/README.md)。
- 现有窗口定位仅按显示屏边界夹取；工作区保留边缘、多显示器迁移、实际系统按钮状态、VoiceOver/IME、原生 GPU 和 Qt 打包仍待验证。
- 组件预览不连接真实任务；独立 browser 入口有 Qt/Rust 只读桥接，尚无完整任务调度、解压或安全文件输出。诊断探针不充当产品 runtime。

测试输出包含四张离屏渲染截图，位于 CTest 工作目录；这些是组件渲染证据，不替换高保真设计稿。原生装饰不会出现在 contentItem 截图里。详见 [阶段记录](../../docs/qa/foundation-components.md)与[目录导航/排序验收](../../docs/qa/navigation-sorting.md)。

任务窗口已按原稿补齐路径卡片、三列统计和正常字号同行底栏，实际截图与保留差异见 [视觉修订记录](../../docs/qa/task-visual-review.md)。

浏览操作：单击行或复选框选择，双击显式目录或从左侧目录树进入目录，“全部文件”返回；搜索框按回车提交。点击前四个列头切换升/降序，更多菜单可恢复归档物理顺序。切换页面/查询会清空本页选择。右侧仅显示条目元数据，尚不读取文件内容。

选中条目后可点击“解压预检”，在独立原生窗口查看展开计数、声明大小和路径/冲突问题；不写出文件。详见 [预检契约](../../docs/engineering/extraction-preflight.md)。

浏览应用已接入正式双句柄运行时、专用事件线程与有界 mailbox；窗口关闭与 Cmd+Q 均等待异步 shutdown 屏障。诊断探针不承担产品请求。
