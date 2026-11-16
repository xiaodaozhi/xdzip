# xdzip: 04 Qt Quick interface design

The project established its formal visual baseline on 2026-10-11; see the [high-fidelity design deliverables](../visual/README.md). Images express the visual target; tokens and component specifications govern implementation dimensions and colors. Image preview remains scheduled for M6 under the established roadmap.

## 1. Information architecture

The main window consists of a top command area, session tabs, a side directory tree, a central virtual file list, and a bottom task area. The first version permits multiple archive sessions; it can begin with a single tab and expose multiple tabs later without changing the model. The global task area is independent of tabs, so users can view tasks from other sessions.

| Page/component | Content | Primary actions |
|---|---|---|
| Welcome page | Open, create, optional recent items | Drop/select a file; recent items are local by default and can be disabled |
| Archive page | Title, format, index state, directory tree, file table | Search, sort, select multiple, extract, inspect details |
| Creation panel | Inputs, format, destination, compression options | Add/remove inputs, preflight, create |
| Extraction panel | Scope, destination, same-name directory, conflict and size summary | Review risks, start |
| Conflict dialog | Existing and new item information, current risk | Skip, rename, overwrite, apply to this task |
| Task drawer | State, actual throughput, completed count, cancellation | Expand results, retry failures through replanning |
| Details panel | Raw/display paths, sizes, format attributes, security notices | Copy safely displayed text |
| Settings/extensions page | Appearance, limits, privacy, version and capabilities | Manage extensions only when needed |

The application menu exposes primary actions. Errors appear first near the relevant task/session; modal dialogs are used only when a decision is required. Routine completion does not interrupt current input. Empty results, scanning, corruption, permission denial, and unsupported formats have distinct states instead of sharing one “Load failed” message.

## 2. Visual design and components

Use a replaceable Qt Quick Controls 2 control layer and shared tokens, without relying on incidental colors from a platform theme. The style is a clean desktop utility: neutral backgrounds, clear hierarchy, limited accent colors, thin separators, and restrained corner radii. Content density supports “Standard” and “Compact.”

| Token category | Initial specification |
|---|---|
| Color | background/surface/raised/text/secondary/border/accent/error/warning/success/focus, defined separately for light and dark themes |
| Spacing | 4/8/12/16/24/32 logical pixels |
| Row height | Standard 40, compact 32; increase for larger text without clipping |
| Corner radius | Controls 8, panels 12; independent operation windows follow the platform, and platform title bars follow the system |
| Font | Fixed UI text uses a bundled Noto Sans SC derived subset. Filenames, directory names, and paths prefer the system default font captured at startup; missing glyphs use platform fallback. Do not bundle a full CJK font package |
| Motion | 100–180 ms opacity/state transitions; disable when the system requests reduced motion |
| Accessibility | Ordinary body text targets at least 4.5:1 contrast; focus and selection are identifiable through non-color cues |

The planned component library includes `AppButton`, `SearchField`, `ArchiveTable`, `EntryDelegate`, `TaskCard`, `InlineMessage`, `PlanSummary`, `ConflictDialog`, and `PasswordDialog`. Controls have busy/disabled/focus/hover/error states. Corner radii, font sizes, and colors must not be hard-coded in scattered locations; icons come from one license-audited set.

## 3. C++ object boundaries

| QObject/model | Exposed to QML | Internal responsibility |
|---|---|---|
| `ApplicationController` | Open, create, session list, exit state | Qt file authorization, command correlation, application lifecycle |
| `ArchiveController` | Format, index state, queries, selection, capabilities | QueryGeneration, snapshot identity, request coalescing |
| `ArchiveListModel` | Roles such as name/size/type/modified/selected/status | Page cache, placeholder rows, accurate model-change signals |
| `DirectoryTreeModel` | Discovered directories and child state | Lazy loading, without building the entire tree at once |
| `TaskListModel` | Task summaries, progress, cancellability, waiting reasons | Merge events by TaskId+sequence |
| `SettingsController` | Theme, density, platform settings | Validate values and produce a core policy snapshot |

QML does not receive raw Rust pointers, access actual output files, or interpret protocol frames. C++ does not decide whether a ZIP entry is safe. `data()` does not make synchronous Rust queries or read files; cache misses return placeholders and coalesce asynchronous requests.

The list defaults to 128 rows per page and caches at most eight pages, for 1,024 rows. Core pages have a maximum size of 256. Prefetch keeps only one adjacent page; rapid scrolling cancels old prefetches. Cache keys include QueryId and revision. After receiving a page, batch `dataChanged` on the GUI thread. Use controlled reset for query changes and role-targeted updates for ordinary selection changes, avoiding a full delegate rebuild each time.

Prefer TableView row/column virtualization and measure delegate counts and destruction behavior. Do not convert the entire model into a QML array or Repeater for table styling. Elide long filenames and reveal the full content on demand; tooltips must not load every field either.

## 4. Events and interaction

Ordinary search input initially uses a 120 ms debounce. Do not submit intermediate pinyin strings while an IME is composing; trigger after commit. Increment generation whenever query conditions change, and validate session, QueryId, generation, and revision before a response enters the model. Coalesce repeated sort clicks and show the requested direction and loading feedback.

| Action | macOS | Windows/Linux | Scope |
|---|---|---|---|
| Open | ⌘O | Ctrl+O | Global |
| New | ⌘N | Ctrl+N | Global |
| Select all current results | ⌘A | Ctrl+A | File list; text inputs retain editing semantics |
| Search | ⌘F | Ctrl+F | Current archive |
| Toggle selection | ⌘ click | Ctrl click | File list |
| Contiguous range | Shift click/arrow keys | Same | Current query snapshot |
| Navigate | Arrow keys, Home/End, adapted to Mac keyboard mappings | Arrow keys, Home/End | List |
| Extract | Menu/toolbar and configurable shortcut | Same | Prepared plan |
| Cancel/close dialog | Escape | Escape | Innermost cancellable interaction; does not automatically cancel all tasks |

Do not assign conflicting system shortcuts; main-menu shortcuts use Qt standard keys and platform conventions. Enter in the list shows details or enters a directory; it does not directly execute an internal file. Double-clicking a file triggers preview only when restricted preview is explicitly enabled; otherwise, it shows details.

External files dragged in can open archives or be added as compression inputs. Dragging out involves temporary file creation and asynchronous drag-and-drop and is separately verified in v0.2; the MVP must not pretend to support it. URLs from browsers are not downloaded automatically.

## 5. Cross-platform experience

macOS uses the system menu bar, native file selection, standard title bar, and Dock behavior. The initial plan is to distribute signed and notarized packages directly; if the Mac App Store is adopted, file authorization and plugin policy are separate release configurations. Windows supports per-monitor DPI and Unicode paths without imposing Windows appearance on Mac. Linux handles Wayland/X11 and file-selection portal capabilities, with a clear usable path even when integration is unavailable.

Display a consistent “Cannot output this name,” with details explaining current filesystem restrictions. Do not concatenate system paths as POSIX strings; the adapter converts platform paths returned by Qt into authorization objects. Handle displayed path direction and bidirectional control characters safely; malicious names must not make file icons/text impersonate controls.

## 6. Accessibility and input

List rows provide readable names, types, sizes, selection states, and logical row numbers; unknown size is read as “Unknown.” Tasks provide their phase and actual progress. Announcements are rate-limited rather than reading a percentage every 100 ms. Dialogs have explicit default buttons; dangerous overwrites cannot be confirmed by pressing Enter by default.

Restore focus after closing a panel. All buttons, tabs, filters, and conflict actions are keyboard reachable; focus must not enter an invisible delegate. Verify VoiceOver, Narrator, and the target Linux screen reader through Qt accessibility interfaces rather than merely checking that properties are populated.

Acceptance covers Chinese pinyin/Wubi candidates, composing text, RTL names, emoji, 100%/125%/150% font sizes (200% is a historical stress scenario only), 125%/150% DPI, light/dark themes, and reduced motion. Programmatic Chinese search in a prototype does not count as IME acceptance.

## 7. Passwords and errors

Password fields disable autocomplete, clipboard suggestions, and logging. Clear the QML property promptly after submission; C++ transfers the secret to a short-lived secret object. Copies inside Qt/system input methods cannot all be guaranteed erased, so do not claim absolute zero residue. Passwords are not remembered persistently by default.

The core supplies error codes; Qt forms messages using translation catalogs. Plugins cannot directly supply rich-text error pages. Sanitized diagnostics may be copied: software version, format, error code, and anonymous TaskId, without passwords, content, or full paths.

## Independent operation windows

Extraction, creation, conflicts, tasks, results, and settings all use independent top-level windows rather than main-window overlays or macOS sheets. macOS uses native system buttons at the upper left. Windows/Linux may customize the title bar while retaining system window behavior. See the [platform window specification](../ux/platform-windows.md) for ownership, modality, close and cancel semantics, prototype boundaries, and actual-device acceptance.
