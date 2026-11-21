# Automatically capture the four Store views

English | [简体中文](../../zh-CN/market/windows/capture.md)

## Export from an approved capture set

Install suitable system fonts for the selected languages, including region-specific Chinese, Japanese and Korean fonts where needed. Review the rendered glyphs before use. Install the shared image tooling once from the repository root.

```powershell
npm install --prefix build/toolchain/windows-store-tools --no-save --no-package-lock sharp@0.34.5
```

To generate ten-language copy, backgrounds and illustrations without screenshots, run:

```powershell
node scripts/build-windows-store-assets.cjs --backgrounds-only
```

This output contains no native screenshots or composites. Add `--output build/windows-backgrounds` for a chosen fresh directory, or `--4k` for additional 4K artwork.

Use the `capture-source` folder from a reviewed export or backup. This reuses the actual screenshot pixels and uses the current artwork and listing copy.

```powershell
python scripts/package-windows-store-materials.py --from-archive "D:/Store-backup/approved/capture-source"
```

The command prints the new output directory under `build/store-listing/`. Override it with `--output` or add `--4k` for 3840 × 2160 artwork. Review the exported images and use each language's 1920 × 1080 files in `promotional/` for the Store desktop screenshot field. Keep the directory as individual files after submitting new artwork.

## Capture languages from a verified MSIX

When screenshots need refreshing, use this command on a Windows desktop. It validates the Store package and its receipts, captures all four scenes in all ten product languages, and exports the same directory layout.

```powershell
python scripts/refresh-windows-store-screenshots.py --package-dir build/store-msix-package
```

The default capture DPI is 144 (150% scaling). Use `--languages en-US zh-CN` to select a subset. Matching completed sessions can be reused with `--languages en-US zh-CN --from-captures EN ZH` in the same order. Archived bilingual captures remain usable with the export command above; other languages receive text and illustrations, not invented screenshots. The raw screenshot provenance is copied to `capture-source/`. The source and output identities are recorded in `manifest.json`.

## Capture just one language

Close any existing xdzip processes first. From the repository root, run either command in Windows PowerShell 5.1:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language zh-CN
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language en-US
```

For a double-click entry point, open [capture-windows-store-zh-CN.cmd](../../../scripts/capture-windows-store-zh-CN.cmd) or [capture-windows-store-en-US.cmd](../../../scripts/capture-windows-store-en-US.cmd). These launch the same script and exit automatically on success. A failed run keeps the console open to show its error.

`-ExecutionPolicy Bypass` applies only to this invocation; it does not change the saved execution policy. The script looks for a complete runtime matching the SHA-256 in the [capture configuration](../../../specs/capture/windows-store.json), including the package output or an existing candidate directory. A worktree can reuse that runtime from the main checkout. It does not build or install the application. Owned demonstration images and archives are generated from [source](../../../scripts/windows-store-demo.psm1) inside each fresh output directory, without downloads.

Pass `-Executable` for another location. To capture a newer build, also supply its intended `-ExpectedSha256`, and optionally its known `-SourceRevision`; see [updating and reusing the tools](../../engineering/windows-screenshots.md). Other builds are not labeled as the pinned Store candidate. `-DemoDirectory` explicitly reuses an owned demo set instead of generating one. It must contain `设计交付.zip` and `图片精选.zip` for Simplified Chinese, or `Design delivery.zip` and `Photo collection.zip` for all other languages, plus `coastline.png`, `dusk.png` and `forest.png`. An existing `-OutputDirectory` is rejected to preserve earlier images.

Use `-CreationDemoDirectory build/Photos-en` (or another fresh directory) to give the creation screenshot a short demonstration path. The script creates it with a protected ACL, copies the three images, sets a localized archive name (`Photos.zip` or `图片合集.zip`) and focuses the ZIP control before capture. Without this option, it uses `create-demo/` inside the output directory. Existing directories are rejected, not overwritten.

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

The default output is a new directory under `build/screenshots/windows-store/`. Each run produces:

- Four `<language>-<scene>.png` files containing the exact client pixels. The main window's custom title area is part of this crop; a separate native title bar, resize borders and shadows are excluded.
- Four `<language>-<scene>-window.png` reference files containing the visible native frame, including the creation window's native title bar. Outer rounded corners can contain desktop background; these references are not transparent cutouts.
- `capture-session.json`, recording language, executable and tool-source hashes, known source revision, confirmed scene states, dimensions, DPI, client crop and PNG hashes. `complete` becomes true only after all four views are saved and the PNG set passes verification. Partial runs remain marked false and record the failure.
- `demo/`, containing the source-generated owned demonstration inputs, unless `-DemoDirectory` was supplied.
- `create-demo/`, containing the three owned demonstration image copies. Failed runs may also retain `owned-ui.txt` or `creation-ui.txt` for diagnosis.

The native helper copies screen pixels after DWM presentation, checks ownership, geometry and occlusion before and after readback, then encodes lossless PNG. The client image is cropped from the same frame without resizing. It does not pass through JPEG, redraw text, sharpen glyphs, add a decorative frame or draw a pointer. Physical image dimensions follow the real window DPI; they are not written back into Qt component sizes. See [windows-capture.cs](../../../scripts/windows-capture.cs) and the [reusable module guide](../../engineering/windows-screenshots.md).

Inspect the saved originals at 100% before using them. This method preserves the application's actual rendering; it cannot improve text already rendered poorly. The script does not upload images or replace the Store artwork automatically. When it exits or is canceled, it releases its cancellation hotkey, closes only the xdzip process it created, restores the pointer and removes any unconsumed private request. It does not start compression or extraction tasks.

## Check without operating the desktop

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -Language zh-CN -ValidateOnly
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/capture-windows-store.ps1 -SelfTest
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check-windows-captures.ps1 -Directory build/screenshots/windows-store/my-capture
```

`-ValidateOnly` checks the expected executable, demo-generation plan or supplied demo paths, and output path without launching xdzip or writing files. `-SelfTest` checks exact PNG RGBA roundtripping, physical-size calculations, crop pixels, lossy-file rejection and refusal to overwrite files under `build/`. The separate validator checks an existing receipt and every client/window pair. These checks do not capture the desktop or establish manual scene correctness. The broader reusable regression entry is `scripts/test-windows-capture.ps1`.
