# cxx 桥接依赖依据

Qt/Rust 采用 `cxx` 生成的类型化所有权桥接；C++ 保留 Qt 对象，Rust 保留领域和运行时。`cxx` 与 `cxx-build` 必须精确同版，以 [Cargo.lock](../../Cargo.lock) 及[工具链策略](toolchain.md)为准。cxx 不作为第三方格式扩展的稳定 ABI。手写 C ABI 虽减少依赖，却增加字符串、释放函数与所有权不变量维护成本。

## 来源与许可

来源为 crates.io，上游为 [dtolnay/cxx](https://github.com/dtolnay/cxx)。依赖的来源哈希、版本、features、构建脚本和许可声明从锁文件及[桥接依赖清单](bridge-dependencies.json)取得，发行时重新核对并汇总实际适用通知。锁文件包含目标条件不成立的依赖，不等于当前二进制实际链接清单，也不覆盖编译器、Qt 或系统库。

## 构建与所有权边界

cxx-build 生成源码和头文件并调用 C++ 编译工具；cxx 编译包内桥接实现，link-cplusplus 负责 C++ 标准库链接。依赖获取由 Cargo 按锁文件校验，新增构建脚本或可选特性必须重新审查其网络与执行行为，不把历史检查当作新版本安全结论。

保持 std/alloc 等实际产品所需特性；裁剪需复验 UTF-8、错误传播、唯一句柄、背压、关闭与排空行为。[运行时契约](m1-runtime.md)和[桥接设计](../design/05-qt-rust-bridge.md)定义实际接口。独立安全审查及发行许可复核按[发行审查流程](../qa/release-review.md)执行。

具体探针、CI 运行与验收结论记录在 [WP-01 issue](https://github.com/xiaodaozhi/xdzip/issues/1)及其 Actions 工件，不在此累积执行报告。
