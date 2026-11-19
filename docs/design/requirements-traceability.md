# xdzip: Requirements, design, and acceptance traceability

English | [简体中文](../zh-CN/design/requirements-traceability.md)

[Product requirements](product-requirements.md) defines the requirement IDs. The matrix links requirements to the design and behavior to verify. See [acceptance cases](../../developers/qa/acceptance.md) for the verification entry points.

## 1. Traceability matrix

| Requirement | Main design | Acceptance suite |
|---|---|--- |
| P01 Browse without automatic extraction | [Domain](domain-data.md), [UI](qt-quick-ui.md) | T01 Read-only open and no changes in temporary directory |
| P02 Browse/query/select | [Domain](domain-data.md), [UI](qt-quick-ui.md), [Performance](performance.md) | T02 Identity, query snapshots, 100,000 entries, stale pages |
| P03 Selective extraction/conflicts | [Domain](domain-data.md), [File security](file-security.md), [Tasks](task-scheduling.md) | T03 Plan scope, duplicate records, overwrite/skip/rename, disk results |
| P04 Archive creation | [Formats](codec-extension-sdk.md), [File security](file-security.md) | T04 Bidirectional ZIP/TAR/TAR.GZ interoperability and old-file protection |
| P05 Capability-driven | [Bridge](qt-rust-bridge.md), [Formats](codec-extension-sdk.md) | T05 Actual features, forged operations, disabled reasons |
| P06 Bounded tasks/cancellation | [Bridge](qt-rust-bridge.md), [Tasks](task-scheduling.md) | T06 Full queues, resource leases, cancellation/commit/shutdown races |
| P07 Security protections | [File security](file-security.md), [Formats](codec-extension-sdk.md) | T07 Native link races, path escape, quotas, special entries |
| P08 Modern accessible UI | [UI](qt-quick-ui.md) | T08 Three-platform keyboard/mouse, IME, DPI, screen readers, and focus |
| P09 Low resource use | [Performance](performance.md), [Tasks](task-scheduling.md) | T09 Actual process tree, GPU, startup, throughput, complete size |
| P10 Decoupling/format extensions | [Architecture](system-architecture.md), [Formats](codec-extension-sdk.md) | T10 Core without Qt, dependency direction, third-format example |
| P11 Privacy/passwords | [UI](qt-quick-ui.md), [File security](file-security.md) | T11 No default networking, redaction, secret lifetimes |
| P12 Auditable releases | [Quality](quality-engineering.md), [Releases](build-release.md) | T12 Locked dependencies, SBOM, required ad-hoc integrity and lack-of-formal-signing disclosure, clean machine, license materials |
| P13 Third-party extensions | [Formats](codec-extension-sdk.md), [Releases](build-release.md) | T13 Independent interoperability, permission changes, rejection of unauthorized actions/refusal to cancel |
| P14 Limited preview | [UI](qt-quick-ui.md), [File security](file-security.md) | T14 Oversized images/text, active content, temporary-file cleanup |
| P15 Recoverable errors | [Bridge](qt-rust-bridge.md), [File security](file-security.md), [Tasks](task-scheduling.md) | T15 Error categories, source changes, full disk, crash recovery |

## 2. Cross-cutting gates

| Suite | Scope | Actual results to check |
|---|---|---|
| T16 Bridge ownership and lifecycle | Every bridge change | Repeated close, receiver wakeup, no callbacks after object destruction, no leaks/out-of-bounds |
| T17 Format interoperability and fuzzing | Every new format/feature | Interoperate with independent references, do not publish corrupt content, bound excessive input |
| T18 Installation, update, and rollback | Every distribution channel | Runs on a clean machine, complete dependencies, active tasks finish before update, documented rollback or recovery |

A new format covers at least T05, T07, T10, and T17. A new output action covers at least T03/T04, T06, T07, and T15. A new release platform covers at least T08, T09, T12, T16, and T18.

## 3. Representative end-to-end scenarios

1. **Regular ZIP:** Drop → display directory → Chinese query → range-select multiple entries → choose a new directory → extract → content matches reference.
2. **Search scope:** Filter with no selection → “Extract all” still means the entire archive; To extract only matches, select them in the result list and review the selection before confirming.
3. **Duplicate/colliding names:** Two records map to the same name → host prompts → renamed/skipped results are traceable, with no silent last-item overwrite.
4. **Cancel commit:** Cancellation before the commit barrier does not publish the current item; after the barrier it retains committed items. A terminal state appears once.
5. **Malicious paths:** `../`, absolute names, ADS, links, and reparse substitutions → reject according to platform policy; no changes outside the authorized directory.
6. **Source changes:** Modify the source after indexing → start/read detects invalidation → stop and explain reopening is needed; no “success” summary.
7. **Create over an old archive:** Full disk/cancellation during writing → old archive checksum remains unchanged; temporary leftovers can be located and cleaned.
8. **Runaway worker:** Out-of-order/oversized frames, refusal to cancel, or unauthorized reads/writes → terminate the extension; host continues other sessions.
9. **Abnormal termination:** Inject kill before/after publication → next recovery does not automatically overwrite or delete user files; explicitly report uncertain state.


## 4. Definition of done

A requirement is complete only when its design is implemented, corresponding acceptance passes, capability documentation is updated, and it is usable in the actual release configuration. Mocks, static screenshots, parse success, or an interface name alone do not count. Design checks can verify document consistency but cannot replace T01–T18.
