# Acceptance execution plan

## Windows MSIX, AC-04 / AC-07 / AC-12

| Case | Procedure and criteria |
| --- | --- |
| MSIX-PACK | After same-commit clean build and tests, generate the experimental package from the audited runtime directory. Check identity, version, manifest and complete file hashes; reject changed files, old product update policies and output overwrites. Do not treat an unsigned package as a release package. |
| MSIX-CHANNEL | About has no in-app update for Store-signed or sideloaded packages and the controller does not access the existing update service; ordinary installations retain the update entry. Check coexistence with installed versions, single-instance handoff, settings and file associations. |
| MSIX-SHELL | The current package registers no modern menu. If implemented later, independently verify Store, Developer, Enterprise, None and query failures; sideloading must not enable it. Record real Windows 11 Explorer display and Windows 10 behavior separately. |
| MSIX-LIFECYCLE | Verify installation, archive opening, drag-and-drop, workers, associations, updates and uninstall on the actual target system, checking defaults and user data. Record licensing, Store certification and manual acceptance separately. |

## File-manager actions, AC-04 / AC-06 / AC-07 / AC-08 / AC-12

| Case | Procedure and criteria |
| --- | --- |
| SHELL-START | Invoke actions on cold start, with an existing main window, hidden to tray and without tray. The main window must not flash, restore or switch tabs. Existing instances deduplicate request identities, reject unsupported protocols, invalid requests and over-limit admission, and show failures. |
| SHELL-OUTPUT | Create ZIP and 7z for single, multiple and cross-directory selections; check complete filenames, parameters and bytes. Quick creation uses no encryption or volumes and asks about collisions. Per-item tasks output beside each source; cross-directory combined creation confirms destination first. |
| SHELL-LAYOUT | Check actual outputs for current, smart, forced per-archive and selected destinations. Implicit top-level directories participate in smart decisions, internal hierarchy is not stripped, existing files/directories/links force numbered new wrappers, and complete volume groups execute once. |
| SHELL-LIFECYCLE | Exercise password retries, conflicts, editing and retry after rejected creation submission, global queueing, continuation after individual failure and batch cancellation. Distinguish success, failure and unexecuted results. Reopen tasks closed to tray; without tray decide to wait or cancel first. Join all Rust connections after the last window closes. By default do not browse new archives or automatically open folders; explicit Open works. |
| SHELL-INSTALL | On three platforms check registration, settings disablement, upgrade, rollback and uninstall. Remove only owned entries without changing default handlers. Registration-only commands can complete while installation transaction holds its lock; normal startup is rejected. Safely update extensions held by file managers; do not hand off to incompatible programs and reread capability after rollback. Windows rejects a retained new root executable after rollback to the old versioned layout; historical install fixtures do not call nonexistent shell-maintenance commands. Portable packages do not auto-register even with manifests. Record macOS Service visibility and system settings separately. |
| SHELL-VISUAL | Actually inspect entries and standalone windows on three target platforms. Creation/extraction progress and results reuse TaskWindow; individual failures and batch states are legible, shell windows center in current display's available area. Verify Windows classic-menu placement, entries and action icons. Pass window contracts, shared controls, both languages, light/dark and three font sizes, preserving main-window/menu proportions. Native accessibility press/toggle updates actual radio and checkbox models. |
| FINDER-SYNC | On actual macOS enable ad-hoc appex, including owned temporary directories outside home. Check dynamic 3/7/8/4 actions for ordinary single, archive single, multiple and mixed selections. Real clicks through relay complete packing and extraction; check output bytes, standalone windows and no automatic folder opening. Independently regress tag-only handoff, stale selection, URL/JSON bounds, absent/incorrect token rejection, preserved UUID and rollback capability. |
| FINDER-INSTALL | Verify disable/re-enable, language and repeated-save menu behavior; unchanged extensions are not redundantly registered and disconnected. Remove only owned exact paths and do not let older copies disable new configuration. Native Service panel still operates independently. Strict nested ad-hoc verification and entitlements match the allowlist. Local results do not establish paid-account signing, App Groups, notarization or other macOS versions. |

Record actual execution in the corresponding issue and local or CI artifacts. Native automation, manual desktop work and unavailable platform environments are recorded separately, never substituted for one another.

## macOS candidate provenance evidence

| Case | Procedure and criteria |
| --- | --- |
| QT-NETWORK-SECURITY | Pin source, five official patches and builder identity; retain original Network features/exports and imports against original QtCore. Load the actual candidate library. Oversized status lines, unterminated headers, chunking and CONNECT reject before fixture cancellation; normal responses, legitimate long headers, each line termination, trusted test CA and CONNECT TLS work, while untrusted certificates still fail. Run upstream malformed and normal authentication vectors. Preparation failure restores the SDK. Missing/stale receipts, replaced library or SDK tools, missing provenance patches or filename-only attribution prevent product/release admission. Record actual platform; macOS 14 remains unverified without its environment and is not inferred from other systems. |
| RELEASE-MAC-EVIDENCE | The dedicated Release runs a real Cocoa/Metal resource probe while the deployed app exists, checking audit, version and source commit. Link inputs bind to that compiled app; the existing Mach-O comparator checks section bytes, addresses, flags, zero-fill sizes, UUID and normalized import versions. Signing/load-command layout and LINKEDIT remain separate. Negative checks reject changed sections/UUID/imports, truncation/out-of-bounds, wrong architecture and mixed resource/link/SDK identities. Verify SDK correspondence, difference analysis, provenance/file SBOM and overall receipt are actually in same-package materials. Retain original gaps/incomplete, reference-machine budgets and human visuals separately; automated results do not stand in for human passes. |
| RELEASE-VALIDATION-UNSIGNED | Local candidate assembly `--validation-only` accepts no update signing key or key ID. Even with injected owned fixture keys it produces no manifest signature or appcast, and rejects stale signature attachments and flags conflicting with mandatory signing. This mode is not in the Release workflow; formal releases use one manual run and mandate complete update signature verification. |

## Title area, task history and drag hints

| Case | Procedure and criteria |
| --- | --- |
| USER-DATA-FONT | Across supported platforms, both languages and 100/125/150% fonts use ordinary Chinese, characters outside the subset, multilingual text, emoji and long paths. Check list/grid, tree, breadcrumbs, tabs/hints, tasks and operation windows prefer startup system font while static copy uses the bundled subset. System fallback covers missing glyphs; record actual platform availability. Check long-text elision, wrapping, width, focus and accessible names; font changes do not change names or file operations. Static root-directory explanation in Add uses UI font; actual in-archive directories use system font. Record native visuals and system glyph coverage separately. |
| HEADER-SHARED | Check create, add, extract, task, result, conflict and settings in both languages, themes and 100/125/150%. Icon 48, title 20, subtitle 13, horizontal padding 24 and normal height baseline 96. Long text grows under the same rules; extra gap to first body group is 0. Measure running, success, failure and cancellation boundaries; retain internal paragraph spacing and reachable body/footer. Settings has no footer. Main-window/menu proportions are checked separately. |
| TASK-PATH-SUMMARY | In both languages, themes and 100/125/150% fonts, alternate short paths, deep directories and long filenames during progress. Current file remains one line with middle elision, full plain text and accessible name, stable card/body/window height and reachable footer. Ordinary extraction captions show only committedFiles for zero, one, multiple, partial-failure and truncated-detail cases, excluding skipped/unfinished files. Creation captions, batch summaries and expandable details retain their behavior. Inspect affected window images and record native visual coverage separately. |
| CLOSED-BROWSER-STATE | After changing directory, selection, search, sorting and grid view, close tab/window and reopen separately with no tasks, running tasks and completed records. New pages start at root, no selection and default list; old task identities and prompts remain. Reopening during archive modification can wait or cancel waiting without cancelling the original task. Cover late receipts, other windows and the 32-session limit. |
| DESTINATION-CHECK | Check create and extraction destinations initially, after directory selection and after editing. Interleaved/rapid edits accept only the latest result for current window, path and extraction session; submission is blocked before passing. Add/remove buttons and start page do not flicker during creation preparation. Both windows' status icons and messages agree; retain extraction path layout, same-name folder and staging explanation across both languages, themes and font sizes. |
| APP-ZIP-DESTINATION | Retain default same-name folder for `*.app.zip`; preview, actual task target and outer ordinary directory name agree and do not end in `.app`. The real app bundle resides inside. Cover case, Unicode, empty-name fallback and other suffixes; path is unchanged when same-name folder is unchecked. Locally compare actual app ZIP to independent extraction for contents, links and safe regular-file permissions, then check signature integrity and launch. |
| TASK-HISTORY-CLEAR | Mix completed, failed, cancelled, running, waiting, cancellation-cleanup and same numeric IDs across sessions. Clear only terminal history; active counts, password/conflict tokens and stable rows remain. Duplicate terminals do not revive entries; result windows close; sources and outputs are not deleted. |
| TASK-MENU-VIEWPORT | Short lists reserve no scrollbar width and right-align status badges. Long lists scroll internally, support Home/End and keyboard, and cap height at 360 and window bounds with fixed clear area. Keep 4 units each side of separator, equal horizontal/vertical item padding and only shared menu padding below the button. Mouse, Enter/Space and native accessibility press actually clear terminal history without revival on reopen; disable when no completed history exists. |
| TASK-MENU-SHORT-REOPEN | Complete a task, clear history, create another and open the menu. Every short-list row is fully visible without scrolling and clicking the new row opens its task immediately. Cover both languages, themes and three font sizes; long lists still scroll to both ends. |
| TASK-MENU-KIND | In both languages/themes and 100/125/150%, check icons, accessible operation names and actual opened window kinds match check, extract, create, add, move and export. Identical numeric IDs across sessions still open original tasks without switching archive tabs. |
| TASK-MENU-ICON-TAGS | Order is operation icon, filename, sequence tag, status tag. Tags share styling, long names elide without covering tags, accessible names retain operation/number, and cross-session identity and opening remain correct. |
| EXPORT-NO-AUTO-OPEN | First run normal extraction explicitly choosing open folder on completion, then drag out and copy files, covering task-first and registration-first order. Export completion does not automatically open a system folder; original file contents and explicit extraction choice retain their own behavior. |
| EXPORT-BROWSER-STABLE | Hand list/grid drags to the system by leaving or deactivating the window. Inspect manifest preparation state notifications and overlay visibility each time: no “Reading” flash, list/selection/directory remain. Release/cancel and actual export remain unchanged; subsequent directory queries still show real waiting feedback. |
| DRAG-SOURCE-TOOLTIP | Show a source name in list and grid, then press and drag elsewhere for over 1200 ms; no source tooltip appears. Native handoff also suppresses it; cached hover cannot revive it afterward without new pointer position and full delay. Escape cancellation does not modify the archive. |
| MAIN-FOOTER-PARITY | Welcome and archive pages have equal bottom-bar heights at normal/large fonts, minimum 32 logical pixels, without height changes on page switch. Task buttons, icons and arrows align on the right, 14 logical pixels from the bottom-bar edge; menus follow button alignment. Check both languages/themes, three font sizes and normal/narrow windows. |
| MAIN-FOOTER-SELECTION | In both languages check no selection, known size, zero bytes, virtual directories, unknown-size files and mixtures. Unknown totals show selection count only; known size remains visible. Welcome and preflight behavior remain unchanged. |
| OWNER-DIALOG-CLOSE | Close the main window with settings, About and operation windows open. Owned native resources are released, unsaved previews revert and tray restoration does not re-show them. Retained tasks can still be explicitly reopened from the global task entry. |
| ADD-CONFLICT-PAGE-CANCEL | Drop a same-named file and start addition; close conflict prompt and task window, then cancel from the main page. The task terminates, original archive bytes remain and reopening restores selection and browsing. |
| ARCHIVE-OPERATION-OVERLAY | During addition and internal moves the original page remains visible but blocks repeated actions, with transparent overlay background. In both languages and large fonts, title, description, progress accessible name and Cancel match the operation and do not call cleanup/refresh loading. |
| TASK-PROMPT-LIFECYCLE | After cancellation from page, task window or global entry, conflict/password prompts close and progress shows current cancellation and terminal. Other task updates do not revive a closed prompt. Explicit reopen, task switch and new tokens respect their identities and reject old decisions. |

These are acceptance requirements. Record actual results and native-platform limitations separately in issues and build/ or CI artifacts.

## User-initiated updates

- Core fixtures check stable/alpha/beta/rc/dev, same versions and downgrades, non-increasing builds, out-of-order pagination, missing packages, wrong platform/architecture/minimum system, mismatched signatures or candidate identities. Missing signatures or unknown trust roots offer manual entry only.
- Network checks cover offline, TLS, timeouts, rate limits, controlled redirects, truncation, insufficient space, cancellation, repeated clicks and ETag. Failure never reports up-to-date or starts installation. Clear SDK search paths after deployment and verify real HTTPS.
- Upgrade old to new on isolated three-platform environments with independent test keys, checking task/password waits, exit cancellation, restart during install, authorization cancellation, failure, recovery and single-instance restart. Settings and default associations survive. Windows interactive installation retains license confirmation; Linux unconfigured dpkg is not success or rollback; Mac full Sparkle archive verification, replacement and restart require native Apple Silicon.
- When password and update waits coexist, exercise the real main program's system Quit route and actual Return/Cancel controls. Return preserves the original task and request token; cancel-and-quit requires all Rust joins and no install handoff. File selectors and unrelated modal windows remain blocking; direct session calls do not replace desktop-entry regression.
- On macOS check application-menu Quit and Command-Q during password and conflict prompts. Return preserves task and prompt identities. Unrelated modals and file selectors block Quit, and Quit triggered while blocked must not execute later after they close.
- Cover all About states, retaining work after close/reopen and explicit cancellation; check both languages/themes, three font sizes and narrow-screen scrolling with shared controls. List unverified production-key and real-platform items separately. Ordinary CI does not deploy/sign/upload Mac app packages.
- Mac manual fallback separately covers ineligible installation identity, no appcast, invalid network configuration and no handoff session. Display the first actual unmet condition and retain release-page/download-location actions; guidance stays legible in both languages/themes and three font sizes. Download completion is not installation completion.

The authoritative T01–T18 mapping is in design/requirements-traceability.md. These are reusable acceptance specifications, not claims that this run did or did not execute them all. Actual results belong in issues and artifacts; existing manual conclusions are in [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). Reverify affected items for new candidates without creating fake pass reports.

| Case | Preconditions and action | Pass criteria |
| --- | --- | --- |
| AC-01 / T01 | Snapshot output/temp directories; open and browse a normal ZIP | No file content extracted; only controlled metadata and explicitly documented caches allowed |
| AC-02 / T02 | 100,000-entry archive; sort/search/range-select and rapidly switch queries | Selection bound to EntryId, stale pages do not pollute new queries, bounded memory |
| AC-03 / T03 | Search, clear selection and extract all; then extract a partial selection | Scope explicit; all is not secretly search results; result bytes match |
| AC-04 / T04 | Create ZIP/TAR/TAR.GZ and read independently; reverse generation and reading | Paths and contents match; unsupported capabilities explicitly rejected |
| AC-05 / T05 | Create AES-256/numbered volumes and independently check; read supported AES-256, numbered and PKZIP volumes, then other encryption | Two-way interoperability for supported password creation/extraction; unsupported reads explicitly rejected and unauthenticated files not published |
| AC-06 / T06/T16 | Full queue, cancel, repeated cancel, app close and commit races | No deadlock, terminal once, pre-cancel committed items traceable |
| AC-07 / T07 | Inject ../, absolute paths, link replacement, ADS and more on native OS | No change outside authorized directory; existing targets not modified without authorization |
| AC-08 / T08 | Chinese IME, keyboard, screen reader and 125/150/200% scaling on three platforms | Critical actions reachable, no clipped text/lost focus, states announced |
| AC-09 / T09 | Cold/warm startup, idle, multitasking and real I/O on a fixed reference machine | Report entire process tree and complete package data, compare with budgets and explain error |
| AC-10 / T10/T13 | Core tests without Qt; later interoperability of two independent external extensions | One-way dependencies and capability-driven behavior; unauthorized/malformed frames rejected while host continues |
| AC-11 / T11 | Default use, errors and diagnostic export | No default network transfer; no secrets or full paths |
| AC-12 / T12/T18 | Clean-machine install, update, uninstall and Qt dependency verification | No implicit development-machine dependencies; delivery includes image worker and offscreen plugin, with real decoding after clearing SDK search overrides. Required ad-hoc integrity, unsigned-release explanation, provenance and license materials complete; active tasks preserved |
| AC-13 / T14 | Preview oversized image/text, HTML/executable contents | Bounded resources, no automatic execution, cleanup on close; M6+ only |
| AC-14 / T15 | Change source after indexing, disk full, terminate process before/after publication | No false success or deletion of committed/user-modified files; recovery explicit |
| AC-15 / T17 | Fuzz metadata/names/frames/recovery logs and minimal regression samples | No out-of-bounds or uncontrolled resource growth; diagnosable failure and regression coverage |

Record commit, case, platform/filesystem/architecture, tool versions, sample hashes, steps, expected and actual results, status, log location, executor and date. Status is only not run/pass/fail/blocked; skipping is not passing.

The first release executes only applicable capabilities; record inapplicability for later third-party extension/preview cases. Select affected cases for each change; execute the complete declared-capability set before release.

## Overlapping creation inputs and output, AC-04 / AC-06 / AC-07 / AC-14

| Case | Input and action | Pass criteria |
| --- | --- | --- |
| CREATE-OUT-01 | Explicitly add an existing ordinary target file plus other inputs to ZIP, TAR and TAR.GZ; compare with selecting a directory containing the target | All input bytes are archived intact. Normal final collision decision appears; overwrite is disabled when target participates as input. Rename/cancel preserve the original; forged overwrite responses cannot bypass protection. |
| CREATE-OUT-02 | Add only the existing target and run real Qt creation | Normal collision guard appears; the renamed archive contains the original target bytes, with no empty-input or special-object error. Bilingual protection messages are clear; body/footer remain reachable at large fonts. |
| CREATE-OUT-03 | System aliases of parent paths, case/Unicode-equivalent entries, same-named files in different directories, distinct files on case-sensitive volumes, hard links, links, directories and special objects | Ordinary inputs are not silently filtered. The final guard determines input-identity protection; link/special-object and hard-linked target limits remain. Private workspace and aliases cannot become input. |
| CREATE-OUT-04 | Split input includes existing `.001`, `.1024`, base `.zip`, `.1025` and other ordinary `.xdzip-*` files | Exclude only identity-matching private staging directories; preserve original bytes of all old volumes and other ordinary files in the new set. Whole-set rename or cancellation does not overwrite old volumes. |

See the [creation contract](../engineering/m4-creation.md) for rules and failure preconditions. Use owned temporary inputs only; verify actual output contents, old bytes after cancellation and staging cleanup. Record results in the corresponding issue and artifacts.

## Password extraction, AC-05 / AC-06 / AC-07 / AC-08 / AC-11

| Case | Input and action | Pass criteria |
| --- | --- | --- |
| AES-01 | Create Store/Deflate AES-256 using this app and independent 7-Zip, including AE-1/AE-2, ZIP64 descriptors, empty files and Unicode names | Physical entries, content and directories agree; AE-1 CRC, AE-2 zero CRC and complete authentication follow the contract. |
| AES-02 | Start without password, try wrong then correct password, different passwords per file, mixed encrypted/plain and duplicate physical names | New tokens consumed once, cache limited to task; correct retries commit per file without mixing other physical entries. |
| AES-03 | Empty password, spaces/control characters, 1,024-byte boundary and oversized/capacity-exceeding input | Independent extraction rules preserve exact input; excess rejected without retaining secrets and creation-password limits are not applied. |
| AES-04 | Corrupt authentication code, CRC, declarations, local/central fields, Deflate termination or tail | Explicit failure, no unauthenticated target published and existing targets unchanged; earlier commits counted honestly and cleanup failure warned. |
| AES-05 | Cancel while waiting, close password window, exit, fill ordinary queue; duplicate/stale/cross-type responses | Control channel remains usable without deadlock; input cleared and old responses cannot revive tasks or become conflict decisions. |
| AES-06 | Consecutive wrong passwords to limit, output before authentication, repeated decoding hitting byte limit, source change during wait | Retry count and cumulative budgets do not reset; source revalidation and authentication failure stop without bypassing quotas or committing old-source contents. |
| AES-07 | POSIX encrypted link graphs, safe execution permissions, invalid links, occupied leaves, Windows/TAR links | Authenticate link graph before output authorization; no-follow, closed graph, permission filtering and platform refusal remain effective. |
| AES-08 | Real Qt create → auto-open → extract → wrong password → correct input, close/exit; both languages/themes, 100/125/150%, DPR 1/1.25/2, small screens and long paths | Password window width 491, maximum 600, fixed reachable footer; password role and accessible name correct; show/hide and submit accessibility actions route real requests without logging passwords. |
| AES-ACCESSIBLE-NAME | Creation and extraction password inputs in both languages; activate accessibility before creation and after display; check name, password flag, text/value masking and show/hide | Readable names, no input leakage while masked; ordinary inputs unchanged. Record automated interfaces, Windows UI Automation and human screen reading separately without recording input values. |

See the [password extraction contract](../engineering/password-extraction.md) for format, secret, output and interaction preconditions. Record execution in issue #11 and local/CI artifacts; automated interface checks are not manual system screen-reader acceptance.

## Password prompts and all-session exit ordering, AC-06 / AC-08

These are repeatable requirements, not pre-recorded passes. Use real AES password waiting and the normal application Quit route; do not manufacture timing by hiding windows, disabling automatic presentation or directly invoking session exit.

| Case | Input and action | Pass criteria |
| --- | --- | --- |
| QUIT-PROMPT-01 | Password window actually visible; system Quit, Return, then Quit, Cancel | Native file pickers still block exit; managed password prompts allow exit confirmation over them. Return retains task/token; cancellation joins all Rust connections without update handoff. |
| QUIT-PROMPT-02 | Real task notification created a token, QML synchronized shown token but password window is still hidden; synchronously send Quit in that notification stack | Exit confirmation appears first and remains current modal after queued password callbacks; Return preserves the same password request and cancellation joins all. |
| QUIT-PROMPT-03 | Same real early request, but another session or main window active at Quit | Application-level decision blocks deferred password display in all sessions; inactive pages/windows cannot steal confirmation. |
| QUIT-PROMPT-04 | After natural password-window display, post Quit normally in a running bounded event loop | Real event traverses application exit filter and records actual destination window state; this is a post-presentation control, not early-window coverage. |
| QUIT-PROMPT-05 | After early Quit choose Wait, then Quit/Return, then Quit/Cancel | Wait releases the password-display pause and retains the token; a new prompt clears the old wait decision, Return leaves no exit pause and final cancellation still waits for all connections. |

Record natural hooks for six data rows, before/after actual Quit processing and deferred callbacks, current modal, task/prompt tokens, active count and join results. Controlled synchronous notification is an ordering regression, not proof of a unique historical macOS posted-Quit order. Qt may remove queued Quit when displaying a new window; do not report an undelivered event as handled or weaken delivery assertions. Check dimensions and bilingual font sizes against the window registry; perform native visuals as needed and record actual results.

## File input and macOS ACL regression, AC-07 / AC-14 / AC-15

| Case | Input and action | Pass criteria |
| --- | --- | --- |
| IO-01 | Verify the actual initial runtime source handle uses nonblocking classification. Replace path with FIFO and FIFO alias; also test directories and special objects. | Ordinary archives browse; special objects return `Io` without waiting for FIFO peers. Use a bounded independent test process so regressions cannot hang the suite. |
| IO-02 | Open via source symlink to a regular file, then substitute paths for source revalidation and task source copy | Legitimate aliases remain usable; special objects or source identity/version changes return `SourceChanged`, without publishing stale queries or starting tasks on incorrect sources. |
| MAC-ACL-01 | Configure owned temporary targets with `everyone deny delete`, external read/execute ACLs and inheritable forms; actually save extraction or creation | Directory grant, staging, parent creation, publication and cleanup succeed with correct bytes and no rewriting of user ACLs. |
| MAC-ACL-02 | Grant external users/groups write, create, append, delete, attribute/extended-attribute write, ACL/owner change and inherit-only modification permissions separately | Reject before output, preserving old contents and outside files without partial publication; preceding deny does not exempt dangerous allow. |
| MAC-ACL-03 | Compare current effective user's modification ACL against external user/group rules | System-resolved and synthetic user UUIDs identify the current user; group principals are not mistaken for that user. Unknown permissions/rules and query failures reject. |
| MAC-ACL-04 | Add dangerous ACL to target parent chain or existing output after grant, then continue staging/publication | Revalidation rejects unsafe objects; POSIX mode, ownership, single-hard-link and no-follow restrictions remain. |

ACL fixtures alter only test-owned objects, never clear or change real user directory permissions. Nonblocking classification addresses special-object opening only; these cases do not prove slow regular-file I/O, atomic content snapshots or malicious same-account concurrency. Record native results and remaining work in issues and local/CI artifacts; cross-platform compilation, mocks or independent-process timeouts do not replace actual safety results.

## Operation and selection regression checklist

This table specifies requirements; runtime results, screenshots and measurements stay in build/ or dist/.

| ID | Input and action | Required observation |
| --- | --- | --- |
| UI-REV-01 | Run create/extract, change terminal state, expand then collapse details | All windows width 491, height from content up to 600, details up to 168, compact height restored on collapse. |
| UI-REV-02 | Select, focus and press three format options in turn | Only outermost left/right corners are rounded; inner corners stay square. |
| UI-REV-03 | Hover, mouse press, Space press and disable blue buttons in both themes | Normal, hover, pressed and disabled clearly distinguished using shared colors. |
| UI-REV-04 | Edit extraction destination; outside click or Escape/Return/Enter | Field loses focus without starting extraction; first Escape does not close window. |
| UI-REV-05 | Display selected and all scopes | Title left, secondary-color count far right; whole row selectable. |
| UI-REV-06 | Creation/extraction conflict at 100/125/150% and historical 200% stress | Conflict window width 491, title 20, body 13, footer 13, max 560, scrolling body and reachable actions. |
| UI-REV-07 | Select all, clear and select individually through last item for same query | Select-all and individual last-item properties agree and action is available; four properties and action hidden without selection. |
| UI-REV-08 | Hover and press preview extraction action | Hover/pressed/focus visible; unauthorized state disabled. |
| UI-REV-09 | Complete multiple tasks and reopen from bottom menu | Accurate history count, active count still 0, status badges right, normal menu density and keyboard navigation. |
| UI-REV-10 | Clear selection and open Extract All dropdown | Selected and preflight disabled without extra selection hint line. |
| UI-REV-11 | HTML 100%/125%, Qt DPR 1/1.25/2, long paths and accessibility fonts | Logical dimensions independent of DPR, correct body/menu ratio and reachable footer. |
| UI-REV-12 | Root, ordinary parent and leaf context menus using right-click, Menu and Shift+F10 in both languages/themes and three sizes | Root Collapse Subdirectories disabled; no unavailable item shows visible hint, reason line or tooltip; accessible descriptions give accurate root/leaf reasons. Parents expand/collapse and disabled actions change no selection/path. Measure label centering within 0.5 logical pixels in the command's local coordinates independently of ancestor translation; do not ignore scale, rotation or custom transforms. Check current text height and conditional slots at first observed rendering after closed and open font changes. |
| UI-REV-13 | Delayed scroll events in shared HTML menus | Real right-click changes non-ancestor position before menu opens; a delayed trusted scroll with position unchanged since opening does not close it, while a later position change does. Retain all 18 directory input/language/text-scale combinations. |
| UI-REV-14 | Launch and open creation window in both languages/themes and three sizes | Labels wrap, all four format captions fit and footer remains reachable; initialization and presentation produce no Qt Quick recursive-layout warning. |
| UI-REV-15 | Mouse/keyboard select and replace text in main search, regular/password inputs and editable dropdowns in both themes | Selected foreground and background readable; password mode and editing behavior intact. |
| UI-REV-16 | One idle welcome page, loading/error, single archive and multiple tabs across themes and sizes | Sole ordinary idle welcome tab hides close and removes it from focus; other cases retain existing mouse, keyboard and accessibility closing behavior. Hover/pressed close backgrounds are circular; main/menu ratio unchanged. |

Implementation regressions are in tst_operationrevisions.qml, browser_test.cpp and shared-control scripts; high-fidelity and HTML revisions are in design/hifi/operation-revisions.html.

## WP-11 TAR/GZIP

| ID | Input / action | Expected |
| --- | --- | --- |
| TAR-01 | System tar/Python USTAR, GNU, PAX, TAR/GZIP and segmented multi-member GZIP | Content detection and complete scan; names and all bytes match, empty directories retained. |
| TAR-02 | Select TAR/TAR.GZ in Qt, create, reopen and extract | Format and extension agree, task succeeds and bytes match without fabricated per-entry compressed sizes. |
| TAR-03 | Enable password/volumes, then switch format | Inapplicable options cleared and disabled, TAR level disabled, TAR.GZ 0/1/6/9 writable. |
| TAR-04 | Path escape, links, devices, FIFO, sparse and global PAX | No unsafe output; explicit refusal without library unpack writes. |
| TAR-05 | Truncation, header checksum, CRC/ISIZE, nonzero tail, extension headers and byte budgets | Complete scan fails without partial index publication; over-budget and cancellation are explainable. |
| TAR-06 | Rename compound-extension conflict and cancel overwrite task during validation | Preserve .tar.gz, leave old archive unchanged and clean current staging. |
| TAR-07 | High-ratio GZIP with default and explicitly allowed intents | Default rejects before write; allowed still checks source, byte quotas and safe publication. |

Test entries are codec-tar, runtime creation tests, Qt tarCreationBrowseAndSafeExtraction and the [independent interoperability script](../../scripts/check-tar-interop.py). Actual logs/screenshots remain in build/review/wp11 or CI artifacts.

Historical desktop permission or device blockers are not current status. Manual acceptance follows corresponding issues; offscreen or Cocoa automation is not reported as human operation.

## W01 welcome integration, AC-01 / AC-04 / AC-08

| Case | Action | Pass criteria |
| --- | --- | --- |
| W01-01 | Start, open a normal archive and close it | Welcome switches to real browsing and returns; an empty archive is still browsed. |
| W01-02 | Create from welcome, close draft | Real creation window opens, no task is created and focus returns. |
| W01-03 | Drop one regular ZIP/TAR/TAR.GZ/TGZ | Existing core detects and opens it without automatic extraction. |
| W01-04 | Drop multiple items, directory, network URL, unsupported extension or damaged ZIP | Reject input shape or show actual core error; no false success. |
| W01-05 | Light/dark, DPR 1/1.25/2, minimum window, 100/125/150% fonts | Cards and shared-menu proportions correct, entries keyboard-accessible and body scrollable. |
| W01-06 | Close archive after starting task, reopen from welcome task menu | Historical count and real state retained; welcome does not cancel background tasks. |
| W01-09 | Retry after failure/cancellation; close a window containing background tasks and reopen the same archive | Reuse an idle welcome page. Retire only an untouched automatic welcome in that window; preserve user pages, other windows, dialogs, password prompts and task identity. |

Automation records go only to build/review/welcome/ or CI artifacts. Select native drag-and-drop, system picker cancellation, screen-reader, IME and platform appearance checks according to changes; record unexecuted limits rather than marking offscreen screenshots passed.

W01-07: welcome and opened-archive title bars, tabs and appearance buttons use the same component positions, dimensions, fonts and icons; welcome-specific rearrangement is forbidden. Automation compares actual Qt geometry and HTML shared-title source coordinates.

### Window-wide focus and title-bar revision

- Shared controls gain no extra outline, halo or thick border on mouse/keyboard focus; normal borders and button states remain.
- Outside click, Escape, Return and numpad Enter leave control focus; Enter executes its action once, search text remains and first Escape does not close the window. Menu arrows and level-by-level closing work; closing does not leave trigger focus behind.
- Application window titles omit “ — xdzip”. On macOS launch, resize and reactivation, traffic-light centers differ from title-bar center by at most 0.5 logical pixels and are not clipped by ancestors.
- Save automation to build/ or dist/ and record actual desktop appearance checks separately.

### S01 settings and internationalization

- Qt component fixtures use an isolated Unicode configuration path under build/ and verify Qt/native environment-variable round trips and actual write capability. For settings switches, check display, model and persisted values. On Windows, use owned configuration and backup read handles without delete sharing to verify recovery from transient contention and rejection of persistent contention, checking originals, backups and saved model. Failures show the specific settings error; a wait timeout does not replace write diagnostics.

- Retain original icon headers, theme illustration cards, list density, recent-history and privacy areas; added language uses existing row layout.
- Standard/compact affects the real table and cancellation rolls back. Disabling history stops additions; clearing first marks and then removes only the record file.
- Title-bar Settings, More menu and platform Settings shortcut open the same standalone nonmodal settings window.
- System/light/dark themes and 100/125/150% text provide real previews. Cancel/system close roll back; Restore Defaults changes only drafts. Successful Save persists; failure retains draft and error.
- Options are auto-detect, 中文 (简体), English (US), default auto. Cover all Chinese and English regions, unsupported/missing languages and English (US) fallback; application numeric locale cannot override system UI-language detection. Manual and auto selection apply immediately; reopening settings and next launch restore selection mode, while save failure restores mode and effective language. Archive contents, protocol codes, paths and user filenames are not translated.
- Both catalogs have complete IDs, plurals and placeholders; QML/C++ product copy has no scattered static Chinese. Verify English welcome, operation titles, file types, errors, numeric/time/byte display and 100/125/150% layouts.
- Check native system themes, keyboard, screen readers and real desktop appearance as needed; automation and artwork do not stand in for unperformed desktop checks.
- At 100/125/150%, settings dropdowns align right and fit complete options plus checkmark column, at least button width, with long entries bounded by window and maximum width.
- Density, text size, language and opening-mode dropdowns have equal heights with unclipped text/indicators in both languages/themes at every supported scale.
- Footer buttons stay on one row at 150% and historical 200% layout testing; hints may wrap separately. Old 200% settings migrate to 150%.
- Check macOS localization declarations and actual build package; native picker uses system language on a Chinese system, and app language switching does not write system preferences. Record native dialog appearance when checked.

### Extraction submission visual stability

- On actual Start Extraction click, toolbar and file list retain their states without briefly showing directory-loading overlay; extraction Change button does not switch to disabled styling.
- During directory, search or sort queries, committed nonempty list/grid and empty results remain visible until new results commit. Check overlay visibility, final directory/query results and busy/cancel behavior.
- Start immediately prevents reentry; repeated calls create one task. Accepted submissions switch task window; failures retain confirmation contents.
- Real opening/directory changes, search and preflight retain request admission and loading feedback; task submission does not reset the model or clear selection.

### Preflight, row boundaries and window ownership

- Preflight width 491, body 13, title 20. Small content contracts; 64 issues at 150% scroll within a client area up to 600 and footer Return remains reachable.
- Visible gaps on either side of main-list color blocks and separators are equal; preview show/close buttons share window coordinates and main/menu font ratio remains.
- All native file/directory pickers belong to their actual calling window and cancellation returns there without raising main above the operation window. Closing the caller cannot redisplay it. Owned nonnative pickers can verify shared ownership policy but do not stand in for unperformed Cocoa picker checks.
- A valid Finder archive drop leaves the receiving main window foreground; invalid drops do not activate it. Verify native behavior as needed and record unexecuted limits.
- POSIX ZIP Framework multi-level symlinks and safe regular-file execute bits can be restored. Synthetic app bundles verify link bytes, dereferenced contents, 0755 execute and stripped privileged bits, including common macOS bundle parent-relative normalization. Absolute, outside-root parent, dangling, cyclic, device, control-byte, link-parent writes and bad CRC reject before output, as does partial selection missing dependencies. Link tasks cannot rename, skip or overwrite occupied leaves. Windows/TAR links remain explicitly unsupported and are not downgraded to regular files. Native publication checks parent-chain substitution, existing symlinks and unchanged external targets; real third-party archive summaries remain local and do not replace change-specific boundaries/regressions.

Preflight footer regression: at 100/125/150% match shared action-only footer height at the same font. Explanation belongs in the body; long reports or explanations scroll only the body and Return stays reachable.

### WP-12 initial desktop automation

- M5-01: actual windows produce rendered frames and record rendering API; hardware-verification mode rejects software and Null backends.
- M5-02: Chinese pre-edit does not submit early, Chinese and non-BMP filenames commit correctly and text remains after Escape leaves focus.
- M5-03: real Qt accessibility provides input role, name, value and text interfaces, with correct button names and disabled states.
- M5-04: selected fills at both ends of format segments have smooth outer corners and square internal seams; Qt software/GPU, DPR 1/2 and HTML do not double-inset.

Historical results are in [private-test admission](https://github.com/xiaodaozhi/xdzip/issues/5). Check system IME/screen readers as needed; synthetic events and interface checks are not performed human checks.

### 2026-10-16 user feedback revision

- W01-08: welcome loading shows body overlay, active progress and Cancel, blocking repeated body actions. Cancellation cannot open later, errors recover and title bar stays unchanged.
- C01-08: dark password placeholder is clear while enabled and empty; creation result says “Saved”. Creation completion never calls the system file manager even when an old caller passes openAfter=true; supported successful results including numbered volumes still open in-app.

See the [loading and creation completion contract](../ux/welcome-opening.md) for parameters and semantics.

## Windows native regression

- PE subsystem checks require product `xdzip.exe` to be GUI; diagnostic/test programs retain console. Deployed Windows platform startup checks still run.
- Windows main has no extra system title bar; minimize, maximize/restore and close to the right of Settings work. Double-click blank title area toggles maximize; close still waits for runtime exit. Native hit testing handles edges/corners and preserves resizable frame, DWM shadow and supported-system corners. Check WS_CAPTION/WS_THICKFRAME, nonclient rendering and 1024×622 client; verify external decorations separately via desktop composition screenshots. Check snapping, native dragging and multiple displays as needed.
- On owned temporary directories add Everyone read/execute permissions and complete stage files, parent directories, publication and cleanup. Then separately add write, delete, change owner, change ACL, generic write and inherit-only write, verifying rejection before output and unchanged old contents. External-owner, link and special-object restrictions remain; existing user-directory ACLs are not rewritten.
- Local/CI logs, measurements and screenshots stay in build/ or short-lived artifacts. Platform conclusions follow actual execution records; this development verification is not unperformed platform acceptance.

## Platform icon padding revision

Check application and ZIP document PNG, ICO and ICNS at 16–1024 pixels. Generic PNG/ICO body height is approximately 90% of canvas, macOS PNG/ICNS approximately 81.2%, allowing small-size rounding. Corners are transparent, radii continuous, paper undistorted and shadows excluded from body bounds. Decode all seven ICO and eleven ICNS representations and match embedded images to corresponding platform PNG sources; macOS Qt runtime must select macOS PNG. Review actual-size contact sheets on light/dark backgrounds; main-window/menu 24-unit UI icon geometry and density stay unchanged. `build-brand-assets.py --check` automatically verifies bounds, transparency, sources and containers; native Finder/Dock/Explorer cache display needs separate manual inspection, not container decoding alone.

## Version and settings About

Product entry filenames are Linux `xdzip`, Windows `xdzip.exe` and macOS `xdzip.app/Contents/MacOS/xdzip`; verify actual entries in ordinary builds and deployed packages. Tests and diagnostics retain their own names and are not user launch entries.

Verify `VERSION.json`, Cargo workspace, CLI, Windows version resources, macOS bundle and settings About agree. Cover development/stable channels, invalid version and build limits, no-Git unknown, dirty markers, both languages/themes, 100/125/150% and high DPR. Copied JSON has no personal paths; all About entries reach the same settings page. HTML does not fabricate native Qt or commit identity. See [WP-12](https://github.com/xiaodaozhi/xdzip/issues/5) for complete process resource observation and limitations.

### Settings, associations and networking

- Six tabs are reachable; sidebar and body meet, with no footer, Save or Cancel. Successful changes apply immediately and survive reopen/cross-process; write failures preserve previous values and show errors.
- Both languages at 100/125/150%: sidebar fits actual words, English wraps only between words, and sidebar/body remain reachable without horizontal overflow.
- The six bodies do not repeat page names or their divider. During update checks, Cancel Check occupies the original button position and works via mouse/keyboard; other update actions remain available.
- During discovery, download, downloaded and recovery states, primary action is right of status and copy/behavior reflects trust and install capability. Download progress and Cancel Download work, available open-folder entries remain, and state changes do not auto-scroll or alter thumb height.
- Light/dark design, HTML and Qt match header divider, sidebar hover and icon left padding. In both languages and 100/125/150%, mouse-driven checks and state changes preserve body scroll position and scrollbar-thumb height; keyboard navigation reveals offscreen controls and bottom licenses remain reachable.
- Language changes immediately refresh UI, native and tray menus including application About/Settings/Quit, task submenu and tooltip without interrupting tasks or restart prompts. Write failure preserves previous language; system items follow system language. Clearing history is immediate without deleting original files.
- Supported types show actual system defaults; refusals/cancellations are not falsified. Verify Windows system confirmation, macOS content types, Linux MIME and shared compound extensions.
- Verify direct, system, custom proxies and HTTPS mirrors for validation, persistence, update traffic, failure messages and signature verification; active updates retain their configuration. macOS native installation supports system, direct and custom HTTP CONNECT/SOCKS5. Disposable hosted runner static-proxy/PAC controls must first prove actual system-mode feed/archive hits, then zero direct hits, checking HTTPS routing, untrusted TLS refusal and restored original settings. Check custom-proxy failure has no direct fallback, actual direct product upgrade and active-configuration snapshot, signatures, task exit and recovery. Download success alone does not prove proxy use.
- Empty creation input lists have no instructional placeholder but retain Add, list border and summary.

## Native menus and tray

For settings placement, place the pointer at the bottom-right of each available display and open/reopen real ownerless Settings and About. In both languages, light/dark themes and 100%/125%/150% fonts, the entire native frame must fit and be centered in the actual QScreen::availableGeometry without restoring an existing main window. Record native geometry separately from actual tray-menu interaction.

Verify Windows tray white fill and dark outline on light, dark and colored backgrounds at every supplied bitmap size, across app theme changes, refresh and tray disable/re-enable. Legibility does not depend on registry theme reads. Pixel checks reject the former black-only icon on dark backgrounds; record actual notification-area observation separately.

Custom radio controls must actually execute QAccessibleActionInterface press and toggle for theme, creation format, repeated activation and disabled items. Check selection model, actual theme, proxy, format extension and all checked states agree and successful settings persist. Mouse-only or Accessible.checked-only checks do not pass this case.

Checkboxes must actually execute QAccessible press/toggle, checking real settings and rollback on failure, list single/select-all tri-state, repeated toggles and disabled items. Settings switches have complete names in the native tree.

Under the [desktop integration contract](../ux/desktop-integration.md), verify macOS menu roles/standard shortcuts, separation of text editing and file selection commands, recent-history privacy, default tray and immediate toggles, close-to-tray and task-detail restoration, exit cancellation/wait/cleanup, file associations and Dock restoration, no-tray and disappeared-tray fallback. Automation does not replace real system panels, scaling, screen readers or Linux desktop verification. Results stay in build/ or CI artifacts.

Opening Settings or About from platform trays must leave existing main windows hidden or minimized or avoid creating them. Exit without tasks does not show main; active tasks show only necessary confirmation. Check Ubuntu X11 integrated title bar, smooth normal-state corners on all four sides, system movement and eight resize directions, and borders/controls after maximize/restore. Record native visuals separately from automated layout tests.

## Read-only archive checking

- Start a whole-archive task from the real toolbar Test; selection, directory and search do not narrow scope. Cover ZIP/ZIP64, Store/Deflate, supported volumes and AES-256, TAR/TAR.GZ, duplicate names, directories, empty files and ZIP link payloads. Create no files, staging directories or links during checks.
- Corrupt local headers, descriptors, CRC, compressed-stream termination or AES authentication. Verify first-error termination, failed entry and verified/unchecked counts; unsupported entries cannot be called whole-archive success. TAR claims only structure/readability; TAR.GZ consumes the tail and checks GZIP CRC/ISIZE.
- Check actual read/verification counts without saved/committed counts or output-directory actions. Do not fabricate unknown scan progress with timers. Wrong-password retries do not reset resource budgets; stale prompts and terminal responses reject.
- Cover cancellation while password-waiting or queued, reopening tasks after closing the browser page, source/volume changes, cross-session serial admission and exit joins. Separately check both languages, font sizes, themes, password and success/failure/cancel windows; record native visuals and platform limits independently.

## Confirmation-window integrity and visual gates

- The registry contains all QML top-level and inline windows with explicit system-picker exceptions. Automatic checks reject unregistered windows and historical 612-wide exit confirmations.
- Exit, stop and restart confirmations remain width 491, title 20 and body 13 across both languages/themes, 100/125/150% and DPR 1/1.25/2. Short content leaves no large blank; long bodies and small screens scroll, with fixed footer reachable in client area.
- Three buttons stay on one row; English large text may wrap inside buttons without clipping, overflow or custom footer styles.
- Verify existing return, wait, cancel exit, later and restart intents; decision buttons disable while stopping. Record function and visuals separately.
- Run `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py`, Qt `ConfirmationWindows` tests and `scripts/check-html-confirmations.py`. Screenshots/measurements go only to build/ or dist/.

## Windows fixed installation directory

- Fresh-install main executable, shortcuts, document icons and open commands point to root; required Qt subdirectories remain and no versions/ is created.
- Upgrade, roll back and upgrade again using a full historical-product old-layout fixture. Verify actual commits, settings, extra user files and system defaults survive.
- Manifests allow only audited bounded relative paths. Reject traversal, device aliases, case duplicates, reparse points and modified obsolete files; on success delete only unchanged obsolete files listed in the manifest.
- Reject upgrade and uninstall while running. Uninstall cleans root and cumulatively recorded historical programs while preserving extra files in root and historical directories.
- Before full Qt compilation, Windows builds and installs the real Inno installer using clearly QA-labeled file fixtures that are never executed, first checking root, shortcuts, actual manifest cleanup and uninstall. Read shortcut Unicode targets with IShellLinkW and compare paths resolved from existing file handles; allow path aliases, avoid WSH ANSI conversion of Chinese to question marks and never treat question marks as wildcards proving a valid target.
- scripts/test-installers.py checks generation/path contracts; native Windows scripts/check-windows-install.ps1 checks actual installation/association/cleanup. Other platform layouts do not change.
- Check candidates, individual icons, commands and real archive opening for every supported Windows extension, preserving old shared ProgID, external candidates and UserChoice. After historical rollback, new format entries open the rolled-back version; uninstall removes only owned registrations. Record final `.tar.gz` extension and existing user-selected icon limitations separately.
- After upgrade and fresh install, native enumeration yields one xdzip app for all nine formats, with real embedded brand icon and identity in the association launcher. Upgrade removes owned old recommendations but retains compatibility commands; REG_NONE prevents automatic default takeover. Verify external candidates and UserChoice/UserChoiceLatest unchanged, and ZIP/other new entries still open after historical rollback. Manually inspect Default Apps and Explorer Open With separately: exactly one branded xdzip option. Do not conflate generic application boxes with document icons or use native API enumeration instead of visible UI.
- Nine per-format Windows ProgID descriptions are `XXX 压缩包`/`XXX archive` after fresh install, upgrades with existing Chinese/English settings and language restart. Portable and isolated settings processes do not modify registry; if installed identity or any open command mismatches, no descriptions change. Preserve defaults and other candidates. Honestly record partial write failures and check next-start retry. Manually inspect Explorer display and cache effects separately.
- On macOS verify each ICNS in bundle, Viewer and None/Alternate declarations, native candidates, real file opening and unchanged defaults; compressed TAR types share an icon. On Linux verify MIME candidates, real GIO opening and unchanged defaults, retaining system icons without adding global MIME icons.
- On isolated macOS without existing third-party type declarations, check RAR, CAB and UDF resolution/candidates; CAB covers system and compatible types, and Settings queries the supported type actually recognized. Existing defaults remain during registration; types previously without handlers may temporarily resolve to the new candidate; all test types return to original state after unregistering.
- On macOS retaining an old invalid UDF declaration, inspect actual `.udf` Open With, None/Alternate candidates and Settings state. Preserve third-party defaults and user preferences and restore prior resolution after unregistering.
- Record browser/offscreen results separately from native Explorer/Finder display and human operation; command or CI success does not establish visual acceptance.

## Session and prerelease acceptance

- Open settings, About, create, extract, tasks, password, conflict and file pickers in native normal, maximized, fullscreen, minimized and tray-hidden states without transient restoration; dialogs belong to callers.
- When minimizing during macOS fullscreen entry or its callback, verify actual final minimization. Newer restore, normal or maximize requests take precedence. Restoration during reissued requests keeps fullscreen and cancels stale requests; destroyed windows must not be recreated.
- Concurrent launch, forwarding before readiness, crash-residue recovery, IPC timeout, Unicode and batch file requests establish one healthy process and tray; forwarders exit after confirmation.
- Save, cancel and restart both opening modes; test welcome reuse, duplicate activation, directory/selection isolation, background continuation after close, password/conflict routing for same numeric task IDs, serial heavy tasks and global async exit.
- Both languages, 100/125/150%, DPR, capsule tabs, shared controls, accessibility press/toggle and unknown old-configuration field preservation.
- Independently start/install all six same-tag package types, Mac signing and relocation, manifest channel/semantic version selection, rejection of missing packages and bad hashes, and re-download verification after publication. Explicitly retain unperformed native manual-visual limitations.

## Multi-volume browsing and temporary opening

| ID | Action | Required result |
| --- | --- | --- |
| OPEN-01 | Open Store/Deflate, ZIP64 and AES-256 from numbered volumes and middle/final PKZIP volumes | Complete list, cross-volume local headers/payload extractable, wrong password retryable; missing/corrupt volumes return no partial list. |
| OPEN-02 | Missing middle volume, duplicate numbers, links/special volumes, replace/modify any volume after scan | Explicit failure, no source volume overwritten by output, cancellation and limits remain. |
| OPEN-03 | Double-click directory, regular file and link in list and grid | Navigate directories; extract only the selected physical regular record to private temporary storage; do not system-open links/special objects. |
| OPEN-04 | Temporary extraction succeeds; system accepts or rejects open | Acceptance auto-hides its progress window; rejection retains error/retry. Failure, cancellation and password waiting do not automatically open or hide. |
| OPEN-05 | Chinese, spaces, percent, command characters, duplicate names, closing tabs and exit | Pass local file URLs exactly without shell; tasks retain unified lifecycle and temporary copies survive asynchronous open. |
| OPEN-06 | Expired, active-locked, foreign-marked and linked cache entries | Clean only expired unlocked app-owned directories, preserving active/foreign objects. Buttons/errors remain reachable in both languages, three font sizes and DPR. |

## Legacy filename encoding adaptation

- ENC-01: owned GBK/GB18030, Big5, Shift-JIS, Korean, Russian, CP437 ZIP and traditional TAR/TAR.GZ names use the same effective spelling for physical identity, directory tree, list, search and directory selection, independently of UI language or OS code page.
- ENC-02: verify encoding precedence and conservative failure for UTF-8 flag, Unicode Path version/CRC/duplicates/corruption, unmarked UTF-8, PAX UTF-8/BINARY and GNU long names; do not repair declarations with replacement characters.
- ENC-03: independently read output names/content for preflight, actual extraction, temporary opening, external export, ordinary archive addition and movement. Corrupt declarations, unsafe paths, Unicode/case collisions, cancellation, budgets and source protection remain effective.
- ENC-04: AES and numbered volumes use the same names and verify content; encrypted/split addition and movement still reject. Independently verify UTF-8 flags or PAX paths after rewriting ordinary archives.
- Manually check actual text display, system opening and drag-and-drop on native desktops. Automation/statistical detection cannot replace real results or claim unique identification for every short name or mixed encoding. Record results and boundaries in issues.

## Isolated 7z and RAR backends

These are capability acceptance requirements, not pre-recorded passes. Record actual results and uncovered scope in issues. See the [isolated backend contract](../engineering/native-codecs.md) for build and permission preconditions.

| ID | Action | Required result |
| --- | --- | --- |
| NATIVE-01 | Check pinned sources, checksums, compile manifest, runtime format enumeration and deployment | 7z/RAR/CAB/ISO/UDF use fixed Rust routes; generated manifest contains no official 7-Zip or UnRAR source. ZIP/TAR/TAR.GZ use in-process Rust with no second container route. |
| NATIVE-02 | Create 7z and read both ways with independent official tool: ordinary/empty files, Unicode, directories, four levels, solid, content and filename encryption | Metadata and per-file bytes agree; commit only after complete validation. Filename encryption available only after setting a password. |
| NATIVE-03 | Open legacy RAR, RAR5 and RAR 7 new-algorithm samples and compare independent UnRAR output | Verify known output for each generation; record password, encrypted header, solid and volume combinations separately. Magic bytes do not prove decoding. |
| NATIVE-04 | Open from any 7z numbered or RAR old/new named volume; missing/wrong volumes, case duplicates and source replacement | Authorize complete groups; RAR preserves boundaries. All volumes have identity/overwrite protection and anomalies fail explicitly. |
| NATIVE-05 | Encrypted directory: no password, wrong input, correct retry, cancellation, page switch and exit | No index before successful scan; old requests do not pollute new sessions; secrets do not enter events/logs. Native password window and exit ordering are accepted separately. |
| NATIVE-06 | Add/replace/move in ordinary and solid 7z; attempt modification of encrypted/split 7z and RAR | Ordinary 7z is fully rebuilt, checked and atomically replaced; unsupported archives reject. Cancellation, corruption or 512 MiB rebuild-cache limit preserves original bytes. |
| NATIVE-07 | Truncation, corrupt checksums, unknown algorithms, dangerous paths, special objects, oversized dictionary, worker crash and cancellation | No failed entry or unverified file published; existing targets preserved; actual reads, retries and solid skips consume budgets. |
| NATIVE-08 | Full product chain: browse, temporary open, export, test, extract, create and modify | Background tasks, errors, passwords and source identities flow end to end; check writes nothing and RAR offers no create/modify. Record actually executed platform automation and desktop checks separately. |
| NATIVE-09 | Create Store 7z larger than 512 MiB, reopen and test whole archive; also create small numbered volumes | Whole output is not accumulated in native memory. Host random writes affect private staging only; cumulative writes and length high-water mark remain separate and header patches neither overcount volume tails nor truncate. |
| NATIVE-10 | Read 24 independently official-authored samples with fixed owned plaintext through ordinary Rust tests, actual worker and host service | Copy, LZMA/LZMA2, Deflate/Deflate64, BZip2, PPMd, filters, BCJ2, solid, compressed headers and AES combinations match complete bytes per file; encrypted headers exercise actual opening-password retry. Small samples do not replace boundary/resource/desktop acceptance. |
| NATIVE-11 | Host source returns typed limit, cancel, unsupported or source-changed errors on initial seek, random seek/read, creation input and EOF probe, including Interrupted wrapping | Preserve stable classification; retry Interrupted only without a typed cause, and cancel immediately. Admitted source counts are not refunded; errors publish no file or replacement archive. |
| NATIVE-12 | Unknown-size RAR and known empty file through real worker, runtime pages, both size-sort directions, Qt selection, preflight and output tasks | Unknown differs from zero; bilingual state and size summaries are not misleading; unknowns sort last. Virtual directories cannot gain preflight via early summary return; unknown items reject before decoding, password request or staging. |
| NATIVE-13 | OS character projection for legacy passwords, local calendar for modern times and auxiliary-work accounting through actual platform callbacks and real workers | Rust retains format decisions and budgets, thin C++ supplies OS primitives only; secret buffers cleared, typed errors retained and auxiliary work not counted as decoded output. Record actual platform evidence separately. |
| NATIVE-14 | Open each CAB volume of a file spanning at least three volumes, where a logical previous-volume reference points to the file's starting volume; check independent known plaintext, missing/wrong sources, truncation and cancellation | Forward order and every file fragment fully checked. A non-adjacent previous volume must be the actual continued file's starting volume; failures publish no partial index/files, while existing adjacent-previous samples remain supported. |

## CLI, AC-04 / AC-05 / AC-06 / AC-07 / AC-11 / AC-12 / AC-14

| Case | Procedure and criteria |
| --- | --- |
| CLI-INTERFACE | Without Qt/display session, run bare commands, bilingual help, version, aliases, literal paths, JSON, pipes and invalid arguments. Check one result object, stable exit codes, unknown sizes and duplicate physical identities. |
| CLI-ARCHIVE | Create/add/selectively extract ZIP, TAR, TAR.GZ/TGZ and 7z; independently read output bytes. Cover passwords, filename encryption, volumes, read-only refusal, filters, source protection, path ambiguity and zero writes before failure. |
| CLI-CANCEL | Use real termination signals and conflict/password waiting. Check cooperative cancellation, terminal, join, actual commit statistics and staging cleanup; forced kills do not replace ordinary cancellation. |
| CLI-INSTALL | Install in owned temporary locations, test stable launcher forwarding, existing-entry refusal, modified-entry protection and deregistration. Record real user PATH, system install authorization and human terminal operation separately. |
| CLI-UPDATE | Private signatures and native executable fixtures verify separate manifests, tampering, offline/rate-limit/timeout errors, digest/length, staging, self-check, atomic pointer, concurrency exclusion and recovery. Simulated transport is not real HTTPS acceptance. |
| CLI-DESKTOP | Same-user bounded IPC remains distinct from file opening and refuses during shutdown. Desktop updater revalidates candidate and all task barriers; installation does not break CLI occupancy and handoff is not installation. Headless identity queries do not consume receipts. Mac ordinary context exists only in Resources, repeated builds leave no stale markers, and actual native signing/fixed GUI-context resolution pass. |

## Large indexes and explicit preview

| Item | Required checks and pass criteria |
| --- | --- |
| INDEX-MILLION | Real million-entry ZIP64 with Chinese, long names, duplicates and nested directories. Check physical and displayed counts separately, read all result batches and record scanning/sorting/searching plus peak memory and disk bytes. Do not generalize one machine's results to all platforms. |
| INDEX-PAGES | Qt retains complete logical row count with distant jumps, search and sorting; at most 1024 rows resident. Reject stale pages on session, query, generation, revision or offset mismatch; changing query clears old selection. |
| INDEX-SELECTION | Cross-page selection and Select All cover the entire current query and survive cache eviction. Freeze stable identities before extraction confirmation and use that set after query changes. Exceeding task limits rejects the whole selection, not a partial submission. |
| INDEX-OUTPUT | Select real regular files from large indexes and output only after payload integrity/authentication and existing safe authorization. Source changes, duplicate offsets, exceeded limits, cancellation, links and unsupported whole-archive operations fail explicitly and preserve existing files. |
| INDEX-CLEANUP | Normal close, query replacement, failure and cancellation clean known private files. Seven-day reclamation retains active leases, unknown objects, links and identity mismatches and never cleans user content. |
| CONTENT-PREVIEW | After explicit shared action, completely verify real text and declared supported images with plain-text semantics. SVG must reject external CSS references even after merging XML/CDATA text segments, while valid internal fragments remain usable. macOS system HEIC decodes under bounded runtime file quota, growing within budget and rejecting above it; the app does not export an entry copy. Reject unsupported types, encodings, dimensions, pixels, input or algorithm-work limits without showing partial contents or launching external apps. |
| PREVIEW-LIFETIME | Missing, wrong and correct passwords, corrupted authentication, cancellation, closing password dialog, changing entries, hiding pages and closing sessions clear contents. Late results cannot restore content; events and Qt mailbox account actual content bytes. |
| PREVIEW-VISUAL | In main and standalone property viewers, text, images, errors and cancellation are legible/reachable in both languages/themes and 100/125/150%; preserve main/menu proportions. Record actual native observation separately from automated functional checks. |

## External SDK read-only trial

Run `scripts/sdk-trial.py test` under the [SDK trial contract](../engineering/sdk-trial.md), independently of enabling or releasing product third-party extensions.

| Item | Required checks and pass criteria |
| --- | --- |
| SDK-WIRE | Rust and independent Python interoperate on actual metadata, multi-block contents and host digest; explicitly reject non-shortest integers, duplicate keys, indefinite lengths, invalid UTF-8, excessive depth/length, unknown required fields and incompatible versions. |
| SDK-BROKER | Reject source/operation/sink identity errors, sequencing, out-of-bounds/overflow, duplicate tokens, unsafe paths, stale credits and incorrect digests. Do not give peers host paths or output handles. |
| SDK-COMMIT | Source changes, target conflicts, missing/duplicate terminals, crashes, late data and cancellation publish no files; staging/directories are cleaned and existing targets preserved. Commit only after final source recheck, normal close and EOF. |
| SDK-PACKAGE | Reject unknown permissions, path escapes and digest mismatches. Verify and privately retain the actual executed copy, constrain environment/inherited handles and disallow sandbox bypass or shell substitution. |
| SDK-KERNEL | First positively control owned read/write files, loopback listener and self-exec, then verify real kernel denial. Bind actual platform, policy and binary identity and enforce resource/response supervision. Unimplemented platforms refuse execution; product third-party extensions remain disabled. |

### Image preview extension

- Actually decode PNG/JPEG/JFIF, first GIF frame, BMP, ICO/CUR, WebP, first TIFF page, TGA, ICNS and SVG; verify pixels and aspect ratio. On macOS verify HEIC/HEIF through system capabilities; other platforms reject them and unauthorized formats such as AVIF remain rejected.
- Use transparent images, 12 MP photos and JPEG orientation metadata to check thumbnail dimensions, transparency and orientation. Bad input, disguised types, huge dimensions and truncation cannot produce displayable success.
- SVG covers ordinary graphics, styles, internal use/gradients, text and metadata, rejecting file/network/relative references, CSS imports/escapes, embedded images, entities, scripts, events, animation, foreign elements and depth/element overages.
- Verify actual private-worker hard deadlines, cancellation, abnormal exit, oversized/trailing/size-mismatched responses. Run new-format regressions through complete archive, Rust admission, bridge and provider to QML, continuing to check password and invalid-selection cleanup.
- Default toolchain preparation can add qtimageformats to an existing SDK and doctor discovers missing plugins early. After build and Windows/Linux deployment, run actual helper format/minimal decoding checks; match source, notices, runtime attribution and packaging handoff to new Qt modules. Reuse valid proportion evidence when layout is unchanged, but actually observe newly added images.
