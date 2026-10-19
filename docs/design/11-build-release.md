# xdzip：11 构建、发布、许可证与更新

## 1. 支持矩阵

应用显示名和可执行文件基名统一为 `xdzip`；平台工件为 `xdzip.app`、`xdzip.exe` 和 `xdzip`，Linux 桌面入口为 `xdzip.desktop`。正式 Rust 模块使用 `xdzip-` 前缀。发布使用项目现有 bundle ID，个人 GitHub 开源发行不要求付费签名身份，见[发布手册](../engineering/release-runbook.md)。首批支持范围和维护方式已由项目所有者于 2026-10-19 批准，权威说明见[支持政策](../engineering/support-policy.md)及 [ADR-011](architecture-decisions.md#adr-011首批支持范围与个人项目维护政策)。

| 平台 | 首批目标 | 最低系统与范围 | 发布核验 |
|---|---|---|---|
| macOS | Apple Silicon arm64 | macOS 14 及以上，不承诺所有未来系统版本 | 原生 GPU/IME/VoiceOver、ad-hoc 完整性、无开发环境运行 |
| Windows | x86_64 MSVC | 仍受维护的 Windows 11 | Unicode/DPI/长路径/NTFS、未签名包与干净环境 |
| Linux | x86_64 | Ubuntu 24.04 LTS、X11、glibc 2.39 及以上 | X11 平台插件、隔离运行、目标发行版原生文件 API |

首批不提供 Intel Mac、Universal、Windows ARM64 或 Linux ARM64 包，不承诺 Windows 10、其他 Linux 发行版或 Wayland。支持范围不等于已逐一覆盖所有系统版本；扩大范围须补充实际验证。个人项目优先维护最新公开版本、尽力处理反馈、不承诺固定响应时限，开发预览保持实验性。

## 2. 工具链和构建入口

产品以 CMake 组织 Qt/C++/QML，Cargo 组织 Rust，顶层脚本/预设负责 target、依赖位置及顺序。Rust 静态核心链接进应用，Qt 默认动态部署，便于许可合规和替换。平台原生 codec/helper 作为明确可审计独立工件。

当前 Rust 工具链由 rust-toolchain.toml 锁定为 1.99.0，Qt SDK 由构建配置锁定为 6.8.2；来源和哈希见[工具链策略](../engineering/toolchain.md)。升级通过对应审查与三平台复验，不把原型版本或宽泛的 ≥6.8 当作可直接替换的构建输入。Qt LTS 名称不代表所有开源分发渠道都提供相同的长期补丁。

`cxx`、Cargo、CMake、编译器、Qt 包 SHA、features 和目标架构保存于版本化构建清单。构建生成文件放 build 目录；QML 编译缓存启用与否也写入基准元数据。缓存 key 含工具链/Qt/feature，避免错误复用不同架构产物。

提供开发 Debug、诊断 ReleaseWithSymbols、发行 Release 三配置。调试符号单独存放，签名前归档；没有实际测量不启用高风险 aggressive LTO/自定义分配器。使用可解释、可回滚的 size 优化，确保 panic/崩溃可定位。

## 3. Qt 模块和部署

首批必要模块为 Core、Gui、Qml、Quick、QuickControls2 及 QML Layouts/相应 Templates、Models 等实际传递依赖。图标可需要 Svg；仅在选用时加入。避免 WebEngine、Multimedia、完整 Qt 开发工具和未使用 SQL driver。

不能仅列直接链接库就删掉所有 transitive DLL/QML 插件。部署工具扫描 QML imports 和动态依赖，在干净机器上验证。QML 包资源和 platform/imageformat 插件用明确允许清单；不从用户当前目录加载未知 Qt 插件。

macOS 使用部署工具生成 app bundle，配置相对加载路径；Windows 部署 Qt DLL/QML 和必要 MSVC runtime；Linux 采用 Ubuntu 24.04 运行目录 tar.gz 和 .deb，按[部署契约](../engineering/linux-deployment.md)收集 Qt 与 OS 依赖。部署工具的输出需审查，不假设自动满足许可、安全或最小体积。

产品安装包仅包含 Qt 产品核心和选择的后端，不包含 Slint/Tauri 原型、Python 基准环境、Cargo target、头文件或字体实验资源。源码包可保留选型实验，但必须与产品二进制包分开。

## 4. 平台发布

### macOS

直接下载使用本地 `.dmg`，按个人开源发行政策只做必要的 ad-hoc 完整性签名，不要求 Developer ID、公证或付费账号。说明未正式签名的系统提示，不要求关闭系统安全保护。权限尽量小，禁止为插件便利开启宽泛禁用保护的 entitlements。

Mac App Store 是独立发行路线，需重新评估 App Sandbox、书签授权、下载插件限制和许可；不能认为直接下载的第三方可执行扩展模式自动可用于商店版本。

### Windows

未签名的当前用户安装程序按 GitHub Releases 渠道交付，明确是否需要管理员权限，默认用户安装可行。工作进程放固定应用/版本化扩展目录；DLL 加载路径不含不可信当前目录。安装/升级过程中不覆盖仍被任务使用的插件包。

### Linux

当前交付 Ubuntu 24.04 运行目录及 .deb，不提供 AppImage 或 Flatpak。扩大渠道前须补充安装、文件访问和来源验证。针对基线系统构建，不在新 glibc 上构建后宣称老系统可运行。Wayland/X11、portal、Qt 插件和沙箱权限分别测试。AppImage/Flatpak/原生包对文件与工作进程限制不同，按发行配置暴露实际能力。

## 5. 许可证和供应链

项目原创内容采用 Apache-2.0；冻结记录见 [许可证决策](../engineering/licensing.md)，完整授权条款以根目录 LICENSE 为准。Rust crate、Qt 模块、7z/RAR 原生库、图标和打包工具各自核查，不能用项目 LICENSE 覆盖依赖条款。Qt 动态部署本身不是全部合规证明。

依赖许可审查至少包含：分发许可、版权和声明、修改说明、源码提供义务、用户替换库和重新链接条件、安装信息及反调试/签名政策的相容性。LGPL/GPL/商业方案由实际模块和发行方式决定；不能把某一 Qt 模块结论推广到全部 Qt。

7-Zip/UnRAR 或其他 RAR 实现涉及的具体条款单独评估，尤其创建 RAR 和逆向相关限制；不把它们简单标作“MIT 兼容”。不满足目标许可时不打包该能力，可作为用户自行安装的合规后端候选，仍需评估宿主分发关系。

发布提供 SBOM（如 SPDX/CycloneDX）、NOTICE、依赖版本/哈希、源码与构建说明、所需许可证材料及文件清单。静态 Rust 第三方代码同样计入 SBOM。当前不使用正式签名密钥；若未来另选签名渠道，凭据不得进入仓库或普通 PR。

## 6. 可复现和离线构建

依赖锁、来源哈希、可选 vendor 缓存和工具链清单支持离线构建。限制未经审查的 build script 下载；不能默认 cargo build 就应访问任意站点。

平台签名/公证时间戳通常会改变最终包字节，分别比较未签名工件的可复现性和已签名产物的验证结果。不要宣称所有签名包 byte-for-byte 相同。

## 7. 更新策略

v0.1 手动下载更新，无后台自动安装。v1.0 可提供可关闭的版本检查，但联网需清楚设置和最小数据，不上传用户归档信息。更新清单签名验证不只依赖 TLS，包含版本、平台/架构、最低系统、哈希和来源；拒绝回滚攻击，必要的安全回滚经可信策略授权。

更新只在任务安全结束后应用，失败保留当前版本。扩展更新独立验证协议、来源和权限变更，正在使用的旧包延后回收。离线用户可以导入核对来源和校验和的完整包，缺失网络不是解压功能不可用的理由。

## 8. 发布检查表

- 同一提交生成源包、平台包、符号、SBOM、许可证与基准报告。
- 核验格式能力清单，未验证加密/分卷不能写进“全格式支持”。
- 三平台干净机器安装、首次打开、系统关联、退出、升级/卸载通过。
- 验证源变化、输出配额、取消、恢复、安全路径及沙箱；第三方未达门槛则默认禁用。
- 记录完整压缩下载/安装体积，而非只记录主 executable。
- 验证必要 ad-hoc 完整性、未正式签名的安全提示、Qt 替换政策和所有声明；公布已知限制。

发布负责人可以拒绝不满足安全/许可门槛的功能，不能为了一个版本日期隐式降低默认保护。
