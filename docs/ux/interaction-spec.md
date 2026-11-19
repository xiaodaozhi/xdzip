# Additional interaction and copy specification

English | [简体中文](../zh-CN/ux/interaction-spec.md) | [繁體中文（台灣）](../zh-TW/ux/interaction-spec.md)

The main-window and extraction-confirmation visual baseline is supplemented by the [operation state specification](hifi-state-spec.md) for creation, conflicts, tasks, results, settings and platform comparisons. See the [welcome-page contract](welcome-opening.md) for welcome and opening progress; current originals and layout references follow the [window registry](../../specs/visual/window-registry.json). A state specification does not mean all designs are implemented or have passed native visual acceptance; implementation boundaries are in the corresponding engineering and interaction contracts.

| Scenario | Presentation and action | Recovery |
| --- | --- | --- |
| First launch | Open or create an archive; local file drag-in | Do not scan all disks or automatically associate all formats |
| Reading directory | Count read and indeterminate progress; total unknown | Session can close; partial results are not called complete |
| Empty archive | “This archive is empty”; retain file properties | Open another; disable extraction and explain |
| No search results | Show search term and scope | Clear search; do not incorrectly report an empty archive |
| Unsupported feature | Show specific format/algorithm/encryption reason | Retain readable metadata and stop unsafe partial output |
| Create archive | Input list, format, destination, level and preflight | Errors near fields; do not overwrite existing files by default |
| Name conflict | Both items, explicit Skip, Rename and Overwrite actions, and scope for subsequent decisions | Close the prompt to keep the task waiting. Reopen it or cancel from the task |
| Task complete | Success, skip, failure and commit counts | Extraction opens the destination on explicit request. Creation can open a supported single archive or the first volume in-app. Retry requires replanning |
| Partial failure | Retain committed files and explain failures | Do not delete user files to simulate whole-task rollback |
| Application exit | Active task count, wait for completion or cancel and exit | Core confirms cleanup; do not block the GUI on an infinite join |
| Settings | System/light/dark theme, density, font size, language, recent history and tray | Apply immediately after successful save; retain old values on failure. Quota editing is not a current setting |
| Format extensions | The product uses built-in formats. An [external SDK trial](../engineering/sdk-trial.md) tests read-only execution separately | Use the [format registry](../engineering/format-registry.md) to find available operations |

The extraction button follows the selection. Its menu offers both scopes, subject to current eligibility. Search covers the current directory and its descendants. The properties panel supports user-requested [text and image preview](../engineering/content-preview.md). Below the inspector breakpoint it can open in a separate window. Larger fonts grow the layout and enable scrolling so confirmation controls stay reachable.

Error copy states what happened and what the user can do; error codes belong in details. Do not show ordinary users call stacks or IPC terminology. Present dangerous filenames as safe text and do not interpret plugin messages as HTML. Shortcuts, IME, focus and screen readers follow the [Qt Quick interface design](../design/qt-quick-ui.md).
