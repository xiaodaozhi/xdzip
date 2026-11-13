# xdzip: 12 Implementation roadmap and prototype migration

## 1. Current implementation and future design

The current product implements a dual-handle runtime, Qt controllers, [bounded million-entry ZIP queries and pagination](../engineering/large-index.md), [limited preview](../engineering/content-preview.md), multiple read/write formats, settings, system integration, and installers. The independent [external SDK trial](../engineering/sdk-trial.md) does not enable third-party product extensions. Public versions and capabilities are governed by the root [README](../../README.md) and respective contracts. WP-12 development and acceptance completion are recorded in [#5](https://github.com/xiaodaozhi/xdzip/issues/5); `v0.1.0` release licensing and closeout are in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53).

See [repository layout](../engineering/repository-layout.md) for actual directories and dependencies. Memory/disk indexing and preview are in runtime; platform file authorization and commit are in io; the experimental plugin host is separate from the product. Independent index/platform crates and a complete extension ecosystem remain future design and cannot be inferred from the target module diagram.

An earlier independent experimental workspace used a fixed 100,000-entry sample, JSON/C ABI, and simulated tasks and provides only technology-selection experience. Experimental source and reports were not migrated into this repository; they are not evidence for real formats, file authorization, or safe output. The migration table below explains design tradeoffs and does not require reimporting experimental code or creating empty directories.

## 2. Implementation work packages

| Milestone | Work package | Exit criteria |
|---|---|---|
| M0 Baseline freeze | Toolchain/platforms, Qt license modules, build manifest, domain IDs, requirements test checklist | Support matrix is explainable; dependency versions and release routes are explicit |
| M1 Product skeleton | cxx dual handles, Qt controllers, idle events, shutdown, core tests without Qt | Minimal three-platform builds and shutdown-race checks pass; no synchronous long GUI calls |
| M2 Read-only ZIP | Detection, real directory index, queries/selection, source versions, capabilities | 100,000 real metadata entries are responsive; corruption/unsupported features are explainable |
| M3 Safe extraction | Platform authorization, path mapping, staging, CRC, conflicts, quotas, cancellation | Native link races, full disk, cancel/commit, no changes outside destination |
| M4 Creation and TAR | ZIP creation, TAR/TAR.GZ composition, input tree, safe commit, summary | Bidirectional interoperability; cancellation preserves old archives; streaming progress is real |
| M5 v0.1 private test | Mac IME/GPU/accessibility, three-platform packages and basic benchmarks | Applicable first-version P01–P12/P15 criteria met, scope explicit, no real extension execution |
| M6 Large index/preview | Disk index, one million entries, paged results, bounded preview | Resource limits and cleanup verifiable; no automatic content execution |
| M7 Additional formats | 7z, read-only RAR when conditions are met, separate encryption/split-volume capabilities | Independent interoperability/cancellation/licensing for each combination, not merely opening one sample |
| M8 SDK trial | CBOR protocol, broker, example extensions, permissions and platform sandbox | Two independent implementations interoperate; enable third parties only after platform privilege-boundary tests pass |
| M9 v1.0 | Security regression, release compliance, performance reference machine, support policy | Three-platform packages, required ad-hoc integrity, and all declared capabilities meet release gates; retain personal open-source policy without formal signing |

M1→M2→M3 is the critical path. Different contributors may develop the design system and test samples concurrently, but UI integration follows service contracts. Without sandbox expertise, first deliver a product with built-in extensions; do not block a reliable ZIP MVP or prematurely enable third-party code execution.

## 3. Prototype asset handling

| Existing asset | Reuse | Must replace/supplement |
|---|---|---|
| Qt virtual pagination model | Cache approach, generation guard, keyboard verification | QuerySnapshot identity, incremental scanning, complete model signals and result types |
| Qt QML main window | Virtualization and interaction experience; approved high-fidelity designs govern production visuals | Tokens, accessibility, IME, capabilities, real errors and task summaries |
| lab-core entry generator | Large-data UI/performance test fixture | Not used as real reading; domain types modeled independently |
| JSON/C ABI | Comparison and regression experiments | Production bridge moves to cxx; experimental ABI does not become an external SDK |
| Single simulated task | Automation workflow example | Multi-task state machine, resource pool, actual bytes, commit permits, and output logs |
| Linux measurement scripts | Environment recording and order rotation method | Native Mac/Windows, real GPUs, complete packaging, and real I/O |
| Slint/Tauri prototypes | Historical research and UI-independence evidence | Excluded from product runtime/installer dependencies |

Review threads and ownership when studying experiments; do not directly import old experimental workspace code. A prototype passing Clippy does not prove product path safety. When task polling is replaced by event reception, update benchmark cancellation metrics accordingly.

## 4. Minimum end-to-end slice and subsequent increments

The first completed product slice is a reference for splitting future capabilities: open a regular ZIP → display its real directory → select a small file → safely extract to a new private subdirectory → verify → show a result summary. It includes cxx, threads, authorization, paths, and cancellation, but did not yet support overwriting existing files.

Conflict overwrite, full extraction, archive creation, and TAR were added in later work packages. New formats should still proceed as separate minimal read-only and write end-to-end slices. Adding all formats at once mixes codec, I/O, and UI problems and makes security boundaries harder to establish.

## 5. Work allocation interfaces

UI maintainers deliver tokens, QObject models, focus, and platform interaction. Core maintainers deliver domain, runtime, index, and tasks. Secure file-layer maintainers deliver platform authorization/commit. Format maintainers deliver adapters and interoperability. Release maintainers deliver packages, required integrity and disclosure of lack of formal signing, SBOM, and benchmarks.

Specific responsibilities are assigned by work package, with explicit handoffs for dependencies, artifacts, acceptance, and risk. Security/licensing items lacking expertise remain release blockers; staffing is not assumed.

## 6. Scheduling method

Do not promise exact completion dates before validating cxx/platform file APIs. M1/M3 are the largest technical uncertainties; use technical validation to reduce uncertainty first, then plan one- to two-week acceptable increments. Each stage demonstrates real operations and failure behavior, not just static UI.

Scope expansion requires explicit requirements and capability tables. If a new format adds native libraries, passwords, split volumes, or special outputs, reevaluate security, scheduling, and licensing rather than only changing the extension list.
