# 开发与发行就绪评审方法

[English](../../planning/readiness.md) | 简体中文

本页定义可重复执行的评审要求。具体阶段的完成记录、人工结论和未完成事项放在 [Issues](https://github.com/xiaodaozhi/xdzip/issues)，路线见[工作清单](../../../docs/zh-CN/design/architecture-evolution.md)，不在本页追加日期报告。

## 开发基线

- 技术路线、模块边界、接口和格式能力与[设计规范](../../../docs/zh-CN/design/index.md)一致。
- 工具链、依赖与来源按锁文件和[工具链策略](../../../docs/zh-CN/engineering/toolchain.md)固定。
- Rust、C++、Qt/QML 及 FFI 变更执行与范围匹配的 CI 和回归，记录实际限制。
- 视觉变更先走[设计流程](../../../docs/zh-CN/visual/design-workflow.md)，实际原生视觉验收与功能测试分开登记。
- 源码、测试和设计输入可以从当前仓库恢复，生成证据只进入本地或 Actions 工件。

## 工作包退出

每个工作包在 issue 明确依赖、产物、验收条件、范围外事项及实际证据。通过必需检查后，继续完成已授权的[主线合入、远端核对及清理](../engineering/repository-hygiene.md)。未解决的缺陷和许可复核单独跟踪。必要外部决定或明确交付边界阻止收尾时，记录剩余步骤并保留相关工作开放。

## 公开发行

公开发行按[发行许可复核](../qa/release-review.md)与[发布手册](../engineering/release-runbook.md)执行，核对真实包、源码、来源许可、通知、安装与回退及支持政策。开发完成或人工验收不代表发行获批；未验证项保留实际状态。正式签名和付费账号不作为本项目必做前提。
