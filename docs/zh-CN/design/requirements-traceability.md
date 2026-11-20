# 需求与验证追踪

[English](../../design/requirements-traceability.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/design/requirements-traceability.md) | [日本語](../../ja-JP/design/requirements-traceability.md) | [한국어](../../ko-KR/design/requirements-traceability.md) | [Français](../../fr-FR/design/requirements-traceability.md) | [Deutsch](../../de-DE/design/requirements-traceability.md) | [Español](../../es-ES/design/requirements-traceability.md) | [Português (Brasil)](../../pt-BR/design/requirements-traceability.md) | [Русский](../../ru-RU/design/requirements-traceability.md)

[产品要求](product-requirements.md)定义需求编号及当前范围。下表说明对应行为改动时应验证什么。执行结果、平台范围和限制记录在对应变更中。[验收用例](../../../developers/zh-CN/qa/acceptance.md)提供检查入口。

## 需求矩阵

| 需求 | 设计参考 | 验证重点 |
| --- | --- | --- |
| P01 浏览时不解压 | [领域](domain-data.md)、[桌面](qt-quick-ui.md) | T01 打开真实归档，核对浏览过程没有解压内容。 |
| P02 浏览、查询与选择 | [领域](domain-data.md)、[性能](performance.md) | T02 同名身份、稳定查询、过期分页、大索引存储与跨页选区。 |
| P03 解压预期范围 | [文件授权](file-security.md)、[任务](task-scheduling.md) | T03 目录展开、同名记录、重命名、跳过、覆盖及实际磁盘结果。 |
| P04 创建归档 | [格式](codec-extension-sdk.md)、[文件授权](file-security.md) | T04 受影响配置的独立往返、暂存验证和既有文件保护。 |
| P05 遵守格式能力 | [桥接](qt-rust-bridge.md)、[格式](codec-extension-sdk.md) | T05 注册表、会话操作及运行时拒绝不支持的配置。 |
| P06 限制任务并支持取消 | [桥接](qt-rust-bridge.md)、[任务](task-scheduling.md) | T06 队列饱和、提示路由、取消与提交竞态及异步关闭。 |
| P07 保护文件与资源边界 | [文件授权](file-security.md)、[格式](codec-extension-sdk.md) | T07 适用的原生链接与身份核对、路径校验、配额和受保护对象。 |
| P08 提供可访问的桌面 | [桌面](qt-quick-ui.md) | T08 模型和控件，以及受影响平台的原生键盘、输入法、辅助功能、缩放及焦点。 |
| P09 限制并测量资源 | [性能](performance.md) | T09 可比的构建、进程树指标，按需核对启动、排序、写出吞吐或包大小。 |
| P10 将 Qt 与格式逻辑分开 | [架构](system-architecture.md)、[格式](codec-extension-sdk.md) | T10 无需 Qt 的核心、显式路由及格式专用回归。 |
| P11 保护密码及数据 | [文件授权](file-security.md)、[隐私](../engineering/privacy.md) | T11 有作用范围的密码、脱敏诊断及明确的网络访问。 |
| P12 审计交付 | [质量](quality-engineering.md)、[发行](build-release.md) | T12 锁定输入和工件身份、组件清单、许可材料、包信任及适用安装检查。 |
| P13 控制外部扩展准入 | [格式](codec-extension-sdk.md)、[SDK 试验](../engineering/sdk-trial.md) | T13 独立试验的互操作和原生权限结果。产品内安装扩展仍是设计选项。 |
| P14 限制预览与打开 | [桌面](qt-quick-ui.md)、[文件授权](file-security.md) | T14 自动预览、受支持内容、工作进程限额和私有临时输出。 |
| P15 报告实际结果 | [桥接](qt-rust-bridge.md)、[文件授权](file-security.md)、[任务](task-scheduling.md) | T15 来源或目标变化、部分提交后的失败与取消、清理及不确定结果。 |

## 横向验证

| 用例 | 适用改动 | 验证重点 |
| --- | --- | --- |
| T16 桥接所有权与生命周期 | 桥接、连接或控制器生命周期 | 当前请求身份、接收线程唤醒、保留任务和安全销毁。 |
| T17 格式互操作与失败路径 | 格式或编解码修改 | 适用时使用独立的有效样本，覆盖截断或不支持的配置及输出限额。 |
| T18 安装、更新与移除 | 平台或发行渠道改动 | 对应安装包、干净环境的运行依赖、安装身份、更新交接及支持的移除或恢复。 |

## 代表性工作流程

1. 打开 ZIP 并搜索，冻结选中的结果，解压后比较输出字节。“解压全部”仍指整个归档。
2. 按指定重名策略解压同名或冲突记录，核对最终名称与结果。
3. 在发布前及发布期间请求取消，核对已完成文件仍有记录，并确认创建在提交前失败时保留原归档。
4. 在自有测试中改动来源或目标并验证实际结果。异常终止后检查遗留对象，恢复过程中无自动重放。
5. 依据各自契约验证随包格式工作进程或独立 SDK 试验异常时的处理，确认暂存内容不会以成功结果发布。

静态写作、格式和链接检查验证文档。夹具、离屏、原生自动化与人工验收分别保留各自的证据范围。[质量工程](quality-engineering.md)、[验证指南](../../../developers/zh-CN/qa/README.md)
