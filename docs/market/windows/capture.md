# Automatically capture the four Store views

English | [简体中文](../../zh-CN/market/windows/capture.md)

[capture-windows-store.ps1](../../../scripts/capture-windows-store.ps1) prepares and captures all four screenshots for one language in a single run. It opens two demonstration archives using the Store candidate and isolated settings, switches tabs, selects files, requests a preview, changes to grid view and opens a populated creation window. No per-image keys or manual scene preparation are required. **F10** cancels, retaining earlier images.

## Run once for each language

Close any existing xdzip processes first. From the repository root, run either command in Windows PowerShell 5.1:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language zh-CN
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language en-US
```

For a double-click entry point, open [capture-windows-store-zh-CN.cmd](../../../scripts/capture-windows-store-zh-CN.cmd) or [capture-windows-store-en-US.cmd](../../../scripts/capture-windows-store-en-US.cmd). These launch the same script and exit automatically on success. A failed run keeps the console open to show its error.

`-ExecutionPolicy Bypass` applies only to this invocation; it does not change the saved execution policy. The script uses the existing candidate at `build/issue272-store-materials/runtime/xdzip.exe` and the owned files in the adjacent `demo/` directory. When run from a Git worktree it can reuse those files from the main checkout. It checks the executable against the SHA-256 recorded in [windows-store.json](../../../specs/marketing/windows-store.json), rather than building or installing a different application.

For other locations, pass `-Executable`, `-DemoDirectory` and optionally a new `-OutputDirectory`. The demonstration directory must contain `设计交付.zip` and `图片精选.zip` for Chinese, or `Design delivery.zip` and `Photo collection.zip` for English, plus `coastline.png`, `dusk.png` and `forest.png`. The assistant does not download demonstration content. An existing output directory is rejected to preserve earlier images.

## Scenes prepared automatically

Leave the desktop available while the script runs. It temporarily activates only its own xdzip windows, moves the pointer outside the capture and restores the original pointer position afterward. It preserves default window dimensions and system scaling. A window extending past its monitor's work area is repositioned without resizing. If its default size cannot fit, the run stops with an error.

| Capture order | Automatic preparation and checks | Scene name |
| --- | --- | --- |
| 1 | Activate the design-delivery tab, use list view and select only `README.md`. Wait for file selection and preview/property actions. | `01-browse` |
| 2 | Activate the photo tab, select only `coastline.png`, request Preview and wait for the image-preview object and the Preview again action. | `03-preview` |
| 3 | Keep both archive tabs open, change to grid view and check the selected image tile. | `04-workspace` |
| 4 | Copy the three demo PNGs into a new private directory and send a `create` shell request. Check the three inputs and the enabled Create archive action, without invoking it. | `02-create` |

The script resolves controls in a fresh Windows UI Automation tree restricted to its process and window. File selection clicks the observed file row after verifying ownership, foreground and the window under the pointer. Other controls use accessibility actions. Creation uses the existing file-manager entry point and an owner/System-only temporary request. The new input directory receives its own protected ACL; existing directories and their permissions are unchanged.

Each scene must satisfy its state checks, then produce an unchanged frame across several samples. This avoids capturing pending layout or image loading. Foreign foreground windows, ambiguous controls, maximization, changed main-window dimensions, off-screen bounds and overlapping windows stop the run instead of requesting manual capture. Do not switch applications or operate xdzip during this short session.

## Output and image quality

The default output is a new directory under `build/issue272-store-materials/native-captures/`. Each run produces:

- Four `<language>-<scene>.png` files containing the exact client pixels. The main window's custom title area is part of this crop; a separate native title bar, resize borders and shadows are excluded.
- Four `<language>-<scene>-window.png` reference files containing the visible native frame, including the creation window's native title bar. Outer rounded corners can contain desktop background; these references are not transparent cutouts.
- `capture-session.json`, recording language, candidate hash, confirmed scene states, dimensions, DPI, client crop and PNG hashes. `complete` becomes true only after all four views are saved. Partial runs remain marked false and record the failure.
- `create-demo/`, containing the three owned demonstration image copies. Failed runs may also retain `owned-ui.txt` or `creation-ui.txt` for diagnosis.

The native helper copies screen pixels after DWM presentation, checks ownership, geometry and occlusion before and after readback, then encodes lossless PNG. The client image is cropped from the same frame without resizing. It does not pass through JPEG, redraw text, sharpen glyphs, add a decorative frame or draw a pointer. Physical image dimensions follow the real window DPI; they are not written back into Qt component sizes. See [windows-store-capture.cs](../../../scripts/windows-store-capture.cs).

Inspect the saved originals at 100% before using them. This method preserves the application's actual rendering; it cannot improve text already rendered poorly. The script does not upload images or replace the Store artwork automatically. When it exits or is canceled, it releases its cancellation hotkey, closes only the xdzip process it created, restores the pointer and removes any unconsumed private request. It does not start compression or extraction tasks.

## Check without operating the desktop

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language zh-CN -ValidateOnly
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -SelfTest
```

`-ValidateOnly` checks the candidate, demonstration paths and output path without launching xdzip or writing files. `-SelfTest` checks exact PNG RGBA roundtripping, physical-size calculations and refusal to overwrite files, using a generated image under `build/`. Neither mode captures the desktop. They do not establish actual native screenshot quality or manual scene correctness.
