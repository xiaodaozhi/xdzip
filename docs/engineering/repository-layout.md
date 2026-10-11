# 仓库规划

当前仅存在文档、视觉资产、协作配置和文档工具。下列产品目录为规划，实施工作包建立时才创建，避免空壳误导贡献者。

| 目录 | 所有权与依赖规则 |
| --- | --- |
| apps/desktop | Qt/QML、C++ 控制器、模型、系统交互；依赖 bridge，不依赖 codec 实现 |
| crates/xdzip-domain | 纯类型与不变量；不得依赖 Qt 或具体 codec |
| crates/xdzip-codec-api | 能力与宿主 I/O 契约；依赖 domain |
| crates/xdzip-codec-zip、xdzip-codec-tar | 格式实现；依赖 codec-api，不获取任意输出路径 |
| crates/xdzip-io、xdzip-platform | 安全 I/O、原生句柄、提交与授权 |
| crates/xdzip-index | 查询与索引；由 runtime 编排 |
| crates/xdzip-runtime | 会话、计划、任务和资源调度 |
| crates/xdzip-qt-bridge | 类型桥接与句柄生命周期，依赖 runtime |
| crates/xdzip-plugin-host | 后续外部协议与隔离宿主，不提前启用 |
| tests/fixtures、tools/codec-conformance | 授权样本和格式一致性验证 |

依赖审查以 design/02 的模块图为权威。避免 domain 反向引用 runtime、codec 直接通知 QML，以及平台层决定 UI 文案。生成绑定和构建输出进入 build/target，不提交到源码目录。
