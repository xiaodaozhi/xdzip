# xdzip: 07 File security, permissions, and output transactions

## 1. Threat model

Archives, filenames, sizes, CRCs, compressed data, extension metadata, recovery logs, and third-party extensions are untrusted. Major threats include path escape, overwriting important files, link attacks, archive bombs, parser memory errors, incorrect name mappings, partial writes masquerading as success, password leaks, and extension privilege abuse.

The trust boundary consists of the signed host, reviewed platform file layer, and user authorization. Third-party code is outside the trusted computing base. This design does not promise protection from an attacker who already controls the same user account or kernel; ordinary file permissions do not defend against an equally privileged process continuously moving directories or reading memory. For untrusted, multi-user-writable destinations, require a user-private subdirectory by default or reject safe-overwrite mode.

## 2. Source and destination authorization

After the user selects a source/destination, the platform layer opens it and retains file identity and handles. Authorization binds to file/directory objects, not just display strings. SourceRef/TargetRef remain within the host; extensions receive operation tokens. Sources are read-only and do not automatically request full-disk access. Other volumes must be identified and authorized by the user before reading; do not locate files using absolute paths supplied by an archive.

Initial archive opening and subsequent source checks use the same regular-file opening boundary, checking object type on the handle actually opened rather than checking a path type before a potentially blocking open. Unix uses nonblocking classification opens and rejects FIFOs, devices, directories, and other special objects. A user-selected source path may follow a symbolic link to a regular file, but source identity and version must still be checked later. This check is not an atomic content snapshot and does not guarantee that regular-file I/O finishes within a fixed time.

The destination directory must exist and have sufficient permissions, or be created by the host within an authorized parent. Working directories use secure random names, exclusive creation, and minimum permissions; temporary names are not predictably derived from archive names. Output and log working directories are inside the authorized destination to allow same-volume commits; cross-volume output cannot be described as atomic rename.

Private-output checks cover the destination root, parent chain, staging directory, and file to be replaced. POSIX mode bits do not replace platform ACL checks. On macOS, require ownership by the current effective user, no group or other-user mode write permissions, and exactly one hard link for regular files. Known deny rules and external read-only permissions are allowed. Modification permissions may be granted only to a principal explicitly matching the current effective user UID; group principals are not treated as the current user. Reject effective or inheritance-only external modification permissions, unknown permissions, and unverifiable ACLs. Create staging files exclusively in verified private directories, with inheritance constrained by directory ACL checks. Checks do not rewrite user ACLs; existing read-only sharing may be inherited by output, so confidentiality from shared accounts is not promised.

## 3. Path processing pipeline

1. Preserve raw paths and encoding origins; removing display ambiguity does not mean accepting an output path.
2. Parse separators according to format rules and reject absolute paths, drive prefixes, UNC, device paths, NUL, and parent escapes.
3. Form component lists and reject empty/invalid components and platform-reserved names. `.` may be normalized as specified, but `..` cannot be blindly removed and then declared safe.
4. Form platform candidate output names and check component lengths, total paths, case, and Unicode filesystem equivalence.
5. Check duplicate paths and file/directory conflicts within the plan and conflicts on disk.
6. After user decisions, revalidate object identity and type at actual open/commit and perform handle-relative I/O.

Windows requires special handling of reserved names such as `CON/NUL/COM1`, trailing spaces/dots, colon ADS, `\\?\\`/device prefixes, and case collisions. A single generic string rule cannot assume that backslashes/colons allowed on POSIX are writable on Windows. Enable and test long paths explicitly according to platform capabilities.

Normalization is for collision analysis and display; do not arbitrarily change original names. `A.txt/a.txt`, NFC/NFD-equivalent names, and duplicate ZIP records form conflict groups. Default to waiting for a decision or skipping, not “the last one overwrites the previous one.” Rename policies retain traceable mappings; extensions cannot modify them.

## 4. Link and race protection

On POSIX, traverse components using directory handles and `openat`-style calls, refusing to follow symbolic links. Linux may use `openat2` beneath/no-symlink constraints where available, but other platforms still need backends. Windows uses handle-relative/verifiable opens, reparse-point checks, and file IDs; `canonicalize + string starts_with` is not a security algorithm.

POSIX ZIP symbolic links are validated as a closed graph within the selected scope under the [safe extraction contract](../engineering/m3-extraction.md), then exclusively published after regular outputs finish; writes do not follow links. Other symbolic links, hard links, junctions, devices, FIFOs, sockets, and setuid/setgid are not restored. Reject ordinary overwrite when an existing destination is a link or special object. Create all staging files exclusively and retain handles until commit; destination changes between scan and commit return to conflict resolution.

Directory handles alone cannot guarantee that an authorized party will never move their paths. The platform layer must define “under the authorized object” and detect destination/parent identity changes. Reject a normal multi-user-writable tree that cannot satisfy safe overwrite, or move to a new private subdirectory. Do not claim that `O_NOFOLLOW` alone eliminates all directory-movement races.

Platform security implementations require native adversarial acceptance tests: swap links, directories, case aliases, and reparse points between check and open. In-memory mocks cannot replace all verification.

## 5. Per-file extraction transactions

```text
Plan and authorization checks
  -> Controlled staging file
  -> Decode and meter actual bytes
  -> Verify content/authentication/expected structure
  -> Check cancellation and destination conflicts
  -> Update transaction log
  -> Atomically publish one file
  -> Record committed item
  -> Clean staging
```

The backend writes to a constrained Sink; the host owns the staging handle. CRC/authentication failure, quota exhaustion, full disk, and cancellation never publish the current partial file. Files without format checks may complete according to backend capabilities, but the summary does not claim a nonexistent CRC was checked. Modes use ordinary safe read/write bits; timestamps are restored on a best-effort basis after commit, with failures reported as warnings.

Do not overwrite by default: use a platform “publish only if destination absent” primitive, not exists followed by ordinary rename. When the user permits overwrite, recheck that the destination is the expected regular file and use a same-volume replacement primitive; do not delete the old file first. On Windows, another program or antivirus may hold the file; report failure and offer retry, without switching to non-atomic truncation. If the platform lacks the required primitive, return a capability/security error.

Overwrite authorization applies to the mapped path. Atomic replacement makes new content appear as a whole but does not inherently provide compare-and-swap semantics that replace only if the destination inode/content is unchanged since the check. Ordinary platform primitives cannot exclude another program changing it after the check. To strictly avoid this kind of overwrite, use no-overwrite/rename/new-private-directory modes, or enable conditional replacement only where the platform actually provides a verifiable primitive. Path safety and protection against concurrent editing are separate guarantees.

Directories and multiple files are not one whole transaction. Failure/cancellation retains committed items and reports them accurately; cleanup does not delete them. The first version does not offer imaginary one-click rollback. A future “publish a new directory as a whole” is a separate mode limited to an absent destination, same volume, and atomic directory rename support.

## 6. Archive creation transactions

The temporary archive is in the final destination directory. After writing the footer/directory, flushing, closing, and required verification, publish only if the destination is absent or perform a user-approved replacement. Failure/cancellation preserves the existing archive; unpublished temporary archives are deleted or recorded as recoverable leftovers. Finishing the last input file does not mean the archive is ready.

Atomic publication visibility differs from power-loss durability. Durable mode synchronizes files and directories/transaction records according to platform capabilities. The default guarantees at least that incomplete output is not published; it does not claim all disks/network filesystems withstand power loss. Network drives and filesystems lacking required primitives explicitly downgrade or reject the relevant mode and are included in testing.

## 7. Quotas and archive bombs

The host meters actual writes and does not trust archive declarations. Initial limits are in [constants](glossary.md): 8 GiB per file, 20 GiB per task, one million scanned entries, 64 KiB per field, and four format layers. Stop before publishing the current file when a limit is exceeded. Raising a limit requires replanning and checking remaining space; one dialog cannot authorize unlimited automatic continuation.

Compression ratios above 1000:1 produce a risk notice, but hard protection comes from absolute actual bytes, index count, decode memory, and work budgets. Zero compressed length, unknown sizes, and format deception cannot bypass metering. Legitimate high-ratio content may be processed after the user explicitly increases the relevant policy.

Solid/streaming archives may decode much preceding content even when only one small item is selected; meter and display “decoded bytes” separately from “output bytes.” CPU/time limits are enforced through cancellation and worker supervision. Native blocking parsing that cannot cooperate with cancellation needs isolation; a stopped UI progress indicator does not prove work has stopped.

## 8. Preview and external opening

Preview is explicitly initiated only. Text is limited to 1 MiB; compressed image content is limited to 20 MiB and decoded pixels to 40 MP, checked after reading the header and before allocating pixels. Image decoders also process untrusted data and should be audited and considered for isolation. The default external temporary-open limit is 100 MiB; exceeding it requires an explicit action, and whole archives are not automatically unpacked.

Private temporary directories are created exclusively. Files are not given executable bits, links are not followed, and dangerous xattr/ADS are not inherited. After user confirmation, invoke the system-associated application, not a shell. Do not render active HTML/SVG scripts/macros. macOS does not proactively strip existing quarantine; Windows does not proactively clear existing download marks. New mark propagation requires platform verification, and the same rule is not claimed for every format.

External applications may hold temporary files. If deletion fails, record a deadline and clean up later. Clean only working directories created by this application whose identities are verified; do not recursively delete using a broad path wildcard.

## 9. Passwords, content, and logs

Secrets live only for the session/task phase, are not written to disk by default, use zeroizable buffers, minimize string copies, and do not pass through ordinary events/errors/traces. OS input methods, GUI internal caches, and backend copies may leave residue; promise only reasonable minimization and cleanup.

Logs contain version, anonymous IDs, error codes, phases, counts, and durations by default. They contain no file contents, passwords, full paths, archive comments, or arbitrary plugin text. When a user exports detailed diagnostics, preview them and save locally; upload requires a separate explicit action. The product has no default telemetry.

## 10. Abnormal termination and recovery

When finding a working directory at startup, first verify ownership, random task ID, file identities, log version, and authorization availability. Treat logs as untrusted; paths must pass the same security checks. If state cannot be determined, show pending leftovers and do not automatically re-extract/overwrite.

A `prepared` but unpublished file may be safely deleted. If publication occurred but the terminal log state was not written, inspect destination identity/checksum records and mark it “Possibly completed; please confirm”; do not simply delete the destination. Recovery after replacing an old file does not promise to restore old content. Insufficient cleanup permissions produce an actionable notice.

## 11. Security test gates

Path escape, link/reparse races, directory alias collisions, duplicate records, ZIP64, unknown sizes, CRC/authentication errors, full disk, insufficient space for logs, destination changes, process kill, corrupt recovery logs, oversized plugin frames, and refusal to cancel are release-blocking cases. Every security failure must verify that files outside the destination are unchanged and the current partial file was not published, rather than merely asserting an error code.
