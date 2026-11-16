# HTML visual fidelity review

High-fidelity images are the visual reference; HTML is the editable implementation. Dimensions above and below the toolbar are measured independently from the [frozen initial main window](../../design/hifi/screens/main-light.png), not inferred backward from current CSS. Current display uses the [light capsule-tab design](../../design/hifi/screens/main-capsule-light.png) and [dark design](../../design/hifi/screens/main-capsule-dark.png), exported from the [editable source](../../design/hifi/sources/main.html); they do not replace independent measurement. The title bar is accepted against the explicit revision in layout-scale.json. Original artwork has softened edges and generated glyphs; test counts cannot establish whole-page pixel-perfect identity.

## Fixed reproducible conditions

- The source canvas matches the PNG at 1586×992; the window is 1548×940 at origin `(20,30)`. Original anchor checks still use a 1588×1000 browser viewport at device pixel ratio 1. Crop the top at `(20,30)-(1568,167)` and content at `(20,167)-(1568,970)` without stretching the original.
- Original SHA-256 values are pinned in [header-reference.json](../../design/html/header-reference.json) and [body-reference.json](../../design/html/body-reference.json). Changes to originals require remeasurement, not silent replacement.
- The bundled offline [Xdzip Design Sans SC](../../design/assets/fonts/README.md) font is measured after `document.fonts.ready`. Real 400–750 weights are fixed to reduce width changes from platform fallback fonts.
- The original font family cannot be established from a generated image. Glyph outlines and system anti-aliasing may still differ; record glyph differences separately and do not claim to have restored the original font.

## Display size and source pixels

Normal design display uses the [1024 logical-pixel window width](README.md), uniformly mapping source coordinates by `1024 / 1548`. The main window displays at 1024 × approximately 621.81 CSS pixels; the original uses the same scale. This ratio is an explicit design convention, not an inference about Retina or DPI from a PNG.

`display-spec.js` is the sole source of all artboard dimensions and display scales. Desktop defaults to normal design size and mobile defaults to fit; fit scale is capped at normal design size to prevent automatic enlargement on wide screens. Original-pixel mode uses `scale=1`, preserving original pixel dimensions and scrolling on both HTML and image sides. Device pixel ratio affects rasterization density only, not CSS logical width.

Do not fit the original and HTML to their containers independently. Header and body anchors remain measured in source-pixel coordinates; display scaling does not change calibrated spacing or font sizes. Size consistency separately covers actual screen bounds, original natural size, mobile scrolling, standalone previews and offline single-file outputs. Retain evidence under the [verification method](../qa/README.md).

## Canvas and content scrolling

HTML embedded in the review tool is a fixed-size artboard with `overflow:hidden` on the root document, preventing classic scrollbars from consuming canvas dimensions and triggering one another. The review page scrolls vertically; the outer canvas pans horizontally only when width is insufficient. `scrollbar-gutter:stable` reserves classic scrollbar width so a threshold height does not repeatedly change fit scale; standalone previews use `auto`. File lists, long properties and operation windows retain scrolling in content containers, and directly opened non-embedded source canvases retain page scrolling. This does not change source canvas, window or component dimensions.

Playwright defaults to `--hide-scrollbars`, so earlier screenshot and size matrices did not cover actual classic scrollbars. Size and preview checks now explicitly remove that argument, checking root scrollbar reservation, visibility of all four window edges, outer horizontal panning and access to the final button in long properties. A canvas 1 px too small and a negative case removing embedding rules reproduce and catch competing scrollbar space. Keep reproduction and fix screenshots in execution artifacts under the [verification method](../qa/README.md); this does not establish scrollbar appearance on every browser or system.

## Header corrections

| Area | Before | Corrected against original |
| --- | --- | --- |
| Active tab | Start x299, width 244 | x293, width 246 |
| Inactive tab | x557, width 244 | x542, width 262 |
| Tab gap | 14 px | 3 px; approximately 1 px uncertainty permitted for softened edges |
| Plus and close buttons | Separate vertical layouts with centers 5 px apart | Same tab row, positioned with approximately 1 px optical offset from the original |
| Close controls | One button and one bare span | Same icon/name/close-control structure and explicit slots |
| Settings icon | Visible graphic small and shifted up-left | Center `(1532,61)`; inspect actual artwork, not only the SVG container |
| Active tab lower border | Overlapped title-bar rule | Joins white toolbar surface, with left-shoulder transition defined independently |
| Search and extract | Horizontal and vertical boundaries shifted | Compare to `(906,101,449,51)` and `(1368,101,183,51)` respectively |
| Toolbar left icons | Layout determined by generic button sizes and gaps | Position and check against original graphic anchors |

Header component slots and typography are centralized in the title-bar, tab and toolbar sections of `hifi.css`, without relying on generic `.icon-button` dimensions or inherited line height. Light and dark themes share geometry. Windows/Linux title decorations remain browser demonstrations; native product windows receive separate acceptance.

## Content below the toolbar

The body covers archive card, directory tree, path bar, table headers and rows, selection, file preview and bottom status bar. Record column boundaries, row heights, padding, icon centers and visible ink and check against the [independent baseline](../../design/html/body-reference.json). Strict geometry and isolated icon-ink tolerances are 2 px; text outlines are recorded only for manual review. Folder optical size is an explicit design revision: the new outline is uniformly enlarged approximately 25.7% in shared assets. Five old folder ink examples remain as comparisons and are checked against independently recorded new design boundaries; other original anchors remain unchanged.

Body review uses equal-size content crops and keeps regional comparisons for sidebar, list, preview and status bar. Before/after screenshots, commits and measurements belong only in the corresponding issue/Actions artifacts, not as a particular run's results in the specification. Mobile review is by region and must not change desktop layout to hide differences.

The original selects one item while leaving Select All empty and labels savings 26% while filling the bar approximately 56%; these are contradictory static states. HTML retains `indeterminate` partial selection and actual 26% fill respectively, as documented semantic exceptions. Preview-specific checks cover normal preview overflow and long-content scrolling; existing interaction checks cover selection, navigation, sorting, search and sample actions. Geometry checks do not replace functional verification.

Save body review, equal-size regional comparisons and measurement evidence separately under the [verification method](../qa/README.md). Preserve explanations of unknown original fonts, generated soft edges, lighting and anti-aliasing differences; this HTML evidence cannot establish corresponding calibration of the Qt product.

## Checks and delivery

Run:

```sh
python scripts/build-html-design.py
python scripts/check-html-header.py
python scripts/check-html-body.py
python scripts/check-html-preview.py
python scripts/check-html-references.py
python scripts/check-html-scale.py
python scripts/check-html-design.py
python scripts/build-html-header-review.py
python scripts/build-html-body-review.py
```

Header checks use independent original anchors: DOM bounds/center deviate by at most 1 px and actual icon-ink boundaries in isolated regions by at most 2 px. White dropdown-arrow ink has a 1 px tolerance; the tab right shoulder samples original white surface and curve with a single-channel tolerance of 10/255 for softening. Glyph outlines are measured and recorded only. Overlays and difference images include anti-aliasing, lighting and glyph differences; total pixel difference must not be converted into fabricated similarity.

The offline header file is `dist/xdzip-header-review.html` and the body file is `dist/xdzip-body-review.html`. Both switch among original, before, after, transparent overlay and pixel differences; body review adds mobile region selection. The single-file UI is `dist/xdzip-hifi-design.html`.

Run these HTML checks and offline reviews locally as needed; they are not default desktop product CI. Investigate anchor failures and inspect overlay evidence even after geometry passes. Main-window HTML checks do not replace calibration of other pages or Qt product and native desktop acceptance.

Normal main-window and menu density follows the [UI standard](design-standard.md). Qt menu/list font-size ratios must be checked together. When reviewing HTML locally, check actual page zoom under the [verification method](../qa/README.md); matching old source-image coordinates does not replace this.
