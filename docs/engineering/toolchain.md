# 工具链与平台冻结表

开发工具版本由 rust-toolchain.toml、Cargo.lock、CMake 和 CI 固定，支持范围遵循[支持政策](support-policy.md)。本表定义升级及发行前的核验要求，具体观测记录留在 issue／Actions，不把单机工具列表当作发行依赖锁。

| 项目 | 当前方向 | 冻结所需证据 |
| --- | --- | --- |
| Rust | rust-toolchain.toml 固定 1.99.0，edition 2024，rust-version 1.99 | 升级时核对依赖 MSRV、三目标测试和 Clippy，保持工具链文件与 Cargo 配置一致 |
| Qt | 当前固定 6.8.2 SDK，动态部署 | 固定来源／哈希、模块许可、三平台 smoke；每次发行检查安全维护状态，升级须复验源码与替换流程 |
| C++ | C++17、Qt 匹配的编译器 | macOS clang、Windows MSVC、Linux 编译器版本 |
| CMake | 根配置要求 3.24 及以上，使用 Ninja | 核对实际构建版本、生成头依赖与干净配置；CI 工具版本由工作流固定 |
| Python | 共用桌面入口要求 3.10+，打包校验工具使用隔离环境 | 同一解释器执行后续 Python 脚本，按阶段检查固定工具依赖；不将浏览器设计工具列为产品构建依赖 |
| cxx | 1.0.205，构建期生成桥接，不提供稳定二进制 ABI | crate／生成器同版本，关闭竞态和异常边界回归 |
| 系统 | macOS 14+ arm64、受维护 Windows 11 x64、Ubuntu 24.04 X11 x64 | 按已批准支持政策核对真实环境，不把 CI 宿主扩大为全部系统版本保证 |

GitHub 托管的 Apple Silicon 作业使用 macOS 15 运行器，产品部署目标仍为 macOS 14.0。运行器升级不等于在 macOS 14 实机上完成验证；平台兼容与人工验收分别记录。

日常准备、检查与执行统一见[开发构建指南](developer-build.md)。`scripts/desktop.py doctor` 默认检查 build；`doctor --for-stage package` 提前检查完整打包链，prepare 的 `--install-rust`、`--install-qt`、`--packaging` 显式准备相应工具与锁定材料。系统编译器／SDK 仍由开发者安装，不把预检当作隐式安装器。

预检运行实际 PATH 中的 Rust／Cargo，核对锁定版本与原生 host，并检查 CMake、Ninja、Qt 前缀及平台工具。编译器已安装但当前 shell 不可调用也视为缺失。打包前还检查 Rust 锁定依赖元数据、Qt SPDX／许可材料及 schema 缓存；所有后继进程使用同一检查过的环境，避免构建成功却在许可收集时找不到 rustc／cargo。缺少 Docker 或 Inno Setup 只影响需要它的阶段，不阻止普通开发构建。

每条记录包含版本、下载来源、SHA256、许可证、目标、负责人、验证提交和日期。Rust/Qt 安全更新触发重新验证，不无限保留已失去维护的补丁。最低系统候选不等于正式支持承诺。

系统维护状态、Qt 安全更新渠道及升级复验责任以[支持政策](support-policy.md)为准，来源与许可结论按[发行审查流程](../qa/release-review.md)登记。

Windows 主窗口的客户区扩展由 WindowsFrame 使用公开 Qt 窗口接口和 Win32 实现，不链接 GuiPrivate。首次绑定普通窗口时按当前 QWindow 客户区逻辑尺寸、DPR 和原生边框度量确定外框，后续主题或 DPI 重应用只刷新边框。在无局部缩放覆盖、Qt PassThrough 舍入的 PMv2 普通窗口中，WM_GETDPISCALEDSIZE 保留待处理 SIZE 的缩放前调整量，缩放客户区后加目标 DPI 的实际边框；无效度量、溢出及其他状态不写入 SIZE，交回 Qt。借用的原生消息及其可写结构仅在同步回调中使用。升级 Qt 或替换 SDK 时须复验初始客户区、真实双屏往返、原生边框、DWM、最大化和全屏恢复；程序化往返不能代替物理拖动和视觉验收。

文档 CI 在 runner 临时目录中独立安装 `rust-toolchain.toml` 固定版本及 rustfmt、clippy，检查实际编译器位于该目录后再执行签名工具与其他现有测试，避免复用宿主不完整的 rustup 工具链。该隔离不跳过任何原有检查，也不解释宿主工具链此前为何不完整。

English: Documentation CI installs the pinned Rust version, rustfmt and clippy in an isolated runner-temporary rustup directory and checks the compiler path before running the existing signing-tool and other tests. Isolation does not skip checks or explain why a previous host toolchain was incomplete.
