# v0.1 interaction prototype and feature mapping

Status: interactive design-validation draft. Sources are in [design/html](../../design/html/README.md), with [index.html](../../design/html/index.html) as the entry. The Figma draft remains historical exploration; current feature and state refinement continues in the repository.

## Covered flows

| State | Entry and exit | Requirements / acceptance |
| --- | --- | --- |
| B01 Browse | Open a sample archive; double-click or Enter a directory, return by breadcrumb | P01/P02/P08, AC-01/02/08 |
| B02 Search and select | Checkbox, single, ⌘/Ctrl multi-select, Shift range and select all; search covers current directory and descendants, or the whole archive at root | P02/P03, AC-02/03 |
| E01 Confirm extraction | Primary button or scope menu; destination, same-name directory, scope and conflict policy are editable | P03/P07, AC-03/07 |
| E02 Conflict | Enter on a collision under ask policy; continue after skip, automatic rename or overwrite | P03/P07, AC-03/07 |
| T01 Task | Start → prepare → process → verify → result; return to browsing or request stop | P06/P15, AC-06/14 |
| R01 Result | Success, partial failure, cancellation, source change; retry unfinished items with reconfirmation | P03/P04/P15, AC-03/04/14 |
| C01 Create | Add/remove inputs, format/level/volumes/password and save location; confirm existing targets first | P04/P05, AC-04/05 |
| S01 Settings | Theme, font size, density, language and recent history; close returns to browsing | P08/P11, AC-08/11 |
| X01 Exceptions | Empty archive, no results, loading, corruption, unsupported features, permissions and task failure | P05/P07/P15, AC-05/07/14 |

“Design annotations” in the prototype header provides these IDs and current state rules. Permission and space outcomes are scenario-driven; successful prototype execution does not pass engineering acceptance P09/P10/P12.

## Interaction decisions

### Selection and search

- Each selection corresponds to a stable entry ID, not a sorted row number. Changing query/directory clears selection to avoid hidden-selection misunderstandings.
- Select all in the current list and extracting the whole archive are separate concepts. Checking a folder expands to deduplicated descendant files.
- Do not submit queries during IME composition; update after compositionend. Browser simulated-event checks do not replace native IME acceptance on macOS/Windows/Linux.
- Enter/double-click on a regular file extracts a temporary copy and opens it with the system association; on a directory it enters the directory. The properties entry remains. HTML demonstrates progress and automatic hiding only; it does not write files or call the system. See the [preview contract](../engineering/content-preview.md) for Qt's explicit content preview.

### Extraction and conflicts

- Display the final path live; append a same-name directory only once. Empty or demonstrably unwritable example paths block start; re-preflight after changing destination.
- Ask about conflicts by default; skip, automatic rename and overwrite are available. Adjacent copy explains overwrite consequences.
- Initial conflict-panel focus is Skip; overwrite is not the default Enter action. Applying to later conflicts is limited to this task; sample data creates one conflict only.
- Renames appear in completion summaries and file details. Skips are not failures; sample pre-existing files are not counted as completed writes.

### Tasks and results

- Returning to browsing does not cancel the task; the status bar can reopen it.
- Request cancel → stopping (repeat cancellation disabled) → cancelled summary. Extraction retains published items; archive creation has no intermediate publication.
- Disk full stops the task and retains successes. Retry selects only failed/unfinished scope and returns to plan confirmation.
- Source changes require reopening the archive rather than silently reusing the old index.
- A creation task counts one final archive; cancellation shows zero created archives and clearly preserves the old archive.

### Archive creation and capabilities

The prototype shows four ZIP compression levels, default 2 normal, without a separate algorithm control. TAR only packs; TAR.GZ uses GZIP 0/1/6/9. Both disable and clear password and volume parameters. Format changes update extensions. Empty input/name/save location disables Create. Prototype format choices are not the product's complete current capabilities: the product supports limited ZIP and 7z creation, passwords and volumes; RAR creation is unsupported. See [format backends](../engineering/native-codecs.md), [password interaction](password-extraction.md) and the [volume contract](../engineering/archive-opening.md) for actual boundaries.

### Visuals and adaptation

Retain warm-gray, cobalt-blue and graphite themes. Standard row height is 40 and compact 32. Dark-theme text accent is independent of button fill for small-text legibility. At narrow desktop widths, details become a closable overlay. Mobile review collapses the sidebar and secondary table columns; operation panels adapt to narrow screens and scroll.

Operation interfaces are standalone windows under the [platform window specification](platform-windows.md). HTML simulates them with dialog: decision windows constrain focus; progress, results and settings are nonmodal; closing attempts to restore focus. Closing a conflict window cancels the task rather than silently skipping; closing progress hides details only. Qt uses actual standalone top-level windows.

### Theme selection

The settings page organizes three native radio inputs, system, light and dark, using `fieldset#setting-theme` and the “Appearance theme” legend. Each theme card uses the shared `theme-system`, `theme-light` or `theme-dark` asset and associates its text label and input. Actual radio controls participate in card clicks, keyboard switching and focus. Selection uses the shared radio mark and synchronizes card border and preview appearance, rather than substituting a noninteractive screenshot.

This describes HTML design-validation behavior. Current settings are in-memory; follow-system resolves only when selected. Runtime system-theme changes, cross-launch persistence, and immediate persistence and failure handling in production Qt settings require separate verification.

## Verification and remaining boundaries

The browser verification script is scripts/check-html-design.py, checking real click flows, statistics, disabled conditions, format coupling, recovery, mobile overflow and offline single-file operation. Reports accompany the exported prototype package; browser checks use Chromium only.

The current main-window delivery includes two independent example archive sessions, list/grid, field sorting, recursive directory tree, hierarchical breadcrumbs, context/dropdown/submenus and button states. See the [component and interaction contract](main-window-components.md) and [verification method](../qa/README.md). `scripts/check-html-controls.py` checks these with real browser input; the [component review page](../../design/html/components.html) uses the same implementation as the main UI.

HTML verifies sample states only; it does not prove real task scheduling, authorization, path races, CRC or platform accessibility. Qt integrates real creation/extraction, drag-and-drop, system pickers, persisted settings and independent archive sessions in optional tabs or multiple windows. Actual scope of multiple formats, million-entry ordinary single-file ZIP indexing and limited preview is in [format backends](../engineering/native-codecs.md), [large indexing](../engineering/large-index.md) and [content preview](../engineering/content-preview.md), respectively; acceptance is recorded in corresponding issues. HTML settings retain demo state only within the page, not across launches, and do not establish native system-theme following or accessibility acceptance.

These limits do not change target capabilities in software design. “Complete” in the prototype describes a demo task state only and does not establish delivery of real compression or extraction.

## Current directory queries and card revisions

The root entry is “Root” and lists direct files and folders only. Search matches descendants in the current scope and shows complete in-archive paths. Ordinary lists/grids show only the final name; copy and navigation preserve complete paths. The right side of the breadcrumb shows current list row count. The directory-tree title is “Archive”, vertically centered with the breadcrumb; tree hover shows no floating tooltip.

List hover covers the whole row. Grids have approximately 16 logical pixels of horizontal padding; an 8-pixel top-left checkbox appears on hover and stays visible when selected. Menus invoked by buttons are 4 units from the button and use a soft shadow. The reopen-preview button is fixed at 28 square. Parameters are in layout-scale.json, directory-tree.json and controls.json; actual verification follows the [general method](../qa/README.md).

## Settings window

Settings retains original icon headings, theme preview cards, density, recent history and privacy explanation, adding text size and UI language. Theme and font previews can be cancelled; Save and Restore Defaults use drafts. HTML retains demo state only within the current page. Language selection shows configuration taking effect at Qt's next startup; neither the design review page nor sample archive contents serve as evidence of product translation coverage. Qt uses complete Chinese/English resources and cross-process persistence verification; see [settings and localization](settings-localization.md).
