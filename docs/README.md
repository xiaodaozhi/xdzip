# 文档导航

基线日期：2026-10-11。产品名固定为 **xdzip**。项目采用 Qt Quick 技术路线，并已建立高保真视觉基线，当前处于开发前准备阶段。

## 阅读顺序

产品与交互贡献者：需求 → 视觉 → 交互补充 → 验收。核心贡献者：架构 → 领域 → 安全 → 桥接/格式契约 → 实施清单。维护者：开发就绪 → 风险 → 贡献规则 → 发布门槛。

| 领域 | 入口 |
| --- | --- |
| 软件设计（16 篇） | [设计总览](design/index.md)、[HTML 阅读版](html/index.html)、[单文件 HTML](html/all.html) |
| 视觉基线 | [交付说明](visual/README.md)、[HTML 画廊](visual/index.html)、[颜色尺寸参数](visual/tokens.json) |
| 高保真状态稿 | [v2 图板与素材](../design/hifi/v2/README.md)、[功能状态契约](ux/hifi-state-spec.md) |
| 交互原型 | [使用说明](../design/prototype/README.md)、[功能映射](ux/prototype-spec.md) |
| Figma 功能设计 | [文件、画板索引与待修项](ux/figma-design.md) |
| 未绘制页面和异常流程 | [交互补充](ux/interaction-spec.md) |
| 开发与仓库约定 | [开发指南](engineering/development.md)、[目录规划](engineering/repository-layout.md) |
| 工具链与依赖 | [工具链决策表](engineering/toolchain.md)、[依赖准入](engineering/dependencies.md)、[许可证](engineering/licensing.md) |
| 接口审查 | [契约冻结清单](engineering/contracts.md) |
| 隐私与诊断 | [数据生命周期](engineering/privacy.md) |
| 持续集成与发布 | [CI 策略](engineering/ci-policy.md)、[发布流程](engineering/release-runbook.md) |
| 项目执行 | [工作清单](planning/backlog.md)、[风险与决策](planning/risks.md)、[就绪评审](planning/readiness.md) |
| 质量 | [验收用例](qa/acceptance.md)、[样本规范](qa/fixtures.md) |
| 开源协作 | [贡献指南](../CONTRIBUTING.md)、[治理](../GOVERNANCE.md)、[行为规范](../CODE_OF_CONDUCT.md)、[安全政策](../SECURITY.md) |
| 历史记录 | [变更日志](../CHANGELOG.md)、[准备工作记录](planning/preparation-record.md) |

## 文档权威与维护

需求编号以 design/01 为准；领域与任务以 design/03、08 为准；桥接以 design/05 为准；外部格式协议以 design/06 为草案权威；视觉以已确认稿及 tokens 为准。新增工程文档补充执行细节，不另定义相冲突的协议或安全策略。

所有“计划/候选/待验证”必须保持标记，不能因文档进入主分支就改写为已实现。设计调整同时更新受影响文档与需求追踪矩阵；重大变化追加 ADR。HTML 为派生阅读材料，修改 Markdown 后重新构建。
