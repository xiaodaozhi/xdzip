# Qt/Rust 桥接与应用服务

[English](../../design/qt-rust-bridge.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/design/qt-rust-bridge.md) | [日本語](../../ja-JP/design/qt-rust-bridge.md) | [한국어](../../ko-KR/design/qt-rust-bridge.md) | [Français](../../fr-FR/design/qt-rust-bridge.md) | [Deutsch](../../de-DE/design/qt-rust-bridge.md) | [Español](../../es-ES/design/qt-rust-bridge.md) | [Português (Brasil)](../../pt-BR/design/qt-rust-bridge.md) | [Русский](../../ru-RU/design/qt-rust-bridge.md)

Qt 桌面通过链接到程序内的 `cxx` 桥接调用 Rust 运行时。每个 `BrowserController` 有自己的 `RuntimeConnection` 和对应的服务实例。桥接在桌面进程内传递类型化命令和自有数据的结果。CLI 直接调用同一个 Rust 服务实现。[系统架构](system-architecture.md)、[桌面会话](../engineering/application-sessions.md)

## 句柄、线程与所有权

`new_runtime()` 启动 Rust 命令线程和任务线程。临时的 `Runtime` 各交出一个 `CommandClient` 和 `EventReceiver`。界面线程使用命令句柄，专用的 Qt 接收线程持有事件句柄并在 `next_event()` 中阻塞等候。Rust 命令线程持有当前归档和索引，独立的任务线程在执行期间持有已准备的作业。

CXX 传递值类型和自有数据的 Rust 容器。Qt 接收方在持有这些值时保存或转换它们。桥接不向 Qt 传递归档对象或原生文件句柄。控制器将需要保留完整 64 位精度的编号和字节数以字符串传给 QML。生成的头文件和静态库与宿主一起构建。该接口属于产品构建，随包提供的归档工作进程使用独立的[进程协议](../engineering/native-codecs.md)。

[桥接声明与转换](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-qt-bridge/src/lib.rs)、[Qt 连接](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/RuntimeConnection.cpp)、[Rust 服务](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/service.rs)

## 请求、查询与任务身份

| 身份 | 用途 |
| --- | --- |
| 请求编号 | Rust 服务接收命令时分配，编号在该连接内有效。`Submission` 返回 `Accepted`、`Busy`、`Closed`、`Invalid` 或 `Exhausted`。拒绝时编号为零。 |
| 会话编号 | 标识当前打开的归档快照。针对该归档的命令必须匹配会话编号。新打开的归档获得新编号。 |
| 查询身份与分页偏移 | 分页绑定会话、查询编号、界面代数、修订号和偏移。控制器核对这些值后才接收列表行。 |
| 物理条目或导航编号 | 物理条目的身份在排序和翻页后保持不变。虚拟目录有独立的导航身份。行下标、显示名和查询范围均不会授予文件系统访问权限。 |
| 任务与提示编号 | 任务编号在所属连接内有效。提示编号指向任务中一个待答复的重名或密码提示。共享任务入口还记录对应的控制器。 |
| 事件序号 | 接收器取出事件时，在连接内分配递增的序号。Qt 丢弃序号不大于已处理序号的事件。任务更新也使用这一序号，并通过原请求编号建立关联。 |

## 当前桥接命令

| 命令或方法 | 用途与普通响应 |
| --- | --- |
| `Open`、`OpenWithPassword` | 打开并扫描所选来源。成功时返回第一页 `Page` 和受限的导航数据。加密目录可能要在打开会话前先提供密码。 |
| `Browse` | 发起查询，或用匹配的身份和偏移继续查询。每次最多返回一个 `Page`，上限为 128 行和 64 KiB 名称数据。 |
| `FreezeSelection`、`Preflight` | 把当前选择器解析为 `Selected` 中自有数据的物理或导航编号，或在 `Preflight` 中汇总元数据检查结果。实际操作还会重新核对快照和选中范围。 |
| `Preview` | 定位一个物理条目。成功时返回自有数据且有容量限制的 `Preview` 字节，以及条目与会话身份。 |
| `CheckTarget`、`InspectInputs` | 运行时打开并清理探测工作区后，在 `TargetChecked` 中返回遗留工作区数量。独立创建检查使用会话编号零。输入概况按传入顺序返回 `Inputs`，只检查浅层元数据，无需递归扫描目录。两种响应均不授予提交权限。 |
| `Extract`、`Create`、`Check`、`OpenEntry` | 校验操作意图并将任务入队。`TaskAccepted` 携带分配的任务编号。目标授权和提交由运行时与 I/O 层处理。 |
| `Update`、`Move`、`Export`、`PublishExport`、`ExportManifest` | 归档修改和数据发布进入重任务队列。接收 `Update` 或 `Move` 后，服务释放当前归档快照，后续请求不能再使用该快照的旧条目编号。`ExportManifest` 作为普通响应返回相对路径等传输元数据。 |
| `Snapshot`、`CloseSession` | 返回当前归档会话编号或零。关闭指定会话时释放浏览快照，任务可以继续持有自己保留的输入。 |
| `cancel`、`cancel_task`、`decide_task`、`respond_password`、`release_export`、`shutdown` | 独立于普通命令队列的控制方法。请求取消有自己的请求编号和 `Finished`，任务控制指向已有任务。 |

Rust 服务还提供 `Catalog` 和 `SelectMembers` 等 CLI 专用操作。桥接在接收命令前校验输入大小与枚举值。例如，创建输入路径每条限 4 KiB，路径合计限 4 MiB。运行时校验会话和操作语义，I/O 层负责打开文件和取得输出能力。[运行时服务契约](../engineering/product-runtime.md)、[命令行契约](../engineering/command-line.md)、[文件授权](file-security.md)

## 事件交付与背压

正常交付时，每个已接收的普通请求最多返回一份数据，随后由一个 `Finished` 结束。出错时 `Finished` 携带错误码。控制器等待请求成功结束，再应用匹配的分页或预览。被后续操作替代的结果会丢弃，对应请求仍会结束。大索引路径在 Qt 中最多缓存 1,024 行，按需请求未加载的分页，并在打开解压确认前冻结跨页选区。[大型归档索引](../engineering/large-index.md)、[内容预览](../engineering/content-preview.md)

`TaskAccepted` 和 `Finished` 属于普通请求交付。`TaskUpdate` 使用每个任务独立的待交付槽，携带原请求编号、任务编号和来源会话编号。任务更新可能早于 `TaskAccepted` 到达，也可能晚于 `Finished`。控制器通过原请求关联提前到达的任务，再将后续更新送到对应任务。后到的任务更新不会再次结束普通请求。待交付槽保留任务的最新快照，进度可以合并，最终快照一直保留到接收器取走。等待答复期间，提示包含在任务快照中。[任务状态与取消](task-scheduling.md)

- **Rust。** 每个连接允许 128 个尚未结束的普通请求，待执行命令的字节预算为 8 MiB，普通事件负载预算为 32 MiB。已接收的普通请求有结果和结束事件的保留容量。请求取消回执有独立的有界队列，关闭无需占用该队列的空位。
- **Qt。** 邮箱最多容纳 256 个事件，其中载荷事件最多 128 个，计入预算的负载最多 32 MiB。容量用尽时，接收线程等待界面线程取走数据。一条排队通知链每轮最多处理 32 个事件或使用 4 毫秒。通知发往界面线程，排队事件持有自己的数据。

## 提示、失败与关闭

- 重名答复使用 `decide_task`，任务密码使用 `respond_password` 和独立的密码提示编号。两者都校验当前仍未答复的提示。密码答复进入 Rust 可清零容器，不进入结果或进度事件。`OpenWithPassword`、`Preview` 和加密创建携带各自操作的密码输入，桥接会把它们转入相应的 Rust 密码类型。[密码处理](../engineering/password-extraction.md)
- 准入、请求和任务结果使用稳定错误码，例如 `stale_session`、`stale_query`、`source_changed`、`unsafe_target`、`target_changed`、`disk_full` 和 `password_required`。控制器根据错误码本地化显示。来源变化会使相关浏览快照失效。`RuntimeFault` 或意外断开时，界面将未完成操作按 `runtime_unavailable` 处理，连接丢失可能使原本的结束事件无法到达。
- `shutdown` 停止接收新命令，取消已接收的工作，包括等待输入的任务。Rust 线程退出期间，Qt 接收线程继续取出事件。Rust 接收器等待工作线程退出后再返回 `ShutdownReady`，随后桥接返回 `End`。Qt 邮箱排空后报告正常断开。连接若提前销毁，会关闭邮箱并唤醒接收线程，界面线程无需等待它结束。[任务关闭](task-scheduling.md)、[运行时生命周期](../engineering/product-runtime.md)

修改这一边界时，可用[桥接探针](https://github.com/xiaodaozhi/xdzip/blob/main/tools/bridge-probe/README.md)、Rust 服务测试和[桌面控制器测试](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/tests/browser_test.cpp)核对各自覆盖的层级。相关构建目标见[开发构建指南](../engineering/developer-build.md)。
