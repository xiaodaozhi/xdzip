# Extraction preflight: metadata phase

This module provides metadata preflight rules. It has no destination authorization, decoding, writing, staging, or commit capability.

## Implementation and boundaries

`xdzip-extract-plan` depends only on domain and metadata interfaces, without Qt, format parsers, or additional third-party dependencies. Input is a session snapshot, a set of physical EntryIds, and cancellation; output is only a bounded report, not paths usable for writing or a “safety approved” flag. `xdzip-runtime` holds the snapshot; Qt calls it through the worker-thread bridge.

Selection is limited to 128 physical IDs, must be nonempty, and all IDs must belong to the current snapshot. Selecting a directory expands all descendants using slash boundaries in original names, independent of search, pagination, or current directory scope. Selecting a directory and child counts the child once. Same-name physical records are not merged by path. Directory statistics count only explicit records and do not synthesize directory objects.

Preflight runs through the M1 production command/event service, reusing bounded admission and independent cancellation. Results do not replace the browsing page or clear selection. Changing selection, query, or session clears the old report and closes the result window, preventing old-selection results from being used for new operations. Reports correspond only to metadata at scan time; this stage does not revalidate source content or identity.

## Rules performed

- Names must resolve to UTF-8, at most 4 KiB, 64 components, and 255 bytes per component; original names are not repaired or rewritten.
- Reject absolute paths, empty components, `.`, `..`, backslashes, colons, control/bidirectional controls, Windows-invalid characters, and trailing spaces/dots in components.
- Reject common Windows device names, including extension forms and superscript COM/LPT digits, conservatively applying a cross-platform portable-name subset.
- Report existing metadata warnings, unsupported encryption, and non-Store/Deflate methods. Entries strictly identified as supported AES-256 separately warn that extraction needs a password; method 99 is not incorrectly reported as an unknown compression method. This filters declared capability only and does not decrypt, authenticate, or decode. See the [format contract](m4-tar-gzip.md) for TAR/TAR.GZ capability mapping.
- Check exact duplicate names, collisions from Unicode lowercase comparison, parent-directory aliases differing in case, and file/directory conflicts in both insertion orders. An implicit parent directory may be supplemented by an explicit directory of the same name; duplicate explicit directory records still conflict.
- Declared per-file size above 8 GiB, total above 20 GiB, or integer overflow produce findings; declared compression ratio above 1000:1 prompts review. Declared directory sizes do not count toward output bytes.

Unicode lowercase comparison is not full case folding or NFC/NFD filesystem equivalence. Any non-ASCII component marks the report “Requires destination filesystem review.” Even a report with no findings does not authorize output.

## Resources and failures

Check at most 100,000 snapshot records; selected/expanded names total at most 4 MiB. Comparison keys and original spelling stored in the collision index total at most 8 MiB, excluding container overhead. Component traversal depth is bounded, and cancellation is checked for each outer record. Reports count all findings, but at most 64 representative issues cross the bridge, containing only physical record IDs and stable reasons, not arbitrary archive error text.

Empty selection, foreign IDs, index limits, or cancellation return errors rather than partially successful reports. Overflowing totals display “Cannot calculate” rather than truncating to zero. Basic findings are only reported; they do not automatically overwrite, rename, ignore, or skip.

Unknown uncompressed sizes report the stable reason `unknown_size`. `declared_bytes=None` means an unknown total and is distinct from `Some(0)` for a known empty file. Unknown values do not fabricate total-size or unknown-method findings; actual integer overflow still produces a total-limit finding. Output tasks separately reject unknown-size records before decoding, staging, or password interaction.

## Desktop entry point

Select entries and click “Extraction preflight.” Results use an independent non-modal top-level `OperationWindow`, with the main window as transient owner and the native title bar retained; macOS traffic-light buttons are not drawn by the application. Closing results closes only that window. Main-window closure uses cooperative cancellation and does not wait for workers on the GUI thread.

## Boundary with actual extraction

[Actual safe extraction](m3-extraction.md) independently implements output accounting, CRC/structural verification, destination authorization, platform path rechecks, conflicts, staging, and file commits. It is not limited to preflight's 128 selections and supports plans for 100,000 physical records and virtual-directory selections. A preflight report is not reused as write permission. Starting a task must reacquire destination authorization, revalidate sources, and check actual content under the [file security specification](../design/07-file-security.md). Native failure behavior and current known defects are tracked by the relevant contracts and issues.

## Window and result presentation

Preflight uses a shared operation window 491 logical pixels wide, with a body adapting to content, a maximum client-area height of 600, a scrollable long issue list, and an accessible fixed footer. Data and metadata-phase authorization boundaries are unchanged. See [preflight parameters](../../specs/visual/preflight-reference.json) for parameters and design source.

Preflight explanation placement was revised on 2026-10-15: move “This report checks metadata only…” to the end of the scrollable body using 12-point explanatory text. The footer no longer carries this explanation; it retains only the Back action and matches the height of a shared action-only footer at the same font size, independent of report length.
