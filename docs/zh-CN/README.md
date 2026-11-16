# xdzip 文档

[English](../README.md) | 简体中文

这里汇集了使用 xdzip、构建项目和了解压缩格式所需的资料。第一次使用，可以先从[项目首页](../../README.zh-CN.md)了解功能、下载方式和支持的系统。

## 使用 xdzip

| 我想…… | 查看 |
| --- | --- |
| 安装或设置文件关联 | [安装与文件关联](engineering/installers-associations.md)、[系统支持范围](engineering/support-policy.md) |
| 使用命令行 | [命令行指南](engineering/command-line.md) |
| 了解文件拖放和包内修改 | [文件拖放与修改](engineering/archive-transfers.md) |
| 了解密码、分卷和文件名编码 | [密码解压](engineering/password-extraction.md)、[分卷与临时打开](engineering/archive-opening.md)、[文件名编码](engineering/filename-encoding.md) |
| 检查压缩包或预览文件 | [完整性检查](engineering/archive-checking.md)、[内容预览](engineering/content-preview.md) |
| 了解大压缩包的限制 | [大归档索引](engineering/large-index.md) |
| 从文件管理器直接压缩或解压 | [文件管理器操作](engineering/shell-actions.md) |
| 了解数据处理和隐私 | [隐私政策](../../PRIVACY.md)、[数据生命周期与诊断](engineering/privacy.md) |
| 检查更新 | [用户主动更新](engineering/software-update.md)、[发布页](https://github.com/xiaodaozhi/xdzip/releases) |
| 准备商店一览 | [Windows 商店文案、截图、宣传图和图标](market/windows/README.md) |

项目首页的[格式支持表](../../README.zh-CN.md#格式支持)列出当前源码支持的格式、操作及限制。发行包的实际能力以对应版本的发布说明为准。

## 开发与贡献

- [开发构建指南](engineering/developer-build.md)：准备环境，构建、运行和测试桌面应用及命令行工具。
- [开发者准备指南](engineering/development.md)和[仓库结构](engineering/repository-layout.md)：了解开发流程、模块分工和验证入口。
- [贡献指南](../../CONTRIBUTING.md)、[行为规范](../../CODE_OF_CONDUCT.md)和[安全政策](../../SECURITY.md)：提交修改、参与协作和报告安全问题。
- [验证方法](qa/README.md)和 [CI 策略](engineering/ci-policy.md)：根据改动选择检查，并区分自动化与实际桌面验证。
- [系统架构](design/02-system-architecture.md)、[领域与数据](design/03-domain-data.md)和 [Qt 与 Rust 桥接](design/05-qt-rust-bridge.md)：了解主要模块及其边界。
- [界面设计规范](visual/design-standard.md)、[设计流程](visual/design-workflow.md)和[交互原型](ux/prototype-spec.md)：查看界面、组件和交互约定。

## 格式与实现细节

这些文档适合需要了解格式能力、边界或实现依据的读者。具体支持范围仍以[格式支持表](../../README.zh-CN.md#格式支持)为准。

- **读取、解压和创建**：[浏览与索引](engineering/readonly-browser.md)、[安全解压](engineering/m3-extraction.md)、[ZIP 创建](engineering/m4-creation.md)、[ISO 创建](engineering/iso-creation.md)、[Windows 自解压包](engineering/sfx-creation.md)。
- **常见压缩流**：[TAR 与 GZIP](engineering/m4-tar-gzip.md)、[单文件压缩流](engineering/single-file-streams.md)、[LZ4](engineering/lz4-streams.md)、[Zstandard](engineering/zstandard-streams.md)、[Brotli](engineering/brotli-streams.md)、[Unix compress](engineering/compress-streams.md)。
- **归档和封装格式**：[7z 实现依据](engineering/sevenzip-format-sources.md)、[RAR 实现依据](engineering/rar-format-sources.md)、[CAB 实现依据](engineering/cab-format-sources.md)、[DEB](engineering/deb-containers.md)、[ASAR](engineering/asar-containers.md)、[AES Crypt](engineering/aescrypt-envelopes.md)、[UU 与 XXE](engineering/text-envelopes.md)。
- **其他格式**：[LHA/LZH](engineering/lha-containers.md)、[PMA](engineering/pma-streams.md)、[ARJ](engineering/arj-streams.md)、[ARC](engineering/arc-streams.md)、[BH](engineering/bh-containers.md)、[ACE、ALZ 与 EGG](engineering/ace-alz-egg-format-sources.md)。
- **设计与依赖**：[格式后端](engineering/native-codecs.md)、[自研格式与算法](engineering/independent-rust-codecs.md)、[格式扩展设计](design/06-codec-extension-sdk.md)、[依赖与许可](engineering/dependencies.md)。

## 发布与项目协作

维护者可以查阅[版本机制](engineering/versioning.md)、[发布协议](engineering/release-protocol.md)、[发布手册](engineering/release-runbook.md)和[发行审查](qa/release-review.md)。项目计划、问题与反馈见 [GitHub Issues](https://github.com/xiaodaozhi/xdzip/issues)，版本变化见[更新日志](../../CHANGELOG.md)。
