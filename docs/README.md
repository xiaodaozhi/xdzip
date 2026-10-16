# 文档导航

维护日期：2026-10-16。产品名固定为 **xdzip**。项目采用 Qt Quick 技术路线，并已建立高保真视觉基线；M1 产品骨架已完成开发与三平台自动化验证，进入一次性人工验收。

## 阅读顺序

产品与交互贡献者：需求 → 视觉 → 交互补充 → 验收。核心贡献者：架构 → 领域 → 安全 → 桥接/格式契约 → 实施清单。维护者：开发就绪 → 风险 → 贡献规则 → 发布门槛。

| 领域 | 入口 |
| --- | --- |
| 软件设计（16 篇） | [设计总览](design/index.md)、[HTML 阅读版](html/index.html)、[单文件 HTML](html/all.html) |
| WP-11 TAR/GZIP | [流式格式与安全提交契约](engineering/m4-tar-gzip.md) |
| M2 完整只读浏览 | [资源和查询契约](engineering/m2-readonly.md) |
| 界面设计准入 | [严格设计规范](visual/design-standard.md)、[设计流程](visual/design-workflow.md)、[逻辑尺寸与密度](visual/layout-scale.json) |
| 视觉基线 | [高保真统一入口](../design/hifi/README.md)、[完整图片画廊](../design/hifi/index.html)、[视觉说明](visual/README.md)、[颜色尺寸参数](visual/tokens.json) |
| 高保真状态稿 | [页面与操作状态原稿](../design/hifi/README.md)、[统一素材库](../design/assets/index.html)、[功能状态契约](ux/hifi-state-spec.md) |
| 任务与检查窗口 | [原生窗口与状态复用契约](ux/task-windows.md) |
| HTML 高保真设计 | [复核方法](visual/html-fidelity.md)、[使用说明](../design/html/README.md)、[功能映射](ux/prototype-spec.md) |
| 中文界面术语 | [用词规范](ux/chinese-terminology.md)、[术语配置](ux/chinese-terminology.json) |
| 设置与界面语言 | [设置与国际化契约](ux/settings-localization.md)、[语言资源](../resources/i18n/README.md) |
| 主窗口组件与交互 | [功能清单与输入契约](ux/main-window-components.md)、[可操作组件页](../design/html/components.html)、[共享菜单与状态参数](visual/controls.json) |
| Figma 功能设计 | [文件、画板索引与待修项](ux/figma-design.md) |
| 未绘制页面和异常流程 | [交互补充](ux/interaction-spec.md) |
| 开发与仓库约定 | [开发指南](engineering/development.md)、[目录规划](engineering/repository-layout.md) |
| 工具链与依赖 | [工具链决策表](engineering/toolchain.md)、[依赖准入](engineering/dependencies.md)、[cxx 诊断记录](engineering/cxx-admission.md)、[许可证](engineering/licensing.md) |
| 接口审查 | [契约冻结清单](engineering/contracts.md) |
| 隐私与诊断 | [数据生命周期](engineering/privacy.md) |
| 产品版本与关于窗口 | [统一版本机制](engineering/versioning.md)、[A01 高保真](../design/hifi/about.html) |
| 持续集成与发布 | [CI 策略](engineering/ci-policy.md)、[发布流程](engineering/release-runbook.md) |
| 项目执行 | [工作清单](planning/backlog.md)、[风险与决策](planning/risks.md)、[就绪评审](planning/readiness.md) |
| 解压预检 | [阶段契约](engineering/extraction-preflight.md) |
| 目录树 B 设计交付 | [正式设计与接入规格](ux/directory-tree.md)、[逻辑参数](visual/directory-tree.json)、[图板与切图 HTML](../design/hifi/directory-tree.html) |
| ZIP 只读浏览 | [实现契约](engineering/readonly-browser.md)、[依赖准入](engineering/zip-metadata-admission.md) |
| M1 产品骨架 | [正式运行时契约](engineering/m1-runtime.md) |
| 实现进展 | [开发预览](../apps/desktop/README.md) |
| WP-12 私测准备 | [准入与桌面验证](qa/private-test-readiness.md)、[人工专项清单](qa/private-test-manual.md)、[部署与 SBOM](engineering/deployment-audit.md)、[Linux 运行目录](engineering/linux-deployment.md)、[组件与许可证据](engineering/runtime-attribution.md)、[安装与关联](engineering/installers-associations.md) |
| WP-12 本地交付 | [macOS 开发 DMG](engineering/macos-deployment.md)、[对应源码与重建材料](engineering/source-delivery.md) |
| 质量 | [验收用例](qa/acceptance.md)、[样本规范](qa/fixtures.md) |
| 开源协作 | [贡献指南](../CONTRIBUTING.md)、[治理](../GOVERNANCE.md)、[行为规范](../CODE_OF_CONDUCT.md)、[安全政策](../SECURITY.md) |
| 历史记录 | [变更日志](../CHANGELOG.md)、[准备工作记录](planning/preparation-record.md) |

测试截图与量测记录不保存在源码树，统一从[本地／CI 验证入口](qa/README.md)获取。结果窗口和创建高级选项分别见 [结果参数](visual/result-reference.json)和[创建契约](engineering/m4-creation.md)。

## 文档权威与维护

需求编号以 design/01 为准；领域与任务以 design/03、08 为准；桥接以 design/05 为准；外部格式协议以 design/06 为草案权威；视觉以已确认稿及 tokens 为准。新增工程文档补充执行细节，不另定义相冲突的协议或安全策略。

所有“计划/候选/待验证”必须保持标记，不能因文档进入主分支就改写为已实现。设计调整同时更新受影响文档与需求追踪矩阵；重大变化追加 ADR。HTML 为派生阅读材料，修改 Markdown 后重新构建。

M3 安全解压：[安全解压实现](engineering/m3-extraction.md)、[依赖准入](engineering/zip-extraction-admission.md)、[验收记录（本地／CI 工件）](qa/README.md)、[Qt／HTML／原稿对照（本地／CI 工件）](qa/README.md)。

WP-10 ZIP 创建：[输入树与整体保存契约](engineering/m4-creation.md)、[验收状态（本地／CI 工件）](qa/README.md)、[实际 Qt／HTML／原稿对照（本地／CI 工件）](qa/README.md)。进度窗口正常密度修订见 [复核记录（本地／CI 工件）](qa/README.md)。

新建窗口控件修订：[原稿参数](visual/create-reference.json)、[实际截图与验收（本地／CI 工件）](qa/README.md)、[HTML／Qt／原稿对照（本地／CI 工件）](qa/README.md)。共享控件禁止窗口覆盖 footer 样式，信息不使用中点分隔。
