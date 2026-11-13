# File association icons

System file icons reuse xdzip's folded-corner document, continuous-curvature badge, four interwoven bands and open U-shaped mark. The design entry is the [high-fidelity review page](../../design/hifi/sources/file-association-icons.html), parameters come from [file-association-icons.json](../../specs/visual/file-association-icons.json), and document icon sources and product platform containers reside in [file-associations](../../design/assets/file-associations). Users still choose their default applications.

## Shape and identification

Paper and brand geometry reuse [brand_assets.py](../../scripts/brand_assets.py). Formats share the same paper, fold, badge position and optical center, with labels identifying formats and accent colors aiding scanning. TAR.GZ, TGZ and GZ form a teal visual family. The generic fallback uses neutral gray-blue and document lines without inventing unsupported formats such as ARC.

Labels strictly follow the original ZIP glyph style: height 61, stroke 14, square ends, rounded joins, top at master Y=817 and color #39465D. ZIP directly reuses the original master's complete text path; other letters are completed under the same metrics and stroke rules. No SVG text, system fonts, third-party icons or embedded bitmaps are used. Maximum width is 504. Preserve the full TAR.GZ label without stretching glyphs; only scale uniformly and vertically center when it exceeds the width budget. Color is not the only means of identification.

The master is 1024 units. Generic PNG/ICO retain the existing `32 32 960 960` viewBox, with paper body approximately 90% high; macOS PNG/ICNS retain `-20 -20 1064 1064`, with body approximately 81.2% high. This continues project optical calibration and is not claimed as a system requirement. Icons use transparent canvases without theme-specific paper, desktop backgrounds or extra rounded masks.

At 16 and 24 pixels, badges are enlarged and paper simplified; 32 pixels uses a slightly smaller badge. These three sizes omit labels and large-image shadows while preserving the brand mark's original aspect ratio. Icons alone cannot reliably distinguish every format at these sizes, particularly the same-color GZIP family; filenames/type columns provide precise identification. Labeled masters are used from 48 pixels; long labels are more legible at 64 pixels and above. Render each size directly from vectors rather than repeatedly shrinking a 1024 PNG.

The UI's 24-unit icons and main-window/menu density continue to follow [layout-scale.json](../../specs/visual/layout-scale.json). These assets add no QML window and change no logical main-window or menu dimensions. Platform PNG dimensions are image sampling dimensions and must not be written into QML component logical geometry.

## Format and platform mapping

| Icon ID | Label | Extension / use | Accent |
| --- | --- | --- | --- |
| `zip` | ZIP | `.zip` | Brand blue |
| `7z` | 7Z | `.7z` | Indigo |
| `rar` | RAR | `.rar` | Purple |
| `tar` | TAR | `.tar` | Amber |
| `tar-gz` | TAR.GZ | Confirmed compound `.tar.gz` suffix | Teal |
| `tgz` | TGZ | `.tgz` | Teal |
| `gz` | GZ | `.gz` system candidate | Teal |
| `cab` | CAB | `.cab` | Steel blue |
| `iso` | ISO | `.iso` | Lake blue |
| `udf` | UDF | `.udf` | Cyan-blue |
| `archive` | None | Fallback for recognized archives without a dedicated icon | Neutral gray-blue |

This table states design intent, not a uniform cross-platform registry. Where possible filename mapping prioritizes full `.tar.gz` before `.gz`; systems recognizing only the final extension may show GZ for both. Do not promise all platforms can distinguish compound suffixes automatically. The GZ icon denotes an extension only: the project still supports TAR.GZ contents rather than standalone GZIP extraction.

Encryption and volume splitting are archive states, not inferred from extension icons; do not draw locks or success marks. Recognized ZIP, 7z and RAR volumes may reuse their family in-app, without registering generic `.001`, `.rNN`, `.zNN` or `.img`. ISO/UDF icons do not imply mounting or execution. A matching extension alone does not establish valid archive contents.

Windows uses per-extension ProgIDs and ICOs while retaining the old shared entry; macOS maps document types to ICNS, with compressed TAR types sharing the TAR.GZ icon. Linux retains system MIME icons without overriding global file icons. Native display depends on default applications and icon caches; see the [installation and association contract](../engineering/installers-associations.md) for association and rollback.

The Windows default-app and Open With application entry uses the [brand application ICO](../../design/assets/brand/xdzip.ico), embedded in both the main executable and association launcher. Per-format document icons do not replace application identity. The system should display one optional xdzip entry with the brand icon; the old shared command serves compatibility with existing choices. This reuses existing assets and does not change main-window, menu or QML logical geometry. Native application icons, document icons and candidate counts are accepted separately; an OS generic application box must not be classified as a format-document icon problem.

## Generation and delivery

Run from the repository root:

```sh
python scripts/build-association-icons.py
python scripts/build-design-assets.py --metadata-only
python scripts/build-association-icons.py --check --check-runtime
# Export local artifacts using existing Pillow, Playwright and Chromium.
python scripts/build-association-icons.py --export build/file-association-icons
python scripts/build-association-icons.py --check --export build/file-association-icons
```

`XDZIP_CHROMIUM` can identify an existing Chromium/Edge path. SVG masters, small-size sources, specification and high-fidelity review page are versioned; asset sources are registered in the main [manifest.json](../../design/assets/manifest.json). The generation entry is [build-association-icons.py](../../scripts/build-association-icons.py), with composition definitions in [association_icon_assets.py](../../scripts/association_icon_assets.py).

Exported artifacts contain generic and macOS PNGs from 16 through 1024 pixels per format, plus ICO and ICNS. The nine ICOs and nine ICNS files used by the product reside beside vector sources in `file-associations/`; ZIP containers retain their original pre-migration bytes and the others come from corresponding exports. Ordinary builds read these containers directly and do not require a browser. Temporary PNGs, complete exports, contact sheets and receipts stay in `build/` or `dist/`.

## Verification and acceptance

Generation checks verify SVGs match the specification and have no font or external-image dependencies. Platform checks cover canvas and RGBA, transparent corners, visible body height and centering, vector digests and decoded pixel agreement for each ICO/ICNS representation. Body bounds are measured at alpha ≥ 128, using the brand tolerance of 1.25 physical pixels.

Visual review covers light, dark and selected backgrounds, sizes 16 through 256 pixels, and actual rendering at normal and high DPR. The review page uses CSS pixels; browser screenshots must not be labeled Explorer, Finder or Linux native acceptance. Later integration must check real filenames, compound suffixes, icon caches, before and after default selection, and display after installation/uninstallation. Record stage status and actual execution scope in GitHub issues rather than accumulating test reports here.
