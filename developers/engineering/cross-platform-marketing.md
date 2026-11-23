# Single-source screenshots for cross-platform marketing

English | [简体中文](../zh-CN/engineering/cross-platform-marketing.md)

Capture the real application once for all ten languages to produce platform-styled promotional images for a website or other presentation. The Windows and Ubuntu exports derived from macOS are labeled as composites in the manifest. Use separate target-platform validation for functionality, native visual acceptance, and store submissions. All platforms use the same [150% capture scale](marketing-screenshot-scale.md). Capture the normal window and preserve its source density.

## Capture once

The composite entry points accept a complete [macOS capture receipt](platform-marketing.md) from a clean build with a full source revision. Use the default light appearance and text size and a measured 200% Retina capture. The expected main window is 1024 × 622 logical pixels and the create window is 491 × 736 plus a 32-pixel macOS titlebar. Keep the original window PNGs and receipt in the capture directory. The [native Windows Store workflow](../market/windows/capture.md) keeps its own candidate and evidence requirements.

Capture the welcome page with one tab, create with ZIP and the controlled demo path, and preview and workspace with two archive tabs. Review the current application, language, window scope, scene and image before recording. Measure the two shortcut labels and the destination field in each language. Include only the old text and sufficient blank padding for its replacement. Record those source PNG rectangles as `[x, y, width, height]` in a JSON region map under `build/`. The map must contain `schema_version: 1`, the `source_receipt_sha256` of `capture-session.json`, and a `regions` object keyed by all ten language IDs. Each language holds two rectangles under `shortcuts` in Open/Create order and one rectangle under `path`. Use the physical source pixels, measured from the upper-left corner of the complete PNG. The compositor checks region bounds, background and intact padding. Check the source text and the target text visually before publishing.

## Title region parameters

The following are logical coordinates. Multiply by 1.5 and round final pixel boundaries; do not change product component dimensions.

| Window | Confirmed implementation | Conversion requirement |
| --- | --- | --- |
| Windows main window | Title height 48, three right-side buttons of width 46, icons of size 18; `ArchiveTabs.qml` lays out the tabs and brand. | Rebuild the title region or retain the Windows original, with minimize, maximize and close on the right. |
| Ubuntu main window | Title height 48, right inset 6, three buttons of width 36 with 28-diameter circular backgrounds; main-window radius 10. | Use the repository's `WindowCaptionButton.qml` and existing light/dark icons and colors; relayout settings and tabs. |
| macOS main window | Title height 48; brand x approximately 78.057 and first tab x approximately 180.589. The native buttons are on the left and `MacTitlebar.mm` aligns their centers to the title's vertical center. | Rebuild the title region and place the buttons using native measurements; overlaying traffic lights on the Windows brand is insufficient. |
| Create and other operation windows | `OperationWindow.qml` uses native window decorations and requests a title and close button, but does not request minimize or maximize by default. | Treat operation windows separately, distinguish frame from client content, and do not invent enabled actions. |

The exporter repositions the captured brand, tabs and settings separately using the shared visual parameters. The supported compositions retain the default tab widths. It draws the Windows and Ubuntu main-window controls from the repository's application icons and dimensions. Creation removes the macOS decoration and adds a separate localized Windows or Ubuntu-style header with a close control, using the modeled light appearance in [cross-platform-frames.json](../../specs/marketing/cross-platform-frames.json). The Ubuntu operation frame models Yaru; installed window-manager themes can vary. [Yaru theme](https://github.com/ubuntu/yaru) and [GTK X11 window decorations](https://gnome.pages.gitlab.gnome.org/gtk/gtk3/class.Window.html).

AppKit and system appearance determine the macOS buttons' exact geometry. The independent probe can be compiled and run on a Mac without capturing desktop contents:

```sh
mkdir -p build/marketing-probe
clang -fobjc-arc -framework Cocoa scripts/measure-macos-marketing-titlebar.m -o build/marketing-probe/titlebar
build/marketing-probe/titlebar
```

In one macOS 27.2.1 probe, the initial button x coordinates were 9, 32 and 55, each 14 × 14; the latter two operation-window buttons were disabled. Coordinates are relative to the client area, so operation-window buttons above it have a negative top offset. The probe records AppKit's initial button geometry; the main window's final vertical center must also be measured against `MacTitlebar.mm`. It does not establish button pixels, system appearance or behavior on other system versions. Prepare platform graphics from owned controls or verified local native references and record the reference platform, appearance and version. Remeasure when the system appearance changes; do not treat one measurement as a universal macOS fact.

## Export and publish

Run these commands from the repository root with the same [image renderer](platform-marketing.md#prepare-the-image-renderer) as the native marketing exporters. Use fresh output directories.

```sh
node scripts/build-windows-store-assets.cjs --composite-from-macos build/macos-captures --composition-layout build/regions.json --output build/windows-marketing
node scripts/build-linux-store-assets.cjs --composite-from-macos build/macos-captures --composition-layout build/regions.json --output build/linux-marketing
```

The manifest records `cross-platform-composites` and identifies each source PNG by digest, scene and language. It records the measured region map, template and source receipt by digest. The composition changes the window chrome, welcome shortcuts and destination path before exporting the main window at 1536 × 933 and the framed create window at 737 × 1152. Promotional images use the shared 1920 × 1080 arrangement. The Mac originals and source receipt stay with the Mac capture.

For a preview or workspace obscured by a breadcrumb tooltip, use a reviewed alternate Mac capture of the same version and build. Add `alternate_capture` to the region map, with the alternate `source_receipt_sha256` and a `scenes` array of `{ "language": "en-US", "scene": "04-workspace" }` entries. Pass `--alternate-macos DIR` to both commands. Each image keeps the digest and revision of the capture that supplied it. Use the primary capture for welcome and create.

Review each scene and language, the changes outside the registered areas, and the output checksums. Compare the platform chrome with the visual specifications and available target references, and record gaps in the issue. Run `node scripts/test-platform-store-assets.cjs` for synthetic regressions. Use [latest-only Drive sync](marketing-archive.md) to update the existing platform folders and publish manifests after the images pass review. Record delivery evidence in [issue #409](https://github.com/xiaodaozhi/xdzip/issues/409).
