# Microsoft Store materials

English | [简体中文](../../zh-CN/market/windows/README.md)

[Copy-ready listing text](listing.md) contains the description, ten product features, supplemental fields, seven keywords, copyright, developer information, links and image captions. Use the English page for `en-US` and the [Chinese page](../../zh-CN/market/windows/listing.md) for `zh-CN`.

These materials target MSIX **1.0.27.0**, package identity `xiaodaozhi.xdzip`, from source `90916a1992188e6e7e5c6b3d91695861ee1bd2d2`. They do not advertise the later format expansion in the current development tree. The [support policy](../../engineering/support-policy.md) remains Windows 11 x64; a manifest's technical Windows 10 minimum does not extend that commitment. Store updates are managed by Microsoft Store, and this candidate does not register a modern Explorer context menu.

To capture updated app views, use the [automatic bilingual screenshot recipe](capture.md). It generates its own demo files and can target a newer build with an explicit executable hash. The [reusable capture module](../../engineering/windows-screenshots.md) also supports other Windows workflows.

## Text fields

| Partner Center field | Material or decision |
| --- | --- |
| Product name | `xdzip`, unchanged in both languages |
| Description | Plain-text block in [listing.md](listing.md), below 10,000 characters |
| What's new | Empty for the first submission, rather than pasting internal test notes |
| Product features | Ten separate entries, each below 200 characters; the portal adds bullets |
| Short title | `xdzip` |
| Sort title | Empty; no alternate alphabetical title is needed |
| Voice title | Empty, because this submission targets desktop rather than Xbox |
| Short description | Localized copy below the recommended 270 characters |
| Keywords | Seven entries, each below 40 characters, within 21 space-separated words |
| Copyright and trademarks | Localized copyright notice below 200 characters |
| Additional license terms | Empty; no new Store-specific supplemental terms are introduced |
| Developer | `xiaodaozhi` |
| Website, support and privacy | Dedicated links in [listing.md](listing.md), not URLs embedded in the description |

The app's existing Apache-2.0 license and third-party notices are unchanged by the blank supplemental-terms field. Pricing and legal acceptance are not determined by this material pack.

## App screenshots and combined promotional artwork

To prepare replacement images, use the [automatic native PNG capture script](capture.md). One run prepares and captures all four views for either language, without manually switching views or confirming individual captures.

Each language has four actual app views using owned demonstration files. Their app build matches the candidate identified above and uses the default logical window dimensions. Published PNGs are **1536 × 932** for the main window and **737 × 1103** for the creation window: one top border row is removed, and old desktop colors are cleared from the corner squares before rounded transparency is applied. Pixels elsewhere remain unchanged, with no text redrawing or resizing. The original native PNGs remain byte-identical in [the source assets](../../../design/assets/marketing/windows-store-native). The root READMEs use the cleaned `01-browse` image in their matching language.

Use the **2560 × 1440 Store screenshots** for the desktop screenshot field. They center the cleaned app view at 1:1 on a neutral background, without marketing overlays. Combined promotional artwork places the same view in the blue brand layout with localized copy, a thin border and a soft shadow. At 1080p the UI is downsampled once to fit; at 4K it remains at native 1:1 size. Neither export upscales the UI. The lower-left platform label is omitted.

| Scene | Default window | Store screenshot | 1920 × 1080 artwork | 3840 × 2160 artwork |
| --- | --- | --- | --- | --- |
| Browse folders and select a file | [PNG](images/screenshots/01-browse.png) | [PNG](images/store-screenshots/01-browse.png) | [PNG](images/combined/01-browse-1920x1080.png) | [PNG](images/combined/01-browse-3840x2160.png) |
| Creation window and ZIP settings | [PNG](images/screenshots/02-create.png) | [PNG](images/store-screenshots/02-create.png) | [PNG](images/combined/02-create-1920x1080.png) | [PNG](images/combined/02-create-3840x2160.png) |
| On-demand image preview | [PNG](images/screenshots/03-preview.png) | [PNG](images/store-screenshots/03-preview.png) | [PNG](images/combined/03-preview-1920x1080.png) | [PNG](images/combined/03-preview-3840x2160.png) |
| Multiple tabs and grid browsing | [PNG](images/screenshots/04-workspace.png) | [PNG](images/store-screenshots/04-workspace.png) | [PNG](images/combined/04-workspace-1920x1080.png) | [PNG](images/combined/04-workspace-3840x2160.png) |

[listing.md](listing.md) provides captions for all four views. The creation window contains three selected demo images and ZIP settings; no compression task was started. The screenshot set uses the light theme, while the listing's appearance options remain as described in the candidate's features.

## Feature illustrations

The original vector artwork remains available for promotion, with an explicit **Feature illustration** label. Use the app captures above for the screenshot field.

| Artwork | 1920 × 1080 | 3840 × 2160 |
| --- | --- | --- |
| Browse and selective extraction | [PNG](images/promotional/01-browse-1920x1080.png) | [PNG](images/promotional/01-browse-3840x2160.png) |
| Creation and supported password protection | [PNG](images/promotional/02-create-1920x1080.png) | [PNG](images/promotional/02-create-3840x2160.png) |
| On-demand content previews | [PNG](images/promotional/03-preview-1920x1080.png) | [PNG](images/promotional/03-preview-3840x2160.png) |
| Multiple archives and appearance choices | [PNG](images/promotional/04-workspace-1920x1080.png) | [PNG](images/promotional/04-workspace-3840x2160.png) |

Captions are localized and below 200 characters. Use the matching language's image set rather than presenting English artwork as localized Chinese UI.

## Icons, avatars and Store artwork

These are xdzip's product assets, not Microsoft corporate logos or a certification badge. Small icons reuse the existing vector brand without redrawing it. Locale-neutral icons and text-free hero artwork are shared by both listings.

| Field or use | Files | Notes |
| --- | --- | --- |
| App tile icon | [300 × 300](images/branding/app-icon-300.png) | Recommended app icon override for Windows 10/11 Store layouts |
| Smaller icon overrides | [150 × 150](images/branding/app-icon-150.png), [71 × 71](images/branding/app-icon-71.png) | Matches the fields shown by Partner Center |
| Reusable avatars | [512 × 512](images/branding/avatar-512.png), [1024 × 1024](images/branding/avatar-1024.png) | Additional brand exports, not separate fields on the observed page |
| Poster art | [720 × 1080](images/branding/poster-720x1080.png), [1440 × 2160](images/branding/poster-1440x2160.png) | These pixel dimensions are **2:3**, despite the portal's “9:16” label |
| Square box art | [1080 × 1080](images/branding/box-art-1080.png), [2160 × 2160](images/branding/box-art-2160.png) | Localized tagline; optional for this desktop app |
| Text-free super hero art | [1920 × 1080](images/branding/hero-1920x1080.png), [3840 × 2160](images/branding/hero-3840x2160.png) | No product title, text, UI or device frame |

The poster and square artwork keep the title and primary mark in their upper two-thirds. The hero composition puts its primary mark centrally and above the bottom overlay area. All images are PNGs below 5 MB per file, meeting the portal's 5 MB icon and 50 MB large-art limits.

Xbox-only branded key art (584 × 800), titled hero art and featured promotional square are intentionally not supplied or enabled: the current package targets `Windows.Desktop`. No Xbox, Holographic or mobile support is claimed. A video trailer is optional and is not included in this image pack.

## Editing and regeneration

The [design gallery](../../../design/hifi/sources/windows-store-artwork.html) is the high-fidelity entry point. Editable composition lives in [build-windows-store-assets.cjs](../../../scripts/build-windows-store-assets.cjs); localized copy, palette and export canvas parameters live in [windows-store.json](../../../specs/marketing/windows-store.json). The [asset inventory](../../../specs/marketing/windows-store-assets.json) records PNGs and source/output hashes, including native inputs and the two README images. The [capture inventory](../../../specs/marketing/windows-store-captures.json) records the candidate, native capture/session hashes, client crops, dimensions, DPI, captions and corner processing. These fixed export canvases do not change Qt logical window dimensions or incorporate screen scaling into the product.

To replace the screenshots, run the native capture recipe for both languages and review the original PNGs at 100%. Pass the two complete session directories to [import-windows-store-captures.cjs](../../../scripts/import-windows-store-captures.cjs). It preflights all eight PNGs, their candidate identity, hashes and exact client/window pixel correspondence before updating the material inputs. JPEG inputs and incomplete or mismatched sessions are rejected. When updating the candidate, update the marketing baseline deliberately before importing.

With Node.js and `sharp` available:

```sh
node scripts/import-windows-store-captures.cjs build/captures-en-US build/captures-zh-CN
node scripts/build-windows-store-assets.cjs
node scripts/build-windows-store-assets.cjs --check
node scripts/test-windows-store-images.cjs
python scripts/check-docs.py
python scripts/package-windows-store-materials.py --output dist/windows-store-materials
```

Use a fresh export path. The packaging script produces a directory and ZIP with separate `en-US` and `zh-CN` folders, 62 upload/promotion PNGs, copy-ready text, caption CSV, supplemental fields, reviewer notes and SHA-256 manifests. Raw capture inputs and cutouts stay in the repository and are excluded from the upload bundle. It contains no product binaries or source archives.

The check validates field limits, language parity, PNG dimensions, transparency, byte limits, exact inventory and source/output hashes. It checks opaque UI pixels against the original cutout, transparent corners against the background, native 1:1 Store/4K placement, stale listing text and the text-free hero composition. Regenerate both languages together after changing the target candidate or copy.

## Reviewer fields

[Reviewer notes](reviewer-notes.md) provide copy-ready `runFullTrust` justification and certification instructions in both languages. Use them in their corresponding submission fields. The material pack does not indicate approval of a restricted capability or submission for certification. Pricing, age ratings and account-level questionnaires remain separate Partner Center sections.

## Official references

The [Store listing field reference](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info) defines descriptions, features and supplemental fields. The [MSIX image reference](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images) defines screenshot and artwork guidance. The current Partner Center form can expose more upload slots than the documentation; this pack provides four app screenshots and four feature illustrations per language and does not infer additional device support from available fields.
