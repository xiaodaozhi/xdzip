# 项目治理

[English](GOVERNANCE.md) | 简体中文

仓库所有者 xiaodaozhi 负责项目范围、发行身份、许可证决定及维护者授权。按核心、界面、平台、安全或发行等受影响领域开展审查，维护权限依据所有者实际授权。

改变已接受架构时使用[架构决策记录](docs/zh-CN/design/architecture-decisions.md)，记录背景、选项、决定、代价及替代关系。产品范围变化时更新相关需求。出现分歧时，以可复现证据和已记录的安全约束为依据，由所有者作最终范围决定。

`main` 是可审查的基线。需要审查或分支保护要求时使用 PR。按[仓库工作约定](AGENTS.md)及[交付指南](developers/zh-CN/engineering/repository-hygiene.md)将已授权改动合入 `main`，核对落地提交并完成清理。

安全相关改动须提供与范围匹配的证据。发行决定绑定具体候选，按[发布手册](developers/zh-CN/engineering/release-runbook.md)核对验证、供应链及平台证据。
