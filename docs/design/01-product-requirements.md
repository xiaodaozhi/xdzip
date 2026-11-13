# xdzip: 01 Product and requirements

## 1. Product positioning

xdzip is a desktop archive manager for macOS, Windows, and Linux. Its primary value is letting users inspect archives like ordinary file folders and reliably extract selected content or create archives. Users do not have to unpack the entire archive to disk first or understand plugins and codec implementations.

The priority experience is the macOS desktop, including Chinese input methods, ⌘ shortcuts, Retina, system file selection, and opening through Finder. All three platforms share information architecture, task behavior, and security semantics while following their own keyboard, window, and filesystem conventions.

## 2. Numbered requirements

| ID | Requirement | Acceptance method |
|---|---|---|
| P01 | Opening an archive only reads and browses it; it does not extract automatically | Opening a test archive creates no files in the destination or temporary preview directory |
| P02 | Directory navigation, search, stable sorting, single selection, and range-based multiple selection | A real metadata set of 100,000 entries can be paged through; selection identities remain correct after changing sort order |
| P03 | Selective and full extraction, conflict preflight, and result summary | Overwrite, skip, rename, failure, and cancellation are observable and match the actual disk state |
| P04 | Create ZIP, TAR, and TAR.GZ from files and directories | System tools or independent reference tools can read the content; cancellation does not damage an existing destination |
| P05 | Expose formats, encryption, split volumes, and other actions according to capabilities | Unsupported actions cannot be submitted, and the core also rejects forged requests |
| P06 | Bounded background tasks, a usable interface, and cancellation | Other sessions remain searchable during large tasks; cancellation results include committed and unfinished items |
| P07 | Protection against path escapes, links, special files, and archive bombs | Adversarial samples cannot escape the authorized destination or exceed actual output quotas |
| P08 | A modern, consistent interface operable by keyboard, IME, and assistive technologies | Three-platform acceptance checklist; focus, screen readers, high contrast, and 200% scaling are usable |
| P09 | Explicit low-resource budgets and reproducible measurements | Release benchmarks include runtime, process tree, fonts, GPU, and workload |
| P10 | Rust core isolated from the interface, with formats extensible according to the specification | Core tests run without Qt; independent format adapters can be integrated without changing QML |
| P11 | Privacy and password protection | No telemetry by default; logs contain no passwords or full user paths; secrets that can be cleared are cleared on exit |
| P12 | Auditable, reproducible, legally distributable releases | Dependency locks, SBOM, license inventory, signing, and install/uninstall verification are complete |
| P13 | Voluntary installation of third-party format extensions with explicit permissions and compatibility | Downloaded extensions are not executed by default; reject them if sandbox, protocol, signature, or authorization checks fail |
| P14 | Limited, explicit preview and temporary opening | Only user initiated; subject to size, type, output quota, and temporary-directory policy |
| P15 | Diagnosable, recoverable errors without false success | Error codes and summaries agree; power loss, full disks, and changed sources have separate tests |

## 3. Workflows

### Open and browse

A file association, drag-and-drop, or “Open” obtains a user-authorized source file handle. Determine the backend from file magic and format probing; the extension is only a hint. A session first displays the archive title and probing status, then incrementally displays content. ZIP can read its directory; streaming formats such as TAR display “Scanning,” and the total entry count may be unknown during the scan. A partial index must not masquerade as complete search results.

When a password is needed, explain whether it is required to read the directory or the content. If the user cancels password entry, retain usable session metadata or return an actionable error; do not silently try an empty password or retry indefinitely. Missing files, unsupported formats, corruption, and permission denial have distinct messages.

### Extract

The toolbar offers “Extract selected” and “Extract all.” Default to the selection when one exists and to all entries otherwise; **do not silently treat the current search results as the entire archive**. A separate “Extract search results” menu action can create an explicit snapshot from the search page, with its count and “current results” scope shown in the confirmation area.

The user chooses a destination directory and may select “Create a subdirectory with the same name.” The core builds a plan covering deduplication of recursive directory selections, path mapping, capability checks, estimated size, known conflicts, and risks. A full-extraction plan waits for an unfinished index to finish scanning. Large plans are built asynchronously, with progress and cancellation.

Existing files are not overwritten by default. A conflict pauses for a decision. The dialog offers skip, rename, overwrite, and an option to apply the decision to subsequent similar conflicts in the same task. Decisions are bound to the task and conflict ID; the task checks the disk again when actually committing. The completion summary shows successes, skips, failures, and items completed before cancellation, and offers “Show destination in system.”

### Create an archive

Add files/directories → choose format and destination → choose a compression level according to backend capabilities → preflight → write a temporary archive in the same directory in the background → verify/close → publish atomically. The default is ZIP/Deflate. Compression levels are labeled “Fast,” “Balanced,” and “Smaller,” with their corresponding backend parameters shown; the same level cannot promise identical results across formats. When the destination is inside an input tree, exclude both the destination and this operation's temporary file, and reject input that would recurse indefinitely.

### Preview and temporary opening

The current source provides user-initiated, limited [text/image preview](../engineering/content-preview.md), with formats, input, pixels, and actual decoding work restricted by allowlists and independent quotas. External opening first fully decodes content into a controlled temporary directory and explains that a system application will be invoked. Programs, scripts, macros, and HTML in an archive must not execute automatically. Executable files do not start automatically as a result of “preview.”

## 4. Scope and versions

The table preserves the original stage breakdown to explain the requirements' origins. It is not a list of current-version features or a release plan. For current support, see the root [README](../../README.md) and the relevant engineering contracts.

| Stage | Deliverables | Explicit follow-up items |
|---|---|---|
| v0.1 private test | Qt product skeleton; ZIP Store/Deflate browsing, creation, and extraction, AES-256 and numbered-volume creation; TAR/TAR.GZ; path security; task cancellation; basic keyboard and Chinese-language experience | Encrypted/split archive reading, 7z, RAR, external extensions, preview, in-archive editing |
| v0.2 public test | 7z capability verification; read-only RAR when conditions are met; text/image preview; external extension SDK trial; large-index optimization | Full third-party ecosystem, general recovery, writing arbitrary formats |
| v1.0 | Verified installers on three platforms; stable core behavior; performance budgets; eligible third-party extensions; licensing and security processes | Encryption and split-volume support must be listed separately for each format; the version number does not imply coverage |

The current implementation supports limited AES-256 ZIP and split-volume reading, 7z passwords, solid archives, and split volumes, as well as rebuilding and modifying eligible regular archives. Traditional ZIP encryption and modification of encrypted or split archives remain unsupported. See [archive opening](../engineering/archive-opening.md), [archive modification](../engineering/archive-transfers.md), and [format backends](../engineering/native-codecs.md) for capabilities and rejection conditions. Third-party extensions and choosing compatible backends remain designs and must not be described as implemented.

## 5. Quality acceptance

- Path security and output transactions are exit criteria for every stage; they cannot be deferred as “future hardening.”
- On the release reference Mac, verify the default font, IME, ⌘A/⌘O, drag-and-drop, Retina, and a real GPU. On Windows and Linux, separately verify platform shortcuts and directory authorization.
- Distinguish opening, scanning, querying, extraction, creation, and preview workloads; see [09](09-performance.md) for all performance budgets.
- Features without real-format tests or actual platform verification are marked unsupported or experimental in release notes.

## 6. Non-goals and product defaults

The first version does not aim to provide cloud synchronization, a network-drive protocol client, a virus-scanning service, disk-image mounting, or arbitrary archive repair. It does not scan user disks in the background or automatically upload error contents. Light/dark appearance follows the system by default, and basic interaction does not use paid components. Extension management does not appear in the main workflow every time an archive is opened.
