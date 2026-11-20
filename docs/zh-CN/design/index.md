# XDZip 设计指南

[English](../../design/index.md) | 简体中文 | [繁體中文](../../zh-TW/design/index.md) | [日本語](../../ja-JP/design/index.md) | [한국어](../../ko-KR/design/index.md) | [Français](../../fr-FR/design/index.md) | [Deutsch](../../de-DE/design/index.md) | [Español](../../es-ES/design/index.md) | [Português (Brasil)](../../pt-BR/design/index.md) | [Русский](../../ru-RU/design/index.md)

从[系统架构](system-architecture.md)了解 Qt Quick 桌面与独立 Rust CLI 如何调用归档运行时。这套文档说明当前实现契约、架构决策和拟议工作的范围。安装与日常操作见[用户手册](../user-guide/README.md)。需要构建或修改项目时，见[开发与贡献](../contributing/README.md)。

## 按主题阅读

| 主题 | 文档 |
| --- | --- |
| 产品行为 | [产品要求](product-requirements.md)定义预期操作和范围，[需求追踪](requirements-traceability.md)提供对应的验证入口。 |
| 归档数据 | [领域模型](domain-data.md)描述会话、来源版本、物理条目、查询及操作输入。 |
| 桌面 | [Qt Quick 界面](qt-quick-ui.md)说明界面、控制器职责和交互，[Qt/Rust 桥接](qt-rust-bridge.md)列出现行命令、身份、事件及生命周期。 |
| 格式与扩展边界 | [格式适配器与扩展试验](codec-extension-sdk.md)介绍内置注册表、随包工作进程和独立 XDP1 试验。 |
| 写出与后台任务 | [文件授权与安全写出](file-security.md)覆盖暂存、验证和提交，[任务调度](task-scheduling.md)说明准入、提示、进度及取消。 |
| 性能与质量 | [性能设计](performance.md)说明预算、测量与目标，[质量工程](quality-engineering.md)将变更关联到对应验证。 |
| 交付与决策 | [构建、部署与发行](build-release.md)涵盖平台、打包和更新。[架构决策](architecture-decisions.md)记录取舍缘由，[架构演进](architecture-evolution.md)说明原型选择如何形成现行产品。 |
| 术语 | [术语与实现限额](glossary.md)解释用语，并链接各子系统的当前限额。 |

## 如何理解这些参考

当前行为附实现入口。设计选项及试验工具各自说明适用范围。[技术参考](../reference/README.md)提供各个工程契约的入口。[项目首页](../../../README.zh-CN.md)和对应发布说明描述支持的分发方式及已交付功能。耗时、资源和原生验证结论适用于当时记录的构建、平台与工作负载。
