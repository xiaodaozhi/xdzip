# Additional interaction and copy specification

The main-window and extraction-confirmation visual baseline is supplemented by the [operation state specification](hifi-state-spec.md) for creation, conflicts, tasks, results, settings and platform comparisons. See the [welcome-page contract](welcome-opening.md) for welcome and opening progress; current originals and layout references follow the [window registry](../../specs/visual/window-registry.json). A state specification does not mean all designs are implemented or have passed native visual acceptance; implementation boundaries are in the corresponding engineering and interaction contracts.

| Scenario | Presentation and action | Recovery |
| --- | --- | --- |
| First launch | Open or create an archive; local file drag-in | Do not scan all disks or automatically associate all formats |
| Reading directory | Count read and indeterminate progress; total unknown | Session can close; partial results are not called complete |
| Empty archive | “This archive is empty”; retain file properties | Open another; disable extraction and explain |
| No search results | Show search term and scope | Clear search; do not incorrectly report an empty archive |
| Unsupported feature | Show specific format/algorithm/encryption reason | Retain readable metadata and stop unsafe partial output |
| Create archive | Input list, format, destination, level and preflight | Errors near fields; do not overwrite existing files by default |
| Name conflict | Information for both items, skip/rename/overwrite and current-task-only option | Overwrite is not the dangerous default Enter action |
| Task complete | Successful/skipped/failed/committed counts | Extraction opens the destination only on explicit choice; creation opens supported single-volume results in-app. Retry requires replanning |
| Partial failure | Retain committed files and explain failures | Do not delete user files to simulate whole-task rollback |
| Application exit | Active task count, wait for completion or cancel and exit | Core confirms cleanup; do not block the GUI on an infinite join |
| Settings | System/light/dark theme, density, font size, language, recent history and tray | Apply immediately after successful save; retain old values on failure. Quota editing is not a current setting |
| Extension management | External execution disabled in first release | Later show provenance, permissions, version and reason for disabling |

The toolbar says “Extract all” without a selection and “Extract selected” with one; search results are not all contents. In the first release the right preview is reduced to a properties panel; limited image/text decoding arrives in M6. Allow growth and scrolling under larger fonts/DPI without clipping confirmation buttons.

Error copy states what happened and what the user can do; error codes belong in details. Do not show ordinary users call stacks or IPC terminology. Present dangerous filenames as safe text and do not interpret plugin messages as HTML. Shortcuts, IME, focus and screen readers follow design/04.
