# WP-11 TAR/GZIP implementation contract

This contract defines TAR/TAR.GZ browsing, creation, and safe extraction boundaries. Platform results and manual acceptance are recorded in issues/CI artifacts. It retains input authorization, staging, conflicts, cancellation, and whole-archive commit from [M3 safe extraction](m3-extraction.md) and [WP-10 creation](m4-creation.md).

## Scope and capabilities

TAR and TAR.GZ browsing, creation, and safe extraction are supported. TAR is a container and GZIP one compression filter; reading probes content instead of selecting a backend by extension. ZIP behavior remains compatible. Automatic nested expansion, encryption and split TAR are not enabled. Ordinary single-file GZIP uses the separate [stream contract](single-file-streams.md).

TAR headers, numbers, PAX, streaming encoding, and rewriting use an independent Rust record layer. GZIP framing, CRC-32, and Deflate LZ77/Huffman use shared `xdzip-compression`. `tar` and `flate2` are development-test references only and do not enter production dependency closures of these crates. See the [in-house format contract](independent-rust-codecs.md) for specifications, sources, and limits.

TAR accepts regular files and directories and parses PAX paths and GNU long names. Symbolic links, hard links, devices, FIFOs, sparse files, and unknown types cannot be extracted. All output names still pass PortablePath and host conflict preflight; format libraries receive no file paths or unpack authority. Directory permissions, owners, and special modes are not restored. Dates are display metadata without restoration guarantees.

## Bounded streaming

Legacy filename encodings, GNU long names, and PAX UTF-8/BINARY declarations are uniformly resolved under the [encoding contract](filename-encoding.md). Publish indexes only after complete scanning; entry counts and totals are unknown during scanning. Metadata retains limits of 100,000 entries, 4,096 bytes per name, and cumulative name budgets. Streaming source encoded input and decode work are each at most 24 GiB; a GNU/PAX extension header is at most 64 KiB, with at most four extension headers per entry. Global PAX accepts only comment text without file semantics, for git archive compatibility. It is fully checked against declared record length and discarded rather than inherited by entries. Global path, size, mtime, linkpath, sparse, and unknown fields remain explicitly rejected; GNU long links and sparse variants are unsupported. Global comments also count toward the four-header-per-entry and stream-byte budgets and do not make unfinished local extensions valid. Normalize common leading `./` in TAR names; root markers are not output entries. Sequential reads and skips also check cancellation and byte budgets; seeking cannot bypass corrupted-stream checks. GZIP must reach EOF and verify CRC and ISIZE. Multiple members form only one continuous TAR byte stream. Nonzero trailing garbage, truncation, and corruption fail. Unverified partial indexes are rejected.

TAR.GZ lacks per-entry compressed sizes. High-ratio admission compares selected declared total against the complete frozen source size. Above 1000:1 and 1 MiB still requires explicit permission, which does not bypass scan-work or output quotas. Metadata preflight does not invent per-entry compression ratios.

Extraction processes selected entries in physical order. Skipping unselected content is metered and cancellable; random reads are not promised. PayloadReader receives only the source and host sink. The host still verifies source versions at scan, decode, and commit boundaries. Per-entry commit and whole-creation failure semantics follow the existing secure I/O contract.

## Creation and interface

The creation window enables existing TAR and TAR.GZ segments without changing density. Format switches update extensions. TAR disables compression level; both TAR formats disable and clear password and split-volume values. TAR.GZ uses GZIP levels 0/1/6/9. Compressed size per entry is unknown and displayed as a dash, not 0 bytes. Rust uniformly derives the archive-named target folder from the source name and provides it through the bounded bridge for preview. TAR.GZ removes the complete compound extension case-insensitively; an empty basename falls back to archive. The UI does not derive another target; PortablePath and the authorized root revalidate it. Main-window format tags adapt to actual format and text width. TAR dates display in UTC and remain unknown outside the existing display range. Creation explicitly writes PAX UTF-8 paths for Unicode/long names and PAX size for large files, avoiding reference readers' local-encoding guesses. Freeze the input tree, stream output, terminate TAR/GZIP, rescan, and decode/verify every entry; publish as a whole only after success. Conflict renaming retains the complete compound extension.

## Verification and limits

Cover bidirectional interoperability with system tar/Python tarfile, long paths, Unicode, empty directories, duplicates, PAX/GNU, special entries, path escape, truncation, CRC, resource limits, cancellation, and old-file protection. Real Qt creation, reopening, and extraction reuse existing task and conflict windows. Actual evidence is generated only under build/ or CI artifacts.

Choose affected desktop and accessibility reverification under [general verification](../qa/README.md); native automation and offscreen checks do not replace manual checks. Actual results and limitations go in the relevant issue. Existing desktop acceptance is in [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5); one-off permission or environment blockers are not retained in this contract. Historical `v0.1.0` release license review is in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53); future candidates need new release issues.

The CLI creation entry accepts .tgz as a same-format alias for .tar.gz, using the same GZIP encoding, verification, and commit. Ordinary non-TAR GZIP uses the separate single-file codec.
