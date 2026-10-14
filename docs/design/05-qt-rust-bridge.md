# xdzip：05 Qt/Rust 类型桥接与应用服务协议

## 1. 决策和版本

正式产品使用 `cxx` 生成 Rust/C++ 构建期桥接。应用、Rust 静态库和生成头必须来自同一提交和工具链组合；不允许替换为任意版本动态库。此接口面向产品内部，不是第三方二进制 SDK，也不是 [06](06-codec-extension-sdk.md) 的进程协议。

原型 `lab_engine_new/lab_request/lab_string_free/lab_engine_free` 及 JSON 请求继续作为实验代码保存；产品不能逐步往实验接口塞入密码、输出路径和插件操作后声称它已冻结。

以下表格定义概念类型与语义，不是可直接编译的 `cxx::bridge` 声明。实施阶段根据 cxx 允许的共享类型拆分结构，生成代码纳入构建而非手工编辑。禁止为性能方便引入未定义生命周期的裸指针。

M1 已落地独立双句柄、Open/Browse/Preflight/CloseSession/Snapshot、取消/关闭控制通道和 Qt mailbox。当前单会话、串行 GUI 请求；完整查询代际、TaskId、输出授权与扩展协议仍是后续目标。已实现签名、资源预算与故障语义见 [M1 运行时契约](../engineering/m1-runtime.md)，测试和开发包见 [M1 验收入口（本地／CI 工件）](../qa/README.md)。

## 2. 客户端和事件接收

桥接创建两个独占句柄：`CommandClient` 和 `EventReceiver`。内部通过 Arc/有界通道引用同一个 Rust Runtime；C++ 以 RAII 包装拥有它们。

- GUI 线程使用 CommandClient 提交短命令；入口不等待工作完成，不操作 QObject。
- Qt 接收线程独占 EventReceiver，调用可阻塞 `next_event()`。运行时关闭/接收器关闭能立即唤醒，不靠长轮询。
- 接收线程把事件转换成值对象，放入容量 256 的 C++ mailbox；对 GUI 最多保持一个使用明确 `QueuedConnection` 的“待处理事件”通知，不为每个 Rust 事件各排一个 Qt callback。GUI 在一轮中按批次/时间预算 drain，仍有数据时再安排一轮。QObject 销毁会取消对应通知，不用捕获悬空 `this` 的 lambda。
- GUI 命令句柄和接收句柄不跨线程并发调用；Rust 内部共享状态经过类型和锁/通道保护。接口文档必须说明哪些对象 Send，不能只因底层用了 Arc 就推断全部线程安全。
- 关闭时保留一个不受普通队列容量影响的控制信号；关闭接收器前明确是否继续 drain 终态，防止工作线程因满事件队列阻塞而无法 join。

## 3. 统一包络

命令包络：`RequestId`、可选 SessionId/TaskId、命令种类和类型化参数。提交立即返回 `Accepted` 或 `Rejected(Error)`；Accepted 表示已入队，不表示操作成功。

被接受的普通请求最终产生一次 `RequestFinished {request_id, outcome}`。异步任务启动的 outcome 是 TaskId，任务终态另由 `TaskUpdated` 给出。取消/关闭控制具有独立准入通道，但仍带 RequestId。运行时崩溃时无法保证最终响应，Qt 连接断开会完成所有 pending 请求为 `runtime_unavailable`。

FetchPage 的内容只由 PageReady 携带一次，RequestFinished 引用页身份并结束 pending 请求；丢弃过期页面仍须结束对应请求。状态事件和请求完成不能重复创建任务或重复应用数据。

事件带 `event_sequence`；任务事件另有单调 `task_sequence`。页结果带完整查询身份。传输顺序不作为跨任务排序依据。一个 TaskId 收到终态后，不接纳更旧的 running/progress 事件。

## 4. 应用服务命令

| 命令 | 必需参数 | 结果/约束 |
|---|---|---|
| `OpenArchive` | 用户授权的 SourceRef、探测选项 | SessionId，后续索引事件；不解压 |
| `CloseSession` | SessionId | 释放浏览；正在运行任务不隐式销毁 |
| `CreateQuery` | SessionId、QuerySpec、generation | QueryId、完整性；旧代次可取消 |
| `FetchPage` | QueryId、revision、offset、limit、generation | 不可变 Page；limit 1～256 |
| `SetSelection` | QueryId、SelectionDelta、selection revision | 新 SelectionRevision、计数；拒绝过期范围 |
| `PrepareExtraction` | 会话、范围快照、目标授权、策略 | PlanId/Revision、摘要；可异步建计划 |
| `PrepareCreation` | 输入授权集、格式、目标授权、参数 | 新建归档计划；检测输入输出递归 |
| `StartPlan` | PlanId、PlanRevision | TaskId；重复 RequestId 不重复创建 |
| `CancelTask` | TaskId | 取消请求已接收；终态另报 |
| `CancelRequest` | 待取消的 RequestId | 用于查询/计划构建等未创建任务的工作；原请求结束为 cancelled 或已完成 |
| `ResolvePrompt` | TaskId、PromptId、类型化决定 | 仅处理当前提示；旧提示为 `stale_prompt` |
| `SupplySecret` | PromptId、SecretRef | 只用于对应阶段；不能写入普通事件 |
| `FetchTaskResult` | TaskId、cursor、limit | 有界结果页，不一次传百万失败项 |
| `GetCapabilities` | 会话或格式 ID | 当前后端版本/具体能力和禁用原因 |
| `GetRuntimeSnapshot` | 无或已知 event_sequence | 有界会话/任务摘要和快照序号；新建视图据此接续事件 |
| `RequestShutdown` | 优雅关闭策略 | 停止准入、取消/完成收尾；发 ShutdownReady |

SourceRef/TargetRef 是通过平台授权工厂建立的宿主引用，不是插件发来的任意字符串。Qt 本身是受信任前端，用户从系统对话框得到路径后，Rust 平台层打开并验证授权对象。文件名、格式参数仍视为不可信数据。

CancelRequest 与 CancelTask 共用独立控制准入。快照不含页数据、密码和完整结果；GUI 忽略早于快照序号的摘要事件，之后按 sequence 应用。关闭会话停止无引用扫描，已被任务持有的源和索引继续按任务生命周期存活。

## 5. 事件

| 事件 | 内容 | 交付策略 |
|---|---|---|
| `RequestFinished` | 请求结果或 Error | 必须交付一次，运行时断开除外 |
| `SessionUpdated` | 探测/扫描/Ready/Failed、索引版本 | 状态转移必达，扫描计数可合并 |
| `PageReady` | 页及查询身份 | 过期页可丢弃；正在使用的请求有完成结果 |
| `SelectionUpdated` | revision、数量和有限可见变化 | 同会话可合并，仅保留最新 revision |
| `TaskUpdated` | 状态、阶段、计数、字节、sequence | 进度可合并；等待及终态必达 |
| `DecisionRequired` | PromptId、冲突/密码/配额等摘要 | 必达；不含密码或未转义富文本 |
| `CapabilitiesChanged` | 后端新增/失效及原因 | GUI 重新计算动作可用性 |
| `RuntimeFault` / `ShutdownReady` | 不可继续错误/关闭完成 | 控制消息，不能被进度挤掉 |

普通请求入口最多 128 个 outstanding；page 请求重复合并，满时返回 `busy`。事件使用容量 256 的必达队列及按对象合并的进度槽。接收端慢时停止接纳普通请求、丢弃过期页/低优先级进度；工作任务不能在文件提交锁内等待 GUI。等待中的任务数量受任务队列限额约束，因此提示和终态占用也有上界。

C++ mailbox 同样合并进度并为控制/等待/终态保留容量；满时接收线程暂停读取普通事件，背压传回 Rust。仅限制 Rust 队列不能限制 Qt 的 queued-event 内存。调度器处理 cancel/shutdown 不阻塞在普通事件入队上，必要时停止新工作并保留有界 pending 控制结果；shutdown 期间 GUI 继续 drain，不能在 GUI 线程 join 一个仍等 mailbox 空位的接收线程。

## 6. 数据和所有权

小 DTO 复制按值交付，字符串 UTF-8；原始非 Unicode 路径字节不直通 QML。页面条目仅有展示字段和 ID，完整属性另查。跨线程结果拥有自己的数据，不借用仍可能变化的索引缓冲。

大内容不通过桥接送到 QML；预览最多按配额获得有限缓冲或受控临时授权。cxx 的 Rust/C++ 容器由其对应类型正确析构，禁止使用 `free/delete` 处理 Rust 分配内存。秘密对象与普通 String DTO 分离，显式销毁/失效，尽可能使用零化容器。

Rust Result 在边界转换为结构化 Error，C++ 异常不跨回 Rust。入口校验所有数值窄化、枚举值、对象状态、长度和 handle 归属。无法恢复的 panic 按发行策略终止/报告，不用捕获 panic 来维护半完成事务。

## 7. 错误契约

`Error {code, category, retryability, context_id, safe_detail}`。category 为 input、capability、permission、integrity、resource、cancelled、backend、internal。safe_detail 只用于转义后的详情，不包含秘密和默认完整路径。UI 根据 code 翻译主文案。

| 错误码示例 | 含义 | 可恢复动作 |
|---|---|---|
| `unsupported_format` / `unsupported_feature` | 无后端或当前能力不支持 | 换后端或取消，不假装继续 |
| `stale_query` / `stale_plan` / `source_changed` | 快照或输入已变化 | 刷新、重新规划 |
| `path_unsafe` / `name_collision` | 输出非法或映射冲突 | 跳过/显式改名，不直接覆盖 |
| `quota_exceeded` / `disk_full` | 达到输出/资源限额 | 当前项不提交；决定后重新规划或取消 |
| `password_required` / `wrong_password` | 需要秘密或认证失败 | 限次提示；不默认记录密码 |
| `archive_corrupt` / `checksum_mismatch` | 结构/内容完整性失败 | 当前输出丢弃，摘要失败 |
| `backend_crashed` / `plugin_protocol_error` | 工作进程或协议失效 | 隔离扩展；重试需新计划 |
| `permission_denied` / `io_error` | 平台文件访问失败 | 保留路径上下文在本地详情中 |

## 8. 构建和迁移验证

CMake 构建 Qt 应用，Cargo 构建 Rust 静态库和 cxx 生成产物。使用一个构建编排入口，明确目标 triple、Qt 架构、编译器、C++ 标准和生成头目录。不能用系统中碰巧存在的另一个 generated header。

首阶段验证 macOS arm64、Windows MSVC、Linux x86_64 的最小往返、Unicode、所有权、队列满、关闭过程中 pending 页、取消优先通道和 RuntimeFault。通过后再迁移原型分页模型。cxx 依赖版本锁定在验证提交；暂不在设计中虚构一个已通过测试的版本号。

## WP-10 当前创建切片

内部桥接增加 Create：输入路径数组、目标目录、ZIP 名称、Store／Deflate、1／6／9 级和重名策略。Qt 在 UTF-16→UTF-8 与整数窄化前拒绝非法值，Rust 再检查单路径 4 KiB、选择路径总量 4 MiB、条目数与便携文件名。CheckTarget 的 session=0 表示独立创建目标检查；非零会话仍绑定解压来源。

创建与解压共用后台任务身份、取消、提示 ID、事件预算与异步退出。TaskSnapshot 增加 creating、total_known，扫描未知总量不能伪装为零字节任务；Verifying 阶段不把再次读取的校验量计入已处理输入。创建只保存完整归档一次，输入条目不能提前计为已保存。字段、错误及整体提交语义见 [WP-10 契约](../engineering/m4-creation.md)。

创建选择摘要通过内部 InspectInputs 命令读取，返回按输入顺序的 Inputs，再返回唯一 Finished。一次最多 256 个绝对路径、每条最多 4 KiB，纳入已有请求与事件字节预算。后台使用 symlink_metadata，不递归目录、不跟随所选叶链接；普通文件报告大小，目录、错误与特殊对象不伪装为已知总量。Qt 仅展示当前路径集合对应的结果，快速变更合并为最新集合。目标检查与摘要请求在元数据通道空闲后执行，不创建重任务；开始创建仍重新授权、扫描并冻结输入树，不使用这些展示数据作为提交依据。
