# Main-window functionality, components and interaction contract

Main-window controls deliver both appearance and behavior, with explicit normal, hover, pressed, keyboard focus, disabled, selected, busy and error states. HTML uses sample archives to verify interaction; Qt enables actions according to actual browsing, creation, extraction, [integrity checking](../engineering/archive-checking.md) and [explicit content preview](../engineering/content-preview.md) capabilities. HTML examples do not represent real background results.

Filenames, directory names, archive names and paths preferentially use the startup system default font, with platform fallback for missing glyphs. Fixed UI copy continues using the bundled subset. Directory tree and drag previews are measured with their respective text fonts; see the [visual standard](../visual/design-standard.md#界面字体与用户数据).

## Features and components

| Area | Shared / composed components | Interaction and capability |
| --- | --- | --- |
| System title bar | Platform decoration, draggable region | Native macOS traffic lights; Windows main window retains native frame, DWM shadow and system corners, hiding only the native title bar and placing minimize, maximize/restore and close to the right of settings. Drag and edge resize use system APIs; double-click blank title space toggles maximize. Linux uses native decoration; button areas do not initiate drag |
| Archive tabs | ArchiveTabs, ArchiveTab, icon buttons | New tabs by default, optional new windows; inactive tabs normal weight, shrink then scroll horizontally with scroll and tab-list entries. In-window drag and Ctrl+Shift+Page Up/Down reorder; closing retains background tasks |
| Toolbar | CommandButton, IconButton, SplitButton | Open, create, test, more, primary extraction and scope menu; mouse/keyboard share command entry |
| Search | SearchField, clear action, scope hint | Do not submit during IME composition; search current directory scope; clearing does not change directory; shortcut focuses |
| Archive summary | ArchiveSummary, format badge, statistics bar | Distinguish ready, reading, empty and read failure; unknown values are not zero |
| Directory navigation | DirectoryTree, DirectoryRow, Disclosure, connecting lines | Multi-level expansion, empty leaf arrow slots, independent current/focus, accessible full paths, collapsed-ancestor marker, directory context actions |
| File area header | PaneHeader, Breadcrumb, ViewToggle | Hierarchical path navigation, list/grid switch and sort menu; shared height and vertical center |
| List and grid | EntryView, SelectableEntry, checkbox, SortHeader | Single/multiple/range select, select all, clear, invert and arrow keys. Double-click/Enter enters directories or temporarily extracts regular files and opens with system association; properties remain available from context menu or details |
| File context menu | ContextMenu, CommandMenuItem, Submenu | Open applies only to the right-clicked item, as with double-click; retain existing multi-selection. Right-clicking an unselected item first selects it alone. Enable extraction, properties, copy name/path and directory navigation according to selection |
| Blank-area context menu | ContextMenu, RadioMenuItem, CheckboxMenuItem | Current-list selection, view, sorting and details pane; do not arbitrarily clear selection |
| Header context menu | Sort menu | Sort field, direction and restore archive order; sort all query results, not only visible entries |
| Properties / preview | InspectorPanel, PropertyRow, detail actions | File, multi-selection, archive and not-ready states; close/reopen, single-file actions, preview image edge aligned with table header |
| Status and tasks | StatusBar, StatusAction, task entry | Selection statistics, index status and background tasks; hiding a task window does not cancel it |
| Hints and operation windows | Tooltip, Notice, Toast, standalone platform windows | Full names and disabled reasons; Escape, focus return, cancellation and close follow each action's semantics |

List and grid share an empty state. A committed search with no results suggests another term. Without search, an archive with no records is identified as empty; otherwise the message describes only the current folder's visible contents and does not infer completeness of a limited directory index. Reading and errors retain actual states; unsubmitted search text does not change current result messages. Guidance before opening a file does not enumerate an outdated format list.

## Unified commands and menus

Buttons, shortcuts, context menus and dropdowns invoke the same commands and recheck current capability before execution. Menus freeze the object and selection scope when opened; close them when the archive or query changes or the object disappears, so an old entry's command cannot apply to a new archive.

Menus share four columns for icon/check slot, name, shortcut and submenu arrow, with separators between action groups. Include only actions relevant to context: directories offer entry and expansion, files offer details and multiple selection retains batch scope. Actions not currently executable remain disabled, with reasons available only to accessible descriptions; do not show explanation sublines or tooltips or substitute “coming soon” on click for disabling.

When font size changes, menu-label height follows actual font metrics and stays vertically centered in its row. The first presentation must meet a half-logical-pixel center-error limit without relying on a later layout pass to fix an old text slot.

Normal, checkbox, radio and submenu rows and separators are shared components. Sorting and view use radio items for current state; details panel uses a checkbox item. Dangerous actions never activate automatically by default; right-clicking and opening a menu perform no action.

## Menu input and lifecycle

- Down/up move between available items; Home/End go to first/last; Enter/Space activate, Right enters a submenu and Left returns to its parent.
- Escape closes one level at a time; closing the root returns focus to the triggering control or original entry. Tab closes the menu and continues normal page focus order. First-character name search is supported; IME composition does not trigger menu type search.
- Hover highlights without execution. Submenus open after a delay and retain the parent item while entering. Mouse and keyboard can follow each other without multiple lingering highlights or root menus.
- Outside clicks close without stealing the outside target's focus. Main-area scrolling, window resizing, entering operation windows or switching archives closes menus; scrolling a menu itself does not. HTML shared controls record actual existing-container scroll positions at opening; events completed before opening but delivered late do not close the menu, while subsequent position changes do.
- Avoid viewport bottom and right edges, opening left or upward as needed. Position in host coordinates while keeping logical dimensions, original source pixels, DPR and review scale distinct.
- Shift+F10/Menu key opens the current entry's context menu. Touch long press and mouse right-click share context and do not destroy multi-selection as a single click would.

## Control states and accessibility

| State | Appearance | Behavior |
| --- | --- | --- |
| Normal | Retain approved original border, radius, icon and baseline | State changes do not change control dimensions |
| Hover | Pale fill; primary button uses deeper primary color | Delayed hints do not cover targets; touch does not retain sticky hover |
| Pressed | More distinct fill than hover | No text/icon movement; cancelled press does not execute |
| Focus | No extra outline, thick border or glow | Retain Tab order and keyboard activation; outside click, Escape and Enter leave control focus |
| Disabled | Explicit disabled text/graphic colors while retaining outlines | No click or shortcut response; accessible description explains why |
| Selected | Background or checkmark clearly indicates state | Synchronize `aria-selected` / `aria-checked` / Qt Accessible |
| Busy | State and disabled scope follow task | Prevent duplicate submission; allow browsing during background work |
| Error | Adjacent copy explains problem and recovery | Color is not the sole information and success is not fabricated |

Shared icons come from `design/assets`. Controls implement text, checks, backgrounds, connecting lines and menu positioning; original-pixel slices are comparisons. HTML retains main-design source coordinates, converting new control parameters from logical pixels first; Qt uses the same logical specification.

## Acceptance

The count and action buttons at the right of the file area form a fixed-width group. List, grid, sort and visible details entries remain 28×28 logical pixels with 6.4 spacing; widening the window only gives breadcrumbs more space. The toolbar More menu contains only Settings; original order, extraction metadata preflight, select all and details use existing dedicated entries.

Welcome and archive pages share a minimum 32-logical-pixel status bar from statusbarHeight in layout-scale.json. Accessibility font growth uses actual task-button height and equal vertical padding, with no height change when switching pages. Both task buttons use task-list in a centered 16×16 logical icon slot from main.interaction.taskIconSize, unchanged by page or font scaling.

The archive status bar shows selection count; for a nonempty selection, append a separator and size only when the total is known. If a virtual directory or unknown-size file makes it unknown, show count alone without an extra separator or `—`. A known zero-byte size is still shown; welcome copy is unchanged.

The file-list checkbox right edge stays 12 logical pixels from the file icon slot; icon, filename and Name header align together. Accessibility font sizes change actual text metrics without enlarging this gap. Row checkboxes and header Select All retain the existing selection model.

All five columns can be resized by dragging the right boundary, with an 8-logical-pixel hit area; double-click restores that column's default. Name fills remaining space by default; other columns start from original design values. Both size columns grow by default to fit measured labels and sort arrows without clipping. User widths are shared across tabs and windows during the run and reset on exit; logical widths stay in memory, not settings or DPR. Name minimum is 160 and other columns 64, increasing proportionally for accessibility fonts; overflow scrolls horizontally. Headers and contents share one width source, and dragging does not trigger sorting. Ascending/descending markers precede the right-aligned Original Size and Compressed Size labels and follow other labels, always 6 logical pixels from displayed text. Active sort columns reserve arrow space on the appropriate side; manually narrowing elides labels first and arrows cannot overlap text or neighboring columns. Numeric headings remain right-aligned with contents.

File-type icons are generated for Qt and HTML selectors from the [extension hint table](../../specs/visual/file-types.json), covering office documents, spreadsheets, presentations, audio, video, archives, installers, fonts and disk images. Extensions are case-insensitive and unknown types use a generic document icon. This describes appearance only, not trusted contents or browsing, creation or extraction support for the archive format.

Use actual mouse and keyboard to check entries, menu edge avoidance, submenu round trips, selection retention, focus restoration, disabled protection, tab isolation, view switching and directory hierarchy. Capture real controls in light/dark themes, three platform decorations, normal/large fonts and scaling. Existing initial main-window title, toolbar, file-list and preview alignment must not regress due to menu work.

Record functional coverage, actual screenshots and unintegrated capabilities under the [general verification method](../qa/README.md), with conclusions in corresponding issues and evidence in build/ or CI artifacts. Static boards, browser behavior and Qt runtime verification are described separately and cannot replace one another.

Qt menus explicitly use `Popup.Item` with shared delegates so AppKit does not ignore custom styling; operation windows remain standalone top-level windows. `CommandTooltip` uses shared hint parameters and plain text so archive names are not interpreted as rich text; hide hints when menus gain focus. Shared JSON eight-digit colors use CSS RGBA; the QML generator converts to QColor ARGB. Verify color and opacity together.

Directory trees and the main toolbar's Open, New, Test, More and Extract hover states have no floating tooltips. Disabled menu actions likewise have no visible hint or tooltip; other buttons use shared delayed hints. Root and ordinary subdirectories list direct files and folders only, while search recursively covers current scope. Ordinary names show the final component; search shows full paths. List hover covers the whole row; grids retain fixed horizontal padding and hover/selected checkboxes. Menus use soft shadows on all sides and a 4-logical-pixel gap from invoking buttons. Retain separate interaction and scaling evidence under the [general method](../qa/README.md).

File-type sorting keeps directories first for both ascending and descending order, grouping by the directory flag in data. Files without extensions cannot share a grouping key with folders. Equal types retain stable identity order. Sort the complete query before splitting internal transfer batches. HTML files without folder set are treated as false; do not convert undefined to NaN for comparison.

## Selection, tasks and hints

Select All retains details focus on the current list's last item, with metadata matching individual selection. Clearing selection hides four properties and “Extract this file”; that button uses shared hover/pressed/focus states. Select All still applies to the complete current query without changing directory or physical-identity contracts.

The bottom “Tasks N” displays retained history count; active count is used only for hints and exit logic. Each task menu entry shows creation/extraction and number on the left and status color on the right; mouse or keyboard activation reopens the same task. Task menus share density, font size and scrolling boundaries with other menus. With no selection, selected-scope and preflight actions in the extraction scope menu are disabled without extra hint lines.

Floating hints appear only after the pointer remains continuously still within the same control for 1200 ms. Movement, leaving, pressing or opening a menu cancels display and old timers; moving between controls requires a full new wait, not rapid wake-up from a previous hint. Prefer above the control; use below when top space is insufficient to prevent edge constraints pushing the tooltip onto the control and causing hover oscillation. Qt shared CommandTooltip uses an explicit stationary timer, synchronized with HTML data-tooltip. Full list names do not use native browser title to bypass this policy. System native menu and tray hint timing is system-controlled and is not claimed to be controllable through QML. Timing does not scale with font size, DPR or density; visual geometry remains unchanged.

Archive tabs likewise show complete filenames in the shared stationary-delay tooltip as plain text; pressing, reordering or opening the tab menu or tab list hides it. Global task entries first close their menu then open only the task window, without switching archive tabs or reopening closed archive pages.

The start page uses a scrollable body fixed at the top and horizontally centered. Errors and warnings appear below format hints in red text and semantic icons without pushing upper content. Closing a tray-resident main window clears its tabs; ordinary restoration shows only the start page. Minimizing retains tabs; background tasks remain accessible globally.

See [file drag-and-drop and modification](../engineering/archive-transfers.md) for bidirectional file-list drops, current-directory addition confirmation and three-platform external copying.

In-archive moves use internal drag. Folder targets in list/grid and directory tree use existing selection fill; immediate feedback reuses tooltip geometry, at most four lines and constrained to the current client area. Selected directories, their descendants, original positions, search and encrypted/split targets have explicit rejection explanations without a new confirmation window. Name conflicts use the file conflict window and move tasks have a distinct name. See the [in-archive move design](../../design/hifi/sources/archive-move.html) and [drag-and-drop contract](../engineering/archive-transfers.md) for transactions and boundaries.
