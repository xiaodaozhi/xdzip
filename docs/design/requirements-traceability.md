# xdzip: Requirements, design, and acceptance traceability

The sole source of requirement IDs is [01](01-product-requirements.md). The table targets the final product; later requirements are not considered complete in the first version. Test IDs identify planned acceptance suites, not tests already present in the repository.

## 1. Traceability matrix

| Requirement | Main design | Acceptance suite | Initial delivery/gate |
|---|---|---|---|
| P01 Browse without automatic extraction | [03](03-domain-data.md), [04](04-qt-quick-ui.md) | T01 Read-only open and no changes in temporary directory | M2, v0.1 |
| P02 Browse/query/select | [03](03-domain-data.md), [04](04-qt-quick-ui.md), [09](09-performance.md) | T02 Identity, query snapshots, 100,000 entries, stale pages | M2, v0.1; one million in M6 |
| P03 Selective extraction/conflicts | [03](03-domain-data.md), [07](07-file-security.md), [08](08-task-scheduling.md) | T03 Plan scope, duplicate records, overwrite/skip/rename, disk results | M3, v0.1 |
| P04 Archive creation | [06](06-codec-extension-sdk.md), [07](07-file-security.md) | T04 Bidirectional ZIP/TAR/TAR.GZ interoperability and old-file protection | M4, v0.1 |
| P05 Capability-driven | [05](05-qt-rust-bridge.md), [06](06-codec-extension-sdk.md) | T05 Actual features, forged operations, disabled reasons | M2–M4; other capabilities M7 |
| P06 Bounded tasks/cancellation | [05](05-qt-rust-bridge.md), [08](08-task-scheduling.md) | T06 Full queues, resource leases, cancellation/commit/shutdown races | M1/M3, v0.1 |
| P07 Security protections | [07](07-file-security.md), [06](06-codec-extension-sdk.md) | T07 Native link races, path escape, quotas, special entries | Blocking for all write stages |
| P08 Modern accessible UI | [04](04-qt-quick-ui.md) | T08 Three-platform keyboard/mouse, IME, DPI, screen readers, and focus | M5 basics; full matrix v1.0 |
| P09 Low resource use | [09](09-performance.md), [08](08-task-scheduling.md) | T09 Actual process tree, GPU, startup, throughput, complete size | M5 baseline, M9 budget exit |
| P10 Decoupling/format extensions | [02](02-system-architecture.md), [06](06-codec-extension-sdk.md) | T10 Core without Qt, dependency direction, third-format example | M1/M4 source SDK; external M8 |
| P11 Privacy/passwords | [04](04-qt-quick-ui.md), [07](07-file-security.md) | T11 No default networking, redaction, secret lifetimes | M5 privacy; secret capability M7 |
| P12 Auditable releases | [10](10-quality-engineering.md), [11](11-build-release.md) | T12 Locked dependencies, SBOM, required ad-hoc integrity and lack-of-formal-signing disclosure, clean machine, license materials | M5 private-test package, M9 official release |
| P13 Third-party extensions | [06](06-codec-extension-sdk.md), [11](11-build-release.md) | T13 Independent interoperability, permission changes, rejection of unauthorized actions/refusal to cancel | M8 trial; sandbox gate in M9 |
| P14 Limited preview | [04](04-qt-quick-ui.md), [07](07-file-security.md) | T14 Oversized images/text, active content, temporary-file cleanup | M6, v0.2+ |
| P15 Recoverable errors | [05](05-qt-rust-bridge.md), [07](07-file-security.md), [08](08-task-scheduling.md) | T15 Error categories, source changes, full disk, crash recovery | M3–M5, v0.1 |

## 2. Cross-cutting gates

| Suite | Scope | Actual results to check |
|---|---|---|
| T16 Bridge ownership and lifecycle | M1 and every bridge change | Repeated close, receiver wakeup, no callbacks after object destruction, no leaks/out-of-bounds |
| T17 Format interoperability and fuzzing | Every new format/feature | Interoperate with independent references, do not publish corrupt content, bound excessive input |
| T18 Installation, update, and rollback | Every distribution channel | Runs on a clean machine, complete dependencies, updates do not overwrite active plugins, old version recoverable |

A new format covers at least T05, T07, T10, and T17. A new output action covers at least T03/T04, T06, T07, and T15. A new release platform covers at least T08, T09, T12, T16, and T18.

## 3. Representative end-to-end scenarios

1. **Regular ZIP:** Drop → display directory → Chinese query → range-select multiple entries → choose a new directory → extract → content matches reference.
2. **Search scope:** Filter with no selection → “Extract all” still means the entire archive; “Extract search results” separately shows scope and freezes a snapshot.
3. **Duplicate/colliding names:** Two records map to the same name → host prompts → renamed/skipped results are traceable, with no silent last-item overwrite.
4. **Cancel commit:** Cancellation before the commit barrier does not publish the current item; after the barrier it retains committed items. A terminal state appears once.
5. **Malicious paths:** `../`, absolute names, ADS, links, and reparse substitutions → reject according to platform policy; no changes outside the authorized directory.
6. **Source changes:** Modify the source after indexing → start/read detects invalidation → stop and explain reopening is needed; no “success” summary.
7. **Create over an old archive:** Full disk/cancellation during writing → old archive checksum remains unchanged; temporary leftovers can be located and cleaned.
8. **Runaway worker:** Out-of-order/oversized frames, refusal to cancel, or unauthorized reads/writes → terminate the extension; host continues other sessions.
9. **Abnormal termination:** Inject kill before/after publication → next recovery does not automatically overwrite or delete user files; explicitly report uncertain state.

## 4. Design risk register

| Risk | Mitigation and verification | Scope adjustment after trigger |
|---|---|---|
| cxx/CMake integration on three platforms | M1 minimal bridge, generated headers, thread/ownership tests | Fix bridge first; replace through a new ADR if necessary while retaining domain |
| Filesystem security semantic differences | Native race and atomic-primitive verification per platform | Reject unsafe destinations or write only into a new private directory |
| Qt runtime and licensing costs | Minimal deployment, complete size, actual module review | Remove unused modules; static linking needs separate approval and ADR |
| Incomplete codec capabilities | Feature matrix, interoperability, candidate-version audit | Restrict support table; do not suppress unsupported cases |
| Large-index/result memory | Compact storage, budgets, disk backend, query cancellation | Explicitly reject at limits; do not grow indefinitely |
| Platform gaps in plugin sandboxing | Permission attack tests, worker limits | Disable third parties; built-in source extensions remain available |
| Power-loss/network-disk durability | Fault injection, durable mode, platform capability descriptions | Do not claim whole-directory transactions or absolute power-loss protection |

## 5. Definition of done

A requirement is complete only when its design is implemented, corresponding acceptance passes, capability documentation is updated, and it is usable in the actual release configuration. Mocks, static screenshots, parse success, or an interface name alone do not count. Design checks can verify document consistency but cannot replace T01–T18.
