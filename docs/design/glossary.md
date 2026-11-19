# XDZip terminology and implementation limits

English | [简体中文](../zh-CN/design/glossary.md) | [繁體中文（台灣）](../zh-TW/design/glossary.md)

## 1. Terminology

| Term | Definition |
|---|---|
| Archive | A container or supported compressed stream and its authorized source |
| Codec | An implementation that detects, scans, reads, or creates a format |
| Session | Open source, backend, index, and browsing context; not a task |
| Entry | An actual archive record; a directory may also have a synthetic display node |
| SourceVersion | Best-effort fingerprint of one admitted source file or volume |
| QuerySnapshot | The runtime's current immutable result order, keyed by session, query, generation and revision |
| Generation | Frontend request replacement relationship that prevents stale results from overwriting newer ones |
| Prepared job | The runtime's bounded execution input, with the source, selection and options relevant to the operation |
| Task | The lifecycle and result of a job admitted to the task queue |
| Grant | Host-held file/directory access authority, not a path string |
| Sink | A write interface under metering, quota, and transaction control |
| Worker | A command or task worker is a thread in the host. The archive and image workers are separate processes. |
| Commit | Publish verified staged output through its authorized file or archive operation |
| Task snapshot | The latest report of task status, progress, prompt or outcome. Intermediate snapshots may be coalesced. |
| Capabilities | Specific intersection of implementation, format features, platform, and policy |
| Solid | Multiple files share compressed blocks; selective reading may require decoding preceding content |
| First presented frame | Actual rendered presentation; an ui_ready log cannot substitute for it |

## 2. Units and values

Memory/file quantities use binary KiB/MiB/GiB, with 1 MiB = 1024² bytes. Transport and accounting retain integer bytes. Time uses ms or s. Elapsed runtime measurements use a monotonic clock. File timestamps retain their calendar representation and source precision where available. Throughput consistently uses MiB/s and specifies whether bytes are compressed input, decoded data, or output.

CPU at 100% of one core means one logical core fully occupied; a process tree may exceed 100%. PSS/RSS/USS/physical footprint are not equivalent. Compressed download size, installed size, and executable size are recorded separately.

## 3. Where to find current limits

| Topic | Implementation reference |
| --- | --- |
| Requests, pages and mailbox limits | [Qt/Rust bridge](qt-rust-bridge.md) |
| Task admission, progress and state codes | [Task scheduling](task-scheduling.md) |
| Extracted byte limits, file authorization and publication | [File security](file-security.md) |
| Metadata and disk-backed ZIP index limits | [Large archive indexing](../engineering/large-index.md) |
| Source and volume limits | [Archive opening](../engineering/archive-opening.md) |
| Text and image preview limits | [Content preview](../engineering/content-preview.md) |
| Worker framing, supervision and resource limits | [Archive worker](../engineering/native-codecs.md) |
| Experimental extension protocol | [SDK trial](../engineering/sdk-trial.md) |
| Benchmark targets | [Performance design](performance.md) |

## 4. Versioning

The application version, internal CXX build interface, external wire protocol and persistent data schema have separate version boundaries. The product version alone does not identify an SDK protocol version. See the [codec extension SDK design](codec-extension-sdk.md).
