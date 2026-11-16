# High-fidelity states and implementation contract

Corresponds to [operation state originals](../../design/hifi/README.md) and the [shared asset library](../../design/assets/README.md). State IDs continue prototype mappings, adding C02, T02, R02 and R03 without inventing a business protocol. Requirements P03/P04/P05/P06/P07/P08/P11/P15; acceptance still compares AC-03/04/05/06/07/08/11/14.

## Screens and task semantics

| State / artboard | Entry condition | Default focus and action | Close and recovery |
| --- | --- | --- | --- |
| C01 / 01 | Creation, no running task | First input button or filename; validate inputs, name, destination and capability before creating | Discard this window's draft; no write |
| E02 / 02 | Current TaskId/PromptId needs a name-conflict decision | Skip by default; rename retains extension; overwrite requires explicit activation; “apply to subsequent” unchecked | Close requests task cancellation, goes through T02, then result; never overwrite or skip by default |
| T01 / 03, 08 | Plan accepted and running event received | Return to browsing and cancel are ordinary buttons; no fabricated pause | Hide view only; task entry can reopen; background completion does not steal focus |
| T01 / integrity check | Same component as task progress, read-only archive and verification stages | Return to browsing, cancel check and real task result; HTML demo and Qt component preview remain separately marked | Closing does not cancel; real checks enter background queue via toolbar |
| R01 / 04 | Task has published and unfinished items | Retry selects only unfinished items and returns to planning confirmation | Closing view does not delete committed files |
| S01 / 05 | Open settings and select one of six tabs | All edits keyboard-accessible; changes take effect immediately after success and addresses must validate | Close retains applied settings; failure preserves previous values |
| R02 / 06 top | Unique successful terminal received | Extraction can open target folder by explicit choice; creation opens only supported results in-app, never automatically opens a system folder. Zero failures use neutral state | Close details without repeated completion notification |
| T02 / 06 middle | Cancellation accepted, cleanup incomplete | Repeated cancellation disabled; returning to browsing remains possible | Can hide; cannot show “Cancelled” prematurely |
| R03 / 06 bottom | Cancellation terminal received | Retain published items; if retry is allowed, replan unfinished scope | Closing does not roll back published items |
| C02 / 07 | Creation target already has an archive of the same name | Default returns to editing; use new name returns to C01 and suggests an unoccupied name; replace is not default | Return to C01 preserving inputs, format and path draft; do not implicitly overwrite |

“Use new name” proposes a name only; actual start still checks whether the destination exists. Filename checking is not final write authorization. Replacement creation must completely generate and verify a temporary archive before rechecking destination conditions for publication. Cancellation arriving after the linearized commit point displays completion according to the core result rather than rewriting success as cancellation.

## Creation capabilities and field errors

- ZIP's four levels map to Store/Deflate 1/6/9, default 2 normal. TAR/TAR.GZ can be created; switching formats synchronizes extensions and clears password and volume splitting. TAR disables level; TAR.GZ levels 0/1/2/3 map to GZIP 0/1/6/9. Format changes synchronize extension but cannot silently overwrite targets.
- Without input, with an empty name, invalid destination or missing permission, Create is disabled and the reason appears beside the field. Error text participates in screen reading rather than changing border color alone.
- The core handles duplicate inputs and directory expansion; UI shows selection summaries only. Image icons in creation are type indicators, not image preview.
- ZIP passwords and volumes are actually created under the WP-10 host contract; RAR and 7z creation remain disabled.

## Task data and exceptions

Example A has three files: 8.4 + 6.2 + 12.8 = 27.4 MB. With two successful, 14.6 MB is saved and the remaining PDF is 12.8 MB; success, skipped and unfinished counts must not be confused. Example B is an entire archive with 30 files and 77.5 MB; 46.5 MB is 60%, not the same moment as example A.

Percentages derive from known total bytes and actual metering. Unknown totals use indeterminate progress without invented remaining time. Re-estimate speed after a phase change; the illustrated 6.2 MB/s and approximately 5 seconds are only an example of a stable phase. File counts specify completed versus being processed and do not use the current entry ordinal as success count.

Disk full: stop and retain published items; replan after freeing space or changing destination. Source change: retry becomes reopening the archive; old indexes cannot be reused. Cancellation alone: unfinished uses amber/neutral, without reporting corruption. Error codes appear in details; ordinary screens do not show call stacks or internal bridge terms.

## Immediately applied settings

Theme, font size, density, language, opening mode and privacy switches persist and apply immediately after successful changes. Clearing recent history happens immediately without deleting original files. Failure retains old settings and shows a message. About and updates are on the settings About page; file associations require OS confirmation and reflect actual results. The network page saves validated proxy and mirror addresses. See the [settings contract](settings-localization.md).

The existing host's default output budgets are 8 GiB per file and 20 GiB per task; the current settings window does not allow editing quotas. If limits are added later, validate positive integers and overflow, ensure the per-file limit is no greater than the task limit, and apply only to new plans. Do not change immutable authorization for active tasks or bypass space checks.

## Window, layout and asset mapping

All operation windows follow the [platform window specification](platform-windows.md). Native macOS title bars do not use sliced traffic lights. Windows/Linux title button position, disabled states, system menus and movement are verified by the platform layer. Linux right-side controls in platform comparison images are not a hardcoded promise for every desktop environment.

Current dimensions follow [layout-scale.json](../../specs/visual/layout-scale.json), the [window registry](../../specs/visual/window-registry.json) and the [design standard](../visual/design-standard.md), replacing early oversized suggestions. Creation, extraction, conflict, progress and results share width 491; settings uses 612 as a registered purpose-specific difference. Title is 20, body 13, footer buttons 13 and supporting text 12. Height follows content and each window's cap; body scrolls. Operation windows with footers keep them fixed and reachable; settings has no footer. Settings offers 100%/125%/150% font sizes; do not derive component dimensions from global scaling or physical screenshot pixels.

| Use | Added assets | Shared / dynamic parts |
| --- | --- | --- |
| Creation title, add inputs | archive-add, image-file, remove | folder, file-pdf, new |
| Conflict, partial failure | document-warning, status-warning, info | file-pdf; comparison cards and text drawn by QML |
| Running progress | document-download, image-file, info | Progress bars and numbers drawn dynamically |
| Success, skip, cancel, stop | status-success, status-skipped, status-stopped, spinner, clock | Qt control rotates spinner |
| Settings | settings-tile, theme-system | Theme previews, switches and numeric inputs are components; settings is a small icon |

Assets reside under `design/assets/`. `reference/` retains original-pixel slices for appearance comparisons; `svg/` and `png/` are transparent runtime assets and must not be called original layers. All SVGs use 24×24 canvases; PNGs export at 24×24, 48×48 and 72×72, retaining the same logical display size. HTML file lists use a 48×48 source-space slot; Qt uses approximately 31.75×39.69 logical pixels. PDF, text, code and Markdown icons share a center and 22-pixel visible height while retaining distinct outline widths; folders are centered in the same canvas. Dark SVGs are asset adaptations; full dark behavior still requires real Qt and contrast checks.

## Directory tree addition

B01 browsing adds the [directory tree B specification](directory-tree.md) with three formal artboards and 26 shared slices. Node arrows depend on subdirectories; expansion, current directory and keyboard focus are independent. Loading, empty archives, root files, navigation limits, failures and hidden current paths are specified separately; the existing read-only domain contract is unchanged. HTML and Qt integrate core navigation, connecting lines and hidden-current-branch indicators; skeletons and dedicated boundary icons remain design assets. See the directory tree specification for integration scope.

## Changes and revalidation

Major states have been used in HTML and Qt. Subsequent changes still check original designs and revised parameters, control-to-function mapping, shared components and window contracts, and perform affected-platform verification. Designs or static checks alone cannot establish native functionality and accessibility. Record actual conclusions in the corresponding issue; historical acceptance is not reset by this list.
