# WP-01 Qt Quick 工具链探针

这是从零建立的独立工程验证工具，不是桌面产品、视觉实现或旧实验代码的迁移。它验证 C++17/CMake/Qt Quick 动态链接和两个顶层窗口的对象关系、显示及重复关闭；不含 Rust 桥接、文件读写或编解码器。

## 标准安装环境

```sh
cmake -S tools/toolchain-probe -B build/toolchain-probe -G Ninja -DCMAKE_PREFIX_PATH="<Qt 安装前缀>"
cmake --build build/toolchain-probe
ctest --test-dir build/toolchain-probe --output-on-failure
```

需要 CMake 3.24+、Qt 6.8+（Core/Gui/Qml/Quick）、C++17 编译器及 Ninja。以上最低要求不是冻结后的发行工具链。Windows 使用匹配 Qt 的 MSVC 开发环境；系统动态加载器须能发现 Qt 运行库和平台插件。

无图形会话的 Linux 可设置 `QT_QPA_PLATFORM=offscreen`、`QT_QUICK_BACKEND=software` 后运行 CTest。此模式不验证 Wayland/X11、GPU、系统装饰、输入法或可访问性。

## 当前工作区实测命令

本机工具安装在非系统前缀，需要显式运行库路径。这里的工作区路径只用于复现当前证据，不是开发者必须采用的安装位置。

```sh
export LD_LIBRARY_PATH=/workspace/.tooling/sysroot/usr/lib/x86_64-linux-gnu
/workspace/.tooling/sysroot/usr/bin/cmake -S tools/toolchain-probe -B build/toolchain-probe -G Ninja -DCMAKE_MAKE_PROGRAM=/workspace/.tooling/sysroot/usr/bin/ninja -DCMAKE_PREFIX_PATH=/workspace/.tooling/sysroot/usr
/workspace/.tooling/sysroot/usr/bin/cmake --build build/toolchain-probe
QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software QT_PLUGIN_PATH=/workspace/.tooling/sysroot/usr/lib/x86_64-linux-gnu/qt6/plugins /workspace/.tooling/sysroot/usr/bin/ctest --test-dir build/toolchain-probe --output-on-failure
```

源码与构建目录含中文/空格的验证使用相同源文件复制到 `build/路径 with spaces/source` 后重新配置及构建。CTest 设置 15 秒超时；失败必须返回非零，不把仅有窗口对象当作显示或关闭成功。

所有窗口由 C++ 栈持有；transient owner 不转移 QObject 所有权。关闭在事件循环中进行，离开循环后逆序析构。这里的重复 close 不等同于 WP-03 的 Rust 运行时关闭协议测试。

后续必须补充 cxx 最小往返、生成器版本一致性、跨语言所有权和真实 Qt 接收线程关闭测试。当前证据见 [WP-01 记录](../../docs/qa/toolchain/wp01-linux.md)。
