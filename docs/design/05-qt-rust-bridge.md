# xdzip: 05 Qt/Rust type bridge and application service protocol

## 1. Decision and versioning

The production product uses `cxx` to generate the Rust/C++ build-time bridge. The application, Rust static library, and generated headers must come from the same commit and toolchain combination; arbitrary-version dynamic libraries must not be substituted. This is an internal product interface, not a third-party binary SDK or the process protocol in [06](06-codec-extension-sdk.md).

The prototype `lab_engine_new/lab_request/lab_string_free/lab_engine_free` and JSON requests remain experimental code. The product must not gradually add passwords, output paths, and plugin actions to the experimental interface and then claim it is frozen.

The following tables define conceptual types and semantics, not directly compilable `cxx::bridge` declarations. Implementation splits structures according to cxx's supported shared types; generated code is built, not edited by hand. Raw pointers with undefined lifetimes must not be introduced for performance convenience.

The product already implements dual handles, cancellation/shutdown control channels, a Qt mailbox, query generations and source versions, plus TaskId and separate output authorization for creation/extraction. The application supports independent archive sessions. Each BrowserController uses its own single-session RuntimeConnection; ordinary requests remain serial within each connection, while heavy tasks are admitted serially across the application. See the [application sessions contract](../engineering/application-sessions.md). The third-party process protocol, CPU token pool, and some target commands below remain future design. See [M1 runtime](../engineering/m1-runtime.md), [M2 queries](../engineering/m2-readonly.md), [safe extraction](../engineering/m3-extraction.md), and the [creation contract](../engineering/m4-creation.md) for actual APIs and resource/failure boundaries. Do not treat the design tables as current cxx declarations.

## 2. Client and event reception

The bridge creates two exclusive handles: `CommandClient` and `EventReceiver`. Internally, they reference the same Rust Runtime through Arc/bounded channels; C++ owns them through RAII wrappers.

- The GUI thread uses CommandClient to submit short commands; entry points do not wait for work to finish or operate on QObjects.
- A Qt receiver thread exclusively owns EventReceiver and calls the blocking `next_event()`. Runtime/receiver shutdown wakes it immediately, without long polling.
- The receiver converts events into value objects and puts them in a C++ mailbox with capacity 256. At most one “pending events” notification using an explicit `QueuedConnection` is outstanding to the GUI, rather than one Qt callback per Rust event. The GUI drains by batch/time budget per turn and schedules another turn if data remains. QObject destruction cancels the corresponding notifications; do not use lambdas that capture a dangling `this`.
- The GUI command and receiver handles are not called concurrently across threads; Rust shared state is protected by types and locks/channels. Interface documentation must identify which objects are Send; Arc alone does not imply that everything is thread-safe.
- Shutdown retains a control signal unaffected by ordinary queue capacity. Before closing the receiver, explicitly decide whether terminal states continue to drain, preventing worker threads from blocking on a full event queue and making join impossible.

## 3. Common envelope

A command envelope contains `RequestId`, optional SessionId/TaskId, command kind, and typed parameters. Submission immediately returns `Accepted` or `Rejected(Error)`. Accepted means enqueued, not that the operation succeeded.

An accepted ordinary request eventually produces one `RequestFinished {request_id, outcome}`. An asynchronous task start returns TaskId as the outcome; the task's terminal state is reported separately through `TaskUpdated`. Cancellation/shutdown control has an independent admission channel but still carries RequestId. A runtime crash cannot guarantee a final response; a disconnected Qt connection completes all pending requests as `runtime_unavailable`.

The current product's separate `TaskUpdate` envelope retains the original RequestId that created the task. TaskSnapshot retains TaskId and source SessionId. Task snapshots and admission receipts have no guaranteed ordering; the GUI uses full correlation to avoid duplicate preparation placeholders rather than guessing from arrival time or task counts. Task snapshots after the original request's Finished still carry the same correlation ID and do not constitute a second ordinary request completion.

FetchPage content is carried exactly once by PageReady. RequestFinished references the page identity and ends the pending request; discarding a stale page must still finish the corresponding request. State events and request completion must not create tasks or apply data twice.

Events carry `event_sequence`; task events additionally have monotonic `task_sequence`. Page results carry full query identity. Transport order is not a cross-task ordering guarantee. Once a TaskId reaches a terminal state, older running/progress events are not accepted.

## 4. Application service commands

| Command | Required parameters | Result/constraints |
|---|---|---|
| `OpenArchive` | User-authorized SourceRef, probe options | SessionId, followed by index events; does not extract |
| `CloseSession` | SessionId | Releases browsing; does not implicitly destroy running tasks |
| `CreateQuery` | SessionId, QuerySpec, generation | QueryId, completeness; old generations may be canceled |
| `FetchPage` | QueryId, revision, offset, limit, generation | Immutable Page; limit 1–256 |
| `SetSelection` | QueryId, SelectionDelta, selection revision | New SelectionRevision and counts; rejects stale ranges |
| `PrepareExtraction` | Session, scope snapshot, destination authorization, policy | PlanId/Revision and summary; planning may be asynchronous |
| `PrepareCreation` | Set of authorized inputs, format, destination authorization, parameters | New archive plan; detects recursive input/output |
| `StartPlan` | PlanId, PlanRevision | TaskId; a duplicate RequestId does not create another task |
| `CancelTask` | TaskId | Cancellation request accepted; terminal state reported separately |
| `CancelRequest` | RequestId to cancel | For work without a created task, such as queries/planning; original request finishes canceled or already completed |
| `ResolvePrompt` | TaskId, PromptId, typed decision | Handles only the current prompt; stale prompts return `stale_prompt` |
| `SupplySecret` | PromptId, SecretRef | Only for the corresponding phase; never written into ordinary events |
| `FetchTaskResult` | TaskId, cursor, limit | Bounded result page; does not send a million failures at once |
| `GetCapabilities` | Session or format ID | Current backend version, specific capabilities, and disabled reasons |
| `GetRuntimeSnapshot` | None or known event_sequence | Bounded session/task summaries and snapshot sequence; new views resume events from it |
| `RequestShutdown` | Graceful shutdown policy | Stop admission, cancel/complete cleanup, emit ShutdownReady |

SourceRef/TargetRef are host references created by a platform authorization factory, not arbitrary strings sent by plugins. Qt is a trusted frontend; after the user selects a path in a system dialog, the Rust platform layer opens and validates an authorized object. Filenames and format parameters remain untrusted data.

CancelRequest and CancelTask share independent control admission. Snapshots contain no page data, passwords, or full results. The GUI ignores summary events older than the snapshot sequence, then applies events by sequence. Closing a session stops unreferenced scanning; sources and indexes held by tasks remain alive for those tasks' lifetimes.

## 5. Events

| Event | Content | Delivery policy |
|---|---|---|
| `RequestFinished` | Request result or Error | Delivered exactly once, except when the runtime disconnects |
| `SessionUpdated` | Probe/scan/Ready/Failed, index version | State transitions must be delivered; scan counts may be coalesced |
| `PageReady` | Page and query identity | Stale pages may be dropped; active requests receive completion |
| `SelectionUpdated` | Revision, count, and bounded visible changes | May be coalesced within a session, retaining only the latest revision |
| `TaskUpdated` | State, phase, counts, bytes, sequence | Progress may be coalesced; waiting and terminal states must be delivered |
| `DecisionRequired` | PromptId, conflict/password/quota summary, etc. | Must be delivered; contains no password or unescaped rich text |
| `CapabilitiesChanged` | Backend additions/failures and reasons | GUI recalculates action availability |
| `RuntimeFault` / `ShutdownReady` | Nonrecoverable error/shutdown complete | Control messages that progress must not crowd out |

The ordinary request entry point permits at most 128 outstanding requests. Duplicate page requests are coalesced; return `busy` when full. Events use a capacity-256 mandatory-delivery queue and per-object coalesced progress slots. If the receiver is slow, stop admitting ordinary requests and drop stale pages/low-priority progress. Workers must not wait for the GUI while holding a file commit lock. Waiting tasks are bounded by task queue limits, so prompt and terminal-state occupancy is also bounded.

The C++ mailbox also coalesces progress and reserves capacity for control, waiting, and terminal states. When full, the receiver pauses ordinary event reads and propagates backpressure to Rust. Limiting only the Rust queue cannot bound Qt queued-event memory. The scheduler must not block cancel/shutdown on ordinary event enqueue. If necessary, stop new work and retain bounded pending control results. During shutdown the GUI continues draining; the GUI thread must not join a receiver that is still waiting for mailbox space.

## 6. Data and ownership

Small DTOs are copied by value; strings are UTF-8. Raw non-Unicode path bytes are not passed directly to QML. Page entries contain only display fields and IDs; full attributes are queried separately. Cross-thread results own their data and do not borrow index buffers that may still change.

Large content is not sent to QML through the bridge. Preview receives at most a quota-limited buffer or controlled temporary authorization. cxx Rust/C++ containers are destroyed by their corresponding types; `free/delete` must not be used on Rust-allocated memory. Secret objects are separate from ordinary String DTOs, explicitly destroyed/invalidated, and use zeroizing containers where possible.

Rust Result becomes a structured Error at the boundary; C++ exceptions do not cross back into Rust. Entry points validate numeric narrowing, enum values, object states, lengths, and handle ownership. Unrecoverable panics terminate/report according to release policy; do not catch panic to maintain a half-completed transaction.

## 7. Error contract

`Error {code, category, retryability, context_id, safe_detail}`. Category is input, capability, permission, integrity, resource, cancelled, backend, or internal. safe_detail is only for escaped details and contains neither secrets nor full paths by default. The UI translates the main message from code.

| Example error code | Meaning | Recoverable action |
|---|---|---|
| `unsupported_format` / `unsupported_feature` | No backend or current capability does not support it | Change backend or cancel; do not pretend to continue |
| `stale_query` / `stale_plan` / `source_changed` | Snapshot or input has changed | Refresh, replan |
| `path_unsafe` / `name_collision` | Invalid output or mapping conflict | Skip/explicitly rename, not direct overwrite |
| `quota_exceeded` / `disk_full` | Output/resource limit reached | Do not commit the current item; replan or cancel after a decision |
| `password_required` / `wrong_password` | Secret required or authentication failed | Bounded retries; do not log passwords by default |
| `archive_corrupt` / `checksum_mismatch` | Structure/content integrity failure | Discard current output and report failure in the summary |
| `backend_crashed` / `plugin_protocol_error` | Worker process or protocol failure | Isolate the extension; retry requires a new plan |
| `permission_denied` / `io_error` | Platform file access failed | Retain path context in local details |

## 8. Build and migration verification

CMake builds the Qt application; Cargo builds the Rust static library and cxx-generated artifacts. Use one build orchestration entry point and explicitly specify target triple, Qt architecture, compiler, C++ standard, and generated-header directory. Do not use another generated header merely because it happens to exist on the system.

The first stage verifies minimal round trips, Unicode, ownership, full queues, pending pages during shutdown, the cancellation priority channel, and RuntimeFault on macOS arm64, Windows MSVC, and Linux x86_64. Migrate the prototype pagination model only after those pass. The cxx dependency version is locked to the verified commit; the design does not invent a version number claimed to have passed testing.

## WP-10 current creation slice

The internal bridge adds Create: input path array, destination directory, ZIP name, Store/Deflate, levels 1/6/9, and name-conflict policy. Qt rejects invalid values before UTF-16→UTF-8 conversion and integer narrowing; Rust additionally checks 4 KiB per path, 4 MiB total selected paths, entry count, and portable filenames. CheckTarget with session=0 means a standalone creation destination check; a nonzero session remains bound to the extraction source.

Creation and extraction share background task identity, cancellation, prompt IDs, event budgets, and asynchronous exit. TaskSnapshot adds creating and total_known; an unknown scan total must not appear as a zero-byte task. The Verifying phase does not count reread verification bytes as processed input. Creation saves the complete archive only once; input entries cannot be counted as saved early. See the [WP-10 contract](../engineering/m4-creation.md) for fields, errors, and whole-archive commit semantics.

The creation selection summary is read through the internal InspectInputs command, which returns Inputs in input order followed by one Finished. A request allows at most 256 absolute paths, each at most 4 KiB, within the existing request and event byte budgets. Background processing uses symlink_metadata, does not recurse into directories, and does not follow selected leaf links. Regular files report size; directories, errors, and special objects do not masquerade as known totals. Qt displays only results corresponding to the current path set and coalesces rapid changes to the latest set. Destination checks and summary requests run when the metadata channel is idle and do not create heavy tasks. Starting creation still reauthorizes, scans, and freezes the input tree; display data is not used as a commit basis.

## WP-14 password extraction slice

Password responses use a separate `respond_password(task, prompt, password)` control entry point, not the ordinary `Command`. At the cxx boundary, the incoming string is first moved into a zeroizing container dedicated to extraction, then the current task, prompt, cancellation, and capacity admission are checked. Rejection also consumes and clears the secret. Qt temporarily holds a bounded UTF-8 buffer only during the synchronous call and does not send queued signals containing passwords. Creation and extraction password rules are validated separately and must not be conflated because window controls are shared.

`TaskSnapshot` fields `password_prompt_id`, `password_prompt_name`, and `password_retry` are separate from the name-conflict `prompt_id`. `waiting_password` describes waiting only, without secrets or summaries. The GUI submits responses using exact string IDs for the task and prompt, preventing duplicate, stale, and cross-type responses. Cancellation/exit wakes waiters; native menus and task windows use the same waiting state. See the [password extraction contract](../engineering/password-extraction.md) for protocol, authentication, and cleanup boundaries.

## Large-index and bounded-preview slice

Current product pages are limited to 128 rows and 64 KiB of name bytes; the conceptual maximum of 256 in the table above is not an implementation parameter. Large queries carry only a logical total row count and on-demand pages; Qt caches at most 1,024 rows. Page requests carry session, query, generation, revision, and offset. Apply a page only when both its receipt and unique Finished match; old pages cannot replace a new query or selection. Physical IDs do not change with sorting or disk storage.

`FreezeSelection` accepts a session and bounded query-range selector. Rust validates full query identity, source, and task budget, returns an owned `Selected` ID list, and then finishes the request. Qt must finish freezing before displaying extraction confirmation; the confirmation window does not hold a q-range whose meaning changes with user sorting. Neither ranges nor pages carry file authorization; see the [large-index contract](../engineering/large-index.md).

`Preview` accepts a session, one physical entry, and an optional decoding password, using request cancellation. A bounded `Preview` result contains content type, entry ID, and owned bytes, accounted against Rust event and C++ mailbox byte budgets. Display only after complete successful termination and while request/session/entry still match. Password failures reuse the host password window with bounded retries; secrets are not returned through ordinary events. Qt displays plain text or uses a controlled asynchronous image provider per engine. Tokens carry neither data URLs nor paths. Selection, query, hiding, closing, and cancellation invalidate content; see the [preview contract](../engineering/content-preview.md).

Image preview kind uses fixed u8 values: 1 plain text, 2 PNG, 3 JPEG/JFIF, 4 GIF, 5 BMP, 6 ICO/CUR, 7 WebP, 8 TIFF, 9 TGA, 10 ICNS, 11 static SVG, and 12 macOS HEIC/HEIF. Rust publishes content only after archive reading and verification complete. Qt does not extend unknown kinds or automatically guess other formats from system plugins. A private bounded process converts images to bounded RGBA; details, selection, and session identity remain frozen and invalidated by existing controllers. See the [preview contract](../engineering/content-preview.md) for complete budgets.
