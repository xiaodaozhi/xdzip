# WP-01 桥接验证记录

日期：2026-10-11。范围：独立诊断工程，不是 WP-02/03 产品运行时验收。

## 本地环境与结果

Linux x86_64，glibc 2.41；Rust 1.99.0（b940084d7，2026-09-28）；GCC 14.2.0；CMake 3.31.6；Ninja 1.12.1；Qt 6.8.2。Qt 通过临时工具链前缀定位，插件路径指向对应前缀，未更改系统安装。

| 检查 | 结果与边界 |
| --- | --- |
| cargo fmt / Clippy `-D warnings` | 通过；覆盖 workspace/all-targets |
| cargo test | 6 项通过：队列满、锁竞争、所有权/字节上限、接收者断连、关闭唤醒、编号耗尽 |
| CMake Debug + Qt | 构建通过；CTest 2/2，通过字符串桥接与窗口生命周期 |
| CMake Release，不带 Qt | 构建通过；CTest 1/1，校验代码不受 NDEBUG 影响 |
| 中文/空格源码及构建路径 | 从提交导出的 `build/中文 source/source` 到独立 `output`，Release 构建与 CTest 通过；仅构建路径含中文/空格的检查也通过 |

C++ 实际覆盖：非法容量与重复取得句柄的 rust::Error、Qt QString/UTF-8/内嵌 NUL 往返、提交后的输入修改、满队列 Busy、过大输入、关闭后拒绝、接受项顺序排空、空消息不等于结束、重复关闭/结束、跨线程接收与关闭竞争。Rust 和 C++ 测试均不靠固定 sleep 同步；关闭发生在等待前或等待中都必须可退出。CTest 使用 15 秒硬超时兜底。

Qt 窗口检查使用 offscreen/software，只证明对象关系和关闭逻辑；无真实窗口管理器、macOS 红绿灯、GPU、IME、读屏或像素对齐证据。Qt Core 字符串测试没有建立产品 GUI 事件投递。

## 自动化与剩余门槛

[Foundation probes 工作流](../../../.github/workflows/foundation.yml) 配置 Linux、Windows、macOS 原生 runner，统一执行格式、Clippy、Rust 测试、Release 静态链接及 CTest，Qt 关闭。运行结果以实际 Actions 记录为准，配置文件本身不算三平台通过。

待完成：三平台 Qt 构建与桌面交互、Apple M1 最新 macOS 实机、最低 glibc、完整依赖/漏洞准入、关闭压力与产品终态保证、安全文件原语。当前不能宣称完整 WP-01 或 WP-03 已验收。

复现命令及 FFI 前提见 [探针说明](../../../tools/bridge-probe/README.md)，依赖边界见 [cxx 准入记录](../../engineering/cxx-admission.md)。

## 首轮远端证据

提交 `4897b89` 的 [Actions 运行 38148538265](https://github.com/xiaodaozhi/xdzip/actions/runs/38148538265) 三平台桥接全部通过：Linux x86_64/GCC 13.3、Windows x86_64/MSVC 19.44、macOS arm64/AppleClang 15。macOS runner 不等同于 M1 最新系统实机。

首轮日志同时暴露 macOS CMake/Cargo deployment target 不一致（14.0/14.5）的链接警告，后续修订显式统一并重新检查。C++ 使用 clang-format 18.1.8 和仓库配置；选定 clang-tidy、sanitizer 和独立 FFI 审查尚未完成。
