# Large archive indexing and on-demand browsing

The source capability limit for regular single-file ZIP/ZIP64 is one million physical records. Other formats and split-volume paths retain their existing 100,000-entry admissions; a format name does not mean all combinations have expanded limits. Directories, duplicate names, and records needing review all count as physical entries; implicit navigation directories add display rows only.

## Scanning and storage

ZIP scans records individually using the ordinary ZIP parser, first completing bounded archive-wide legacy-encoding name sampling, then performing a second pass with the same encoding decision. Both passes together read at most 512 MiB; the central directory is at most 256 MiB; raw names total at most 128 MiB; decoded and display names together at most 128 MiB; each raw or decoded name remains limited to 4 KiB. A damaged tail, cancellation, or source change fails the whole result without publishing a partial index.

At 64 MiB hot metadata or over 100,000 entries, migrate records individually to private disk storage and release existing names and records without holding two complete indexes simultaneously. The physical-position table contains at most one million u64 offsets. Disk metadata is at most 512 MiB; filenames and contents do not enter diagnostic strings. The directory tree remains limited to 32,768 nodes, 64 levels, and a 4 MiB name budget. If exceeded, do not publish the tree; root browsing and physical search remain available.

Temporary files reside in a randomly named `xdzip-index-*` directory beneath the host temporary directory, readable/writable only by the current user and operated through held directory and regular-file handles. Normal close, query replacement, failure, and cancellation clean known files and identity-verified empty directories. Crash leftovers are identified by a 16-byte marker matching the directory's random identity and an exclusive lease. Later creation performs bounded scans and cleans known unused files older than seven days only. Links, identity mismatches, and unknown objects are retained. Do not recursively delete unfamiliar content or restore/reuse indexes across launches.

## Queries and selection

Search and sorting produce an immutable complete row order. Sort keys are written into private merge runs of at most 8 MiB each, at most 128 runs and 512 MiB temporary sort storage. Final queries store row tokens only, without another full name copy. Comparisons retain original integers, existing name/type rules, and stable physical identity. Unknown sizes sort after known values; descending order does not change identity order for duplicates. Cancellation is checked around scanning, sort runs, and merging, not by changing ordering relationships inside the comparator.

The bridge transfers at most 128 rows and 64 KiB of names per batch. Queries of at most 1,024 rows retain the existing full-commit approach. Larger Qt models retain the full logical row count and load only visible positions, with at most 1,024 resident cached rows. Unloaded rows cannot submit entry actions. Every deferred page binds session, query, generation, revision, and request offset; stale pages cannot enter new results.

Cross-page selection uses bounded ranges in the current immutable query, at most 4,096 ranges. Select All covers the entire current query; leaving the cache does not change selection. Before extraction confirmation, Rust freezes ranges into physical/navigation identities, which remain in use even if the query changes. Selections exceeding the task's 100,000-entry limit are explicitly rejected rather than silently reduced. Total size for nonresident selections is not currently shown. Bulk clipboard copying beyond resident content returns a resource limit rather than silently copying an incomplete list.

## Task boundaries

Regular files selected from a disk index may be extracted after complete source revalidation, safe planning, whole-entry verification, and existing authorized commit. A frozen plan remains limited to 100,000 entries and 64 MiB metadata; file and actual decoded-byte limits follow the [extraction contract](m3-extraction.md). Directory expansion cannot skip unresolved names; selected links are rejected in this index configuration. The ZIP reader rechecks physical offsets for the entire central directory, retains only selected payload locations, and preserves overlap, local-header, size, CRC, authentication, and cancellation checks.

Archives using the disk index do not currently support whole-archive modification or testing; an empty in-memory projection cannot be treated as a complete archive. Existing limits for creation, other formats, split volumes, and archive rewriting are unchanged. See [content preview](content-preview.md) separately. CLI physical listing and selection reuse the same index; [explicit pagination](command-line.md#归档行为) obtains the complete physical range of large archives. Unpaged responses retain 100,000-entry and 16 MiB serialized-data/text budgets and fail as a whole when exceeded.

## Verification

`scripts/probe-large-index.py` generates a real ZIP64 under `build/` with Chinese names, duplicates, and nested directories, then invokes the actual `xdzip-m2-probe`. The probe reads every query batch and separately records scanning, sorting, retrieval, disk bytes, and platform-available memory metrics. The sample generator does not construct an array of one million filenames in memory. Scope, failure, cancellation, cleanup, stale pages, and actual selected-file output follow [acceptance requirements](../qa/acceptance.md); results remain in issues/local or CI artifacts. Measurements are evidence for their actual machine and workload, not universal performance guarantees.
