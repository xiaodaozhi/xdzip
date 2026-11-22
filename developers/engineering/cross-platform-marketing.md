# Single-source screenshots for cross-platform marketing

English | [简体中文](../zh-CN/engineering/cross-platform-marketing.md)

The goal is to capture the real application once for all ten languages, then produce Windows-, macOS- and Ubuntu-styled promotional images for a website or other presentation. Cross-platform outputs are composites, not native screenshots of the target platform, and do not establish its functionality, visual acceptance or store compliance. All platforms use the same [150% capture scale](marketing-screenshot-scale.md); do not upscale captures from another scale and claim native 150% capture.

## Capture once

The existing Windows automation is the recommended source: it prepares four owned demo scenes and saves client images, native window references and a receipt with hashes. For a newer build, pass its executable, expected SHA-256 and full source revision explicitly, then verify the product version, scenes and all ten languages before capture. The current Windows Store importer accepts only its older pinned candidate; the new workflow must adapt the receipt separately and must not falsify the pinned-candidate flag. Do not present an older screenshot as the current version or confuse the window reference with the client image. The macOS and Linux recorders prepare scenes manually; their receipts can also be normalized to the same source format.

The four scenes are browse, create, preview and workspace. The create window displays an actual operating-system path: do not show a Windows path in a macOS or Linux composite. Use owned demo data without sensitive paths and prepare an explicit demo-path layer for each target. If a path, platform-specific label or icon cannot be reliably located and replaced, reject the scene instead of publishing an incorrect image. Creation formats are also platform-filtered, so use the common ZIP scene.

## Title region parameters

The following are logical coordinates. Multiply by 1.5 and round final pixel boundaries; do not change product component dimensions.

| Window | Confirmed implementation | Conversion requirement |
| --- | --- | --- |
| Windows main window | Title height 48, three right-side buttons of width 46, icons of size 18; `ArchiveTabs.qml` lays out the tabs and brand. | Rebuild the title region or retain the Windows original, with minimize, maximize and close on the right. |
| Ubuntu main window | Title height 48, right inset 6, three buttons of width 36 with 28-diameter circular backgrounds; main-window radius 10. | Use the repository's `WindowCaptionButton.qml` and existing light/dark icons and colors; relayout settings and tabs. |
| macOS main window | Title height 48; brand x approximately 78.057 and first tab x approximately 180.589. The native buttons are on the left and `MacTitlebar.mm` aligns their centers to the title's vertical center. | Rebuild the title region and place the buttons using native measurements; overlaying traffic lights on the Windows brand is insufficient. |
| Create and other operation windows | `OperationWindow.qml` uses native window decorations and requests a title and close button, but does not request minimize or maximize by default. | Treat operation windows separately, distinguish frame from client content, and do not invent enabled actions. |

Windows and Ubuntu brand and tab positions are calculated by QML; tab count, labels, text size and right-side controls affect available width. The generator should lay them out using the current QML and visual parameters instead of shifting one captured titlebar wholesale. AppKit and system appearance determine the macOS buttons' exact horizontal coordinates, size and colors. The independent probe can be compiled and run on a Mac without capturing desktop contents:

```sh
mkdir -p build/marketing-probe
clang -fobjc-arc -framework Cocoa scripts/measure-macos-marketing-titlebar.m -o build/marketing-probe/titlebar
build/marketing-probe/titlebar
```

In one macOS 27.2.1 probe, the initial button x coordinates were 9, 32 and 55, each 14 × 14; the latter two operation-window buttons were disabled. Coordinates are relative to the client area, so operation-window buttons above it have a negative top offset. The probe records AppKit's initial button geometry; the main window's final vertical center must also be measured against `MacTitlebar.mm`. It does not establish button pixels, system appearance or behavior on other system versions. Prepare platform graphics from owned controls or verified local native references and record the reference platform, appearance and version. Remeasure when the system appearance changes; do not treat one measurement as a universal macOS fact.

## Follow-on code and delivery interface

1. Normalize capture receipts while retaining source platform, executable identity, revision, DPR, language, scene, original client PNG, window reference and their hashes. Validate all inputs before generating a new directory and preserve the originals. Mark each transformed image with its source platform, target platform and `cross-platform-composite`; record title and demo-path transformation parameters, template version and hashes.
2. Generate the target title region by window type, accounting for button positions, tab and settings layout, corners and borders. Handle native operation-window decorations and demo paths separately. Reject unsupported layouts, missing measurements and occluded input. Preserve original pixels outside registered title and demo-path areas.
3. Connect composites to the existing platform marketing exporters as illustrations. Extend validation and manifests so a composite cannot pass native-capture validation or be labeled a native capture. Output images and promotional artwork by platform, language and scene, retaining source and transformation receipts.
4. Check all ten languages and four scenes for complete content, size, sensitive paths, unchanged pixels outside approved regions, target layout, composite provenance and hashes. Use owned synthetic fixtures for invalid inputs, occlusion, limits and existing output directories. Compare at least one result per target platform with a corresponding native window reference; this is one platform reference comparison, not repeated acceptance for every language.
5. Once complete and checked, update each existing Google Drive platform folder in place using [latest-asset synchronization](marketing-archive.md), preserve unrelated files and update the manifest and checksums last. Replace a complete screenshot set only when all three platforms, ten languages and four scenes are ready.

This page is a conversion design and parameter preparation; it does not claim that the converter, target templates or new ten-language screenshots have been produced. Record actual progress and evidence in [issue #409](https://github.com/xiaodaozhi/xdzip/issues/409).
