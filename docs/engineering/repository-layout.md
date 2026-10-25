# 仓库目录与依赖

当前产品模块以根目录 Cargo.toml 和 CMakeLists.txt 为准。下表描述实际目录；架构设计中的未来拆分不代表已有独立 crate。

| 目录 | 所有权与依赖规则 |
| --- | --- |
| apps/desktop | Qt/QML、薄 C++ 控制器、模型、设置与系统交互；通过 bridge 访问核心 |
| crates/xdzip-domain | 纯类型与不变量；不得依赖 Qt 或具体 codec |
| crates/xdzip-codec-api | 能力、借用输入、编码输出和取消契约；依赖 domain |
| crates/xdzip-compression | 无第三方生产依赖的共享压缩算法与 GZIP 封装，不拥有路径或发布能力；独立实现规则见[契约](independent-rust-codecs.md) |
| crates/xdzip-codec-zip、xdzip-codec-tar | ZIP、TAR/GZIP 格式实现；不获取任意输出路径 |
| crates/xdzip-codec-native、tools/native-codec | 官方 7-Zip／UnRAR 的 Rust 适配及原生工作进程，固定来源、裁剪和 I/O 边界见[契约](native-codecs.md) |
| crates/xdzip-extract-plan | 元数据预检与有界报告；不拥有目标授权或写入能力 |
| crates/xdzip-io | 输入树、原生授权与身份、安全暂存和文件提交 |
| crates/xdzip-runtime | 会话、内存索引、查询、选择、创建／解压任务和调度 |
| crates/xdzip-qt-bridge | cxx 类型桥接、双句柄及生命周期；依赖 runtime |
| tools | 独立桥接、格式、I/O 与进程计数探针，不作为 GUI 产品入口 |
| scripts、.github | 生成、检查、部署工具与 CI、协作模板 |
| release-notes | 可审阅的版本专属发布说明及 version/build 元数据，不存执行报告或生成包 |
| design/hifi、design/html、design/assets | 高保真原稿、可操作设计及运行素材输入 |
| docs | 持久设计、工程契约与可复用验证方法；不累积执行报告 |
| third_party | 随代码保留的第三方来源与许可材料 |

独立 xdzip-index、xdzip-platform、xdzip-plugin-host 及外部格式一致性工具属于后续架构规划；目前索引在 runtime、平台文件操作在 io 与桌面适配层实现。测试主要随 Rust 模块、apps/desktop/tests 和 scripts 的检查工具维护，不创建无实现空壳。

依赖设计见[系统架构](../design/02-system-architecture.md)。避免 domain 反向引用 runtime、codec 直接通知 QML，以及平台层决定 UI 文案。生成绑定、样本、报告和构建输出进入 build/、target/ 或 dist/，不提交到源码目录。历史兼容测试的持久输入另见[安装与关联](installers-associations.md#兼容基线的长期输入)。
