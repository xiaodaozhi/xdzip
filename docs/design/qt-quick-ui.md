# Qt Quick desktop interface

English | [简体中文](../zh-CN/design/qt-quick-ui.md) | [繁體中文](../zh-TW/design/qt-quick-ui.md) | [日本語](../ja-JP/design/qt-quick-ui.md) | [한국어](../ko-KR/design/qt-quick-ui.md) | [Français](../fr-FR/design/qt-quick-ui.md) | [Deutsch](../de-DE/design/qt-quick-ui.md) | [Español](../es-ES/design/qt-quick-ui.md) | [Português (Brasil)](../pt-BR/design/qt-quick-ui.md) | [Русский](../ru-RU/design/qt-quick-ui.md)

The desktop uses Qt Quick for presentation and interaction. C++ controllers translate intent and maintain view state. The Rust runtime owns archive interpretation, task policy and file authority. The [system architecture](system-architecture.md) and [bridge contract](qt-rust-bridge.md) describe the calls below.

## Windows, sessions and models

| Component | Current responsibility |
| --- | --- |
| `Browser.qml` and `SessionPage.qml` | Host the welcome page, archive tabs, path and search controls, directory navigation, list or grid, properties, and operation entry points. A page presents the controller's current state and invokes commands. |
| `ApplicationSessions` | Routes open requests to independent browsing sessions and coordinates tabs or windows, the shared task entry, and exit. Opening mode applies to future opens. Closing a page releases browsing presentation while a task's controller can stay alive. |
| `BrowserController` and `ArchiveTableModel` | Maintain selection, listing state and result identity. Small results commit as a complete model. Large results expose a logical row count with at most 1,024 resident rows and fetch missing pages. Unloaded rows cannot submit entry actions. |
| `TaskListModel` and operation windows | Present bounded task snapshots, progress, results and the current decision. A task command is routed to its owning controller using the composite desktop identity. |
| Settings and platform adapters | Maintain appearance, locale, text size and application behavior. Native adapters handle the tray, activations, file dialogs, associations and transfer handoffs according to the operating system. |

[Desktop entry](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/browser_main.cpp), [Session page](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/ui/SessionPage.qml), [Controller and models](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/BrowserController.h), [Application sessions](../engineering/application-sessions.md), [Settings and localization](../ux/settings-localization.md)

## Browsing and operation flow

Opening publishes a completed metadata snapshot. A new search, sort or navigation request carries its own result identity. A stale page cannot replace the current model. Views render the existing committed result while a new query is pending where the controller keeps it valid. The core transport sends at most 128 rows and 64 KiB of names per batch. [Domain data](domain-data.md), [Large archive indexing](../engineering/large-index.md)

Entering a directory navigates into it. Double-click or the corresponding open action on an eligible regular entry starts the temporary-open task. The properties view exposes a separate Preview action. Conflict and password prompts keep the identity of the current operation. Results and progress use the application's task entry, including when the browsing page has closed. [Browsing](../user-guide/browse.md), [Temporary opening](../engineering/archive-opening.md), [Content preview](../engineering/content-preview.md), [Task windows](../ux/task-windows.md)

Drag-in can open archives or supply creation inputs depending on the target. Eligible in-archive moves and drag-out use task preparation and platform-specific handoffs. The platform determines transfer capabilities, including the handling of asynchronous content and destination writes. [Archive transfers](../engineering/archive-transfers.md)

## Shared controls and native boundaries

Operation screens reuse `OperationButton`, `OperationFooterButton`, `OperationFrame`, `OperationSelect` and `OperationSegmentedGroup`. The shared components own visual metrics and interaction. Discrete choices use the shared intent controls to route mouse, keyboard and accessibility actions into the same state model. UI text and file data follow the font and localization rules in the [design standard](../visual/design-standard.md). Archived names remain data and are displayed safely.

Independent operation windows use the window registry and per-window modality and close behavior. macOS uses native title-bar controls. Windows and Linux use their system adapters. The [platform window contract](../ux/platform-windows.md) defines acceptance for real windows, and the [desktop integration contract](../ux/desktop-integration.md) describes tray and activation behavior.

## Design and verification scope

Source code and tests describe product behavior. Native focus, menus, IME, accessibility actions, display scaling and system file-manager behavior still need platform-specific evidence when affected. Extending desktop support to more platform configurations or adding new operation windows requires the corresponding implementation and acceptance. [Visual workflow](../visual/design-workflow.md), [Quality engineering](quality-engineering.md)
