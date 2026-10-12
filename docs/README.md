# 文档导航

基线日期：2026-10-11。产品名固定为 **xdzip**。项目采用 Qt Quick 技术路线，并已建立高保真视觉基线；M1 产品骨架已完成开发与三平台自动化验证，进入一次性人工验收。

## 阅读顺序

产品与交互贡献者：需求 → 视觉 → 交互补充 → 验收。核心贡献者：架构 → 领域 → 安全 → 桥接/格式契约 → 实施清单。维护者：开发就绪 → 风险 → 贡献规则 → 发布门槛。

| 领域 | 入口 |
| --- | --- |
| 软件设计（16 篇） | [设计总览](design/index.md)、[HTML 阅读版](html/index.html)、[单文件 HTML](html/all.html) |
| 素材修正 | [问题与修复记录](qa/asset-review.md)、[原稿与矢量逐项对照](qa/asset-review/index.html) |
| 视觉基线 | [高保真统一入口](../design/hifi/README.md)、[完整图片画廊](../design/hifi/index.html)、[视觉说明](visual/README.md)、[颜色尺寸参数](visual/tokens.json) |
| 高保真状态稿 | [v2 图板与素材](../design/hifi/v2/README.md)、[功能状态契约](ux/hifi-state-spec.md) |
| HTML 高保真设计 | [使用说明](../design/html/README.md)、[功能映射](ux/prototype-spec.md) |
| Figma 功能设计 | [文件、画板索引与待修项](ux/figma-design.md) |
| 未绘制页面和异常流程 | [交互补充](ux/interaction-spec.md) |
| 开发与仓库约定 | [开发指南](engineering/development.md)、[目录规划](engineering/repository-layout.md) |
| 工具链与依赖 | [工具链决策表](engineering/toolchain.md)、[依赖准入](engineering/dependencies.md)、[cxx 诊断记录](engineering/cxx-admission.md)、[许可证](engineering/licensing.md) |
| 接口审查 | [契约冻结清单](engineering/contracts.md) |
| 隐私与诊断 | [数据生命周期](engineering/privacy.md) |
| 持续集成与发布 | [CI 策略](engineering/ci-policy.md)、[发布流程](engineering/release-runbook.md) |
| 项目执行 | [工作清单](planning/backlog.md)、[风险与决策](planning/risks.md)、[就绪评审](planning/readiness.md) |
| 解压预检 | [阶段契约](engineering/extraction-preflight.md)、[测试与实际窗口](qa/extraction-preflight.md) |
| 目录导航与排序 | [实现、实际截图与验收](qa/navigation-sorting.md) |
| ZIP 只读浏览 | [实现契约](engineering/readonly-browser.md)、[依赖准入](engineering/zip-metadata-admission.md)、[测试与实际截图](qa/zip-browser.md) |
| M1 产品骨架 | [正式运行时契约](engineering/m1-runtime.md)、[完整验收入口](qa/m1-acceptance.md) |
| 视觉修正复核 | [布局、排版、主题修正与新截图](qa/visual-refinement.md) |
| 工具栏按钮修正 | [原稿对照与实际截图](qa/toolbar-review.md) |
| 顶部结构修正 | [实施说明与实际截图](qa/header-review.md) |
| 视觉严格复核 | [设计与实现差异报告](qa/visual-audit-review.md)、[运行时量化检查表](qa/visual-audit/measurements.md) |
| 实现进展 | [领域与 Qt 组件验证](qa/foundation-components.md)、[任务窗实际截图与对照](qa/task-visual-review.md)、[开发预览](../apps/desktop/README.md) |
| 质量 | [验收用例](qa/acceptance.md)、[样本规范](qa/fixtures.md) |
| 开源协作 | [贡献指南](../CONTRIBUTING.md)、[治理](../GOVERNANCE.md)、[行为规范](../CODE_OF_CONDUCT.md)、[安全政策](../SECURITY.md) |
| 历史记录 | [变更日志](../CHANGELOG.md)、[准备工作记录](planning/preparation-record.md) |

## 文档权威与维护

需求编号以 design/01 为准；领域与任务以 design/03、08 为准；桥接以 design/05 为准；外部格式协议以 design/06 为草案权威；视觉以已确认稿及 tokens 为准。新增工程文档补充执行细节，不另定义相冲突的协议或安全策略。

所有“计划/候选/待验证”必须保持标记，不能因文档进入主分支就改写为已实现。设计调整同时更新受影响文档与需求追踪矩阵；重大变化追加 ADR。HTML 为派生阅读材料，修改 Markdown 后重新构建。
