# Qt 开发应用与组件预览

`xdzip-browser` 是连接 Rust 核心的真实产品入口；`xdzip-ui-preview` 只展示明确标识的本地示例任务，不访问用户文件，也不模拟真实压缩进度。产品能力以根目录 [README](../../README.md)为准，交互与窗口规则见[主窗口契约](../../docs/ux/main-window-components.md)及[任务窗口契约](../../docs/ux/task-windows.md)。

## 构建运行

准备根目录锁定的 Rust、CMake、Ninja 和 Qt Core、Gui、Qml、Quick、QuickControls2、QuickTest、Widgets。将 Qt 前缀加入 `CMAKE_PREFIX_PATH`，版本与支持范围见[工具链](../../docs/engineering/toolchain.md)及[支持政策](../../docs/engineering/support-policy.md)。

```sh
cmake -S . -B build/ui -G Ninja -DCMAKE_BUILD_TYPE=Debug -DXDZIP_BUILD_DESKTOP_PREVIEW=ON
cmake --build build/ui
./build/ui/apps/desktop/xdzip-ui-preview
ctest --test-dir build/ui --output-on-failure
```

macOS 另设置 `-DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_OSX_DEPLOYMENT_TARGET=14.0`，真实应用为 `build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip`。Windows 可执行文件带 `.exe`。Linux 开发环境需要 OpenGL/EGL 和 xkbcommon 开发包，Ubuntu 可安装 `libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev`。

无显示服务时可设置 `QT_QPA_PLATFORM=offscreen` 与 `QT_QUICK_BACKEND=software` 测试；普通桌面运行不设置这些变量。离屏渲染不证明 GPU、系统输入法或读屏已验收。产物、截图和日志位于 `build/`，不提交 Git；实际执行状态由对应 issue 与 Actions 记录。

## 模块与开发边界

Rust 核心负责格式、查询、任务及输出安全，QML 只表达状态和用户意图。浏览器通过正式双句柄运行时、事件线程及有界 mailbox 接收数据；退出等待异步 shutdown 屏障，不在 GUI 线程同步 join。诊断探针不承担产品请求。

界面复用共享窗口、按钮、菜单和选择控件。视觉参数由 `docs/visual` 生成，运行 `python3 scripts/generate-qml-theme.py --check` 检查同步；新窗口先登记窗口契约。产品设置支持主题、100/125/150% 字号及中文 (简体)／English (US)，保存与重启规则见[设置与国际化](../../docs/ux/settings-localization.md)。开发测试用 `XDZIP_SETTINGS_PATH` 隔离配置，`XDZIP_LANGUAGE` 指定进程语言；它们不是产品设置入口。

格式与安全边界见[只读浏览](../../docs/engineering/readonly-browser.md)、[预检](../../docs/engineering/extraction-preflight.md)、[安全解压](../../docs/engineering/m3-extraction.md)、[ZIP 创建](../../docs/engineering/m4-creation.md)及[TAR/GZIP](../../docs/engineering/m4-tar-gzip.md)。内容预览、额外格式和第三方扩展不因设计稿存在便视为实现。

平台开发包、安装器、系统关联及本地 DMG 按[构建说明](../../docs/engineering/developer-build.md)和[安装契约](../../docs/engineering/installers-associations.md)执行。macOS CI 不部署或上传应用，验证副本的 LaunchServices 注册按[Mac 维护方法](../../docs/engineering/macos-deployment.md)自动收尾。
