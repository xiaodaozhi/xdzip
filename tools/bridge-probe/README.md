# Rust / C++ 桥接验证工程

这是 WP-01 的独立诊断工具，不是归档引擎或插件 ABI。根 Cargo workspace 现已包含 xdzip-domain，CMake 可选构建 Qt 组件预览；本目录继续保持独立诊断用途，不作为产品运行时。

## 构建

需要 Rust 1.99.0（含 rustfmt、Clippy）、CMake 3.24+、Ninja、C++17 编译器。默认还需要 Qt 6.8+ 的 Core/Gui/Qml/Quick；通过 `CMAKE_PREFIX_PATH` 指向 Qt 安装目录。Windows 在 x64 MSVC 开发者终端运行；macOS 使用与 Rust host 一致的原生架构，M1 使用 arm64 Qt 和 Rust。macOS 运行命令前设置 `MACOSX_DEPLOYMENT_TARGET=14.0`，或向 CMake 显式传入相同目标；该值是探针候选，不是产品最低系统支持承诺。

在仓库根目录执行：

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cmake -S . -B build/foundation -G Ninja -DCMAKE_BUILD_TYPE=Debug
cmake --build build/foundation
ctest --test-dir build/foundation --output-on-failure
```

无 Qt 时在配置命令追加 `-DXDZIP_BUILD_QT_PROBE=OFF`，只验证原生 C++/Rust。Linux 无显示服务时运行 CTest 前设置 `QT_QPA_PLATFORM=offscreen`、`QT_QUICK_BACKEND=software`；真实桌面测试不设置这两个变量。Release 使用单独构建目录并设置 `-DCMAKE_BUILD_TYPE=Release`。当前不支持交叉编译、Universal Binary 和多配置生成器。

Cargo 首次联网获取锁定依赖；缓存完整后设置 `CARGO_NET_OFFLINE=true` 可离线重建。生成的头文件和静态库只写入构建目录。修改源码由 CMake 的显式依赖触发 Cargo。升级依赖必须一起更新 Cargo.lock 和准入记录。

## 所有权与关闭

- `Probe` 只负责一次性转移生产者、接收者两个 `rust::Box`；重复取得句柄通过 `rust::Error` 返回。
- 每个 C++ 句柄仅由一个线程持有，可以移动，不允许别名并发调用。输入 `rust::Str` 只在 submit 调用期间借用；接受后 Rust 拷贝最多 4096 字节。返回的 `rust::String` 独立拥有数据，支持内嵌 NUL。
- 容量为 1–128 条；队列满或锁竞争返回 Busy，不占用编号。编号耗尽拒绝提交，零仅用于拒绝/结束标记。容量与文本上限限定的是负载预算，不是进程总内存。
- shutdown 在控制路径关闭准入并唤醒接收者，不依赖队列空位；它可能短暂等待互斥锁，并非无锁实时操作。已接受消息先排空，再返回显式 end。重复 shutdown/end 有定义。
- 释放生产者等同于关闭；释放接收者关闭准入并丢弃剩余诊断消息。该丢弃语义不是产品任务的终态交付保证。
- 阻塞接收只能在工作线程执行。本测试命令行无 GUI 事件循环；Qt Core 仅参与字符串边界验证。后续 Qt 事件投递、mailbox 合并、任务取消/提交竞争尚未实现。
- 手写 Rust 禁止 unsafe；仅 cxx 生成桥接局部放行。预期错误使用 Result/枚举；内部锁中毒是程序缺陷，panic 不作为可恢复的跨语言错误。C++ 异常在调用端处理，不回调穿越 Rust 栈。

原生三平台 CI 执行 [检查脚本](../../scripts/check-foundation.ps1)，不安装 Qt，因此不证明三平台窗口行为。实际证据见 [桥接验证记录（本地／CI 工件）](../../docs/qa/README.md)。
