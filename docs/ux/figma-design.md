# Figma functional design draft

Design file: [xdzip — v0.1 product design and interaction specification](https://www.figma.com/design/ib4gjPbNSFNpaOI1ld3zaF).

Further functional detail continues in the [repository interaction prototype](prototype-spec.md); this file remains a record of design exploration.

Status: initial functional design, without complete visual review or prototype links. It is not the final development acceptance baseline. Early visual direction images are no longer in docs/visual. This page describes the structure and limitations of the historical exploration; current originals are in [design/hifi](../../design/hifi/README.md) and the visual specifications. Do not treat the historical tasks below as current Qt product defects.

## File contents

- 00 Design specification and requirements mapping: scope, requirements, button and input components.
- 01 v0.1 core flows: 11 screens covering open, browse, selection, search, extraction, tasks, results, creation, dark theme and settings.
- 02 Error and boundary states: 9 screens covering loading, empty archive, no results, unsupported features, conflicts, permissions, cancellation, partial failure and source changes.

The file contains 16 component variants, 6 text styles and 57 variables including raw colors, theme semantics and dimensions. The Starter plan uses separate Light/Dark single-mode collections. Text, controls and layouts are editable layers, not full screenshots standing in for UI layers. The first property panel does not contain later image preview.

## Artboard index

| ID | Screen | Requirements | Figma node |
| --- | --- | --- | --- |
| F01 | Open and create | P01, P04 | 4:2 |
| F02 | Browse archive, no selection | P01, P02, P08 | 4:22 |
| F03 | Select 3 files | P02, P03 | 4:134 |
| F04 | Search and selection scope | P02 | 4:246 |
| F05 | Confirm extraction | P03, P07 | 4:321 |
| F06 | Extraction in progress | P06 | 4:351 |
| F07 | Extraction complete | P03, P15 | 4:374 |
| F08 | Create archive | P04, P05 | 4:398 |
| F09 | Creation in progress | P04, P06 | 4:432 |
| F10 | Dark browsing | P08 | 4:454 |
| F11 | Settings | P08, P11 | 4:566 |
| S01 | Reading directory | P01, P02 | 6:2 |
| S02 | Empty archive | P01, P02 | 6:23 |
| S03 | No search results | P02 | 6:41 |
| S04 | Unsupported encryption feature | P05 | 6:59 |
| S05 | Name conflict | P03, P07 | 6:80 |
| S06 | Destination not writable | P07, P15 | 6:110 |
| S07 | Stopping and cleaning up | P06 | 6:131 |
| S08 | Partial failure and retry | P15 | 6:150 |
| S09 | Source file changed | P15 | 6:172 |

Append `?node-id=4-134` to the file URL to locate an artboard. Each has entry, behavior, recovery path and AC acceptance IDs below it. Engineering requirements P09/P10/P12 remain subject to test and release evidence, not proof by image.

## Historical exploration checks and limitations

Representative screenshots of browsing, extraction confirmation, creation and button components were checked. Corrections addressed component-instance label scaling misalignment, multi-line core-flow text height and status-bar spacing. Node results confirmed 47 component instances in core flows and 19 on error pages.

Design exploration not completed at the time:

- Modal title-bar text appeared clipped and needed layout-boundary review and correction.
- Error, dark, specification and remaining flow pages had not received page-by-page visual acceptance.
- Prototype transitions were not connected; inputs, checkboxes and scope choices represented design states, not operational forms.
- File rows, creation input lists and others needed further componentization; toolbar entries, format switching, single/multiple selection and error recovery needed precise interaction branches.
- Cancellation completion, creation completion and damaged-file states needed further development; 20 artboards did not cover every v0.1 state.
- Native file pickers, system menus, platform keyboard labels and narrow-window/font-scaling layouts needed to be added.

Figma Starter MCP call limits were reached during editing on 2026-10-11, pausing remaining review and prototype connections. Before further editing, check current nodes, preserve existing components and variables, and do not recreate the file.
