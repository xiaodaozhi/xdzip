# 工具链与平台冻结表

开发工具版本由 rust-toolchain.toml、Cargo.lock、CMake 和 CI 固定，支持范围遵循[支持政策](support-policy.md)。本表定义升级及发行前的核验要求，具体观测记录留在 issue／Actions，不把单机工具列表当作发行依赖锁。

| 项目 | 当前方向 | 冻结所需证据 |
| --- | --- | --- |
| Rust | stable，统一 edition 与 MSRV | 具体版本、三目标编译、依赖 MSRV；写入 rust-toolchain.toml |
| Qt | Qt 6.8 或更新的受维护版本，动态部署 | 精确补丁/来源/哈希、模块许可、三平台 smoke、维护渠道 |
| C++ | C++17、Qt 匹配的编译器 | macOS clang、Windows MSVC、Linux 编译器版本 |
| CMake | 支持所选 Qt 和生成式 cxx 桥接 | 精确最低版本、Ninja/presets、生成头依赖验证 |
| cxx | 构建期生成桥接，不提供稳定二进制 ABI | 锁定 crate/生成器同版本，关闭竞态和异常边界通过 |
| 系统 | macOS arm64 优先，Windows/Linux x86_64 | design/11 候选最低版本经过干净机器运行 |

每条记录包含版本、下载来源、SHA256、许可证、目标、负责人、验证提交和日期。Rust/Qt 安全更新触发重新验证，不无限保留已失去维护的补丁。最低系统候选不等于正式支持承诺。

系统维护状态、Qt 安全更新渠道及升级复验责任以[支持政策](support-policy.md)为准，来源与许可结论按[发行审查流程](../qa/release-review.md)登记。

Windows 主窗口的客户区扩展使用 Qt 6.8.2 随 SDK 提供的 GuiPrivate Windows native interface，仅由 WindowsFrame 适配层引用。它不是稳定的公共 API；升级 Qt 或替换 SDK 时必须重新编译并验证客户区尺寸、原生边框、DWM、DPI 变化和最大化工作区。Linux／macOS 不链接此私有接口。
