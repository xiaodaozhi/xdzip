# 文档导航

文档保存通用规则、设计和可复用方法。具体阶段进度与验收记录见 [Issues](https://github.com/xiaodaozhi/xdzip/issues)，执行证据见 [Actions](https://github.com/xiaodaozhi/xdzip/actions)或本地工件，不在 docs 累积报告。

| 领域 | 入口 |
| --- | --- |
| 产品与架构 | [设计总览](design/index.md)、[架构决策](design/architecture-decisions.md)、[需求追踪](design/requirements-traceability.md) |
| 界面规范 | [严格设计规范](visual/design-standard.md)、[设计流程](visual/design-workflow.md)、[逻辑尺寸](visual/layout-scale.json)、[视觉参数](visual/tokens.json) |
| 高保真与素材 | [原稿入口](../design/hifi/README.md)、[可操作 HTML 设计](../design/html/README.md)、[素材目录](../design/assets/index.html) |
| 交互契约 | [主窗口](ux/main-window-components.md)、[任务窗口](ux/task-windows.md)、[密码解压](ux/password-extraction.md)、[系统集成](ux/desktop-integration.md)、[设置与国际化](ux/settings-localization.md)、[中文术语](ux/chinese-terminology.md) |
| 浏览与任务 | [运行时](engineering/m1-runtime.md)、[只读索引](engineering/m2-readonly.md)、[浏览契约](engineering/readonly-browser.md)、[解压预检](engineering/extraction-preflight.md) |
| 文件与格式 | [安全解压](engineering/m3-extraction.md)、[密码解压](engineering/password-extraction.md)、[ZIP 创建](engineering/m4-creation.md)、[TAR/GZIP](engineering/m4-tar-gzip.md) |
| 工具链与依赖 | [工具链策略](engineering/toolchain.md)、[依赖准入](engineering/dependencies.md)、[cxx 桥接依据](engineering/cxx-admission.md)、[许可](engineering/licensing.md) |
| 构建与验证 | [统一构建入口](engineering/developer-build.md)、[开发指南](engineering/development.md)、[Qt 应用模块](../apps/desktop/README.md)、[CI 策略](engineering/ci-policy.md)、[验证方法](qa/README.md) |
| 部署与来源 | [部署核验](engineering/deployment-audit.md)、[运行时归属](engineering/runtime-attribution.md)、[Linux 部署](engineering/linux-deployment.md)、[Mac DMG](engineering/macos-deployment.md)、[安装与关联](engineering/installers-associations.md) |
| 源码与候选 | [对应源码](engineering/source-delivery.md)、[SDK 对应核验](engineering/qt-sdk-correspondence.md)、[自动证据汇总](engineering/wp12-automation.md)、[候选材料](engineering/release-candidate.md) |
| 版本与发行 | [版本机制](engineering/versioning.md)、[发布协议](engineering/release-protocol.md)、[应用会话](engineering/application-sessions.md)、[支持政策](engineering/support-policy.md)、[发行审查流程](qa/release-review.md)、[发布手册](engineering/release-runbook.md) |
| 执行与风险 | [工作包路线](planning/backlog.md)、[风险控制](planning/risks.md)、[就绪评审方法](planning/readiness.md) |
| 仓库管理 | [内容与历史管理](engineering/repository-hygiene.md)、[目录与依赖](engineering/repository-layout.md)、[贡献指南](../CONTRIBUTING.md)、[变更日志](../CHANGELOG.md) |
| 开源协作 | [治理](../GOVERNANCE.md)、[行为规范](../CODE_OF_CONDUCT.md)、[安全政策](../SECURITY.md) |

## 按任务阅读

- 使用与构建：先读根目录 [README](../README.md)和[开发构建指南](engineering/developer-build.md)，使用同一入口分阶段准备、构建、测试、打包与验证；平台部署文档只补充差异，Qt 应用文档说明模块与低层调试。
- 修改核心：按[实际模块目录](engineering/repository-layout.md)定位，再读对应浏览、预检、创建或解压契约；设计中的外部 SDK、磁盘索引和内容预览仍属于后续能力。
- 修改界面：先读视觉规范与流程，原稿位于 design/hifi，行为参照 ux 文档；HTML 示例与 Qt 实际能力分别核对。
- 准备交付：按源码、部署、安装契约及发行手册复验，区分开发验收与独立发行审查。支持范围以[已批准政策](engineering/support-policy.md)为准，不以 CI runner 名称推断。

新缺陷与任务先查重并记录到 issue。已完成的 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)不承接后续新待办；独立发行审查见 [#4](https://github.com/xiaodaozhi/xdzip/issues/4)，文件输入与 macOS ACL 修复的范围及验证见 [#6](https://github.com/xiaodaozhi/xdzip/issues/6)。这些入口用于查阅实际状态，本文不复制执行报告。

## 文档权威

需求以 design/01 为准；领域与任务以 design/03、08 为准；桥接以 design/05 为准；格式扩展以 design/06 为准；视觉以已确认原稿和 tokens 为准。工程文档补充实施边界，不另定义冲突协议。候选、未实现与未验证内容保持明确标记，不因文档存在便写成已实现。

设计 Markdown 是阅读版的源。执行 `python3 scripts/build-design-html.py` 可在忽略的 `build/docs-html/` 生成多页与单文件 HTML；生成物不提交到 Git。
