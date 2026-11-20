# Platform windows and title bars

English | [简体中文](../zh-CN/ux/platform-windows.md) | [繁體中文（台灣）](../zh-TW/ux/platform-windows.md) | [日本語](../ja-JP/ux/platform-windows.md) | [한국어](../ko-KR/ux/platform-windows.md) | [Français](../fr-FR/ux/platform-windows.md) | [Deutsch](../de-DE/ux/platform-windows.md) | [Español](../es-ES/ux/platform-windows.md) | [Português (Brasil)](../pt-BR/ux/platform-windows.md) | [Русский](../ru-RU/ux/platform-windows.md)

xdzip operation interfaces use standalone top-level windows, not sheets attached to the main window or overlay panels drawn inside it. A standalone window does not imply a separate process or a taskbar entry for every auxiliary window. Dropdown menus and tooltips remain lightweight Popups and are not operation windows.

## Window contract

| Window | Relationship to main window | Close / Escape | Minimize / maximize |
| --- | --- | --- | --- |
| Extract or create form | Nonmodal `OperationWindow` with a transient owner when opened from browsing | Dismiss the draft or confirmation without submitting a new task | Disabled / disabled |
| Task conflict prompt | Nonmodal prompt owned by the operation window's host. The task waits for a decision | Dismiss the prompt. Reopen it from the task, or explicitly cancel the task there | Disabled / disabled |
| Password, quit or restart prompt | Application-modal. The native file picker is also application-modal with a transient caller | Password, exit and restart each have explicit dismissal or confirmation actions | According to native dialog capabilities |
| Task progress and result | Nonmodal `TaskWindow`. Browsing can continue when no other modal is active | Dismiss details and reopen them from the task entry. Standalone work retains a way to reach active tasks | Minimize / disabled |
| Settings and About | Nonmodal. A tray entry may open it without a browsing owner | Close the view. Settings already saved remain active | Disabled / disabled |

Cancelling a task is an explicit task command independent of closing its progress window. Background completion does not steal foreground focus; update open details or otherwise update the task entry. Application exit still follows task-exit policy; closing the main window cannot directly map to closing auxiliary windows.

## Platform appearance

- macOS: use a system-decorated title bar with red, yellow and green controls at upper left; do not substitute custom-colored dots for native controls. Fixed-size operation windows retain red Close and disable unavailable minimize/zoom; progress windows can minimize. Actual button availability depends on AppKit/Qt window capabilities and is accepted on a real device. The main window retains normal resize, minimize and fullscreen capabilities.
- Windows uses a client-extended main title bar with QML caption buttons, a native resizable frame and DWM shadow. Dragging, edge resizing and system snapping use platform facilities. Standalone operation windows use native decorations. See the [Windows and Ubuntu main designs](../../design/hifi/sources/main.html?platform=windows).
- Ubuntu X11 uses integrated right-side controls and a rounded outline in normal state. Composited sessions with input shaping also show a soft, click-through window shadow. Maximized and fullscreen windows have square edges and fill the available area. The window system handles movement and resizing. Standalone operation windows keep system decorations. Other backends outside the custom X11 path use normal decorations. Wayland is outside the supported scope. See the [Ubuntu main design](../../design/hifi/sources/main.html?platform=linux).

Each operation window has its own title, border, shadow and draggable title bar; the main window behind it is not dimmed or blurred like a webpage. Initial opening centers within the available area of the owner's display. Restored positions must be clamped to visible screens. Toolbars or content must not cover the title bar.

## Qt Quick implementation constraints

`OperationWindow` uses a standalone `Window` with an optional `transientParent` and defaults to `Qt.NonModal`. `PasswordWindow` and `OperationConfirmationWindow` explicitly use `Qt.ApplicationModal`. The native picker takes the actual calling window as its transient owner and restores activation only while that caller is visible. See the [window registry](../../specs/visual/window-registry.json) and [task windows](task-windows.md).

Custom Windows/Linux title bars need actual platform window movement, resizing and system-menu support, not simulated system dragging by manually changing QML positions. `WindowController` handles window lifecycle, ownership, close requests and reopening task views. The core owns tasks; destroying a view cannot destroy an active task. Cross-thread window state updates use only defined bridge events.

## Prototype evidence and limits

The HTML prototype simulates standalone window appearance, title-bar dragging, blocking/nonblocking interactions and close semantics. The top “Platform” selector compares macOS, Windows and Linux and is not a product setting. Browser `dialog` remains in one browser page and cannot provide actual native windows, system buttons, system minimization, taskbar or multi-display behavior. Minimizing a task window is simulated by hiding details.

Platform controls on the prototype main window are appearance illustrations. Returning from overwrite confirmation to creation still resets the example form; the product implementation must preserve the draft. These limitations cannot serve as evidence of completed native acceptance.

## Real-device acceptance criteria

1. Each operation window is a standalone platform window correctly associated with its main window, with no sheet or webpage-style overlay.
2. Traffic lights/right-side button order, disabled state and close requests match platform and window capabilities; titles and system buttons do not overlap at different font sizes and scales.
3. Window dragging, screen boundaries, multiple displays, 125%/150%/200% scaling and light/dark themes work correctly.
4. Closing task details leaves the task running and the task entry can reopen it. Closing a conflict window leaves the task waiting until the user reopens the prompt or explicitly cancels the task.
5. Owner modality, focus restoration, keyboard navigation, screen-reader titles and system shortcuts work; unrelated archive windows are not unnecessarily blocked.
6. Verify Windows system menu/snapping and Linux X11 separately; Wayland is outside initial support. Existing Qt real-device conclusions are in [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). This list is for rechecking affected items later and does not reset completed acceptance.
