# Directory tree B: design, assets and integration

Status: **core directory interaction is integrated in HTML examples and Qt read-only browsing; boundary-state assets await integration**. Scope covers B01 browsing, P01/P02/P08 and WP-05/06, following the [read-only browsing contract](../engineering/readonly-browser.md). Normal directory browsing uses direct children, search recursively covers the current scope, and output authorization remains read-only.

The three current artboards are [multi-level expansion](../../design/hifi/screens/directory-tree-capsule.png), [node and interaction states](../../design/hifi/screens/directory-tree-states.png) and [empty and navigation boundaries](../../design/hifi/screens/directory-tree-boundaries.png). The [shared gallery](../../design/hifi/index.html#components) shows full artboards and the [asset library](../../design/assets/index.html#directory-tree) provides individual slices. Multi-level expansion is exported from the [editable source](../../design/hifi/sources/main.html?tree=expanded), with the title bar synchronized to current capsule tabs. Top and central lists in the board provide context only and do not redefine title bar, toolbar or file table. The [historical multi-level original](../../design/hifi/screens/directory-tree.png) remains unchanged as the source of 26 slices and an independent measurement reference.

Artboards are enlarged and individual card scales differ. Text, radii and pixel dimensions in images cannot directly define control sizes; [component parameters](../../specs/visual/directory-tree.json) explicitly record implemented logical dimensions and the asset manifest separately records physical crop coordinates. The implementation uses a 1024 logical-width and shared source-canvas transform; do not enlarge the entire new component set again.

Qt uses the logical values below directly. The existing HTML main design maps a 1548 source-pixel window to 1024 logical pixels according to [display-spec.js](../../design/html/display-spec.js): `source CSS = logical / (1024/1548)`. Thus row height 32 corresponds to 48.375 source CSS pixels, font 11.2 to 16.93125, icon slot 16 to 24.1875 and indentation 12.8 to 19.35. Preview scaling applies uniformly to the whole window after conversion. DPR determines sampling density and line alignment only and does not enter logical-size conversion.

## Tree structure and counts

The complete index provides explicit and implicit directories. A leaf means **no subdirectories**, not no files. If Glacier Lake contains a subdirectory it has its own expansion arrow, independent of its parent's arrow.

```text
Root                               ← current directory
▼ 01 Photos                     12
  ▼ Glacier Lake                 6
    ▸ Lakeside Trail             3  ← deeper directory, not expanded
      Snow Mountain View          3  ← files only, retains empty arrow slot
  ▸ Mountain Sunrise             4
    Other Photos                 2  ← no subdirectories
▸ 02 Documents                   8
  03 Maps                        4  ← no subdirectories
```

The example archive contains 12 photos, 8 documents, 4 maps and 6 root files; the root list shows three direct directories and six direct files without expanding descendants. Descendants of collapsed branches are also listed fully in the parameter file so leaves are not decorated with fake arrows. The explicit empty directory in another board is a separate state example, not added to this 30-entry example.

Directory numbers represent all descendant **physical archive records** below the prefix, including duplicate names but excluding the directory record identical to the prefix. They are not counts of subdirectories and do not change with name search. Tree rows do not display counts. The archive summary retains the complete physical record count, while the right side of the list title shows current-directory row count; these use different definitions. Ordinary lists show the final name; search results show the full in-archive path.

Arrows depend only on known subdirectories: right when a parent is collapsed and down when expanded. Nodes containing only files or empty nodes have no arrow, but peers retain identical arrow slots, icon slots and text starts. During complete-index loading, do not guess per-node arrows, counts or structure or add per-node asynchronous loading states.

## Layout, fonts and dynamic geometry

| Item | Normal logical size / behavior |
| --- | --- |
| Default sidebar width | 193.16; main-window structural boundary rounds to 193 |
| Row height | Minimum 32; grows according to the parameter formula with larger fonts |
| Per-level indentation | 12.8; independent of node type |
| Arrow hit area / graphic slot | At least 19.2×19.2 / 14.4×14.4; leaves retain the same slot |
| Folder icon slot | 16×16; shared 24-unit SVG scales proportionally, replacing the current tree's 18 slot |
| Icon-to-text / end padding | 6.4 / 16; no visible count |
| Body | 11.2; Qt rounds to 11, using the same-source font |
| Record count | Retained in data and accessible semantics; not shown in rows and no count column reserved |
| Current directory | Pale blue background, 6.4 right corners, straight left edge and full-height 2.4-wide indicator |
| Keyboard focus | No extra outline or focus bar and no change in row occupancy; current-directory indicator remains |
| Hierarchy lines | Width 1; restrained hierarchy against background |
| Hidden current directory marker | 4.8-unit dot in a transparent 19.2-unit slot, sharing the name's row center |

Connecting lines are drawn dynamically from the visible ancestor chain and last-sibling flags. Parent vertical lines cross expanded subtrees through the last sibling's center; the last connection terminates in an L and does not extend into the next top-level group. Each vertical line's center X aligns with its parent's downward arrow tip X, including larger accessibility fonts. Inner lines terminate independently; a generic line image cannot span all rows. Align segments to target DPR; text, arrows and folders share the row's vertical center.

The hierarchy region scrolls horizontally. Width uses only actual names, indentation and 16 end padding for currently expanded visible nodes. Names remain complete without number overlays. Recompute width after collapse so hidden deep nodes leave no blank space; the left indicator stays fixed to the viewport edge. Row hover shows no floating tooltip; full paths are available in accessible descriptions and details. Vertical scrolling occurs only within the directory viewport; scrollbars do not enclose the main-window border.

All directories use the same folder asset. Branch dots, filled information/warning icons and segmented spinners have light/dark SVGs and 1×/2×/3× PNGs; lines, focus frames, backgrounds, skeletons, text and prompts are dynamic components. Dark parameters reuse global theme with additional line and marker colors; **the complete dark directory-tree original remains outstanding** and dark icon exports do not replace the full artboard. Existing rendering and desktop acceptance conclusions are recorded separately in corresponding issues and artifacts.

Record HTML/Qt interaction consistency under the [verification method](../qa/README.md). Artboards and slices retain original design history; current row counts, focus and corners follow this page and the logical specification.

## Interactions and independent states

Expansion, current directory, hover and keyboard focus are stored independently and can coexist. Current directory also differs from file checkbox selection on the right.

Root is a separately navigable entry that is always expanded and does not participate in ordinary node expansion/collapse. When right-click, Menu key or Shift+F10 opens its menu, “Collapse subdirectories” remains disabled without a visible explanation line or tooltip. The accessible description is “根目录始终展开” / “Root is always expanded”; lack of an ordinary tree row does not imply no subdirectories. Ordinary parent expansion/collapse depends on actual subdirectories; ordinary leaves retain the accessible description “This node has no subdirectories”. Reasons are accessible only, not visible hints. Opening the menu or clicking disabled items must not change directory, file selection or tree expansion.

| Action | Result |
| --- | --- |
| Click arrow | Expand/collapse only that node; directory query, file selection and details remain unchanged |
| Click name or Enter | Enter using the current session's directory ID; after successful query replacement reset current-directory query, file selection and details under the existing contract |
| Up / down | Move focus among visible directory rows without automatically switching the file list |
| Right | Expand a collapsed parent, then move to first child if already expanded; leaves stay put |
| Left | Collapse an expanded parent, otherwise move to parent; top-level nodes stay put |
| Collapse current directory's ancestor | Keep current directory and file list; hidden focus returns to nearest visible ancestor |
| New archive session | Clear old node IDs, expansion, focus and branch dots; do not reuse another archive's state |

For example, when the path is `01 Photos / Glacier Lake`, collapsing `01 Photos` keeps Glacier Lake displayed on the right. The Photos row shows only a blue end dot, without receiving Glacier Lake's current-directory fill; root cannot become current either. The blue dot is on the **nearest visible collapsed ancestor**; one hidden current directory has one branch marker. Expanding again restores Glacier Lake's current background.

The tree itself is one keyboard navigation entry. Nodes retain accessible semantics for stable ID, name, level, expansion, position and record count. Leaves do not claim to be expandable; decorative folders, lines and dots are not announced redundantly. Hidden current paths have accessible descriptions and cannot rely on blue alone. Arrow keys and arrow clicks must not propagate into row activation.

## State coverage and recovery

| State | Directory tree | File list / recovery |
| --- | --- | --- |
| Reading | Loading message, unknown count; skeleton is a future design asset; navigation disabled | Follow global loading state; replace tree once after complete index publication |
| Empty archive | Root row has no number; “This archive is empty” | Empty list; another archive can still be opened |
| Root files only | Root row has no number; “No subdirectories” | Show six files normally; do not turn the whole page into an empty state |
| Directory index limit exceeded | Do not publish half a tree; show “Directory exceeds display limit” | Root, search and eligible explicit directory browsing continue within current capabilities |
| Read failure | Dim navigation and show explicit error | Reopen archive; do not interpret failure as zero or empty directory |
| Node with subdirectories | Independent arrow at every level | Expansion is not entry into that directory |
| Nonempty / empty leaf | Count retained in data; neither shows row count or arrow | Name remains activatable; empty directory can be browsed |
| Long name / deep nesting | Full name, accessible path, horizontal scrolling and visible-node width | Path filtering remains consistent |
| 150% font (200% is a historical stress case only) | Slots and rows grow; lines recenter on rows | Names, arrows and focus are not clipped |
| Hidden current directory | Nearest visible ancestor shows branch dot | List retains current path; re-expanding restores current row |

Navigation limits and global read failure are different states; reasons, available actions and count sources cannot share an “empty directory” template. Existing budgets of 32,768 nodes, 64 levels and 4 MiB directory names come from the read-only contract; design does not raise or bypass them.

## Asset delivery and actual integration

The 26 original-pixel slices are under `design/assets/reference/tree-*.png`, each with source, SHA-256, `x,y,width,height` and purpose. `component_reference_coverage` individually points to shared vectors or dynamic components. Do not describe static whole-row images as interactive controls or remove white backgrounds from slices to fabricate transparency.

| Part | Asset / drawing | Current integration |
| --- | --- | --- |
| Folder, right/down arrows | `folder`, `chevron-right`, `chevron-down` from shared assets | Existing shared resources available; B icons and dimensions used by recursive HTML rows and Qt delegates |
| Loading, information, warning | New `tree-spinner`, `tree-info`, `tree-warning` | Part of eight light/dark SVGs and 24 transparent PNGs; retain filled and segmented board forms rather than substituting existing outline icons |
| Hidden current branch | New `tree-active-branch` | Six transparent PNGs for two themes delivered; HTML and Qt consume the asset to mark nearest visible collapsed ancestor |
| Lines, current background, focus, skeleton, hints | Dynamically drawn using dimensions and themes in parameter file | Style and logic specified; no second set of bitmap controls delivered |
| Formal artboards | `hifi/screens/directory-tree*.png` | Retain confirmed originals' bytes for design comparison |

HTML and Qt integrate connecting lines, branch dots, 16-unit folder slots and directory context menus. HTML uses a complete recursive example hierarchy; Qt uses the real bounded index. Collapsing the current ancestor does not submit a query. Loading skeletons and dedicated info/error icons remain design assets; the complete dark artboard is outstanding. Icon integration does not prove platform-by-platform screen-reader or HiDPI acceptance; existing manual conclusions follow [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5).

## Post-integration acceptance

1. Capture the actual directory region at the same 1024 logical width, font configuration, DPR and state data, and compare with the corresponding confirmed artboard region and specification. Record scale relationship; retain visual judgment and structural assertions separately.
2. Align arrow, icon and name starts at each level; no numeric row column. Icons, text, dots and arrows share row centers. Target geometry deviation is at most one physical pixel; record platform and actual fallback for font differences.
3. Expand at least three levels, including a deeper collapsed branch and a nonempty leaf. Accept connection-line continuity/termination through last siblings and expanded subtrees.
4. Check current, hover and focus separately and together. Arrow clicks do not submit directory queries or clear file selection; Enter activation follows the successful-query reset rule.
5. With Glacier Lake current, collapse Photos: list path remains, root is not selected and Photos shows only a branch dot. Re-expand to restore current row. Keyboard navigation remains possible when focus is hidden.
6. Check empty archives, root files, explicit empty directories, read failure and directory limit from a real bounded index; separately check counts, disabling and recovery actions.
7. With long names, 100%/125%/150% fonts and the historical 200% stress case, minimum window and maximum permitted index depth, names remain fully visible, horizontal scroll affects only hierarchy and full paths are accessible. Tree scrolling does not move archive summary or bottom statistics.
8. Complete actual light/dark rendering, normal/Retina/Windows high DPI, keyboard and screen-reader acceptance. Passing image checks or asset generation does not replace these runtime checks.

Under the [verification method](../qa/README.md), separately check originals, slices, assets and actual mouse, keyboard, menu and tree behavior. Explicitly mark untested accessibility and platform differences.
