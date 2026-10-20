# Qt 开发应用与组件预览

`xdzip` 是连接 Rust 核心的真实产品入口，Windows 文件名为 `xdzip.exe`；`xdzip-ui-preview` 只展示明确标识的本地示例任务，不访问用户文件，也不模拟真实压缩进度。产品能力以根目录 [README](../../README.md)为准，交互与窗口规则见[主窗口契约](../../docs/ux/main-window-components.md)及[任务窗口契约](../../docs/ux/task-windows.md)。

## 构建运行

日常环境准备、预检、构建、测试、打包与核验统一使用 [desktop.py](../../scripts/desktop.py)，完整参数及平台前置条件见[开发构建指南](../../docs/engineering/developer-build.md)：

```sh
python3 scripts/desktop.py doctor
python3 scripts/desktop.py build
python3 scripts/desktop.py test
```

默认构建目录为 build/desktop。Linux 直接启动 `build/desktop/apps/desktop/xdzip`，Windows 为同目录 `xdzip.exe`，macOS 为 `xdzip.app/Contents/MacOS/xdzip`。组件示例另运行同目录的 `xdzip-ui-preview`（Windows 加 `.exe`），不用于真实文件验收。更换构建目录时同步调整路径。

共用 test 仅对子进程启用 `QT_QPA_PLATFORM=offscreen` 与 `QT_QUICK_BACKEND=software`，普通桌面启动不设置这些变量。离屏渲染不证明 GPU、系统输入法或读屏已验收。生成物位于忽略的构建目录，实际状态由 issue 与 Actions 记录。

## 低层 CMake 调试

排查 CMake 或 IDE 集成时仍可直接运行底层工具；这不是另一套日常编排。准备锁定 Rust、Ninja 与 Qt Core、Gui、Qml、Quick、QuickControls2、Widgets，测试还需要 QuickTest／Test，然后按需要设置 SDK 前缀：

```sh
cmake -S . -B build/ui-debug -G Ninja -DCMAKE_BUILD_TYPE=Debug \
  -DXDZIP_BUILD_DESKTOP_PREVIEW=ON -DCMAKE_PREFIX_PATH=/path/to/Qt/6.8.2/gcc_64
cmake --build build/ui-debug
ctest --test-dir build/ui-debug --output-on-failure
```

上述 SDK 子目录示例适用于 Linux。macOS 使用 macos 前缀，并在配置时加 `-DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_OSX_DEPLOYMENT_TARGET=14.0`；Windows 使用 msvc2022_64 前缀及 x64 MSVC 开发环境。工具版本、系统依赖与支持范围见[统一构建说明](../../docs/engineering/developer-build.md)和[工具链](../../docs/engineering/toolchain.md)。

macOS 的 `XDZIP_MACOS_HANDLER_RANK` 仅接受 `None`、`Alternate`，普通开发和测试默认为 `None`，避免副本进入系统“打开方式”。主应用或 DMG 输入由共用入口的 `build --macos-handler-rank Alternate` 显式选择；低层 CMake 对应 `-DXDZIP_MACOS_HANDLER_RANK=Alternate`。此设置不更改默认关联，文件拖入、命令行与显式指定应用仍可用。

## 模块与开发边界

Rust 核心负责格式、查询、任务及输出安全，QML 只表达状态和用户意图。浏览器通过正式双句柄运行时、事件线程及有界 mailbox 接收数据；退出等待异步 shutdown 屏障，不在 GUI 线程同步 join。诊断探针不承担产品请求。

界面复用共享窗口、按钮、菜单和选择控件。视觉参数由 `docs/visual` 生成，运行 `python3 scripts/generate-qml-theme.py --check` 检查同步；新窗口先登记窗口契约。产品设置支持主题、100/125/150% 字号及中文 (简体)／English (US)，保存与重启规则见[设置与国际化](../../docs/ux/settings-localization.md)。开发测试用 `XDZIP_SETTINGS_PATH` 隔离配置，`XDZIP_LANGUAGE` 指定进程语言；它们不是产品设置入口。

格式与安全边界见[只读浏览](../../docs/engineering/readonly-browser.md)、[预检](../../docs/engineering/extraction-preflight.md)、[安全解压](../../docs/engineering/m3-extraction.md)、[ZIP 创建](../../docs/engineering/m4-creation.md)及[TAR/GZIP](../../docs/engineering/m4-tar-gzip.md)。内容预览、额外格式和第三方扩展不因设计稿存在便视为实现。

平台开发包、安装器、系统关联及本地 DMG 按[构建说明](../../docs/engineering/developer-build.md)和[安装契约](../../docs/engineering/installers-associations.md)执行。macOS CI 不部署或上传应用，验证副本的 LaunchServices 注册按[Mac 维护方法](../../docs/engineering/macos-deployment.md)自动收尾。
