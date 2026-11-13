# xdzip: Terminology, units, and initial constants

## 1. Terminology

| Term | Definition |
|---|---|
| Archive | A container or supported compressed stream and its authorized source |
| Codec | An implementation that detects, scans, reads, or creates a format |
| Session | Open source, backend, index, and browsing context; not a task |
| Entry | An actual archive record; a directory may also have a synthetic display node |
| QuerySnapshot | A view with fixed source/index version and result order |
| Generation | Frontend request replacement relationship that prevents stale results from overwriting newer ones |
| OperationPlan | An operation description with fixed scope, destination, capabilities, parameters, and policies |
| Task | Lifecycle and result of executing a plan |
| Grant | Host-held file/directory access authority, not a path string |
| Sink | A write interface under metering, quota, and transaction control |
| Worker | A separate process; sandbox-constrained only when permission restrictions are implemented |
| Commit | Safe point publishing verified staged content as a final file |
| Mandatory-delivery event | Result/state/terminal event that cannot be coalesced away while the runtime connection is healthy |
| Capabilities | Specific intersection of implementation, format features, platform, and policy |
| Solid | Multiple files share compressed blocks; selective reading may require decoding preceding content |
| First presented frame | Actual rendered presentation; an ui_ready log cannot substitute for it |

## 2. Units and values

Memory/file quantities use binary KiB/MiB/GiB, with 1 MiB = 1024² bytes. Transport and accounting retain integer bytes. Time uses ms or s; event timestamps come from a monotonic clock, while user file times use calendar time with precision/origin. Throughput consistently uses MiB/s and specifies whether bytes are compressed input, decoded data, or output.

CPU at 100% of one core means one logical core fully occupied; a process tree may exceed 100%. PSS/RSS/USS/physical footprint are not equivalent. Compressed download size, installed size, and executable size are recorded separately.

## 3. Initial constants

This table records initial design values and future target constraints, not current product settings or measured optima. The current product's 100,000-entry scan limit, single-session browsing per connection, and file-output quotas are governed by [read-only indexing](../engineering/m2-readonly.md), [safe extraction](../engineering/m3-extraction.md), and the [creation contract](../engineering/m4-creation.md). Application-level multiple sessions, the 32-live-session limit, and serial heavy-task admission follow the [application sessions contract](../engineering/application-sessions.md). The UI exposes only implemented settings. Adjustable user limits must pass core validation; wire-protocol hard limits cannot be arbitrarily relaxed within the same protocol major version.

| Constant | Initial value | Category/notes |
|---|---:|---|
| Default/maximum list page | 128 / 256 rows | Application contract; fixed page maximum |
| Qt page cache | 8 pages, 1,024 rows total by default | UI policy; density optimization must remain within memory budget |
| Search debounce | 120 ms | Committed input only; does not trigger on intermediate IME composition |
| Ordinary outstanding requests | 128 | Cancellation/shutdown have separate admission |
| Mandatory-delivery event queue | 256 | Progress uses separate coalesced slots; terminal states are not overwritten by progress |
| Queued/active heavy tasks | 32 / 2 | Reduce further for insufficient resources or conflicts |
| Default CPU tokens | `max(1,min(4,cores-1))` | Backend threads count against the same pool |
| Total codec admission memory | 512 MiB | Excludes GUI/index; in-process estimates are not OS hard isolation |
| Default third-party process memory | 256 MiB | Enforced by platform hard limits; re-admit when higher needs are declared |
| Hot index budget/disk-migration preparation threshold | 128 MiB / 64 MiB per session | In v0.1, explicitly reject overages when disk backend is unfinished; do not OOM |
| Global index budget/open-session limit | 256 MiB / 8 | Indexes referenced by tasks also count |
| Unstarted plan limit/plan-result hot data | 8 / 64 MiB globally | Large data moves to disk later; first version returns resource errors on overage |
| Task history summaries | Latest 100 | Full results cleaned on explicit close/expiry; active references are retained |
| Actual per-file/task output limits | 8 GiB / 20 GiB | User may explicitly raise them with a new plan |
| Default scan-entry limit | 1,000,000 | Can be raised, subject also to index budgets |
| Per-metadata-field limit | 64 KiB | Raw names, comments, etc. have separate limits; platform output paths are usually smaller |
| Format stream composition depth | 4 | Nested archives are not automatically expanded recursively |
| Maximum magic probe read | 64 KiB per candidate | Candidate count is also bounded; arbitrary full scans are forbidden |
| Stream buffer/IPC data block | 256 KiB | May be optimized independently; IPC remains within protocol hard limits |
| Maximum IPC CBOR frame | 1 MiB | Check length before allocation |
| CBOR nesting depth | 16 | Reject unbounded lengths and duplicate key fields |
| Unacknowledged IPC data blocks | 4 per direction | Credit management; data limits cannot crowd out control messages |
| EntryBatch maximum rows | 128 | Must also fit the 1 MiB frame limit; do not blindly send 128×64 KiB |
| Handshake deadline/cancellation cooperation deadline | 2 s / 2 s | Host cleanup remains necessary after termination |
| Progress coalescing frequency | 10 Hz | States/prompts/terminal states are immediate, without waiting for the interval |
| Application shutdown soft wait | 5 s | On timeout offer wait/force quit, not false success |
| High compression ratio warning | >1000:1 | Warning only; absolute byte/resource quotas are hard protection |
| Text preview/image content/pixels | 1 MiB / 20 MiB / 40 MP | Explicit user preview; check before pixel allocation |
| External temporary-open limit | 100 MiB | No automatic execution of programs/scripts |

Implemented [password extraction](../engineering/password-extraction.md) counts at most ten password-verification-value failures per physical entry. Authentication or structural corruption stops immediately; each prompt has a one-shot ID. The 32 authorized sources mentioned in the table are an initial design value. Current ZIP, 7z, RAR, and CAB split-volume reading limits are specified by the [split-volume contract](../engineering/archive-opening.md). Waiting for a user does not reset or expand output quotas.

## 4. Standard state names

Standard task names are Queued, Preparing, Running, WaitingForUser, Committing, Cancelling, CleaningUp, Succeeded, Failed, and Cancelled. Code enums and event schemas derive from these semantics; UI translation does not change protocol values. Archive index states are Detecting, Scanning, Ready, Failed, and Closed.

## 5. Versioning rules

Design baseline version, application version, Rust SDK SemVer, internal build-bridge version, external wire-protocol version, and persistence schema version are managed separately. Product v0.2 does not automatically imply plugin protocol 0.2. Clearly mark draft protocols experimental before freezing; draft updates also update examples and conformance tests.

## 6. Reference authority

Product scope is governed by [01](01-product-requirements.md), internal services by [05](05-qt-rust-bridge.md), third-party protocols by [06](06-codec-extension-sdk.md), file-output policy by [07](07-file-security.md), scheduling by [08](08-task-scheduling.md), and engineering budgets by [09](09-performance.md). This table centralizes defaults; update references and related acceptance when values change.
