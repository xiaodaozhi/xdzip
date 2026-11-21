# macOS and Linux marketing assets

English | [简体中文](../zh-CN/engineering/platform-marketing.md)

This workflow produces reusable screenshots, backgrounds and promotional images for a website or other presentation. It does not publish anything or assert that the images meet a particular store's submission requirements. The two platforms use the same visual composition as the [Windows materials](../market/windows/README.md), with separate [macOS copy](../../specs/marketing/macos-store.json) and [Linux copy](../../specs/marketing/linux-store.json). Each covers the ten languages in [the product language inventory](../../resources/ui-languages.json) and four scenes: browse, create, preview and workspace.

## Prepare the image renderer

Node.js and the same pinned `sharp` dependency as the Windows generator are required. Install suitable system fonts for the selected languages, including region-specific Chinese, Japanese and Korean fonts when needed, and review the rendered glyphs before use. Install it once in the shared local toolchain:

```sh
npm install --prefix build/toolchain/windows-store-tools --no-save --no-package-lock sharp@0.34.5
```

For backgrounds and illustrations without screenshots, run either or both commands from the repository root. Output directories must not already exist.

```sh
node scripts/build-macos-store-assets.cjs --output build/macos-marketing
node scripts/build-linux-store-assets.cjs --output build/linux-marketing
```

Omit `--output` for a timestamped directory under `build/<platform>-store-assets/`. Add `--4k` for additional 3840 × 2160 backgrounds, illustrations and composites. Artwork includes text, backgrounds with screenshot frames, feature illustrations, icons, avatars, posters, square artwork and heroes. A background-only export contains no screenshots or composites.

## Capture real application windows

Capture on the target OS using the actual xdzip build. By default the recorder requests all ten languages. Use `--languages ja-JP fr-FR` to select only the languages needed. Prepare each scene and language in the application, using non-sensitive demonstration data. Review each window for correct content and unobstructed presentation. On macOS, interactive capture uses `screencapture` and asks you to select a window. On Linux it uses `gnome-screenshot`; install it if necessary and activate the desired window during the countdown. Scene preparation and window selection are manual; this workflow does not run the Windows UI automation on other platforms.

| Scene | Prepare in the application |
| --- | --- |
| `01-browse` | Browse the archive tree, select a file and show its properties. |
| `02-create` | Open archive creation for selected files and show ZIP settings. |
| `03-preview` | Show an image preview beside the file list and properties. |
| `04-workspace` | Open two archives in tabs and show the selected files in grid view. |

Calculate the executable's SHA-256 (`shasum -a 256 /path/to/xdzip` on macOS, `sha256sum /path/to/xdzip` on Linux). Then use the matching platform and executable path, replacing the placeholders:

```sh
python3 scripts/capture-platform-store-assets.py --platform macos --executable /path/to/xdzip --expected-sha256 LOWERCASE_SHA256 --interactive --output build/macos-captures
python3 scripts/capture-platform-store-assets.py --platform linux --executable /path/to/xdzip --expected-sha256 LOWERCASE_SHA256 --interactive --output build/linux-captures
```

On macOS select the executable inside the `.app/Contents/MacOS/` bundle. The recorder reads its `--version-json` identity, checks the digest before and after capture, and records the operating system, architecture, scene, language, PNG dimensions and digest. `--source-revision` optionally records a full commit matching the executable's abbreviated revision. The identity belongs to the chosen executable: it does not prove which window was captured. The operator must verify the screenshot subject and scene.

Alternatively, arrange already captured native PNGs under one directory per selected language, such as `ja-JP/` and `fr-FR/`, using `01-browse.png`, `02-create.png`, `03-preview.png` and `04-workspace.png`, then replace `--interactive` with `--from-directory /path/to/images`. Run the recorder on the corresponding OS with the actual build. The receipt records these as operator-supplied; it cannot establish when, where or from which executable the files were created.

To synchronize a completed export, see [latest-only Drive sync](marketing-archive.md).

## Export and review

```sh
node scripts/build-macos-store-assets.cjs --captures build/macos-captures --output build/macos-marketing-with-screenshots
node scripts/build-linux-store-assets.cjs --captures build/linux-captures --output build/linux-marketing-with-screenshots
```

The exporter validates the platform, complete four-scene set for each captured language, screenshot digests and dimensions, and refuses to overwrite an existing output. Per language, `screenshots/` contains original PNGs, `backgrounds/` contains the reusable frames, `promotional/` contains composites only for captured languages, and `feature-illustrations/`, `branding/` and `text/` contain other artwork and copy. `capture-source/` preserves the original receipt and PNGs for re-export. `manifest.json` records the output roles and source identity, and `SHA256SUMS.txt` covers all exported files including the manifest. Screenshots are not enlarged; review the generated images and their provenance before use. Do not commit captures or exports.

Run `python3 scripts/test-platform-store-captures.py` and `node scripts/test-platform-store-assets.cjs` for the focused capture and rendering checks. The rendering test uses synthetic fixtures, not actual application screenshots.
