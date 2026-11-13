# Split-volume reading and temporary opening

## Split ZIP

The read layer supports numbered continuous byte streams such as `name.zip.001`, `.002`, and PKZIP volumes such as `name.z01`, `.z02`, and final `name.zip`. Any volume may be used as the open entry point. Volumes must be in the same directory with consecutive names, at most 1,024 volumes; missing volumes, links, and special files are rejected. Extensions are used only to discover adjacent volume sets. The actual container is probed from source bytes and fully scanned by its backend.

When the ZIP64 classic disk number is 0xFFFF, first discover the actual volume set using the bounded total-volume count in the adjacent locator. Do not mistake the placeholder for 65,535 missing volumes. This discovery does not replace full end-record and directory validation.

The Rust host holds the identity, length, and modification time of every volume. Logical input reads across volumes on demand without combining the entire archive. PKZIP disk-relative offsets are converted into a bounded single-volume central-directory view. The existing codec continues to check the directory, local headers, CRC, password authentication, and extraction quotas. Revalidate all source volumes on each use and before commit; any change stops the task. All source-volume identities are protected from output overwrite.

PKZIP normalization handles only admitted central records and does not support the central-directory digital-signature end record. This rejection does not mean signature verification failed. A regular single-volume archive can read that record within bounds but does not verify its cryptographic signature and refuses modification to avoid discarding or damaging unknown metadata during relocation.

Acceptance covers numbered volumes, PKZIP, ZIP64, headers and payloads spanning volumes, missing or nonconsecutive volumes, source replacement, invalid disk numbers, Chinese names, and regular single-volume regression.

## 7z and RAR volumes

Numbered 7z volumes use continuous byte streams such as name.7z.001 and .002. RAR uses name.partN.rar or legacy names such as name.rar and name.r00, preserving each physical RAR volume boundary rather than passing one concatenated RAR stream to UnRAR. Both locate a same-directory volume set from any existing part. Limits are 1,024 volumes and 100,000 directory-discovery entries. Missing volumes, conflicting name numbers, links, and reparse points are explicitly rejected.

Rust retains each volume handle's identity, length, and timestamps and declares authorized boundaries to the native adapter through ReadSource::volume_lengths(). The native process can request that the host read only authorized ranges; changing volumes does not accept arbitrary host paths from the archive. All volumes participate in source revalidation and output-overwrite protection. A change to the first or any later volume stops the task.

7z and RAR filename encryption may require a password during opening. When ordinary Open returns password_required, no index is published. Qt reuses the password window and resubmits the original selected file and bounded secret with OpenWithPassword; a wrong verification value returns wrong_password. After success, the password is held only by the session and its admitted tasks. Closing the session releases its copy; it is not written to settings or recovery records.

## CAB volumes

CAB volume names come from headers and can be any valid filenames; numbered-volume byte concatenation rules do not apply. The next reference advances one volume at a time. A logical previous reference may point to an earlier volume where a continued file first appeared rather than the immediately preceding one; existing samples may also use adjacent previous references. From any user-selected volume, Rust follows logical previous references backward, then discovers the complete same-directory set forward. After validating set identity, consecutive volume order, and identities, it freezes each volume's length and original name. The decoder verifies all file fragments; a non-adjacent previous volume must be the actual starting volume of a continued file. Limits are 1,024 volumes and 100,000 same-directory discovery entries. Missing, incorrect, cyclic, duplicate-identity, absolute-path, directory-separator, parent-escape, link, and reparse-point cases are rejected. An opened subset is not published as a complete archive.

Worker volume-change callbacks may select names only from this authorized set; each volume provides its own bounded stream. All volumes participate in source revalidation before tasks and file commits and in output-overwrite protection. Native-handler warnings or missing-volume results cannot become a successful index. Single-volume and split CAB archives are read-only.

## Double-clicking regular files

Double-clicking a directory still enters it. Double-clicking a regular file submits a background extraction task containing only that physical entry, outputting to a randomly named directory in the user's private cache. Links and special objects cannot be opened through this entry point. Password, CRC, authentication, cancellation, and application-level serial heavy-task admission still apply; the user need not select an extraction destination.

After successful output, invoke the system association using a local file URL, without a command interpreter. Automatically hide the corresponding task-progress window only if the system accepts the open request. Extraction failure, cancellation, or system refusal leaves it visible with a readable error. System acceptance does not mean the external application has finished reading the file; the temporary file cannot be deleted when the request is sent.

Temporary directories use unpredictable 128-bit random names, with payload and identity marker separated. The output payload directory allows access only to the current user and trusted system principals. On Windows, set a protected DACL through the held handle only on an empty directory exclusively created for this operation, and recheck before writing plaintext; do not modify shared permissions on a user-selected directory. POSIX uses 0700 and checks inherited macOS ACLs. Refuse output if permissions cannot be ensured; see the [Win32 DACL API](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setsecurityinfo).

Do not clean while a lock is held. Retain completed output for at least seven days, then clean expired unlocked directories during a later temporary open. The cache retains at most 256 entries and explicitly refuses at the limit; each scan is limited to 4,096 entries. Cleanup validates private permissions, markers, and names and does not traverse links. Changes to a temporary copy are not written back to the archive.

Temporary-file cleanup is limited to the application-created private cache; it does not scan or delete user extraction directories. Acceptance covers automatic hiding on success, retaining windows on failure, cancellation, password retries, duplicate filenames, Unicode and special characters, task routing after page closure, temporary-directory safety, and exit cleanup policy.
