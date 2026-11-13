# M2 complete read-only ZIP browsing contract

This contract defines direct root entries, descendant search, stable physical identity, and directory-first type sorting. Read-only browsing writes no files, decodes no content, and makes no CRC integrity claim. Actual acceptance and measurements are recorded in issues/Actions.

See [large archive indexing](large-index.md) for the current regular ZIP large index and on-demand Qt model. This page retains the M2 100,000-entry baseline and query-identity requirements; later scale expansion does not remove error, cancellation, or identity regressions.

## Phase boundaries

- ZIP/ZIP64 metadata, at most 100,000 physical records. Split volumes are read after conversion by the [split-volume host](archive-opening.md); the codec still rejects raw multidisk metadata.
- 4 KiB per name, 16 MiB total raw names, 16 MiB total decoded and escaped display names, 64 MiB total reads, and 256 KiB scan buffer. Declared 64-bit sizes remain exact integers.
- Directory index: at most 32,768 nodes, 64 levels, and 4 MiB total names/prefixes. On overage discard the entire navigation index and explain degraded behavior; do not publish half a tree.
- Reuse direct-directory indexes and query results; transport batches for the same query do not refilter or resort. Each batch separately limits names to 64 KiB. The UI still displays the complete current directory without pagination buttons.
- Pages validate session, query ID, GUI generation, snapshot revision, and offset. Query changes cancel old work and coalesce the latest intent; stale responses cannot replace new results.
- Check source length, modification time, Unix device/inode, and Windows volume number/file index before and after scanning and at query and preflight boundaries. On source_changed, Qt queries and preflight clear the invalid list, selection, directory tree, and details; reopen to obtain a new session. This is best-effort detection, not a filesystem atomic snapshot or secure-write authorization.

## Acceptance exit criteria

A genuinely generated 100,000-entry ZIP64 completes scanning, full-result sorting, retrieval, directory navigation, complete Qt list, and selection; every batch shares query identity. Verify damaged ZIP64, split volumes, name/display/read budgets, cancellation, stale responses, source modifications, and replacements. Retain actual duration, resources, event queue, and delegate counts. When three-platform builds or Apple Silicon development packages are verified on demand, record actual results separately.

CI queries use the repository workflow run list filtered by head_sha and actual event. Routine checks run on push/PR; full Foundation and Qt checks are manual on demand. Record actual results and unexecuted scope in the relevant issue; an empty result for a different event does not establish check status.
