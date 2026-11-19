# 领域模型与归档数据

[English](../../design/domain-data.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/design/domain-data.md)

归档会话、已完成的索引和会话内最新查询由运行时持有。Qt 保存结果的视图，CLI 通过独立的清单命令读取物理记录。两者都把操作交给同一个 Rust 服务实现。[系统架构](system-architecture.md)、[Qt/Rust 桥接](qt-rust-bridge.md)

## 身份与来源版本

| 身份或数据 | 当前表示与作用范围 |
| --- | --- |
| `SessionId` | 打开归档后产生的非零运行时身份。重新打开成功时建立新会话。文件路径用于输入和界面路由。 |
| `SourceVersion` | 运行时记录各个已接收分卷对应普通文件的身份、长度和可用时间信息，在相关读写边界复核已打开对象及对应路径。这是一种尽力检测。 |
| `EntryId` | 标识会话内的一条物理归档记录。同名记录各有独立身份。 |
| 导航身份 | 表示已索引的目录。通过后代记录合成的虚拟目录可显示在目录树或列表中，无需对应物理条目编号。 |
| `QueryKey` | 由会话、查询编号、代数和快照修订号组成。偏移表示该结果中的位置。 |
| 请求、任务与提示身份 | 服务在连接内分配请求和任务编号。提示归属某个具体任务。桌面共享任务入口使用控制器的会话身份及任务编号。 |

领域类型保证归档和查询身份非零。原生桥接通过生成的 CXX 数据传递类型化 Rust 值。需要完整保留 64 位的编号以不透明字符串交给 QML。行号只表示某个查询中的位置。[领域身份源码](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-domain/src/identity.rs)、[来源实现](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/source.rs)、[应用会话](../engineering/application-sessions.md)

## 物理条目与输出名称

`xdzip-codec-api::Entry` 保存格式、物理身份、名称的不同表示、大小信息、条目类型标志、修改信息以及格式专用的方法和加密字段。原始名称字节、可选的 UTF-8 解码名、编码判定及转义后的显示文本分别存储。写出时，运行时对有效名称执行可移植路径校验，并核对目标授权。未解析的名称可以显示，但解压该记录或目录时可能拒绝执行。

原始大小带有独立的 `size_known` 标志，用于区分已知的零字节文件与大小未知。有些压缩容器没有逐条目的压缩后大小。方法值按格式解释。时间保留各格式的约定，例如 ZIP 的本地 DOS 时间，缺失时仍显示未知。`payload_supported()` 判断当前记录能否解码，实际写出还取决于所选操作、来源状态、名称与文件系统规则。[条目字段](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/lib.rs)、[文件名处理](../engineering/filename-encoding.md)、[文件授权](file-security.md)

## 索引、查询与分页生命周期

运行时打开来源并完成有界扫描后，才发布归档及第一页。扫描可以取消。尾部损坏、需要密码或来源变化时，不会发布部分索引。目录导航有独立预算，超出目录树限额时仍可支持根目录浏览及物理条目搜索。普通单文件 ZIP/ZIP64 的私有磁盘路径最多可存储一百万条记录。其他格式和分卷路径遵循各自限额。

`Archive::query_page` 接收目录、文本、排序方式、偏移和预期查询身份。新查询从偏移零开始，使用新的代数。运行时建立完整的结果顺序，每个归档保留一个当前不可变查询。分页从这个顺序取出行身份。搜索和排序保留归档记录原有身份。各排序方式都将目录排在文件前，以确定性的身份顺序处理并列值，并在同组内将大小未知项排在已知项之后。

通用的领域类型 `PageRequest` 最多接受 256 行。桌面运行时每批最多请求 128 行，名称数据预算为 64 KiB。Qt 完整提交小结果，大结果最多常驻 1,024 行。会话、查询、代数、修订号或偏移不匹配的响应不能替换当前数据。[查询实现](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/query.rs)、[列表与排序](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/listing.rs)、[大型归档索引](../engineering/large-index.md)

## 选区与操作输入

Qt 模型对完整加载的结果按身份记录选区，对大结果则记录当前结果内的有界区间。全选覆盖当前结果。任务能区分整包操作与明确选中的搜索结果。进入解压确认前，Rust 通过 `FreezeSelection` 按完整查询键解析跨页选区。展开目录会解析实际后代，并按物理条目编号去重。超过操作限额会返回错误。CLI 通过 `SelectMembers` 解析成员路径和物理条目编号。

解压、检查和修改分别准备自身的作业，持有操作所需的来源及元数据。创建接收选项，再扫描和验证独立的 `InputTree`。元数据预检可说明潜在问题，但不会给任务输出权限。写出前，运行时复核输入并取得相应授权。作业一旦持有自己的选区，后续界面操作便不会改变其范围。[桌面模型](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/BrowserController.cpp)、[运行时命令](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/service.rs)、[任务调度](task-scheduling.md)

## 结果与数据生命周期

| 数据 | 生命周期 |
| --- | --- |
| 当前归档、查询和磁盘索引 | 归属当前服务会话。更换查询或关闭会话会释放相应私有数据，作业独立持有的资源可继续使用。重新打开归档时重建磁盘元数据索引。 |
| 任务快照 | 运行时发布受限的快照，记录实际状态、字节数、项目数、清理状态和有限明细。桌面通过所属控制器在本次应用会话内保留任务结果。 |
| 用户设置 | 与归档索引和任务结果分别管理。数据保留和隐私见设置与隐私契约。 |
| 工作区与日志 | 私有输出暂存及相对路径发布记录用于清理和检查。重启不会重放未完成的归档写入。 |

[快照字段](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/extraction/mod.rs)、[文件提交](file-security.md)、[设置与本地化](../ux/settings-localization.md)、[数据生命周期与隐私](../engineering/privacy.md)

## 尚未实现的设计选项

增量发布可搜索索引、带独立修订号和启动令牌的可复用计划，以及持久化的分页任务历史，属于设计选项。当前命令分别准备有界作业并入队，只保留当前查询和有界任务快照。持久化恢复还需要单独设计来源及输出的验证流程。
