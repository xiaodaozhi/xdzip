# Requirements and verification traceability

English | [简体中文](../zh-CN/design/requirements-traceability.md) | [繁體中文（台灣）](../zh-TW/design/requirements-traceability.md) | [日本語](../ja-JP/design/requirements-traceability.md) | [한국어](../ko-KR/design/requirements-traceability.md) | [Français](../fr-FR/design/requirements-traceability.md) | [Deutsch](../de-DE/design/requirements-traceability.md) | [Español](../es-ES/design/requirements-traceability.md) | [Português (Brasil)](../pt-BR/design/requirements-traceability.md) | [Русский](../ru-RU/design/requirements-traceability.md)

[Product requirements](product-requirements.md) defines the requirement IDs and their current scope. Each row below describes what to verify when that behavior changes. Execution results, platform scope and limitations belong to the corresponding change record. The [acceptance cases](../../developers/qa/acceptance.md) identify verification entry points.

## Requirement matrix

| Requirement | Design reference | Verification focus |
| --- | --- | --- |
| P01 Browse without extraction | [Domain](domain-data.md), [Desktop](qt-quick-ui.md) | T01 Open a real archive and confirm browsing produces no extracted payload. |
| P02 Browse, query and select | [Domain](domain-data.md), [Performance](performance.md) | T02 Same-name identities, stable queries, stale pages, large-index storage and selection across pages. |
| P03 Extract the intended scope | [File authorization](file-security.md), [Tasks](task-scheduling.md) | T03 Directory expansion, duplicates, rename, skip, overwrite and exact disk results. |
| P04 Create an archive | [Formats](codec-extension-sdk.md), [File authorization](file-security.md) | T04 Independent round trips for the changed profile, stage verification and old-file protection. |
| P05 Honor format capabilities | [Bridge](qt-rust-bridge.md), [Formats](codec-extension-sdk.md) | T05 Registry, session-specific actions and runtime rejection of unsupported profiles. |
| P06 Bound jobs and allow cancellation | [Bridge](qt-rust-bridge.md), [Tasks](task-scheduling.md) | T06 Full queues, prompt routing, cancel/commit races and asynchronous shutdown. |
| P07 Protect file and resource boundaries | [File authorization](file-security.md), [Formats](codec-extension-sdk.md) | T07 Applicable native link and identity checks, path validation, quotas and protected objects. |
| P08 Provide an accessible desktop | [Desktop](qt-quick-ui.md) | T08 Model and control behavior, native keyboard/IME/accessibility, scaling and focus on affected platforms. |
| P09 Bound and measure resources | [Performance](performance.md) | T09 Comparable build, process-tree metrics, startup, sorting, output throughput or package size as applicable. |
| P10 Keep Qt out of format logic | [Architecture](system-architecture.md), [Formats](codec-extension-sdk.md) | T10 Core without Qt, explicit adapter routing and format-specific regressions. |
| P11 Protect secrets and data | [File authorization](file-security.md), [Privacy](../engineering/privacy.md) | T11 Scoped passwords, redacted diagnostics and intentional network access. |
| P12 Audit delivery | [Quality](quality-engineering.md), [Releases](build-release.md) | T12 Locked inputs and artifact identity, inventory, license material, package trust and applicable install checks. |
| P13 Gate external extensions | [Formats](codec-extension-sdk.md), [SDK trial](../engineering/sdk-trial.md) | T13 Trial-only interoperability and native permission results. Product extension installation remains a design option. |
| P14 Bound preview and opening | [Desktop](qt-quick-ui.md), [File authorization](file-security.md) | T14 Automatic preview, supported content, worker limits and private temporary outputs. |
| P15 Report the actual outcome | [Bridge](qt-rust-bridge.md), [File authorization](file-security.md), [Tasks](task-scheduling.md) | T15 Source/target changes, failures and cancellation after earlier commits, cleanup and uncertainty. |

## Cross-cutting verification

| Suite | Applicable changes | Verification focus |
| --- | --- | --- |
| T16 Bridge ownership and lifecycle | Bridge, connection or controller lifetime | Current request identity, receiver wakeup, retained tasks and safe destruction. |
| T17 Format interoperability and negative paths | Format or codec changes | Independent valid fixture where applicable, truncation or unsupported profiles, and bounded output. |
| T18 Install, update and removal | Platform or distribution channel changes | The actual package, clean-environment runtime dependencies, installation identity, update handoff and supported removal or recovery. |

## Representative workflows

1. Open a ZIP and search it, freeze selected results, then extract and compare output bytes. “Extract all” still means the entire archive.
2. Extract same-name or colliding records under a defined conflict policy and confirm the resulting filenames and statuses.
3. Cancel before and during publication, check that completed files remain reported, and confirm earlier verified archives are preserved if creation fails before commit.
4. Change a source or destination during an owned test and verify the actual outcome. Following abnormal termination, inspect leftovers without assuming automatic replay.
5. Fault a bundled format worker or the separate SDK trial according to its contract and verify that incomplete data is never published as a successful result.

Static writing, format and link checks validate the documentation. Fixture, offscreen, native automated and manual acceptance evidence retain their own scope. [Quality engineering](quality-engineering.md), [Verification guide](../../developers/qa/README.md)
