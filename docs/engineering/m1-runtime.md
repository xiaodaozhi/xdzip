# M1 product runtime and Qt lifecycle

M1 scope follows design/12: product skeleton, dual handles, events/backpressure, shutdown, and minimal three-platform builds. M2's 100,000-entry index, M3 safe extraction, and M5 private-release gates are not completed merely by this phase.

## Implemented services

`xdzip-runtime::service` is the product implementation; diagnostic probes remain separate. The GUI owns only `CommandClient`; a dedicated Qt receiver thread exclusively owns `EventReceiver`. Each handle is taken once. cxx supplies ABI glue for this bridge; platform file-access security boundaries are separately described in the [file security specification](../design/07-file-security.md). A single Rust worker owns source files and archive snapshots; Archive handles and synchronous scan entry points are no longer exposed to Qt.

M1 commands are Open, Browse, Preflight, CloseSession, and Snapshot; all existing real browsing and preflight use them. Each accepted ordinary request gets a non-reused ID, returns at most one bounded data event, then exactly one Finished; errors return Finished directly. Pages carry session, request, and offset identity. Each RuntimeConnection has one session and serial GUI pending requests. M2 added full query ID, GUI generation, and snapshot revision; see the [M2 contract](m2-readonly.md). Cross-session requests return stale_session.

GUI ordinary admission uses try_lock without waiting for queue space, validating 4 KiB paths, 1 KiB search, 128 selected items, 21-byte directory IDs, and sort codes 0–12. Rejection consumes no ID; Busy may be retried. Cancellation has a separate control channel with at most 128 items and its own Finished acknowledgement; acknowledgement does not guarantee the target has not already finished. Shutdown retains an independent, idempotent RequestId and reserves the last available ID so it works even when ordinary IDs are exhausted or the queue is full.

The Qt controller retains only one command awaiting admission. Transient Busy is retried in the event loop using a one-shot 1 ms timer for at most 32 attempts; only sustained saturation prompts retry. This is not a completion deadline. Cancellation, shutdown, or disconnection immediately withdraws unaccepted commands; there is no retry timer when idle. Invalid arguments and exhausted IDs are reported separately rather than disguised as busy. Open is disabled after disconnection.

## Bounded transport

- At most 128 ordinary requests are outstanding, with capacity released only when the receiver takes Finished. Two event slots are reserved per request so data cannot crowd out terminal states.
- Rust data events have a cumulative 32 MiB content budget, including images of at most 16 MiB and their containers. Publication pauses beyond budget; cancellation/shutdown wakes waiting and can instead emit a cancelled terminal state. An oversized single event returns resource_limit.
- Rust commands retain an 8 MiB cumulative content limit; excess is rejected on admission and is unaffected by the larger image-event budget.
- The Qt mailbox has at most 256 events and 32 MiB content bytes. At most 128 slots are ordinary data; the rest are reserved for terminal/control events. It does not create one Qt callback per Rust event.
- One notification chain drains at most 32 events or approximately 4 ms per turn, posting another turn if unfinished; handling a single event is not preemptible. Empty queues stop scheduling, and the receiver blocks on a condition variable.
- Open's Page additionally carries the complete bounded directory index (at most 32,768 nodes and 4 MiB total names/prefixes), marked for replacement by `replace_directories`; an empty index also explicitly replaces. Browse does not resend the index; `navigation_limited` explicitly indicates downgraded indexing. Rust accounts name/prefix bytes; the Qt mailbox also counts bridge-generated node/parent ID strings. Node containers and depth have separate limits. Apply the new page and directory model together only after the corresponding request's successful Finished.
- Page `total` is current list rows; `archive_total` independently retains physical archive record count. Ordinary browsing lists direct files/directories in the current directory; search recursively returns physical matches. Directories without explicit records use `navigation_id` and `physical=false`; Qt row keys are `dN`, raw physical IDs remain separate, and virtual nodes cannot submit Preflight. Full paths remain in row metadata; display names use basename/full path for ordinary browsing/search respectively.
- String budgets are not RSS: containers, snapshots, and one event in conversion require additional space, but entries and pages remain bounded. Bridge batches contain at most 128 items. See the [M2 contract](m2-readonly.md) for the M1/M2 100,000-entry scan baseline and [large archive indexing](large-index.md) for later ordinary single-file ZIP expansion.

Control and ordinary events are taken alternately so continuous cancellation acknowledgements cannot starve ordinary terminal states. Increasing sequence numbers are assigned at delivery. Qt drops old sequences; control acknowledgements and stale request_ids do not modify the current view.

`Task`/`TaskUpdate` envelopes on the independent task channel retain the original task-creation request_id. Correlation records remain with bounded task control slots and are released when the task terminal state is delivered. They may precede acceptance data or follow request Finished. They consume no second ordinary-data slot and do not release or complete the ordinary request again; they accurately bind preparation placeholders to actual TaskId and source session. This ID is not a cancellation request ID; task control still uses TaskId.

## Shutdown, destruction, and failure

Native window close and application Quit, including macOS Cmd+Q, enter one asynchronous shutdown path. Stop admission, cancel accepted work, continue draining terminal states, and mark work finished after Rust releases sessions. The receiver thread joins the Rust worker, then receives ShutdownReady and EOF. Only after the Qt receiver ends and the mailbox drains may the GUI exit. The GUI never joins threads waiting on events or file work.

Cancellation/shutdown control uses short critical sections containing bounded state operations, independent of data-queue capacity. Cooperative cancellation cannot interrupt an OS-blocked read; this is not a hard real-time exit guarantee. Read-only requests here do not produce output transactions. Creation/extraction commits, staging cleanup, and exit results follow the [creation](m4-creation.md) and [safe extraction](m3-extraction.md) contracts separately.

If a QObject is destroyed early, close the mailbox, wake the receiver, and stop the service. Background work holds only shared queues and the Rust receiver handle, not controller pointers. Qt automatically cancels notifications for destroyed objects. QPointer also protects lifetime if an event-consumption callback synchronously destroys the connection. When abandoning reception, do not join in the destructor; work ends cooperatively. Normal ShutdownReady alone establishes that join completed.

If a read-only worker handler encounters an unwindable panic, the boundary catches it, discards the entire service, sends RuntimeFault, and ends incomplete requests as runtime_unavailable; the session is not reused. This defines read-only failure behavior only and does not promise recovery of half-completed creation/extraction transactions. Committed items, uncertain results, and cleanup for writes follow their contracts. Unrecoverable allocation failure, process abort, or poisoned internal locks do not guarantee terminal delivery. Qt uniformly ends pending states on unexpected EOF/connection failure to prevent permanent busy state.

## Verification and build

See [general verification](../qa/README.md). Rust core tests require no Qt; Qt window, model, stress, and shutdown cases are natively built and run on three platforms. macOS CI verifies arm64 and SDK-environment Cocoa startup without deploying or uploading apps; local development DMGs may be produced on explicit request. Windows/Linux deploy bundled Qt and verify SDK-free execution; see the [build guide](developer-build.md). These checks do not amount to formal release approval.

M2 initial queries allocate immutable result identity; later batches must match every QueryKey field and parameter. Qt cancels old requests and coalesces the latest intent while retaining sequence and request-ID checks. A batch also limits names to 64 KiB and does not refilter/resort. Historical M1 acceptance is not rewritten to claim M2 already existed then.

The application layer may create multiple independent connections while the domain runtime isolates sessions per connection. See the [application sessions contract](application-sessions.md) for global task identity and serial heavy-task admission.
