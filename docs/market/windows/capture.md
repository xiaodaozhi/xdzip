# Capture the four Store views

English | [简体中文](../../zh-CN/market/windows/capture.md)

[capture-windows-store.ps1](../../../scripts/capture-windows-store.ps1) assists with all four screenshots for one language in a single run. It opens two demonstration archives using the Store candidate and isolated settings. Prepare each view yourself, then press **F8** while xdzip is active. The assistant saves and names the PNGs and advances to the next view. **F10** cancels, retaining earlier images.

## Run once for each language

Close any existing xdzip processes first. From the repository root, run either command in Windows PowerShell 5.1:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language zh-CN
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language en-US
```

For a double-click entry point, open [capture-windows-store-zh-CN.cmd](../../../scripts/capture-windows-store-zh-CN.cmd) or [capture-windows-store-en-US.cmd](../../../scripts/capture-windows-store-en-US.cmd). These launch the same assistant and keep the console open to show the result.

`-ExecutionPolicy Bypass` applies only to this invocation; it does not change the saved execution policy. The script uses the existing candidate at `build/issue272-store-materials/runtime/xdzip.exe` and the owned files in the adjacent `demo/` directory. When run from a Git worktree it can reuse those files from the main checkout. It checks the executable against the SHA-256 recorded in [windows-store.json](../../../specs/marketing/windows-store.json), rather than building or installing a different application.

For other locations, pass `-Executable`, `-DemoDirectory` and optionally a new `-OutputDirectory`. The demonstration directory must contain `设计交付.zip` and `图片精选.zip` for Chinese, or `Design delivery.zip` and `Photo collection.zip` for English, plus `coastline.png`, `dusk.png` and `forest.png`. The assistant does not download demonstration content. An existing output directory is rejected to preserve earlier images.

## Prepare each view, then press F8

Keep the default window size and current system scaling. Wait for loading to finish, close menus and tooltips, move the pointer away and activate the requested xdzip window before pressing F8. The console displays the current step. A successful capture produces a sound and a saved-file message.

| Capture order | View to prepare | Scene name |
| --- | --- | --- |
| 1 | Switch to the design-delivery archive, use list view and select a file so its properties appear on the right. | `01-browse` |
| 2 | Switch to the photo archive, select `coastline.png`, request Preview and wait for the complete image. | `03-preview` |
| 3 | Keep both archive tabs open, switch the photo archive to grid view and select an image. | `04-workspace` |
| 4 | Press Ctrl+N, add the three demo PNGs, set the destination to the demo directory and leave ZIP compression options visible. Do not start creation. | `02-create` |

The assistant does not click these controls or judge whether the scene is ready. F8 confirms your preparation. It rejects a foreign foreground window, the wrong xdzip window, maximization, changed main-window dimensions, off-screen bounds or overlapping windows. Fix the indicated condition and press F8 again. A full window must fit inside one monitor's work area. The assistant does not resize windows or change monitor settings to make them fit.

## Output and image quality

The default output is a new directory under `build/issue272-store-materials/native-captures/`. Each run produces:

- Four `<language>-<scene>.png` files containing the exact client pixels. The main window's custom title area is part of this crop; a separate native title bar, resize borders and shadows are excluded.
- Four `<language>-<scene>-window.png` reference files containing the visible native frame, including the creation window's native title bar. Outer rounded corners can contain desktop background; these references are not transparent cutouts.
- `capture-session.json`, recording language, candidate hash, dimensions, DPI, client crop and PNG hashes. `complete` becomes true only after all four views are saved. Partial runs remain marked false.

The native helper copies screen pixels after DWM presentation, checks ownership, geometry and occlusion before and after readback, then encodes lossless PNG. The client image is cropped from the same frame without resizing. It does not pass through JPEG, redraw text, sharpen glyphs, add a decorative frame or draw a pointer. Physical image dimensions follow the real window DPI; they are not written back into Qt component sizes. See [windows-store-capture.cs](../../../scripts/windows-store-capture.cs).

Inspect the saved originals at 100% before using them. This method preserves the application's actual rendering; it cannot improve text already rendered poorly. The script does not upload images or replace the Store artwork automatically. When it exits or is canceled, it releases its hotkeys and closes only the xdzip process it created. Do not run real archive operations in this capture session.

## Check without operating the desktop

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language zh-CN -ValidateOnly
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -SelfTest
```

`-ValidateOnly` checks the candidate, demonstration paths and output path without launching xdzip or writing files. `-SelfTest` checks exact PNG RGBA roundtripping, physical-size calculations and refusal to overwrite files, using a generated image under `build/`. Neither mode captures the desktop. They do not establish actual native screenshot quality or manual scene correctness.
