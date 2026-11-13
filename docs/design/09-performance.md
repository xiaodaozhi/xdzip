# xdzip: 09 Performance design and benchmarks

## 1. Metric boundaries

Performance goals are user-perceived open/query latency, actual throughput, low idle overhead, bounded task peaks, and complete distribution size. Rust does not automatically guarantee these metrics, and one Linux cloud environment cannot establish performance on every Mac.

Existing evidence: a Qt 6.8.2 Linux software-rendered prototype recorded startup-ready marker P50 of 158 ms, median peak PSS of 81.9 MiB, and warmed Home/End reference-pixel match P50 of 15.3 ms. The 0.80 MiB figure for only the C++ executable and Rust library excludes Qt. The prototype used 100,000 fixed records and simulated tasks. It is neither a production baseline nor cold-cache, real extraction, or physical-display latency evidence.

## 2. Reference machine and initial budgets

Target reference configuration: Apple Silicon Mac, 16 GiB RAM, local SSD, 60 Hz display, system default font, real GPU, Release build, and required ad-hoc integrity signing. Actual samples must identify machine model, system, backend, and commit; see the [automated verification contract](../engineering/wp12-automation.md#性能与资源证据) for existing measurement methods. These are engineering budgets, not performance guarantees for all supported devices based on a single host observation.

| Scenario | Initial target | Measurement conditions |
|---|---|---|
| Empty window interactive | Repeated-start P95 ≤ 500 ms; report first-file-cache conditions separately | No archive loaded; startup to interactive frame, at least 30 runs |
| Prepare 100,000 ZIP metadata entries | P95 ≤ 1 s | Local SSD, directory already in OS cache; CRC content not read at this stage |
| Indexed query to visible | P95 ≤ 80 ms | 100,000 entries, Chinese/long names, search and sort; includes bridge/rendering |
| Warm visible-page jump | P95 ≤ 50 ms | Real GPU, separately capture input and frame times |
| UI scheduling | GUI continuous blocking P95 ≤ 8 ms | Browse while a large task runs; separately record maximum single long block |
| Empty-window memory | Process-tree physical memory target ≤ 150 MiB | macOS footprint or available equivalent; do not call it PSS |
| Browse 100,000 entries only | Target ≤ 256 MiB | Entire process tree, no decode dictionary/preview, stable for at least 30 s |
| Idle CPU | Average ≤ 0.5% of one core | Sample 60 s after 10 s stabilization, without tasks/motion/diagnostics |
| Cancellation request feedback | P95 ≤ 50 ms | Excludes determination that work has stopped |
| Cooperative streaming task stop | P95 ≤ 250 ms | Local I/O, block-level cancellation supported; worker limit separately in 08 |
| Archive throughput | GUI product loses ≤ 10% relative to the same core driven without UI | Same input, algorithm, threads, level, cache, and output device |
| Single-architecture Mac distribution | Compressed download target ≤ 100 MiB, installed ≤ 250 MiB | Includes Qt/QML, Rust, platform plugins, declarations, and resources |

Do not apply these warm-cache figures to first launch/system-cache changes, network filesystems, huge directories, or high-dictionary modes. Windows/Linux establish their own baselines and budgets; physical-memory metrics across platforms are not ranked directly.

## 3. Memory model

`Total = Qt/fonts/GPU + hot metadata + query index + page cache + active tasks (dictionaries + stream buffers) + workers + staging management`.

Optimization priorities: do not create 100,000 GUI objects; avoid redundant path copies; do not transfer every entry at once; bound query snapshots and result pages; reduce backend internal threads/dictionaries; only then consider unnecessary unsafe code or a custom allocator.

Metadata uses compact records and string pools/shared buffers. Raw and display paths are separated on demand. The default hot-index budget is 128 MiB; prepare to migrate to a disk backend at 64 MiB. Migration has its own peak budget and does not first duplicate the entire index. Externalize large searches/sorts or return explicit resource errors rather than waiting for OOM.

Regular single-file ZIP/ZIP64 currently uses a [million-entry streaming index](../engineering/large-index.md), switching to disk after 64 MiB hot metadata or 100,000 entries. Qt caches at most 1,024 rows for large queries, with a default transfer batch of 128. Other formats and split-volume paths remain limited to 100,000 entries. Queries of at most 1,024 rows are collected in batches and committed once; large queries retain their complete logical row count and fetch on demand. Resource admission is separate for each format and query; merely enlarging constants is insufficient. Retain at most the current query snapshot plus old snapshots needed by plans/tasks; repeated searches support cancellation and reclamation. Failure results use bounded paged result storage rather than accumulating every error string in the UI.

## 4. Throughput and I/O

Choose high-quality mature codecs first; do not write ZIP/Deflate from scratch merely to be “fast.” Initial stream buffers are 256 KiB; test 64/256/1024 KiB. Reuse buffers but do not return secret-containing buffers to ordinary pools. File reads, decoding, and output verification may be pipelined, with bounded credits/queues.

Parallel independent ZIP entries require evaluation of source seeks and destination file counts; TAR/solid 7z generally decode sequentially. Small-file workloads may bottleneck on creation, synchronization, or logging rather than compression algorithms. Report durable mode, disk cache, and security-check costs separately; do not disable default safety to improve numbers.

Backend internal threads participate in the global budget. “Smaller” creation mode may be slower/use more memory; the UI warns users. Measure the additional cost of third-party broker copies first. Replace ordinary block transfer only after ownership, boundaries, and reuse of controlled shared memory are proven.

## 5. Benchmark workloads

| ID | Workload | Observe |
|---|---|---|
| B01 | Empty window, first/repeated starts | Interactive frame, fonts, loaded modules, peaks |
| B02 | 100,000 and one million directory records, CJK/long/duplicate names | Scan, index migration to disk, query, sort, peaks |
| B03 | One 4 GiB low-compressibility file | Stream throughput, buffers, cancellation; specify generation and cache conditions |
| B04 | 100,000 small files and deep directories | Path preflight, creation cost, logs, cleanup |
| B05 | Highly compressible data, malicious declared sizes | Quotas, CPU, actual output limits |
| B06 | Small tail entry in TAR.GZ, solid formats | Sequential decode amplification, progress, cancellation |
| B07 | Two tasks with concurrent browsing/search | UI latency, fairness, CPU/memory budgets |
| B08 | Idle, unfocused window, theme switching | CPU timers, GPU, retained memory |
| B09 | Workers/installer | Entire process tree, IPC, full deployed size |

Configure quotas for throughput tests according to the declared workload in advance; do not secretly permit unlimited runtime output. Benchmark inputs include generation scripts, sizes, SHA-256, naming distribution, and licenses. Do not commit real user content.

## 6. Measurement methods

Use Release builds and save symbols separately. Record commit, toolchain, Qt modules, features, signing status, machine, OS, power mode, font, display DPI, GPU, and cache conditions. Name cold-file-cache and repeated-process-start conditions separately; do not forcibly clear system caches on every platform.

Use Instruments/system memory and frame tools on macOS, ETW/performance tools on Windows, and perf, PSS/USS, and presentation tracing on Linux. Use a monotonic clock. Instrument input, command acceptance, backend, model application, and frame presentation separately. Printing `ui_ready` establishes semantic readiness only, not a first frame.

Report median, P95, maximum, and sample count for at least 30 startup/interaction samples. Run long throughput tasks at least five times and report the distribution; do not pretend five-sample tail latency is stable. Compare a reference CLI using the same algorithm/level/thread count; comparisons across libraries identify opportunities and are not directly attributed to the UI.

## 7. Regression handling

On stable dedicated machines, 10% latency/memory changes and 5% throughput changes trigger retesting. Do not block every PR based on one fluctuation on a shared cloud runner. After reproduction, locate the CPU/heap/I/O/frame/dictionary source and verify before/after under identical conditions. Prioritize architectural and security optimization; do not reduce checks, shrink test data, or hide processes to improve metrics.

Routine CI runs functional checks and coarse resource ceilings; reference machines run the full suite nightly/before release. Optimization PRs include raw data and applicability, stating whether algorithms, compression results, or disk durability changed.
