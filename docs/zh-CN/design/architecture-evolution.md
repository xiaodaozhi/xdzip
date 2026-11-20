# 架构演进与当前边界

[English](../../design/architecture-evolution.md) | 简体中文 | [繁體中文](../../zh-TW/design/architecture-evolution.md) | [日本語](../../ja-JP/design/architecture-evolution.md) | [한국어](../../ko-KR/design/architecture-evolution.md) | [Français](../../fr-FR/design/architecture-evolution.md) | [Deutsch](../../de-DE/design/architecture-evolution.md) | [Español](../../es-ES/design/architecture-evolution.md) | [Português (Brasil)](../../pt-BR/design/architecture-evolution.md) | [Русский](../../ru-RU/design/architecture-evolution.md)

## 从框架试验到现行产品

早期框架试验使用合成归档列表和模拟任务。产品选用 Qt Quick 与类型化 CXX 桥接，再由 Rust 实现归档、任务及文件操作。原型的 JSON/C ABI 和耗时数据属于当时的试验。[架构决策记录](architecture-decisions.md)保存了选择依据及复核条件。

产品使用有界行模型，并校验查询身份，避免旧结果覆盖新视图。在此基础上实现物理条目身份、显式格式能力、文件授权、暂存和验证后发布。独立 CLI 无需 Qt 即可调用同一核心。[系统架构](system-architecture.md)、[领域模型](domain-data.md)

## 当前实现、独立试验与拟议工作

| 范围 | 当前状态 |
| --- | --- |
| 运行时与索引 | 已在宿主实现。普通单文件 ZIP/ZIP64 可迁移到私有磁盘索引，任务限额和元数据限额分别计算。[性能设计](performance.md) |
| 格式隔离 | 内置路由选择进程内 Rust 实现或随包工作进程。操作是否支持，由注册表配置及运行时准入共同决定。[格式适配器](codec-extension-sdk.md) |
| 外部扩展 | 独立的只读 XDP1 试验有受限协议与平台权限门槛，通过单独宿主运行。产品内的扩展发现、安装与激活仍属拟议工作。[SDK 试验](../engineering/sdk-trial.md) |
| 长期状态与并发 | 当前调度一次只准入一个桌面重操作。跨启动的持久索引和任务恢复、通用的共享调度仍是设计选项。[任务调度](task-scheduling.md)、[文件提交](file-security.md) |

当前源码能力见[格式支持表](../../../README.zh-CN.md#格式支持)。具体发行版本以对应发布说明和附件为准。原生实现、试验工具及测量结果各有适用范围。
