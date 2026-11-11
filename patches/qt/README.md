# QtNetwork 安全补丁

桌面继续使用 Qt 6.8.2，只有 QtNetwork 动态库替换为 `6.8.2-xdzip-network.1`。
原始 QtCore、其他模块、TLS 插件和 SDK 声明保持各自原有来源，不声明整套 SDK 已从源码重建。

来源、下载地址、SHA-256 和顺序固定在
[qt-network-security.json](../../scripts/qt-network-security.json)。QtBase 原始源码的地址、摘要和提交固定在
[corresponding-sources.json](../../scripts/corresponding-sources.json)。五份补丁完整保留 Qt 作者、许可和原始字节：

1. [CVE-2026-116337 官方通告](https://www.qt.io/blog/security-issue-cve-2026-116337?hsLang=en)，先应用 `0002` 限制接收中的响应头，再应用 `0001` 限制状态行。涵盖普通 HTTP 和 HTTP CONNECT 的共同读取边界。
2. [CVE-2026-105946 官方通告](https://www.qt.io/blog/security-advisory-cve-2026-105946)，按 `0001`、`0002`、`0003` 顺序修复 NTLM 数据长度、Digest 指针运算和 NTLM 偏移相加。应用未提供凭据不能排除 Qt 自动认证路径。

## 构建与替换

从对应发行标签获取本仓库即可取得完整补丁、构建脚本及其版本：

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
git checkout v0.2.0
python3 scripts/desktop.py prepare --install-qt
python3 scripts/prepare-qt-network.py --qt-prefix build/toolchain/Qt/6.8.2/macos --check-only
```

Windows 将末尾平台目录改为 `msvc2022_64`，Linux 改为 `gcc_64`。需要原生平台编译器、CMake 3.24+、Ninja、Perl、make（Windows 为 nmake）和 OpenSSL 命令行工具；构建脚本使用 CMake 的 `--fresh`，实际要求 CMake 3.24+。Ubuntu 还需 `zlib1g-dev libpcre2-dev libicu-dev libglib2.0-dev libzstd-dev libkrb5-dev`，以保留 SDK 的核心构建特性。macOS 只构建 arm64，最低部署目标保持 14.0；这不代表已在 macOS 14 验证。

入口校验源码和补丁摘要，精确匹配补丁上下文，从 QtBase 根配置只构建 Network 及必需的构建依赖。只替换 QtNetwork，不安装重建的 QtCore。必须保留原库所有导出符号、对原 QtCore 的符号兼容性及全部 QtNetwork 功能开关。TLS、DTLS、OCSP 等原有功能不能因安全修复被移除。macOS 框架作必要 ad-hoc 完整性签名，不宣称 Developer ID 或公证。

OpenSSL 3.5.9 仅生成编译所需声明，固定来源和摘要在同一配置中；不编译或分发它的密码库，运行时继续采用 SDK 的原有 TLS 后端与平台依赖。该构建输入不代表随包 OpenSSL 版本发生更新。

替换后检查实际加载库的摘要，执行有限回环 HTTP、分片输入、CONNECT、HTTPS、受信测试 CA 与不受信证书拒绝，并执行补丁内的上游认证测试及正常认证向量。失败时恢复原库、框架封印和回执；进程被终止后，缺失或与库摘要不符的回执会阻止后续产品配置与打包。删除失败的专用构建目录后可重新准备，禁止将缺失回执当作已修复。

SDK 的 `xdzip-network-security.json` 绑定源码、补丁、构建器、原 QtCore、实际 QtNetwork、功能开关和真实验证记录。发行材料 `notices/Qt-network-security/` 包含回执、完整补丁和本说明，实际部署归属记录将它与最终库连接，macOS 的重定位与重签另由部署核验记录。

macOS 的 `qmlimportscanner`、`qmlcachegen` 和 `qmllint` 原有供应商签名启用了库验证，会拒绝本地补丁库的 Team ID。准备阶段只对这三个开发工具作 ad-hoc 重签，去除其运行时签名选项，记录原始和实际摘要，验证启动并在失败时恢复。此处理不改变产品签名、TLS 或运行库的安全策略；其他 SDK 工具保持原样。

Linux 另有两个受摘要约束的项目原始兼容输入：[实例化单元](../../tools/qt-network-security/linux-abi-retain.cpp)和[CMake 注入](../../tools/qt-network-security/retain-linux-abi.cmake)。它们仅显式实例化并保留原 SDK 导出的 57 个 QtCore 内建类型 constexpr 常量，值仍来自同一 Qt 头文件；没有网络逻辑或可执行初始化。该单元不会进入 Mac/Windows 产品库。上游五份补丁保持原样，运行库来源记录、通知和源码获取说明同时包含这两个兼容输入；原库所有导出及原 QtCore 导入的检查继续强制执行。本机可验证编译、链接和保留常量，但 Linux 结论仍由实际 Linux SDK 构建给出。

English: Two hash-pinned original compatibility inputs retain the Linux SDK’s 57 exported builtin constexpr constants by explicit instantiation from the same Qt headers. They add no networking logic or executable initialization, and are not injected into Mac/Windows product libraries. Notices and source provenance include both inputs alongside the unmodified official patches. Complete export and Core-import checks remain mandatory; native Mac fixture checks do not substitute for Linux verification.

## 许可与安装信息

Linux 读取原 SDK 的 separate-debug 配置，采用相应的 `RelWithDebInfo` 构建并保留链接器选择和已导出的边界符号；构建模式与原 SDK 对齐，编译器仍会省略的模板常量由显式实例化单元保留。原库导出必须完整保留，不能以“私有符号”名义跳过 ABI 失败。Windows 探针将源码参数作为规范化 CMake PATH 传入，避免反斜线被解释为转义。

English: Linux retains the SDK's separate-debug build mode, linker selection and exported boundary symbols; missing exports remain fatal. Windows probe source paths use a normalized CMake PATH to avoid backslash escape errors.

QtNetwork 生产源码使用其原有 LGPL-3.0/GPL/商业多重授权，本项目继续选择动态链接的 LGPL-3.0 路径。原文及 Qt 第三方通知随 `notices/Qt-6.8.2` 和 `Qt-runtime` 提供；保留补丁中的作者和版权。补丁内上游测试文件单独标有 GPL-3.0-only，它们仅用于开发验证，不进入应用或发行材料中的可执行文件。

修改库的完整源码可由材料内的固定 QtBase 下载地址、这里的五份补丁和两个 ABI 兼容输入获得，构建入口、配置和对应标签公开可取，不提供额外的对应源码压缩包。替换库后的部署、重链接、macOS 本地完整性签名与应用重签方法见
[SDK 对应核验](../../docs/engineering/qt-sdk-correspondence.md)、[源码与重链接说明](../../docs/engineering/source-delivery.md)和
[macOS 部署](../../docs/engineering/macos-deployment.md)。原 SDK 的 SPDX 摘要继续保留，不能冒充修改库的字节来源证明。

English: Only QtNetwork is rebuilt from pinned QtBase 6.8.2 plus the five unmodified official patches, in the order recorded in `scripts/qt-network-security.json`. The original QtCore, other modules and TLS backends retain their existing provenance. Run the shared `desktop.py prepare` entry on each native platform; it preserves features and exported symbols, verifies imports against the installed QtCore, runs bounded HTTP/CONNECT/TLS and upstream authentication controls against the actual loaded library, and rolls back on failure. Only the three macOS QML build tools that load Network are locally ad-hoc re-sealed, with before/after hashes and rollback; product hardening policies are unchanged. macOS remains arm64 with deployment target 14.0; macOS 14 execution remains unverified. OpenSSL 3.5.9 provides generated build-time declarations only, with no new crypto library distributed. The source archive URL/hash, complete patches, both ABI compatibility inputs, build entry and immutable release tag provide source access; no corresponding-source archive is generated or attached. LGPL notices, replacement/relinking instructions and modified-library receipts remain part of release materials. GPL-only upstream test executables are never distributed.
