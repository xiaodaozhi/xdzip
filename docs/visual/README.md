# xdzip visual specifications

Specification entries: [UI design standard](design-standard.md), [mandatory design workflow](design-workflow.md), and [density and unit contract](../../specs/visual/layout-scale.json). Reverification of main-window and menu proportions is recorded under the [general method](../qa/README.md).

Current originals are in [design/hifi](../../design/hifi/README.md); the interactive HTML delivery is in [design/html](../../design/html/README.md). Unadopted early proposals are not the current baseline. Do not rely on reorganized main history to find old designs; the current originals and manifest govern.

[tokens.json](../../specs/visual/tokens.json) holds theme and shared implementation parameters; visual machine-readable inputs reside in `specs/visual/`. Main-window geometry and fonts are generated from the independently frozen [qt-main-reference.json](../../specs/visual/qt-main-reference.json), based on high-fidelity originals and accepted HTML measurements, scaled by 1024/1548. The prior 1280-wide baseline was converted to normal density. Rounding, actual screenshots and unimplemented capabilities are recorded separately under the [verification method](../qa/README.md); implementation parameters cannot become their own design acceptance baseline.

## Research basis and adaptation

1. [Official Bandizip Windows UI](https://www.bandisoft.com/bandizip/screenshots/): retain a directory tree, file table, clear extraction entry and status bar; simplify the large colorful toolbar into text with fine-line icons and emphasize one primary action.
2. [Official Bandizip macOS UI](https://www.bandisoft.com/bandizip.mac/screenshots/): review the official file browsing demonstration and archive-creation panel; continue the browse-first flow before choosing contents and extraction location.
3. [Bandizip extraction instructions](https://www.bandisoft.com/bandizip/help/how-to-unzip/): opening a file does not automatically extract it. Tasks begin only after the user confirms destination, scope and conflict handling.
4. [Official Transmit UI](https://panic.com/transmit/): draw on file-manager information density, path navigation and restrained separators.
5. [Linear 2026 design refresh](https://linear.app/now/behind-the-latest-design-refresh): reference quieter sidebars, compact navigation, soft borders and neutral backgrounds that emphasize central content.

Research was conducted 2026-10-11 using public official screenshots, demonstrations and articles; these are not claimed to represent each product's latest installed version. Third-party research materials remain in a separate experimental workspace and are not included in this repository.

## Visual system

Use warm-gray backgrounds, cobalt primary actions and a graphite dark theme. Do not replace information hierarchy with large gradients or frosted backgrounds. Normal file lists retain a clear flat structure; shadows are reserved for overlays. The left answers “Where?”, the center “What is here?”, and the right “What is selected?”.

HTML and Qt share glyphs and metrics from [Xdzip Design Sans SC](../../design/assets/fonts/README.md); Qt uses same-source TTF and platform fallback for archive names outside the subset. Main list text is 13, headers and toolbar 12, pane titles 13 logical pixels, rounded to integer Qt font pixels. Filenames retain complete accessible names; visual truncation uses ellipsis and a full-name hover hint. File icons share an approximately 31.75-logical-pixel slot without per-type size compensation.

Initial main window is 1024 × 622, minimum 832 × 544 logical pixels. Sidebar and preview source measurements are approximately 193.16 / 257.98, rounded to Qt structural boundaries 193 / 258. Directory rows are approximately 43 and file rows 44.65. Pane titles are 46 high, table headers 34, search and primary button approximately 33.74. Directory tree B independently retains height 32 and icon slot 16. Below width 928 the preview collapses and opens via a button in a standalone native window. Settings offer 100%/125%/150% font size; historical 200% reads as 150%. Large fonts use real layout, scrolling and a standalone details window rather than global scaling to conceal overflow. Pane dragging and width persistence are not implemented.

## Main-window interaction

- Qt supports independent archive sessions, creating capsule tabs or main windows according to saved open mode; reopening activates an existing session. Pages retain path, selection, sort and scroll state; tasks continue after closing pages. See the [application session contract](../engineering/application-sessions.md). HTML multi-tab behavior validates design only.
- The primary button is “Extract all” without selection and “Extract selected” with selection; the dropdown always offers the other scope.
- The file table supports sorting, multi-selection, range selection and keyboard navigation. Double-click a folder to enter; double-click a regular file to extract a temporary copy and open with the system. The properties card offers explicitly requested limited content preview; see the [preview contract](../engineering/content-preview.md) for scope and safety. HTML sample photos are not real decoding results.
- Search recursively covers the current directory and descendants; root search covers all contents. Results show full in-archive paths, while ordinary directory lists show final names.
- Details can close and reopen. Preview requires asynchronous reads and resource limits; unsupported items show properties and extraction entry. Discard late preview results after task cancellation.
- The status bar shows selected count and original size, index state and reopenable task-history count. Active task count separately governs hints and exit protection. Mark an incomplete index as “Reading” without inventing a complete count.

## Extraction-panel interaction

The panel groups archive, destination, selection scope, conflict policy and completion behavior. The sample selection is 8.4 + 6.2 + 12.8 = 27.4 MB. The file list is a summary; larger lists offer “View all”.

Destination defaults to the last directory. “Create folder with archive name” updates final-path preview immediately to avoid redundant nesting. Show text for path permission, free-space and archive-read failures; checking is a separate state, and unconfirmed space is “Unknown”.

Conflicts default to Ask; Skip, automatic rename and Overwrite are also available. Clearly mark overwrite consequences. During execution show actual stage, processed file count, byte progress and speed when calculable; use indeterminate progress when totals are unknown. After cancellation show “Stopping” until the background confirms it has stopped.

## Component states and accessibility

| Component | Required states and feedback |
| --- | --- |
| Primary button | Normal, hover, pressed, keyboard focus, disabled and submitting; prevent duplicate start |
| File row | Normal, hover, selected, selected while window unfocused and keyboard focus; selection changes more than text color |
| Search | Empty, input, searching, no results and clear; Escape leaves focus while retaining text |
| Preview | No selection, loading, loaded, unsupported, error and cancelled |
| Task | Waiting, running, waiting for password/conflict decision, stopping, complete, partial failure and failure |
| Modal panel | Focus constraint, Escape closes an unsubmitted panel, focus returns to triggering button |

After Qt implementation verify WCAG AA body contrast for theme colors. Focus appearance and exit follow the current design standard without drawing additional focus rings. Provide screen-reader names and state announcements and respect system reduced-motion settings. Suggested ordinary transitions are 120–160 ms; do not continuously animate decorative file-list effects.

## Qt Quick implementation constraints

Establish unified Theme and dimension tokens; QML components must not scatter hardcoded theme colors. Suggested components include ArchiveTabs, ArchiveToolbar, ArchiveTree, ArchiveTable, InspectorPanel, ExtractDialog and TaskPanel. The file list uses a virtualizable model; preview, indexing and compression run in the background. QML presents state and user intent and does not decode formats.

macOS, Windows and Linux share content layout and interaction models; window controls, menus and shortcuts follow platform conventions. The macOS frame illustrated here does not require other platforms to copy traffic lights. Large real-time blur is not an implementation requirement; a static mask can reproduce background deemphasis in screenshots.

Current complete originals cover light archive browsing, extraction confirmation, creation, conflicts, tasks, results and settings. Dark icons are adapted, but a complete dark main window, plugin management and some exceptional pages remain outstanding; not all product pages are finalized.

## Platform window revision

Operation panels uniformly become [standalone platform windows](../ux/platform-windows.md). The close button in the content area, rounded panel and mask in older extraction artwork are replaced by a standalone title bar and platform decoration; colors, typography and content structure remain. Current file-type icons, list alignment and three-platform title bars have an interactive prototype as supplemental review material.

## Originals and shared asset library

[Complete originals](../../design/hifi/README.md) reside in `design/hifi/screens/`, including the main window, extraction confirmation and eight operation-state boards. See the [state contract](../ux/hifi-state-spec.md) for features and recovery. Preserve original bytes; originals and slices must not be modified to accommodate implementation.

The [shared asset library](../../design/assets/README.md) consolidates main-window and operation-state assets rather than splitting them by delivery batch. SVG is the sole graphic source for transparent runtime icons, embedded directly in HTML and rendered as same-source multi-resolution PNGs in Qt. Forty-five original-pixel comparison slices in `reference/` retain original artwork, crops and backgrounds; three HTML example photos are in `demo/`. One manifest identifies their purposes to prevent background-bearing slices being mistaken for transparent layers.

All transparent icons share a 24×24 canvas. PDF, text, code and Markdown icons share center and 22-pixel visible height while retaining distinct silhouette widths and internal styles. Folders use a centered horizontal visible silhouette of 22×approximately 18.86. This height constraint applies to runtime icons, not original slices. 1×/2×/3× PNGs are 24×24, 48×48 and 72×72; scale does not change logical size. HTML file lists use 48×48 source-space slots; Qt uses approximately 31.75×39.69 logical pixels after equivalent conversion. Remove per-type CSS enlargement, translation and stroke compensation.

File-list breadcrumbs and right preview/details titles share bar height, font size and vertical text center. HTML uses shared title-bar variables; Qt uses `PaneHeader` and `Theme.mainPaneTitleSize`. Titles remain above scrolling content and large fonts increase both pane heights together. Actual measurements are retained in execution artifacts under the [verification method](../qa/README.md).

HTML photo previews start directly below the pane title, with top aligned to the file-list header and no extra top gap. When search hints appear, align according to their actual occupied space. Keep existing image height and spacing below. Photo alignment rechecks cover two sample images, light/dark themes, three window decorations and search changes.

File-list original-size and compressed-size headers and values align right; type headers and content align left, with shared content padding per column. HTML retains original column widths and row heights: size-column right padding 26 and 22 source-space pixels, type-column left padding 28 source-space pixels. Reverification records text layout edges for all visible rows.

Record icon silhouette, original-pixel slice dimensions and runtime canvas dimensions separately. Folders need not be stretched into paper shapes; SVGs must not be distorted to fill slots. Make visual changes at the graphic source, review in the asset gallery, then synchronize both implementations.

## Directory tree B revision (2026-10-12)

Three formal boards define multi-level expansion, node interaction and navigation boundaries; see the [directory tree specification](../ux/directory-tree.md). Twenty-six original-pixel slices enter shared `assets/reference`; branch dots, filled status icons and segmented spinners use two-theme SVGs and three-resolution PNGs. Connecting lines, state backgrounds, focus and text are dynamically drawn from [component parameters](../../specs/visual/directory-tree.json); these do not automatically override global tokens. Boards are enlarged visual explanations; exact logical dimensions and source-pixel rectangles are recorded separately. HTML and Qt integrate recursive directories, connecting lines and hidden-current-branch markers; dedicated boundary icons/skeletons remain design resources and a complete dark board remains outstanding.

## Current main-design toolbar revision (2026-10-12)

Earlier generic control suggestions above no longer govern the main toolbar. Current original measurements are 51 source pixels high, 18 source-pixel font and 32 source-pixel icon, converted by 1024/1548 to approximately 33.74 height, 12 integer font and 21.17 icon canvas. Toolbar region height is 53. The previous 1280-wide 42.17/15/26.46 conversion has been replaced by normal density; ordinary operation windows retain their registered mappings.

Ordinary left/right toolbar actions and search/primary actions form groups. Large fonts may increase height and search may shrink to keep right-side actions fully visible at minimum window size. Disabled text does not receive group opacity; the primary button has its own disabled background. The extract primary button retains disabled divider and arrow appearance; actual extraction and scope menus follow runtime capability. Without a selection, selected scope and metadata preflight are disabled; integrity checking follows archive capability. Record revalidation under the [general verification method](../qa/README.md).

## Application branding and platform icons

System document icons for each format use the [file association icon specification](file-association-icons.md) and [high-fidelity review page](../../design/hifi/sources/file-association-icons.html). This family reuses brand geometry below with distinct format labels, accents and small-size simplifications as design assets for later per-format integration.

Branding uses four interwoven bands and an open U-shaped bottom, with geometry maintained in `scripts/brand_assets.py`. Standalone mark, application icon and ZIP document master are in `design/assets/brand/`. UI `app-mark`, `archive` and `archive-grey` identify the application, blue archive and gray archive, all generated from the same brand geometry. The application uses a blue rounded exterior and documents use paper with a folded corner: identity is shared while frames serve different roles.

The current mark retains a slender 2:3 aspect ratio and is not stretched to the old mark's width. Its base uses a rotationally symmetric continuous-curvature curve at all four corners, smoothly increasing curvature from zero on straight edges to the diagonal midpoint, avoiding a beveled middle. Lighting and base share one path; do not overlay strokes of different radii or open highlight lines. Document badges use the same outline definition.

This is an explicit new branding revision, not an indistinguishable reproduction of the ZIP zipper icon in old high-fidelity originals. Ten complete originals, 45 original-pixel icon references and three example photos retain original bytes. Newly generated `brand/identity-reference.png` is a review reference for the current branding direction, not a runtime asset. The main asset manifest records reference coverage and explicitly labels old-brand references as historical design roles.

Small UI icons retain 24×24 canvases. Platform icons export transparent PNGs at 16/24/32/48/64/128/256/512/1024 pixels. Sizes 16/24/32 simplify illegible text and large-image lighting while retaining the interwoven bands and U-shaped recognition structure. Application and ZIP document icons each provide ICO with seven sizes and ICNS with eleven density representations. Do not infinitely enlarge a 24-unit UI icon for system use or shrink large shadows into small-size noise.

System application icons are integrated into Qt windows, macOS app bundles, Windows executable resources and Linux desktop launchers/hicolor directories. ZIP document platform formats are used by installers and file associations; system open requests route to real browsing through unified routing. See the [installation and association contract](../engineering/installers-associations.md). Register only open candidates without changing user defaults. Record platform verification under the [general method](../qa/README.md).

Missing theme, success, cancel, conflict and progress graphics are completed from original-pixel references as distinct semantic assets. Large cancellation art uses a square stop symbol; system theme retains the complete window structure. HTML uses native radio inputs, while Qt uses IntentRadioButton to route mouse, keyboard and accessibility actions to one selection model; assets handle appearance only. Record generation checks, target-platform display and differences from originals separately; existing acceptance cannot replace this revision's results.

## Interactive control specification

[controls.json](../../specs/visual/controls.json) defines menu columns, row heights, radii, colors, focus and hint timing; [directory-tree.json](../../specs/visual/directory-tree.json) defines directory hierarchy. Both generate HTML control parameters and QML Theme; checks avoid manually duplicating dimensions and themes. Main-design logical/source-pixel ratios come from `display-spec.js`; the [component review page](../../design/html/components.html) uses 1:1 logical dimensions while the main canvas uses source coordinates. Feature lists and input contracts are in the [main-window component description](../ux/main-window-components.md); actual rendering and behavior go to corresponding issues and artifacts under the [verification method](../qa/README.md).
