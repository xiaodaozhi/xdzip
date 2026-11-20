# Coding conventions and quality engineering

English | [简体中文](../zh-CN/design/quality-engineering.md) | [繁體中文（台灣）](../zh-TW/design/quality-engineering.md) | [日本語](../ja-JP/design/quality-engineering.md) | [한국어](../ko-KR/design/quality-engineering.md) | [Français](../fr-FR/design/quality-engineering.md) | [Deutsch](../de-DE/design/quality-engineering.md) | [Español](../es-ES/design/quality-engineering.md) | [Português (Brasil)](../pt-BR/design/quality-engineering.md) | [Русский](../ru-RU/design/quality-engineering.md)

## Code and ownership

- **Rust.** Follow the workspace toolchain, lints and lockfile. Validate external lengths, offsets and integer conversions. Preserve unknown sizes and typed errors. Document unsafe and FFI preconditions, ownership, lifetime and failure behavior. Keep synchronization critical sections short and keep archive I/O outside them.
- **C++ and Qt.** Use C++17, RAII and clear QObject ownership. Mutate models on their GUI thread, pass owned values across threads, and bind queued notifications to a live receiver. Keep shutdown asynchronous where the receiver still needs the GUI to drain events. Rust remains responsible for archive and file policy. [Qt/Rust bridge](qt-rust-bridge.md)
- **QML.** Express state and user intent through controllers. Reuse shared controls and generated visual and text resources. Keep file I/O and archive interpretation out of delegates. Window changes follow the registry, native behavior and visual standards. [Qt Quick interface](qt-quick-ui.md), [Visual workflow](../visual/design-workflow.md)

Document invariants and what failure leaves behind. In file code, for example, explain what a commit permit protects, which identities are checked, and which parts can already be visible after an error. [File publication](file-security.md)

## Verification layers

| Change | Relevant evidence |
| --- | --- |
| Domains, queries and service | Rust tests for identity, bounds, duplicate names, ordering, stale responses, bounded delivery and cancellation. |
| Format operation | A project-owned fixture, independent round trips where applicable, and rejection, corruption, unsupported-profile and quota cases. |
| Native files and tasks | Actual file identities and written bytes, cancellation and commit races, protection of existing output, and cleanup results on the affected platform. |
| Desktop, Qt and accessibility | Model/controller tests, QML and shared-control checks, native interaction where relevant, and actual visual or assistive-technology evidence for the target behavior. |
| Performance and delivery | Commit-bound workloads, renderer and resource metrics, or channel-specific build identity, install/update results and dependency receipts. |

The [acceptance guide](../../developers/qa/README.md) determines the necessary checks for the change. Protocol conformance, fixture or offscreen verification, native automated runs and manual desktop observation establish different facts. Existing regression entry points and deliverable bounds are linked from the [developer build guide](../engineering/developer-build.md) and [traceability matrix](requirements-traceability.md).

## Existing CI and additional techniques

The ordinary [CI workflow](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml) checks documentation, generated resources, layout contracts, formatting and workflow structure. Heavier native Rust/format and Qt checks run in separately dispatched Foundation and Qt workflows. Dedicated release jobs handle release artifacts. The [CI policy](../../developers/engineering/ci-policy.md) describes these triggers and gates. Code paths and platforms affected by a change determine which local or dispatched tests are needed.

Fuzzing, sanitizers, model checking and performance benchmarks are methods to apply where they give useful coverage. Select bounded, redistributable fixtures and keep the actual evidence separate from this design guidance. For sensitive security reports, follow the [security policy](../../SECURITY.md). Do not treat planned checks or example test methods as a scheduled pipeline or an acceptance result.

## Maintaining the design

When an implementation changes an identity, command, capability, task or write boundary, update the matching design and engineering reference and its acceptance links. Keep all language editions aligned. Use [architecture decisions](architecture-decisions.md) for design rationale and [performance design](performance.md) for measurement rules. Record actual results and limitations separately for the change under review.
