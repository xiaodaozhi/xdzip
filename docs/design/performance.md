# Performance design and measurement

English | [简体中文](../zh-CN/design/performance.md)

XDZip bounds the memory and work of indexing, transport, task execution and image preview separately. The code limits below are implementation rules. Timings, memory measurements and throughput depend on the build and workload. The reference targets later on this page are engineering goals.

## Limits and mechanisms in the current code

| Area | Current behavior and cost |
| --- | --- |
| Ordinary single-file ZIP/ZIP64 | The streaming scanner admits up to one million physical records, subject to name, directory and read budgets. When hot metadata exceeds 64 MiB or a new record would exceed 100,000 resident entries, records move into private disk storage. The metadata file is bounded to 512 MiB, with a separate position table. Browsing scans metadata rather than decoding every payload. |
| In-memory and disk queries | The runtime builds the current immutable result order once. The disk path spills sort runs of up to 8 MiB, then merges them to a token order. Name and sort storage are bounded. The cache contains the current query rather than a full rendered view or indefinite query history. |
| Navigation and Qt transfer | The directory tree has a 32,768-node, 64-level and 4 MiB name budget. If the tree exceeds its budget the runtime can retain root browsing and search. The bridge sends up to 128 rows and 64 KiB of names per page. Large views retain up to 1,024 resident rows, with a logical row count covering the entire result. |
| Commands, tasks and writes | The metadata and task threads use separate bounded queues. Heavy desktop tasks are admitted one at a time. Decode work, output bytes, file commits and worker resources have different accounting. Solid and streaming formats can read or decode preceding data before reaching a selected item. |
| Preview and native workers | Text and encoded images have separate byte limits. Image dimensions and pixel count, worker execution and thumbnail size are bounded independently. Format workers use their own protocol and process limits. |

[Disk index implementation](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/disk_index.rs), [Large archive indexing](../engineering/large-index.md), [Qt/Rust bridge limits](qt-rust-bridge.md), [Task admission](task-scheduling.md), [Output budgets](file-security.md), [Preview budgets](../engineering/content-preview.md)

The memory cost includes Qt, fonts, rendering and models, archive metadata, the position table and query buffers, decoder state, child processes, and I/O buffers. The 64 MiB migration trigger is a metadata threshold. Process-tree memory also includes everything outside that index. Larger metadata admission does not enlarge extraction quotas or enable whole-archive testing and editing for disk-backed sessions. Private indexing writes and extraction staging are separate disk costs.

## Measuring the current implementation

| Entry point | What it measures or verifies |
| --- | --- |
| `scripts/probe-large-index.py` and `xdzip-m2-probe` | Generate an owned ZIP64, exercise the real scanner, sort and paged queries, and report phases and storage separately. Include duplicates, long names and directory structure, and identify whether the disk path was reached. |
| `scripts/measure-desktop.py` and `xdzip-process-metrics` | Measure the desktop process tree, startup and browsing. With the extraction option, drive a real extraction and verify output bytes independently. Record the renderer, build identity and exact workload. |
| `scripts/run-performance-suite.py` | Collects Release observations for startup, steady idle and browsing, and compares extraction against a locked core probe on owned fixtures. The report records build and fixture identities. Its comparison is observational and retains reference-machine approval as a separate condition. |
| `scripts/check-runtime-resources.py` | Checks the runtime resource and font receipt from the probe. It establishes resource identity and coverage for that probe, without measuring interactive latency. |

[Large-index probe](https://github.com/xiaodaozhi/xdzip/blob/main/scripts/probe-large-index.py), [Desktop measurement](https://github.com/xiaodaozhi/xdzip/blob/main/scripts/measure-desktop.py), [Performance suite](https://github.com/xiaodaozhi/xdzip/blob/main/scripts/run-performance-suite.py), [Runtime resource probe](https://github.com/xiaodaozhi/xdzip/blob/main/scripts/check-runtime-resources.py), [Evidence collection](../../developers/engineering/candidate-verification.md#性能与资源证据)

For a comparison, record commit/configuration, OS and architecture, renderer, hardware, number of runs, fixture, device and cache conditions. Keep cold and repeated starts separate. Report first interaction, archive scan, query ordering, page transfer and visible rendering as distinct phases when the measurement supports them. Distinguish cancellation feedback from actual stop and cleanup. Separate input, decoded and committed bytes when reporting throughput.

The process-tree tool labels Linux PSS, macOS physical footprint and Windows private commit distinctly and also reports RSS/working set where available. Compare the same metric under comparable conditions. Report download and installed sizes for actual packages separately. Offscreen timings and prototype runs describe only their own conditions.

## Workloads for comparative tests

| Workload | Measurements |
| --- | --- |
| B01/B08 Empty window, idle and theme changes | Startup and frame timing, idle CPU, retained memory and renderer. |
| B02 100,000 and one million ZIP records | Metadata scan, migration, sorting, page retrieval and separate memory and disk costs. |
| B03/B04 A large ordinary file or many small files | Throughput, cancellation, destination work, synchronization and cleanup. Small files can be dominated by file operations. |
| B05/B06 Highly compressed or deep-stream entries | Quota handling and the extra decode work before reaching a selected entry. |
| B07 Pending heavy work while browsing | Search and page latency while one heavy operation runs and another waits. |
| B09 Bundled workers and the complete package | Process-tree usage, protocol cost, download size and installed size. |

These are comparative scenarios. Select the workload that matches the change and use project-owned fixtures with recorded sizes and hashes.

## Reference targets and planned research

The existing design uses an Apple Silicon reference with 16 GiB memory, local SSD, 60 Hz display, system font, real GPU and a Release build. The targets are candidates for comparable measurements. They make no assertion about the latest release or a different platform.

| Scenario | Design target |
| --- | --- |
| Repeated empty-window startup | P95 at most 500 ms to an interactive frame, with at least 30 runs |
| Open and query 100,000 cached ZIP records | P95 at most 1 s to prepare metadata and at most 80 ms from indexed query to visibility |
| Warm page jump and GUI scheduling | P95 at most 50 ms for a visible page jump and 8 ms for continuous GUI-thread blocking |
| Memory | Process-tree target of at most 150 MiB empty and 256 MiB when browsing 100,000 entries without preview or a decode dictionary |
| Idle CPU | Average at most 0.5% of one logical core after stabilization |
| Cancellation | P95 at most 50 ms for feedback and 250 ms for local cooperative streaming stop, measured separately |
| Extraction and distribution | Compare GUI and core under identical workload and I/O settings, with a design target of at most 10% throughput loss. Single-architecture Mac targets are at most 100 MiB downloaded and 250 MiB installed. |

Block-level parallel extraction, persistent index reuse and a shared CPU-token scheduler remain design options. Profile the current serial task path and memory/disk paths before changing concurrency or storage. Maintain equivalent byte verification, cancellation and safe publication in any comparison. [Architecture evolution](architecture-evolution.md), [File publication](file-security.md)
