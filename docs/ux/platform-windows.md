# Platform windows and title bars

xdzip operation interfaces use standalone top-level windows, not sheets attached to the main window or overlay panels drawn inside it. A standalone window does not imply a separate process or a taskbar entry for every auxiliary window. Dropdown menus and tooltips remain lightweight Popups and are not operation windows.

## Window contract

| Window | Relationship to main window | Close / Escape | Minimize / maximize |
| --- | --- | --- | --- |
| Extract, create, open file | Has a transient owner; blocks its main window | Discard current form without starting writes | Disabled / disabled |
| Name conflict | Blocks its main window; task waits for a decision | Cancel the current task, stop and clean up, then show result; never overwrite by default | Disabled / disabled |
| Existing archive overwrite confirmation | Blocks its main window | Return to creation form; implementation preserves draft | Disabled / disabled |
| Task progress | Nonmodal; browsing its archive may continue | Close details only; task continues and can be reopened from task entry | Minimize / disabled |
| Task results, settings | Nonmodal | Close window and restore appropriate focus | Disabled / disabled |

Cancelling a task is an explicit task command independent of closing its progress window. Background completion does not steal foreground focus; update open details or otherwise update the task entry. Application exit still follows task-exit policy; closing the main window cannot directly map to closing auxiliary windows.

## Platform appearance

- macOS: use a system-decorated title bar with red, yellow and green controls at upper left; do not substitute custom-colored dots for native controls. Fixed-size operation windows retain red Close and disable unavailable minimize/zoom; progress windows can minimize. Actual button availability depends on AppKit/Qt window capabilities and is accepted on a real device. The main window retains normal resize, minimize and fullscreen capabilities.
- Windows: custom title bars are allowed, with icon/title left and minimize/maximize/close right. Available buttons support system menu, keyboard access, high contrast and hit areas. The main window retains system resize, dragging and snapping; fixed-size operation windows cannot maximize.
- Linux: custom title bars are allowed; the prototype shows buttons at right, while the product respects desktop environment conventions. Initial support is X11; future Wayland expansion needs separate verification. Unavailable features appear disabled. Environments where custom decoration cannot adapt reliably fall back to server-side decoration.

Each operation window has its own title, border, shadow and draggable title bar; the main window behind it is not dimmed or blurred like a webpage. Initial opening centers within the available area of the owner's display. Restored positions must be clamped to visible screens. Toolbars or content must not cover the title bar.

## Qt Quick implementation constraints

Each operation window uses a standalone `Window` / `QQuickWindow` with `transientParent`. Blocking decisions use `Qt.WindowModal`; task details, results and settings use `Qt.NonModal`. Do not use QML `Popup` as a top-level operation window or make macOS frameless and draw traffic lights. File pickers open as standalone system windows, not sheets; if native integration cannot provide this mode, explicitly implement and accept platform adaptation.

Custom Windows/Linux title bars need actual platform window movement, resizing and system-menu support, not simulated system dragging by manually changing QML positions. `WindowController` handles window lifecycle, ownership, close requests and reopening task views. The core owns tasks; destroying a view cannot destroy an active task. Cross-thread window state updates use only defined bridge events.

## Prototype evidence and limits

The HTML prototype simulates standalone window appearance, title-bar dragging, blocking/nonblocking interactions and close semantics. The top “Platform” selector compares macOS, Windows and Linux and is not a product setting. Browser `dialog` remains in one browser page and cannot provide actual native windows, system buttons, system minimization, taskbar or multi-display behavior. Minimizing a task window is simulated by hiding details.

Platform controls on the prototype main window are appearance illustrations. Returning from overwrite confirmation to creation still resets the example form; the product implementation must preserve the draft. These limitations cannot serve as evidence of completed native acceptance.

## Real-device acceptance criteria

1. Each operation window is a standalone platform window correctly associated with its main window, with no sheet or webpage-style overlay.
2. Traffic lights/right-side button order, disabled state and close requests match platform and window capabilities; titles and system buttons do not overlap at different font sizes and scales.
3. Window dragging, screen boundaries, multiple displays, 125%/150%/200% scaling and light/dark themes work correctly.
4. Closing task details leaves the task running and the task entry can reopen it; closing a conflict window safely stops without overwriting files.
5. Owner modality, focus restoration, keyboard navigation, screen-reader titles and system shortcuts work; unrelated archive windows are not unnecessarily blocked.
6. Verify Windows system menu/snapping and Linux X11 separately; Wayland is outside initial support. Existing Qt real-device conclusions are in [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). This list is for rechecking affected items later and does not reset completed acceptance.
