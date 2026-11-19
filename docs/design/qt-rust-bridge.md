# Qt/Rust bridge and application service

English | [简体中文](../zh-CN/design/qt-rust-bridge.md) | [繁體中文（台灣）](../zh-TW/design/qt-rust-bridge.md)

The Qt desktop calls the Rust runtime through the linked `cxx` bridge. Each `BrowserController` owns a `RuntimeConnection` with a separate service instance. The bridge transports typed commands and owned results within the desktop process. The CLI uses the same Rust service directly. [System architecture](system-architecture.md), [Desktop sessions](../engineering/application-sessions.md)

## Handles, threads and ownership

`new_runtime()` starts the Rust command and task threads. The temporary `Runtime` gives out one `CommandClient` and one `EventReceiver`. The GUI thread uses the command handle. A dedicated Qt receiver thread owns the event handle and blocks in `next_event()`. The Rust command thread owns the current archive and its index. The separate task thread owns the prepared job while executing it.

CXX transports value types and owning Rust containers. The receiving Qt code holds or converts these values while it owns them. No archive or native file handle crosses into Qt. When IDs or exact byte counts reach QML, the controller uses strings for values that need to preserve all 64 bits. The generated headers and static library are built together with the host. This interface belongs to the product build. The bundled worker uses a separate [process protocol](../engineering/native-codecs.md).

[Bridge declarations and conversions](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-qt-bridge/src/lib.rs), [Qt connection](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/RuntimeConnection.cpp), [Rust service](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/service.rs)

## Request, query and task identities

| Identity | Purpose |
| --- | --- |
| Request ID | The Rust service assigns an ID on admission, scoped to the connection. `Submission` returns `Accepted`, `Busy`, `Closed`, `Invalid` or `Exhausted`. Rejections return ID zero. |
| Session ID | Identifies the currently open archive snapshot. Commands that act on that archive must match it. New opens get a new session ID. |
| Query identity and offset | A page is bound to the session, query ID, UI generation, revision and offset. The controller validates these values before accepting rows. |
| Physical entry or navigation ID | A physical entry keeps its identity across sorting and pagination. Synthesized directories carry their own identities. A row index, display name or query range never grants access to a filesystem path. |
| Task and prompt IDs | A task ID belongs to its connection. A prompt identifies one pending conflict or password response within that task. The application task entry also knows which controller owns it. |
| Event sequence | The receiver assigns a monotonically increasing sequence per connection as it takes events. Qt drops events whose sequence is no newer than the last one consumed. Task updates share this sequence and use the original request ID for correlation. |

## Commands in the current bridge

| Command or method | Purpose and ordinary result |
| --- | --- |
| `Open`, `OpenWithPassword` | Open and scan the selected source. A successful response includes the first `Page` and bounded navigation data. An encrypted directory may require a password before a session opens. |
| `Browse` | Start a query or continue it with matching identity and offset. Each command returns at most one `Page`, bounded to 128 rows and 64 KiB of names. |
| `FreezeSelection`, `Preflight` | Resolve current selectors to owned physical or navigation IDs in `Selected`, or summarize metadata findings in `Preflight`. The snapshot and selected scope are revalidated for the actual operation. |
| `Preview` | Resolve one physical entry. On success, return owned, bounded `Preview` bytes along with the entry and session identity. |
| `CheckTarget`, `InspectInputs` | Return the leftover count in `TargetChecked` after the runtime opens and cleans a probe workspace. The creation check uses session zero. `Inputs` reports shallow metadata in input order, without recursively scanning directories. Neither response grants permission to commit. |
| `Extract`, `Create`, `Check`, `OpenEntry` | Validate the intent and enqueue a task. `TaskAccepted` contains the assigned task ID. Target authorization and commit are performed by the runtime and I/O layer. |
| `Update`, `Move`, `Export`, `PublishExport`, `ExportManifest` | Archive edits and data publication enter the heavy-task queue. After admitting `Update` or `Move`, the service drops the current archive snapshot so later requests cannot use its old entry IDs. `ExportManifest` returns relative transfer metadata as an ordinary response. |
| `Snapshot`, `CloseSession` | Return the current archive session ID or zero. Closing the identified session drops its browsing snapshot while independently retained task inputs can remain alive. |
| `cancel`, `cancel_task`, `decide_task`, `respond_password`, `release_export`, `shutdown` | Control methods outside the ordinary command queue. Request cancellation has its own request ID and `Finished`; task controls operate on the existing task. |

The Rust service exposes additional CLI-specific operations such as `Catalog` and `SelectMembers`. The bridge validates input size and enum values before submission. For example, creation paths are bounded to 4 KiB each and 4 MiB in total. The runtime validates session and operation semantics, while the I/O layer opens files and obtains output capabilities. [Runtime service contract](../engineering/product-runtime.md), [CLI contract](../engineering/command-line.md), [File authorization](file-security.md)

## Event delivery and backpressure

An admitted ordinary request produces at most one data result followed by one `Finished` under normal delivery. An error is represented by `Finished` carrying an error code. The controller waits for successful completion before publishing the matching page or preview. A superseded result is discarded while its request still finishes. The large-index path caches up to 1,024 rows in Qt and requests missing pages. It freezes a cross-page selection before presenting extraction confirmation. [Large archive indexing](../engineering/large-index.md), [Content preview](../engineering/content-preview.md)

`TaskAccepted` and `Finished` belong to ordinary request delivery. `TaskUpdate` uses separate per-task pending slots and carries the original request ID, task ID and source session ID. An update may arrive before `TaskAccepted` or after `Finished`. The controller correlates an early task with the original request and routes subsequent updates to that task. A later task update never completes the ordinary request again. The pending slots retain the latest snapshot for a task. Task progress can be coalesced, and the terminal snapshot persists until it is taken by the receiver. Prompts appear in snapshots as long as the task waits for their response. [Task state and cancellation](task-scheduling.md)

- **Rust.** Per connection, ordinary admission allows 128 outstanding requests and an 8 MiB queued-command byte budget. The ordinary event payload budget is 32 MiB. Accepted ordinary requests reserve room for their result and completion. Request-cancellation acknowledgements have their own bounded queue, and shutdown does not need space in it.
- **Qt.** The mailbox holds up to 256 events, up to 128 payload events and 32 MiB of accounted payload. If capacity is exhausted, the receiver waits for the GUI to drain it. A single queued notification chain drains up to 32 events or 4 ms per turn. The notification goes to the GUI thread and events retain owned data while queued.

## Prompts, failures and shutdown

- Conflict decisions use `decide_task`. Task passwords use `respond_password` and a distinct password prompt ID. Both validate the current, unanswered prompt. The password response enters zeroizing Rust storage and never appears in a result or progress event. `OpenWithPassword`, `Preview` and encrypted creation carry operation-specific input secrets that the bridge moves into their corresponding Rust password types. [Password handling](../engineering/password-extraction.md)
- Stable error codes describe admission, request and task outcomes. Examples are `stale_session`, `stale_query`, `source_changed`, `unsafe_target`, `target_changed`, `disk_full` and `password_required`. The controller translates codes for display. A source change invalidates the relevant browsing snapshot. `RuntimeFault` and unexpected disconnects invalidate pending UI work with `runtime_unavailable`; connection loss can prevent the original terminal events from arriving.
- `shutdown` closes admission and cancels accepted work, including tasks waiting for input. The Qt receiver keeps draining events as the Rust threads exit. The Rust receiver joins them before yielding `ShutdownReady`, then the bridge yields `End`. Qt reports a clean disconnect after its mailbox drains. If the connection is destroyed early, it closes the mailbox and wakes the receiver without joining it on the GUI thread. [Task shutdown](task-scheduling.md), [Runtime lifecycle](../engineering/product-runtime.md)

For changes at this boundary, the [bridge probe](https://github.com/xiaodaozhi/xdzip/blob/main/tools/bridge-probe/README.md), Rust service tests and [desktop controller tests](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/tests/browser_test.cpp) cover different layers. Use the [developer build guide](../engineering/developer-build.md) to locate the relevant targets.
