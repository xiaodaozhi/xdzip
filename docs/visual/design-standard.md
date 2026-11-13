# UI design standard

This standard governs UI creation, HTML design delivery and Qt implementation. Dimension conversions follow [layout-scale.json](../../specs/visual/layout-scale.json); colors, shared controls and directory trees follow [tokens.json](../../specs/visual/tokens.json), [controls.json](../../specs/visual/controls.json) and [directory-tree.json](../../specs/visual/directory-tree.json), respectively. Original measurements are in [qt-main-reference.json](../../specs/visual/qt-main-reference.json); do not rewrite originals backward from implementation results.

<a id="主动内容预览"></a>

## Explicit content preview

The [content states](../../design/hifi/sources/content-preview.html) retain existing properties-card width, height, radius, main-window density and text roles. Insets, gaps, icons and loading-indicator parameters are generated from main.content_preview in layout-scale.json. Content is read only after the user clicks shared ArchiveAction. Loading, password waiting, plain text, images and errors share the card; cancellation remains reachable. Text is selectable plain text. Images preserve aspect ratio; raster and limited static SVG share an image slot without introducing browser, HTML or external-link execution. SVG is rasterized in a restricted worker and its source is not handed to QML. Main-window and standalone property viewers reuse InspectorPanel; closing preview or changing entries clears it immediately. The password window retains P02 and shared footer, changing copy only for the preview task. Main-window and menu proportions still require simultaneous revalidation.

<a id="界面字体与用户数据"></a>

## Interface fonts and user data

At Qt startup, capture the system default font family before installing the bundled Noto Sans SC derivative subset for fixed UI text. Filenames, archive and directory names, paths and messages or fields containing them prefer the captured family, with Qt/platform fallback for missing glyphs. Composite messages containing filenames may use the system font throughout; static placeholders and action text retain the UI font. Display, directory-tree width and drag-preview measurement use the corresponding same font. Font sizes still follow logical dimensions and accessibility scaling; system DPI does not alter component logical dimensions. HTML review examples use the bundled subset; actual user-data glyphs and widths depend on runtime platform, while structural dimensions follow the same logical specification.

## Units and density

1. Deliver all dimensions in logical pixels: HTML CSS px and Qt device-independent pixels. Divide physical screenshot pixels by DPR before comparing to design values. Asset 1×/2×/3× only affects sampling clarity, not icon slots.
2. Treat the previous 1280-wide main-window baseline as 125% and convert to 100% using `1 / 1.25 = 0.8`. New default client area is 1024×622, minimum 832×544, preview-collapse threshold 928. Apply this density correction to title, toolbar, panes, file list, directory tree, status bar and their fonts.
3. Pop-up menus and hints use normal density rather than shrinking with the main window. Menu defaults are width 240, minimum row height 32, font 13 and icon slot 18. Target main-list font is approximately 12.57, rounded to 13 in Qt; menu/list font ratio must be 0.95–1.10. Check the ratio as well as individual parameters.
4. The original main client area 1548×940 is measurement coordinates. HTML main-canvas conversion is `1024 / 1548`; menu source CSS coordinates convert back to retain the logical dimensions above. Ordinary operation windows retain `1280 / 1548`. The extraction original uses a separate `819.2 / 1548` mapping and approximately 491 window width, an explicit density correction for a standalone entry previously 25% larger than the embedded entry. All entries share final logical sizes; component fonts cannot differ between embedded and standalone display.
5. Browser review scale, browser page zoom, system scaling and DPR must not enter component size parameters. Qt does not use `Item.scale`, downsized screenshots or an extra 1.25 multiplier to implement density correction; specification parameters generate real layout and integer pixelSize.
6. One-logical-pixel separators and focus strokes retain specified values; OS owns native window decoration. Keep fractional spatial dimensions; round structural boundaries and window sizes to nearest integer. Round Qt font sizes after density conversion and before accessibility scaling.

## Title-area revision

Main title-bar height is 48 logical pixels. Tabs are floating capsules height 32 and radius 16, with 8 above and below. Active tabs have complete borders and do not join the toolbar. Tab close, plus and settings share the title-bar vertical center. Larger fonts first increase capsule height while retaining equal top/bottom padding. This replaces bottom-attached tabs in the archived original.

Active tab weight is 600, inactive 400 without an active outline, using shared fill only on hover. Tabs divide available width between existing preferred width and minimum 112; upper and lower limits increase with larger text. Overflow uses a horizontal viewport without visible scrollbar, retaining New, bidirectional scrolling and tab-list entries without squeezing settings or system buttons. Wheel, trackpad, keyboard switching, menu activation and drag-edge scrolling share one bounded viewport. Active tabs remain fully visible after open, switch, close and resize; explicit user scrolling is not pulled back. In-window drag shows a floating capsule and insertion marker; release reorders by stable session identity, while Escape or input cancellation abandons the drag. Parameters come from titlebar in layout-scale.json, not screenshot pixels.

Tab close retains its square hit area and centered position, with circular hover/pressed backgrounds. Hide it for a single ordinary idle welcome page without error; retain existing visibility for loading, errors, multiple tabs and archive tabs. Hidden buttons cannot receive keyboard focus or appear as visible accessibility actions.

The welcome loading card's body scrolls; “Cancel loading” uses shared OperationFooterButton in a fixed lower-right action area. Loading is not extraction and does not reuse “Cancel extraction”. After cancellation show cancelling and disable duplicate submission; restore welcome focus when loading ends. Card logical width is 360 with 24 padding, retaining the translucent overlay and shared button dimensions without overriding footer appearance.

Modification waiting for an open archive reuses card parameters with layout-scale.json operation_overlay transparency. The original file page stays visible while duplicate actions are blocked. Title, description, progress accessible name and Cancel name the actual add or in-archive move; restore interaction after task cleanup and directory refresh. Accepted cancellation immediately closes invalid conflict and password prompts, while progress shows cancelling and follows the real terminal. Conflict windows bind a specific task and prompt token and close when the task changes or token expires.

Current [light main window](../../design/hifi/screens/main-capsule-light.png), [dark main window](../../design/hifi/screens/main-capsule-dark.png) and [multi-level directory design](../../design/hifi/screens/directory-tree-capsule.png) use this title area and come from the [editable source](../../design/hifi/sources/main.html). Old main and directory originals remain in gallery history with independent measurements, SHA-256 and slice provenance unchanged; current exports do not replace independent geometry acceptance.

## Component rules

### Settings-window category revision

Ownerless tray Settings and About reuse shared screen centering against the current display work area and complete native frame. Recenter after native creation and height adjustment rather than retaining a bottom-right notification-area pointer location. Settings dimensions, typography and density remain unchanged.

For Windows MSIX identity, About omits the update title, divider and entire update section; version information uses existing subsequent shared group spacing. Window dimensions, fonts and other content remain unchanged, with no channel-specific scaling or density.

Settings and About share a 612-logical-unit nonmodal window with preferred client height 532, constrained by available screen height. A one-unit divider below shared OperationHeader precedes sidebar navigation and an independently scrolling right page. Sidebar baseline width is 144, normal row height 40, icon slot 18 and icon 15 from the left. The first row touches the top and selection meets the right divider. Larger text widens the sidebar for the actual widest word and icon spacing; English wraps between words, and rows grow or scroll. Unselected hover uses shared controlHover distinct from sidebar in both themes. Page padding is 20 and group gap 16. Parameters follow operation.settings in layout-scale.json, not screenshots, DPR or review scale.

Pages are Appearance, General, Privacy, File Associations, Network and About. Settings has no footer or Save, Cancel or Done buttons. Successful changes apply immediately; failures retain applied values and show why. About retains version, license and user-initiated updates; all About entries open this page rather than a standalone About window. Theme cards, settings icons, brand and existing state assets are reused without regenerating unchanged slices.

Page bodies do not repeat sidebar titles or their separators; actual section titles and separators remain. About's primary update action shares the status row and aligns right: Cancel Check while checking; download, download-and-install or release page when available; Cancel Download during download; install or open download location after completion. Check and Cancel Check keep the same position and size, with progress below. Secondary download-location actions align right.

Every component specifies dimensions, font, icon slot, padding, alignment edges, hit area, states and data source. Default state alone is insufficient. Buttons cover normal, hover, pressed, focus, disabled and busy. Menus cover keyboard selection, submenus, checked, disabled, edge flipping and scrolling. Trees cover hierarchy, leaves, collapsed current paths, deep scrolling and long names. Lists cover header/content edge alignment, sorting, multi-selection, empty and unknown states.

Numeric columns align right on a common edge and type columns left. Breadcrumb and preview-title height, font and baseline match; preview top aligns with file-header top. Center icons in slots and align text by font metrics. Use shared assets rather than temporary slices from background-bearing screenshots.

The gap from file-list checkbox right edge to icon slot is 12 logical pixels, from fileSelectionGap in layout-scale.json. This tightens the original approximately 21.17 gap; file icon, filename and Name header move left together while retaining icon slot, icon/text gap and other column boundaries. Frozen originals and measurements remain unchanged.

Main-window normal-density correction and accessibility font scaling are separate steps. Current options are 100%/125%/150%; old 200% settings read as 150%. Larger fonts may increase heights, scroll or open standalone details; do not shrink text to counteract the selection. Tests directly injecting 200% represent historical stress only, not a current product option. Record reasons and verification scope for all visual exceptions.

## Acceptance gates

Compare HTML and Qt at the same logical width; crop only the original outer background and display at an explicit scale. Maximum geometry error is 1 logical pixel, text/icon vertical-center error 0.5, and aligned pane-top error 0.01. Screenshot reports specify commit, platform, renderer, browser zoom, review scale, DPR and logical window dimensions.

For optional HTML comparisons verify actual 100%/125% browser page zoom and DPR 1/1.25/2. Corresponding Qt measurements verify the same logical window, font and menu parameters under high DPR. Tools cannot pass CSS zoom off as real browser page zoom or offscreen screenshots as native desktop acceptance. Check counts are not a percentage of visual fidelity.

Follow the [UI design workflow](design-workflow.md). Do not mark full acceptance when specification, generated output, actual coordinates, screenshots or interaction evidence are missing.

## Main-window interaction revision

Main toolbar icon/text gap is fixed at 6; standalone icons are centered in content boxes. List/grid/sort buttons are fixed 28×28 rounded squares with 6.4 header gap; breadcrumbs absorb remaining space. Breadcrumb buttons are normally 28 high with vertical padding. Menus sit 4 units from invoking buttons; bottom menus open upward. Tree rows show no file counts; selected left edges are straight with full-height indicator, and child nodes have no four-sided outlines. Tree width uses actual text metrics of expanded nodes. Record revised interactions and measurements under the [verification method](../qa/README.md).

## Operation-window and task-state assets

### Shared icon header

Create, add, extract, task/result, conflict and settings windows share OperationHeader. About is in Settings and integrity checking inherits it via TaskWindow. The icon slot is 48×48, title 20, subtitle 13, horizontal padding 24, vertical padding 20, icon/text gap 16, title/subtitle gap 6 and normal-height baseline 96. Icons preserve aspect ratio and center without window-specific canvas/artwork overrides. Titles and subtitles are plain text using text/textSecondary in light/dark themes; font sizes change only with application accessibility settings. Long text or larger fonts grow the header using shared font metrics, without shrinking text or covering body/footer.

[operation.header](../../specs/visual/layout-scale.json) is the sole parameter source; standalone state designs are in [operation-headers.html](../../design/hifi/sources/operation-headers.html), with window mappings in [window-registry.json](../../specs/visual/window-registry.json). This replaces per-window icon slots, centered 24-unit About branding and window-specific header heights; historical images, hashes and measurements remain unchanged. Text-only password, preflight and exit/restart headers gain no invented decorative icons and retain safety interactions. Main-window and menu density remain separately measured.

Operation-window zipper illustrations share scripts/zipper_assets.py geometry and inventory: two interlocking rows of rounded teeth, low-contrast tape, separate slider and open pull tab. Teeth and metal use restrained highlights; scale proportionally inside assets and select colors for backgrounds without changing document silhouettes, icon slots or UI density. New consumers must register in ZIPPER_ICONS; asset checks verify light/dark completeness and compatible inline gradient IDs.

| Current icon | Zipper colors and internal layout |
| --- | --- |
| archive-add | Silver white; retain blue document and add badge |
| existing-archive | Silver white; proportionally enlarge and center for the taller document silhouette |
| archive-warning | Warm white metal and brown tape with amber warning document |

file-archive retains its simple flat file-category symbol with ochre horizontal teeth and rectangular pull tab, using the same symbol color in both themes; its source, SVG and PNG exports retain that style.

Shapes use a shared 24-unit vector canvas while headers retain shared 48-logical-pixel slots. Light/dark SVGs and 1×/2×/3× PNGs share one definition; archive-add and file-archive also retain 9× exports. Branding and association icons retain their brand definitions; historical artwork and slices remain unchanged.

Ordinary standalone windows retain 1280/1548 source-coordinate conversion and approximately 612 width. The progress original maps by 819.2/1548 to normal width 491, with content-adaptive client height up to 600; title 20, body 13 and shared footer fonts 13/12. Operation windows convert back to source coordinates for the current review context and cannot shrink again when entering progress from extraction. Extraction uses original hash, measurements and mapping frozen in [extract-reference.json](../../specs/visual/extract-reference.json): width 491, title 20, body 13, path field approximately 35 high and embedded Change button approximately 69×24. Qt uses real layout sizes without Item.scale or screenshot shrinking. HTML converts normal size back to current review source coordinates; standalone and embedded entries must yield the same final dimensions.

Extraction destination and conflict cards use uniform backgrounds; scope uses a pale header strip and white body. Light cards use #F4F6FA, borders #DFE5EF and path/dropdown borders #DCE2EC; dark equivalents are in tokens.json. Path icon, editable path and Change share one border. Checkboxes use shared selected graphics. All operation windows reuse xdzip's built-in dropdown with selection, disabled, keyboard navigation, Escape, focus return and edge flipping. Menus stay 4 logical pixels from callers.

All footer buttons use text 13, supporting text 12 and icon slot 16 logical pixels generated from layout-scale.json. Body scrolls separately and footer stays reachable. Fonts, colors and 1-logical-pixel strokes do not change by embedded/standalone entry. Record actual comparisons under the [verification method](../qa/README.md).

“Create folder with archive name” and destination-check feedback share 24-unit icon slots, 16-unit visible marks and 8-unit icon/text gaps. Optical centers align, text starts match, and each line centers text and marks vertically. Retain icon slots during checking/failure so text does not jump left. Right padding of Change equals its vertical padding, calculated as half the difference between path-field and actual button heights; retain this relationship for larger fonts.

Task-result discs use solid fill and white semantic marks from [status-icons.json](../../specs/visual/status-icons.json), with 24-unit canvas and visible diameter 20, without gradient, highlight, thin border or shadow. Export light/dark SVG and 1×/2×/3× PNG together. Existing originals and slices retain historical pixels rather than being rewritten in the new style. Record actual verification under the [general method](../qa/README.md).

## Root and file-item interaction

“Archive” at the top of the directory tree shares vertical center with the breadcrumb title. Normal directory browsing shows direct files and folders only; the tree entry is “Root”. Full archive paths remain in data, ordinary lists show final names and search results show full paths. Tree hover has no floating hint; full paths remain accessible descriptions. Parent vertical-line center X aligns with the parent's expansion-arrow center X.

List hover covers all columns across the row, with selection fill taking precedence. Grid horizontal padding is 16, card slot 144×124, slot gap 12 and icon slot 40. Checkboxes sit 8×8 from the card's upper left, appearing on hover and remaining visible when selected. Accessibility fonts grow slots according to real metrics. The reopen-preview button is also 28×28. Menu shadow radius is 12 with vertical offset 4; Qt uses a shared nine-slice PNG compatible with software/GPU rendering, HTML uses CSS blur from the same parameters. Verify body and shadow appear together on the first complete menu frame and check actual pixels in light/dark states. Dark split-button dividers use the dark border color.

ZIP creation uses the C01 original's three-part structure and white inner panels: normal width 491, title 20, body 13, supporting text 12, fields/body buttons height 28, input rows 38, footer buttons height 32 and font 13. Client area maxes at 736; smaller desktops scroll only the body and keep footer reachable. Qt and HTML use the same logical parameters. TAR/TAR.GZ use existing format segments; TAR disables level, both disable password and volumes, and switching clears inapplicable options and synchronizes extensions. Directory totals await background scanning; sample values do not replace real preflight. Record actual function and comparison under the [verification method](../qa/README.md).

## Shared operation controls and creation-window revision

[create-reference.json](../../specs/visual/create-reference.json) pins the C01 original hash, control crop coordinates and current revision. Operation buttons, inputs, dropdowns and footer buttons share radius 5, light border #DCE2EC, dark border #485469 and normal neutral button fill #FAFBFD. Body actions are 28 high and footer actions 32; footer text is 13 and support text 12. Individual windows no longer override shape, stroke, fonts, colors or density. Dangerous actions retain semantic danger colors; focus adds no extra stroke.

Settings density, text size, language and opening-mode dropdowns share logical height: at least body-button height, or text height plus 8 when needed to avoid clipping. Editable selections use existing accent background and accentText foreground in both themes. Main search, operation fields, editable dropdowns and HTML examples follow this relationship without changing unselected or placeholder text.

Qt uses OperationButton, OperationFooterButton, OperationFrame, OperationSelect, OperationRemoveButton and OperationSegmentedGroup. Windows provide only copy, data, state and actions; shared components own appearance and behavior. HTML uses the same generated parameters, shared styles and segmented-option generator. Dropdown icon slot is 18, with 6 from slot right to control right. HTML draws dropdown strokes with inset outline to prevent browsers flooring fractional CSS borders and shifting icons.

The creation list uses a circular minus from shared assets with 28 hit area and 24 icon slot; file size aligns right. Compression format is one continuous segmented radio group, equal-width, zero gap and one outer border; disabled formats accept neither mouse nor keyboard selection. File summaries read regular-file metadata in the background; the three original same-named fixtures are actually 8.4, 6.2 and 12.8 MB and display “3 files, 27.4 MB”. Directory totals remain unknown and the input tree is frozen again when starting.

Separate information with commas, periods, line breaks or explicit labels, not middle dots. This applies to current UI and maintenance documents, not existing high-fidelity bitmaps. Historical screenshots and captured JSON do not enter the current source tree. scripts/check-operation-components.py enforces shared controls; record actual measurements, interactions and limitations under the [verification method](../qa/README.md).

## Results and advanced creation options

Result windows are 491 wide, with client height calculated from actual body and footer up to 600; title 20, subtitle 14, card title 15, body 13, support 12 and result numbers 28. R01 keeps one left-aligned header. Statistic labels sit above numbers; unfinished cards and destination use uniform pale outer and white inner panels. Cards have radius 6, padding 12 and gap 10; detail rows are at least 56 high. Detail expansion mark, title and record count share vertical center. See result-reference.json. Footer retains global shared parameters without separate font or radius.

The creation window removes a separate algorithm row; four levels default to 2 normal. Volume presets, custom input and unit selector use OperationSelect's border and arrow slot. Empty or invalid custom input disables creation rather than substituting defaults. Cover password-enable checkbox and password-field disabled, input, error and clear states. Advanced options retain C01 fields height 28, body 13 and support 12. Additional controls grow scrollable body only; do not enlarge UI to 125%. New high-fidelity originals are design/hifi/screens/create-options.png and result-details.png, with slices under shared reference/; runtime controls use existing transparent icons.

Screenshots, measurements and comparison reports stay in ignored build/ or dist/ or short-lived CI artifacts. The repository retains only design originals, assets, specification parameters and check code. Deleted historical implementation screenshots cannot serve as test inputs or design authority.

## Integrity checking and system windows

Integrity checks retain task-progress width 491, adaptive client height up to 600, title 20, body 13, shared footer and normal density. The high-fidelity original is screens/integrity-progress.png, with frozen reference in integrity-reference.json. Keep the same layout for header, object path, gradient progress, bytes/files read, three metrics, current file, explanation and footer; replace only operation name, blue document-check illustration, target, average read, read-and-verify and cancel check. Do not use a separate browser-native progress bar or differently sized check card.

HTML reuses taskProgressBody, taskProgressFooter and progress styles; Qt IntegrityWindow inherits TaskWindow. Checks are read-only and terminals do not show saved-output statistics or open-output-folder actions. Component development preview must clearly state no actual verification occurs; the real toolbar command follows the [checking contract](../engineering/archive-checking.md). Light/dark, unknown totals, long paths, accessibility fonts, close and cancel use shared contracts.

Fixed Qt row heights must also be the layout row-slot heights; do not add extra font leading into fixed slots. Percentage row slot is 24 and byte/file-count slot 18. Path-card title and metrics retain prescribed row heights; multiline metrics grow by actual line count. At default font sizes all three progress-window body descriptions display fully; small screens or larger text can scroll the body while footer stays fixed and reachable.

Each main window currently owns one ExtractWindow, CreateWindow and TaskWindow, all standalone OS windows. Extraction, creation and real integrity-check progress/results reuse TaskWindow; selecting another task updates the same window rather than creating one per state. Component preview separately uses IntegrityWindow and performs no real check. See the [task-window contract](../ux/task-windows.md).

## 2026-10-15 operation and selection revision

This section replaces historical progress, result and name-conflict window dimensions. Original PNGs, hashes and crop references remain; screenshots are not scaled to change dimensions. See [operation windows and button states](../../design/hifi/operation-revisions.html).

- Extract, create, progress, result and file-exists windows share width 491 logical pixels. Task body plus footer determines height up to 600; details lists are at most 168 high and collapsing restores actual content height. Conflict body uses 0.8 normal density, title 20, body 13, support 12, max height 560 and shared fixed reachable footer. Larger fonts grow content or scroll.
- Compression formats form a continuous segmented radio group. Only the first option keeps two left rounded corners and last keeps two right rounded corners; all interior corners are square in selected, hover, pressed and focus states.
- Shared tokens distinguish all blue-button normal, hover, pressed and disabled states. Pressed uses accentPressed and disabled uses accentDisabled/accentDisabledText, not group opacity or gray-blue mixing; keyboard and mouse press share the state.
- Clicking outside, Escape, Return or numpad Enter leaves extraction destination editing focus without starting extraction. First Escape retains the window; subsequent Escape closes under window rules. Scope title and secondary count occupy separate columns while the row retains one radio hit area.
- Select All and individual selection share focused-details semantics: Select All focuses the current list's last item and clearing it clears details. With no selection hide four properties and preview extraction action. Preview extract retains hover, pressed, focus and disabled.
- Status-bar task count is reopenable history; active count separately governs hints and exit barrier. Completion cannot display zero tasks. Task menus reuse width 240, font 13 and minimum row height 32, showing operation icon, filename, sequence tag and status tag without visible type prefixes. Creation, extraction, checking, addition, moving and export use shared icons. Sequence and status use CommandMenuTag with controls.json padding/radius; sequence is neutral and status semantic. Filenames elide to reserve tags; accessible names retain operation, number, filename and state. History holds at most 64; oversized menus scroll and keyboard navigation/activation reopen matching tasks.

Welcome and archive pages share task-button internal layout and the 14-logical-pixel right inset in layout-scale.json. Button, icon, disclosure and menu anchor remain horizontally aligned when switching pages, independently of font scale and DPR.

Tag text width and height bind directly to current font implicit metrics, with positions computed from tag and text dimensions so the first rendered frame remains centered after menu opening or font change. First-frame checks verify text size, shared insets and centers without waiting for later layout correction. Unavailable menu actions remain disabled without visible reason lines, hints or tooltips; exact reasons are accessible descriptions only. “Extract selected” and “Extraction metadata preflight” follow this rule without selection. Real operation and safety errors still appear explicitly in the task or field.

Main-window format badges display content-detected ZIP, TAR, TAR.GZ, 7z or RAR. Minimum width is 32 with 6 logical pixels horizontal padding per side; width grows with actual text metrics. Do not put longer names in a fixed ZIP-width badge. Main-window/menu font ratios still meet existing gates. C01 adds 7z to the existing segmented group, with four formats equally dividing existing width; minimum group width is four times measured TAR.GZ text plus track padding. With insufficient translation space at larger fonts, left option labels wrap rather than shrinking fonts or widening the window. Solid and filename encryption use OperationCheck; the scrolling body holds new content and footer stays reachable.

## 2026-10-15 focus and system title-bar revision

All product windows remove focus-specific outlines, halos, thickening and color-changing strokes. Normal borders, hover, pressed, disabled, checked and current menu-item backgrounds remain. Shared focusWidth is 0 and is not adjusted by physical screenshot pixels. This replaces historical original/specification focus-stroke requirements.

Outside clicks first leave the old control's focus while executing the target normally. First Escape leaves the current control; subsequent Escape closes under window rules. Return and numpad Enter first perform existing control submission/activation, then leave focus. Pop-up menus retain arrow navigation and level-by-level close; final close leaves trigger focus. Tab order, shortcuts and accessibility semantics remain, as does appropriate focus restoration after programmatic window opening. Qt's unified policy applies only to owned Quick windows; system file pickers retain platform behavior.

Window titles show only task or archive names, removing the “ — xdzip” suffix. Without an archive the main title is xdzip. macOS traffic-light centers align with the existing 48-high title bar; realign after AppKit resize, activation, layout or exiting fullscreen. Verification measures actual layout directly without correcting it before measurement.

## 2026-10-15 preflight and list-boundary revision

Extraction metadata preflight uses [preflight-reference.json](../../specs/visual/preflight-reference.json): width 491, title 20, body 13, support 12, card radius 6 and body padding 20. Height follows actual contents up to 600; body scrolls independently and footer uses shared OperationFooter and OperationFooterButton. Native title is “Extraction preflight”; conclusion title reflects real report state. Standalone design entry is [P01](../../design/hifi/sources/preflight.html).

Main-list separators occupy 1 logical unit, with color blocks leaving 1 above and 2 below so visible gaps from previous block to line and line to next block are each 1. Preserve row height, font and menu density. Preview show/close directly anchor to the title area's right edge with identical padding, avoiding RowLayout allocation offsets. HTML synchronizes blocks and boundaries without modifying historical main-window originals.

2026-10-15 preflight explanation revision: move “This report checks metadata only…” to the end of the scrollable body in support font 12. Footer contains only Return, with the same height as a shared action-only footer at that font, independent of report length.

Main-toolbar Open, New, Test, More, Extract and extraction-menu triggers have no hover tooltip; text and accessible names remain, while needed hints elsewhere use shared controls. During task admission the Start Extraction button retains normal blue appearance; intent handling and controller block duplicates, and errors remain in confirmation. This revises interaction only, not main-window 0.8/menu 1 size parameters.

Chinese UI follows the [Chinese terminology standard](../ux/chinese-terminology.md) and JSON: container “压缩包”, size property “压缩大小”, scopes “全部内容” and “所选文件和文件夹”. See [terminology states](../../design/hifi/terminology.html). Historical images and text-ink coordinates remain; copy revisions do not adjust geometry tolerances.

<a id="2026-10-15-操作控件描边校正"></a>

## 2026-10-15 operation-control stroke correction

C01 inputs and neutral buttons use light-gray borders; the former shared light #C9D1E0 was too dark and is now #DCE2EC. Original straight-edge sampling regions and old value remain in create-reference.json without replacing bitmaps or hashes. Qt and HTML generate from tokens.json for create, extract, settings, task, result, preflight, conflict and exit windows. Unselected settings theme cards use this role too; selected blue remains. Dark #485469, card separators, menus and error/danger states retain their roles. Stroke remains 1 logical pixel without changing density, radius, focus or window size.

Segmented groups use outer radius 5 and inner fill radius 4, inset by only one 1-logical-pixel border. Qt end segments use antialiased ordinary rounded rectangles extended inward and clipped to square seams, avoiding per-corner curve differences between software and GPU backends. HTML no longer adds extra padding within the border. Check actual curve pixels for selected, hover and pressed ends; internal seams remain square.

## 2026-10-16 loading and creation completion revision

Welcome-page body uses shared parameters from the [loading overlay contract](../ux/welcome-opening.md) without overriding the title bar. Unknown totals use an active progress bar and real phase text, not simulated percentages. Operation-field placeholders consistently use textSecondary; dark password hints must not inherit light system defaults. Successful creation automatically opens supported results in-app only and does not show the system directory; result label is “Saved”.

## 2026-10-16 Windows main-window revision

The Windows main window hides the extra system title bar and displays minimize, maximize/restore and close in that order to the right of existing Settings. Normal title area remains 48 logical pixels; each button is 46 wide and full height, with 18-unit shared icon slot. Restore reuses maximization artwork offset by 3. Colors derive from tokens.json captionCloseHover/captionClosePressed and existing text, canvas and neutral-button roles. The old Settings right inset moves to the left of the window-control group. Parameters are in layout-scale.json and do not change with DPR or screenshot size.

The Ubuntu X11 main window integrates controls into the same title area, with normal radius 10, three 36-unit control slots, 28-unit circular backgrounds, 6-unit right inset and 6-unit resize edges from layout-scale.json. Transparent window backgrounds and antialiased title/status corners preserve edge alpha gradients. Blank-title dragging, double-click and eight-direction resize use the window system; maximized/fullscreen states have square corners. Operation windows keep system decorations; window-manager shadows and capabilities require separate native checks. See the [Ubuntu title design](../../design/hifi/sources/main.html?platform=linux).

Blank title dragging calls startSystemMove and double-click toggles maximize/restore. Windows retains WS_CAPTION/WS_THICKFRAME while incorporating only the native title-bar area into the client area. Native hit testing handles edges and corners; maximization uses system work area and native resize margins. DWM supplies system borders, shadows and supported Windows 11 corners, without QML rounded-rectangle or shadow substitutes. Caption buttons retain keyboard focus and accessible names; close follows existing task decisions and asynchronous exit barriers. Operation windows retain system decoration and macOS native traffic lights remain unchanged. The HTML Windows original synchronizes button slots but browser decoration does not execute OS window operations.

Qt automation simultaneously checks title area, button geometry, fonts and menu/list ratio; DPR 1/1.25/2 retains logical dimensions. Native Windows launch, maximize/restore and exit are executed separately. Offscreen results do not constitute complete manual Windows drag/snap, system scaling, IME or accessibility acceptance.

The Windows native frame retains system caption and resizing, removing FramelessWindowHint and QML edge-resize layers. WindowsFrame uses public Qt and Win32 interfaces without synchronizing Qt private client margins. Initial binding to a normal window preserves requested logical client area; later border reapplication does not repeat initial compensation. For ordinary PMv2, PassThrough and no local scale override, cross-screen outer size negotiates pending client adjustment and target-DPI border; other configurations return to Qt. Native min/max tracking sizes use current HWND DPI and the same border metrics, not Qt screen scale that may not have updated during cross-screen drag. Only tracking dimensions are handled; system maximization position and size remain. With a global scale override, tracking constraints return to Qt. DWM extension and restoration retain one native device pixel at the top; native resize metrics come from system metrics for window DPI. These are Win32 boundaries, not QML logical sizes. Dark frames update with theme and are reapplied after DPI, system theme and composition changes. Qt upgrades rerun relevant size and state regressions; physical dual-monitor dragging and native visuals are checked as needed, with unperformed limits recorded. Unsupported corner properties or composition fall back to system capability; do not claim identical decoration across versions.

Regressions check native window style, DWM nonclient rendering, queryable corner preferences and visible border, actual client dimensions, maximize/restore, minimize and close. contentItem screenshots cannot establish external frame and shadow; desktop-composition screenshots and state records are saved separately to build/, not replaced by internal canvas screenshots.

Fullscreen uses system borderless client-area calculations without the ordinary one-native-pixel top border or edge-resize hit area. Otherwise Qt may misidentify the state as normal after minimize/restore because the client area does not cover the screen. Normal and maximized windows retain their system border, corner capability and shadow rules; these Win32 parameters do not enter QML logical sizes.

Historical restart confirmation retains shared-window rules; settings language changes now apply immediately and do not trigger it. An empty creation input list remains a blank list box without instructional placeholder; Add and the actual summary below remain.

## Native desktop menus

Soft in-app menu shadows are generated as shared nine-slice PNG from menuShadowAsset in controls.json and appear with menu body on the first frame, without generating blurred Canvas pixels at popup time. Light/dark and 100%/125%/150% accessibility fonts have separate assets; 200% remains a test asset. Shadow radius and offset do not change with DPR.

The macOS menu bar and tray menus on all platforms are natively drawn, with system-determined fonts, row heights, selection and accessibility. They do not use in-app popup width 240 or row height 32. Verify design and behavior under the [desktop integration specification](../ux/desktop-integration.md); main-window density and title parameters remain unchanged.

## 2026-10-19 confirmation-window normal density

Exit, stop-task and historical restart confirmations share [operation.confirmation](../../specs/visual/layout-scale.json): client width 491, title 20, body 13, content padding 24, paragraph gap 12, maximum height 600 bounded by available screen. Height follows actual body and shared footer. Do not use fixed tiers such as 270/540 or old default operation-window 612/25/16. Body scrolls and footer stays fixed. Three buttons may wrap their text inside buttons and increase shared row height at English 150%, without shrinking fonts, overflowing or becoming a vertical stack. While stopping, preserve geometry policy and disable decisions.

[Standalone states](../../design/hifi/sources/confirmations.html) and HTML quit/restart entries are synchronized. SettingsWindow width 612 and standalone properties viewer width 360 are registered purpose-specific differences, not inconsistent sizes for the same confirmation type. About is merged into settings; no standalone AboutWindow is registered. See [window-registry.json](../../specs/visual/window-registry.json) for complete list and exceptions.

## Password extraction requests

PasswordWindow follows [password-reference.json](../../specs/visual/password-reference.json) and the [P02 standalone design](../../design/hifi/sources/password-extraction.html): width 491, max height 600, body padding 24, title 20, body 13 and support 12. Native title bars remain platform-drawn without changing the main-window title bar. The body scrolls and shared footer stays fixed; inputs reuse OperationField and OperationFrame, while show/hide and footer actions use shared buttons. Normal, hover, pressed, disabled, pending, error retry and light/dark use existing roles; no new outline or footer override. Password cleanup, request identity, cancellation and empty-password rules are in the [interaction contract](../ux/password-extraction.md).

The start-page body begins a fixed 28 logical pixels below the welcome area top, centered horizontally only, not vertically by body height. Error feedback follows format hints without changing headline, description, main action or drop-area coordinates; long messages grow and scroll naturally. Feedback uses shared danger text and an 18-logical-pixel status-warning icon with 8 gap and 10 top spacing; accessible role is AlertMessage and the icon is not redundantly announced. Parameters are in welcome-reference.json.

## Archive addition confirmation

UpdateWindow uses C01 [creation artwork](../../design/hifi/screens/create.png) as independent reference, with logical width 491, title 20, body 13, support 12 and shared footer. Below the icon header are three pale cards, Add Files, Add Location and Add Options, each with a white inner panel. Horizontal content padding is 18, card padding 13, inner-panel padding 10 and card gap 10. Target archive and in-archive directory use read-only fields; conflict policy uses shared dropdown. Height adapts to body plus footer up to 736 and available screen height; body scrolls independently and footer stays reachable.

Creation and addition share ArchiveInputPanel, using original C01 file icons, names, right-aligned sizes and circular minus while preserving existing creation control identity. The input list uses a virtualized view, empty/row baseline 38 and viewport max 148. Larger fonts grow rows by actual metrics without changing shared removal role's 28 hit and 24 graphic. Users may add files/folders, deduplicate and remove; bounded background summaries provide regular-file sizes and directory totals remain unknown. Hidden windows do not update or steal another window's summary; metadata updates do not change selected paths.

Authoritative parameters are operation.adding in layout-scale.json, reusing existing create and shared control parameters; independent mapping is in [add-reference.json](../../specs/visual/add-reference.json). The [editable design](../../design/hifi/sources/archive-add.html) supports Chinese/English, light/dark, 100%/125%/150%, long names and empty selection. HTML and Qt separately check cards, controls, scrolling and footer; main-window/menu proportions are measured separately. Offscreen success does not replace native desktop acceptance.

## In-archive move states

Internal dragging changes no main-window, menu or row-height ratios. Targets use existing list selection and treeSelectionColor. Feedback uses tooltipSurface/tooltipText, tooltipMaxWidth, tooltipPadding and mainBodySize, at most four lines, flipped/constrained within client area without a fixed operation window. Invalid targets show specific reasons and source rows retain selection color. Reuse progress and conflict-window dimensions; file conflicts offer skip, rename, replace and apply to later conflicts. See [archive-move.html](../../design/hifi/sources/archive-move.html).

Once file drag exceeds the system threshold, immediately show a pointer-following icon/name card without waiting for a target directory. Single items show type icon and name; multiple add selection count. Names are plain text, middle-elided; target explanations have at most four lines. The card handles no input and cannot block drop detection; target cursor and specific text still express allowed/refused drops. Offset 16 down/right, constrain to client bounds and clear immediately on cancellation or completion. Maximum width 280, icon 24, padding 10, gap 8 and radius 8; use shared hint colors and menu text, growing text and maximum width for accessibility fonts. See [drag-preview.json](../../specs/visual/drag-preview.json) and [archive-drag.html](../../design/hifi/sources/archive-drag.html).

External copying uses platform drag sessions. Windows Shell and Linux QDrag use transparent 64-logical-unit previews, 48-unit icons and multi-selection count badges; counts above 99 show 99+. Only bitmap storage samples at window DPR; logical dimensions do not change. Mac retains AppKit native file-promise images and system feedback. Native cursors express copy or refusal; do not depict external copying as in-archive movement or show completion before success. Native image preparation failure retains safe transfer and default system cursor without weakening content verification. See [Windows Shell drag feedback](https://learn.microsoft.com/en-us/windows/win32/shell/dataobject) and [Qt 6.8 QDrag](https://doc.qt.io/qt-6.8/qdrag.html).

The extraction confirmation's native and body titles are “解压文件” / “Extract files”; the toolbar Extract action retains a separate translation ID. Title revision changes no window dimensions, body or footer.

The first body group follows the shared header with operation.header.bodyGap, currently 0 logical pixels. Header padding already supplies separation; do not add legacy hero gaps. Card padding, body paragraphs and settings-section gaps remain inside body containers. Qt and HTML measure the header-bottom-to-first-group boundary, covering running and all three terminal task views.

## About license notice

The scrollable About settings page identifies copyrights of xdzip original code and Qt, the xdzip license and Qt's selected LGPL-3.0 option, and points to bundled GPL-3.0 and LGPL-3.0 texts. Reuse existing text roles and wrapping without a new window or footer; HTML and Qt share bilingual copy.

## Dynamic operation-window height

The [task path and result revision](../../design/hifi/sources/task-windows.html) keeps the running Current file slot on one line with middle elision and the full plain-text accessible name. Path length alone does not resize its card or window. Ordinary extraction result captions contain only the committed file count, independent of retained detail records; creation, batch summaries and expandable details keep their existing behavior. Existing logical dimensions, density and larger-font adaptation remain authoritative.

Each operation window retains its existing target client height based on content, font and available screen. During adjustment, first widen the constraint in the target direction, set target height, then restore both bounds to the target, avoiding platform refusal under stale constraints. The client height remains fixed afterward. Reopening, content changes and late platform geometry callbacks use the same logical units without screen-scale multipliers or new size parameters. Runtime geometry checks do not replace native desktop visual acceptance.
