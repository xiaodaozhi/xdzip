# 文档导航

[English](../README.md) | [简体中文](README.md)

文档保存当前源码的通用设计、实现契约和可复用方法。`docs/` 只存放 Markdown 与必要图片；供生成器、产品资源和检查工具使用的机器可读输入位于 `specs/`，按视觉、术语和工程依赖分类。用户安装入口与版本区别见根目录 [README](../../README.zh-CN.md)，具体版本以[发布页](https://github.com/xiaodaozhi/xdzip/releases)为准。进度与验收结论记录在 [Issues](https://github.com/xiaodaozhi/xdzip/issues)，执行证据保存在 [Actions](https://github.com/xiaodaozhi/xdzip/actions)或本地工件，不在 docs 累积报告。

## 从哪里开始

- **安装和使用**：阅读 [README](../../README.zh-CN.md)、[安装与文件关联](engineering/installers-associations.md)和[支持政策](engineering/support-policy.md)。[命令行工具](engineering/command-line.md)自 `v0.2.0` 起提供独立发布包，旧版附件保持原样。
- **隐私与数据处理**：面向用户的[隐私政策](../../PRIVACY.md)与 [English 版本](../../PRIVACY.en.md)，工程实现边界见[数据生命周期与诊断](engineering/privacy.md)。
- **构建和贡献**：从[开发构建指南](engineering/developer-build.md)准备环境，按[仓库结构](engineering/repository-layout.md)定位代码，再按[验证方法](qa/README.md)选择检查。
- **修改产品行为**：先查下方的浏览、任务、格式和交互契约，区分当前实现、设计和未验证内容。
- **修改界面**：先读[视觉规范](visual/design-standard.md)和[设计流程](visual/design-workflow.md)，再查高保真原稿及相关交互契约。
- **准备发行**：阅读[发布协议](engineering/release-protocol.md)与[发行审查流程](qa/release-review.md)，具体候选的证据和决定以 issue 为准。

## 主题索引

| 领域 | 入口 |
| --- | --- |
| 命令行 | [独立 CLI、脚本接口与安装更新](engineering/command-line.md) |
| 产品与架构 | [设计总览](design/index.md)、[架构决策](design/architecture-decisions.md)、[需求追踪](design/requirements-traceability.md) |
| 界面规范 | [严格设计规范](visual/design-standard.md)、[设计流程](visual/design-workflow.md)、[逻辑尺寸](../../specs/visual/layout-scale.json)、[视觉参数](../../specs/visual/tokens.json) |
| 高保真与素材 | [原稿入口](../../design/hifi/README.md)、[可操作 HTML 设计](../../design/html/README.md)、[素材目录](../../design/assets/index.html) |
| 交互契约 | [主窗口](ux/main-window-components.md)、[任务窗口](ux/task-windows.md)、[密码解压](ux/password-extraction.md)、[系统集成](ux/desktop-integration.md)、[文件管理器右键操作](engineering/shell-actions.md)、[设置与国际化](ux/settings-localization.md)、[中文术语](ux/chinese-terminology.md) |
| 浏览与任务 | [运行时](engineering/m1-runtime.md)、[只读索引](engineering/m2-readonly.md)、[大归档索引](engineering/large-index.md)、[主动预览](engineering/content-preview.md)、[浏览契约](engineering/readonly-browser.md)、[解压预检](engineering/extraction-preflight.md) |
| 文件与格式 | [文件名编码](engineering/filename-encoding.md)、[分卷与临时打开](engineering/archive-opening.md)、[文件拖放与修改](engineering/archive-transfers.md)、[完整性检查](engineering/archive-checking.md)、[安全解压](engineering/m3-extraction.md)、[密码解压](engineering/password-extraction.md)、[ZIP 创建](engineering/m4-creation.md)、[TAR/GZIP](engineering/m4-tar-gzip.md)、[CAB 来源与实现依据](engineering/cab-format-sources.md) |
| 工具链与依赖 | [工具链策略](engineering/toolchain.md)、[依赖准入](engineering/dependencies.md)、[cxx 桥接依据](engineering/cxx-admission.md)、[许可](engineering/licensing.md)、[Windows 许可](engineering/windows-licensing.md) |
| 构建与验证 | [统一构建入口](engineering/developer-build.md)、[开发指南](engineering/development.md)、[Qt 应用模块](../../apps/desktop/README.md)、[CI 策略](engineering/ci-policy.md)、[验证方法](qa/README.md) |
| 部署与来源 | [部署核验](engineering/deployment-audit.md)、[运行时归属](engineering/runtime-attribution.md)、[原生库材料](engineering/native-runtime-materials.md)、[Linux 部署](engineering/linux-deployment.md)、[Mac DMG](engineering/macos-deployment.md)、[安装与关联](engineering/installers-associations.md) |
| 源码与候选 | [对应源码](engineering/source-delivery.md)、[SDK 对应核验](engineering/qt-sdk-correspondence.md)、[自动证据汇总](engineering/wp12-automation.md)、[候选材料](engineering/release-candidate.md) |
| 版本与发行 | [版本机制](engineering/versioning.md)、[主动更新](engineering/software-update.md)、[发布协议](engineering/release-protocol.md)、[应用会话](engineering/application-sessions.md)、[支持政策](engineering/support-policy.md)、[发行审查流程](qa/release-review.md)、[发布手册](engineering/release-runbook.md) |
| 执行与风险 | [工作包路线](planning/backlog.md)、[风险控制](planning/risks.md)、[就绪评审方法](planning/readiness.md) |
| 仓库管理 | [内容与历史管理](engineering/repository-hygiene.md)、[目录与依赖](engineering/repository-layout.md)、[贡献指南](../../CONTRIBUTING.md)、[变更日志](../../CHANGELOG.md) |
| 开源协作 | [治理](../../GOVERNANCE.md)、[行为规范](../../CODE_OF_CONDUCT.md)、[安全政策](../../SECURITY.md) |

## 按任务阅读

- 使用与构建：先读根目录 [README](../../README.zh-CN.md)和[开发构建指南](engineering/developer-build.md)，使用同一入口分阶段准备、构建、测试、打包与验证；平台部署文档只补充差异，Qt 应用文档说明模块与低层调试。
- 修改核心：按[实际模块目录](engineering/repository-layout.md)定位，再读对应浏览、预检、创建或解压契约；大归档索引和主动预览按对应工程契约声明范围，外部 SDK 的目标协议与宿主设计另见格式扩展规范。
- 隔离格式工作进程：Rust 7z/RAR/CAB/ISO/UDF 的受控流、工作进程和许可边界见[格式后端契约](engineering/native-codecs.md)，产品不编译官方 UnRAR/7-Zip 来源，不将格式名称等同于全部特性已验收。
- 外部扩展：独立[只读 SDK 试验](engineering/sdk-trial.md)包含两个独立协议实现、受控 I/O 和沙箱门禁；桌面与 CLI 不自动启用第三方包。
- 自研 Rust 后端：独立实现来源、逐格式替代与生产依赖退出门槛见[自研格式与算法契约](engineering/independent-rust-codecs.md)。
- 宿主日历与编码：格式无关的可选 `ReadSource` 服务、OS 回调、借用与秘密缓冲边界见[宿主平台服务](engineering/host-platform-services.md)。
- 7z 后端：流式方法图、共享算法、创建、重写和密码封装见[读写规格](engineering/independent-rust-codecs.md#7z读写规格)，来源与参考分析分工见[7z 实现依据](engineering/sevenzip-format-sources.md)。产品固定使用 Rust，官方 7-Zip 源码不进入产品编译或部署。
- CAB 后端：只读容器、共享算法和完整帧工作计量见[自研规格](engineering/independent-rust-codecs.md#cab只读规格)，规范版本与参考分析边界见[来源记录](engineering/cab-format-sources.md)。产品固定使用 Rust，官方 Cab 处理器及 CAB 专用 LZX/Quantum 不进入构建，没有失败回退。
- 修改界面：先读视觉规范与流程，原稿位于 design/hifi，行为参照 ux 文档；HTML 示例与 Qt 实际能力分别核对。
- 准备交付：按部署、安装契约及发行手册复验，区分开发验收与发行许可复核。正式 Release 不生成或附带对应源码压缩包；常规发行验收不运行源码交付收集器，仅在用户针对源码收集另行明确要求时用于本地审查或重建验证。支持范围以[已批准政策](engineering/support-policy.md)为准，不以 CI runner 名称推断。

新缺陷与任务先查重并记录到 issue。已完成的 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)不承接后续新待办；`v0.1.0` 的历史发行许可复核见 [#4](https://github.com/xiaodaozhi/xdzip/issues/4) 和 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)，文件输入与 macOS ACL 修复的范围及验证见 [#6](https://github.com/xiaodaozhi/xdzip/issues/6)。这些入口用于查阅实际状态，未来候选须建立新的发行 issue，本文不复制执行报告。

## 文档权威

需求以 design/01 为准；领域与任务以 design/03、08 为准；桥接以 design/05 为准；格式扩展以 design/06 为准；视觉以已确认原稿和 tokens 为准。工程文档补充实施边界，不另定义冲突协议。候选、未实现与未验证内容保持明确标记，不因文档存在便写成已实现。

`docs/zh-CN/design/` 中的中文设计 Markdown 是中文 HTML 阅读版的源。执行 `python3 scripts/build-design-html.py` 可在忽略的 `build/docs-html/` 生成多页与单文件 HTML；生成物不提交到 Git。
