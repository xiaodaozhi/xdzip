# 构建、部署与发行架构

[English](../../design/build-release.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/design/build-release.md) | [日本語](../../ja-JP/design/build-release.md) | [한국어](../../ko-KR/design/build-release.md) | [Français](../../fr-FR/design/build-release.md) | [Deutsch](../../de-DE/design/build-release.md) | [Español](../../es-ES/design/build-release.md) | [Português (Brasil)](../../pt-BR/design/build-release.md) | [Русский](../../ru-RU/design/build-release.md)

## 构建与打包输入

CMake 构建 Qt 桌面、C++ 适配层及原生助手。Cargo 构建 Rust 运行时并生成要链接的 CXX 桥接。独立 CLI 使用自己的构建入口，无需 Qt。`scripts/desktop.py` 与 `scripts/cli.py` 组织构建和测试。当前源码锁定 Rust 1.99.0 和 Qt 6.8.2 SDK。比较结果时要同时核对构建身份、目标架构、锁文件和工具链来源。[工具链](../engineering/toolchain.md)、[开发构建指南](../engineering/developer-build.md)

桌面部署各平台包需要的 Qt 库、QML 模块、系统资源和随包助手。专用图片进程使用 Qt 进行图片解码及静态 SVG 渲染。CLI 包携带所需的归档工作进程。构建配置、打包清单和部署检查由脚本和平台契约维护。单个可执行文件的大小无法说明完整依赖或安装体积。[部署与依赖审计](../engineering/deployment-audit.md)、[CLI 分发](../engineering/command-line.md)

## 当前分发渠道

| 目标 | 安装包与信任边界 |
| --- | --- |
| macOS Apple Silicon | GitHub 提供 DMG 和应用包 ZIP。目标基线为 macOS 14，应用包使用 ad-hoc 完整性签名。安装和更新行为见平台说明。 |
| Windows 11 x86_64 | GitHub 提供未签名的每用户安装程序及便携 ZIP。Microsoft Store 管理 MSIX 的包身份、签名和更新渠道。 |
| Ubuntu 24.04 LTS x86_64 | GitHub 提供运行时目录压缩包和 DEB。支持基线包含 X11 及 glibc 2.39 或更高版本。 |
| 独立 CLI | 单独分发，拥有自己的包与更新身份。桌面安装中附带的 CLI 遵循其所在安装的集成和交接规则。 |

[支持政策](../engineering/support-policy.md)界定系统及架构范围。[项目首页](../../../README.zh-CN.md)与对应发布说明列出实际附件，[安装程序指南](../engineering/installers-associations.md)说明文件关联与集成限制。

## 常规检查、原生构建与发行

常规 [CI 工作流](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml)执行轻量检查。原生 Foundation 和 Qt 检查由独立工作流手动运行。普通 Apple Silicon CI 执行编译、测试和启动验证，无需部署可供下载的应用包。专用发行工作流按授权执行部署、签名、打包和发布。[发行协议](../engineering/release-protocol.md)和 [CI 策略](../../../developers/zh-CN/engineering/ci-policy.md)定义门槛与证据。

发行清单、校验和、组件清单、SBOM 和许可材料描述实际交付的工件。发行工作流不生成或上传项目源码压缩包。GitHub 自带的标签快照通过独立机制提供。依赖义务依据实际随包组件与项目的[许可记录](../engineering/licensing.md)和[源码交付政策](../engineering/source-delivery.md)。

## 更新与后续分发

更新选择和验证独立于归档任务。符合条件的安装版按更新契约核对可信元数据、已验证工件和安装身份，并按活动任务状态完成交接。Windows 和 Linux 使用对应的助手流程，macOS 使用 Sparkle，商店版由 Microsoft Store 提供更新。便携与开发配置各有限制。[软件更新](../engineering/software-update.md)

扩大架构、系统基线、包格式或商店范围时，需要独立的构建和验收证据。历史发行决策与复核触发条件见[架构决策记录](architecture-decisions.md)。
