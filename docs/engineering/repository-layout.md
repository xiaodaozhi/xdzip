# 仓库目录与依赖

当前产品模块以根目录 Cargo.toml 和 CMakeLists.txt 为准。下表描述实际目录；架构设计中的未来拆分不代表已有独立 crate。

| 目录 | 所有权与依赖规则 |
| --- | --- |
| apps/desktop | Qt/QML、薄 C++ 控制器、模型、设置与系统交互；通过 bridge 访问核心 |
| crates/xdzip-domain | 纯类型与不变量；不得依赖 Qt 或具体 codec |
| crates/xdzip-codec-api | 能力、借用输入、编码输出和取消契约；依赖 domain |
| crates/xdzip-compression | 无第三方生产依赖的共享 Deflate/Deflate64/GZIP、LZMA/LZMA2、BZip2、PPMd7、MSZIP、LZX、Quantum 算法与封装，不拥有路径或发布能力；独立实现规则见[契约](independent-rust-codecs.md) |
| crates/xdzip-crypto | 格式无关、固定内存的可清除敏感状态 SHA/HMAC 与 BLAKE 哈希，复用已准入基础原语；不解释归档密码或决定发布 |
| crates/xdzip-codec-zip、xdzip-codec-tar | ZIP、TAR/GZIP 格式实现；不获取任意输出路径 |
| crates/xdzip-codec-7z | 自研 7z 容器、方法图、过滤器、AES 组合、流式创建及有界重写，不拥有文件路径 |
| crates/xdzip-codec-cab | 自研 CAB 只读容器，使用宿主授权卷和共享算法；保留原始名称、UTF-8 声明及普通重名记录，不自行发现卷或发布输出；见[自研契约](independent-rust-codecs.md#cab只读规格)与[来源](cab-format-sources.md) |
| crates/xdzip-codec-optical | 自研 ISO9660/Joliet/Rock Ridge/UDF 只读解析与内容流，不拥有文件路径 |
| crates/xdzip-codec-bridge | 隔离工作进程到 Rust 7z、CAB 和光盘后端的共享薄 CXX 边界，复用授权流、取消、有界输入、输出与暂存协议，不提供文件路径 |
| crates/xdzip-codec-native、tools/native-codec | Rust 7z／RAR／光盘／CAB 后端的隔离工作进程，薄 C++ 仅提供协议及系统日历、编码服务；Native 独立核验预计费输出与辅助工作；I/O 边界见[契约](native-codecs.md) |
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

CAB 产品固定使用 Rust，官方 Cab 处理器和 CAB 专用 LZX/Quantum 不进入编译清单。调用链继续使用同一个隔离工作进程，不新增产品可执行文件，没有测试开关或失败回退。完整帧工作预付由 codec-api 定义，容器持有每帧额度，Native 接收端和 runtime 分别核验自己的额度与输出限制，见[自研契约](independent-rust-codecs.md#cab只读规格)。

依赖设计见[系统架构](../design/02-system-architecture.md)。避免 domain 反向引用 runtime、codec 直接通知 QML，以及平台层决定 UI 文案。生成绑定、样本、报告和构建输出进入 build/、target/ 或 dist/，不提交到源码目录。历史兼容测试的持久输入另见[安装与关联](installers-associations.md#兼容基线的长期输入)。
