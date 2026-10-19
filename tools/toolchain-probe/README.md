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

## 路径与生命周期回归

源码与构建目录含中文／空格时，在自有临时目录配置和构建。CTest 设置有界超时，失败返回非零，不把只有窗口对象当作显示或关闭成功。

所有窗口由 C++ 栈持有；transient owner 不转移 QObject 所有权。关闭在事件循环中进行，离开循环后逆序析构。这里的重复 close 不等同于 WP-03 的 Rust 运行时关闭协议测试。

桥接及正式运行时行为由各自测试覆盖。本工具只验证其声明的 Qt 窗口边界，具体执行结论使用对应 issue 和 CI 工件。
