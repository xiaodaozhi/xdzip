# Microsoft Store materials

English | [简体中文](../../zh-CN/market/windows/README.md)

[Build and submit the Store package](build.md).

[Copy-ready listing text](listing.md) contains the description, ten product features, supplemental fields, seven keywords, copyright, developer information, links and image captions. Use the English page for `en-US` and the [Chinese page](../../zh-CN/market/windows/listing.md) for `zh-CN`.

The listing targets Windows 11 x64 under the [support policy](../../../docs/engineering/support-policy.md). Store updates are managed by Microsoft Store. The Store build declares modern and classic Explorer menus, and modern commands require a Microsoft Store signature.

Use the [local export and capture guide](capture.md) to get the bilingual submission files. For routine package updates with accurate existing artwork, carry forward the images in Partner Center and update the release notes as needed.

## Text fields

| Partner Center field | Material or decision |
| --- | --- |
| Product name | `xdzip`, unchanged in both languages |
| Description | Plain-text block in [listing.md](listing.md), below 10,000 characters |
| What's new | Localized text in [listing.md](listing.md#whats-new-in-this-version) |
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

Export the approved capture set or [capture both languages from a verified MSIX](capture.md). Open the exported `preview.html` and review the PNGs at their intended size. The capture uses default window dimensions and owned demonstration files. The exporter trims the top pixel row, cleans the corners and adds a thin inner border before compositing the window with localized copy and a soft shadow.

Upload the four **1920 × 1080** combined images in each language's `promotional/` directory to the desktop screenshot field in this order.

| Order | Scene | File |
| --- | --- | --- |
| 1 | Browse folders and select a file | `01-browse-1920x1080.png` |
| 2 | Creation window and ZIP settings | `02-create-1920x1080.png` |
| 3 | On-demand image preview | `03-preview-1920x1080.png` |
| 4 | Multiple tabs and grid browsing | `04-workspace-1920x1080.png` |

Each export also includes plain app views under `screenshots/` and labeled feature illustrations under `feature-illustrations/`. Use `--4k` to add 3840 × 2160 artwork. [Listing copy](listing.md) provides the matching captions.

## Icons, avatars and Store artwork

Each language's `branding/` directory includes 71, 150 and 300 px app icons, 512 and 1024 px avatars, posters at 720 × 1080 and 1440 × 2160, square artwork at 1080 and 2160 px, and a 1920 × 1080 text-free hero image. The optional 4K export also includes a 3840 × 2160 hero image.

## Editing and regeneration

Edit [the marketing specification](../../../specs/marketing/windows-store.json) for bilingual copy and the [artwork generator](../../../scripts/build-windows-store-assets.cjs) for composition. Refresh the Markdown copy with `node scripts/build-windows-store-assets.cjs`. Use the [local export command](capture.md#export-from-an-approved-capture-set) to re-render archived screenshots with updated copy or artwork. The [preview template](../../../design/hifi/sources/windows-store-artwork.html) is included in the export.

After submitting updated images, copy the reviewed export directory to your backup location. Keep `capture-source/`, the uploaded images and `manifest.json` so you can re-render or reproduce the submission. The manifest records the capture package separately from the listing export version. Root README screenshots are maintained independently.

## Reviewer fields

[Reviewer notes](reviewer-notes.md) provide copy-ready `runFullTrust` justification and certification instructions in both languages. Use them in their corresponding submission fields. The material pack does not indicate approval of a restricted capability or submission for certification. Pricing, age ratings and account-level questionnaires remain separate Partner Center sections.

## Official references

The [Store listing field reference](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info) defines descriptions, features and supplemental fields. The [MSIX image reference](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images) defines screenshot and artwork guidance. The current Partner Center form can expose more upload slots than the documentation; this pack provides four app screenshots and four feature illustrations per language and does not infer additional device support from available fields.
