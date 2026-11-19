# 贡献指南

[English](CONTRIBUTING.md) | 简体中文

构建命令、源码入口和验证方式见[开发与贡献](docs/zh-CN/contributing/README.md)。原创贡献按项目的 [Apache-2.0 许可证](LICENSE)授权。提交前确认你有权分享相关内容，并保留第三方通知及来源说明。

## 工作流程

1. 查找或新建议题，记录范围与验收标准。重大架构变更须附架构决策记录。
2. 从最新 `main` 开始，按需使用短期 `coding/` 分支，保留工作区中的其他改动。
3. 提交前在本地构建或验证受影响路径。在 PR 中说明改动、实际检查及剩余要求。
4. 同步更新受影响的中英文文档，Git 中只保留允许再分发的测试输入。
5. 遵守仓库保护规则及约定的交付范围。维护任务通过必需检查后继续合入、远端核对与清理。见[仓库交付约定](developers/zh-CN/engineering/repository-hygiene.md)。

每次提交聚焦一项变更，标题说明具体内容，例如 `docs: clarify archive support`、`feat: add an option` 或 `fix: preserve selection`。

## 编码和测试

遵循[质量工程指南](docs/zh-CN/design/quality-engineering.md)，按需说明所有权、不变量和失败边界。性能改动应记录工作负载及前后量测。按 [CI 策略](developers/zh-CN/engineering/ci-policy.md)和[验证指南](developers/zh-CN/qa/README.md)选取与改动范围匹配的检查。

## 报告问题

使用仓库问题模板及自制的最小复现样本。分享前去除个人信息。软件漏洞按[安全政策](SECURITY.zh-CN.md)私密报告。参与项目时遵守[社区行为规范](CODE_OF_CONDUCT.zh-CN.md)。
