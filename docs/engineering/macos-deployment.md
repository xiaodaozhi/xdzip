# Local macOS development DMG

The dedicated Release workflow permits native deployment, necessary ad-hoc signing, app.zip and DMG packaging, and uploads. The on-demand Qt workflow's macOS job retains SDK verification. One deployed, signed, verified .app is used for both release packages; the unzipped ZIP also receives relocated startup verification. See the [release protocol](release-protocol.md).

A local development DMG is an allowed development deliverable. The on-demand Qt workflow verifies native Apple Silicon build/startup without packaging, signing, or uploading a Mac app. Local scripts do not install the application, change default associations, upload files, or read release credentials.

## Build and verify

Environment preflight and stage options follow the [developer build guide](developer-build.md). Prepare packaging tools, build a separate Release primary app from a clean commit, then package and verify:

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

Stages do not implicitly rerun predecessors. The default Qt SDK is `build/toolchain/Qt/6.8.2/macos`; pass `--qt-prefix` to each command for another location. Ordinary development/test builds default to `None`; the primary app above explicitly selects `Alternate` without changing user defaults. Local macOS package supports only `all`. The on-demand Qt workflow does not run packaging or upload Mac apps; see the release protocol for the dedicated Release exception.

Before macdeployqt, shared package checks native arm64, system SDK, deployment tools, Rust PATH/pinned metadata, Python validation dependencies, and Qt license/schema materials. build and test permit uncommitted source; package requires a clean commit and matching actual CLI identity and cannot bypass these with compilation caches. System compiler, SDK, lipo, codesign, and hdiutil are supplied by the system toolchain, not installed by preparation.

macdeployqt also relocates the image worker. Qt 6.8's default platform-plugin selection includes only Cocoa, so the packaging script explicitly copies offscreen before scanning app libraries to process its Qt framework dependencies. Missing plugins or workers refuse packaging. After integrity signing and import auditing, clear Qt/QML/DYLD search overrides and verify the actual bundled image worker's format set and an owned PNG decode; the receipt is `image-preview-deployed.json`. Script unit checks do not establish deployed-package acceptance.

Underlying [build-macos-dmg.py](../../scripts/build-macos-dmg.py) and [verify-macos-dmg.py](../../scripts/verify-macos-dmg.py) retain image and native-verification details. Use their `--help` for troubleshooting rather than maintaining separate routine steps. Underlying output paths must be new and inside build/. Every document type in the DMG input must declare `Viewer/Alternate`; otherwise rebuild rather than rewriting plist or signatures on the original bundle. macdeployqt scans actual QML and deploys Qt frameworks, plugins, and QML. Add Qt system-control translations, project notices, Rust upstream notices, font licenses, source SBOM, and pinned Qt license materials. Retain framework-internal symlinks and reject links escaping the app. Remove developer-machine absolute RPATHs and recursively check that all native imports resolve only to the app or system. Build identity must be neither dirty nor unknown and must match the current clean commit to bind actual notices.

Qt optional QML LocalStorage brings in QtSql, so deployment also copies database drivers. The project has no server-database capability; remove drivers other than built-in SQLite to avoid PostgreSQL, ODBC, Mimer, and other SDK-external clients in the image. Record removals in the receipt and do not hide unresolved imports as passes.

## Architecture and style trimming

Development DMGs deliver Apple Silicon `arm64` only. Main-program CMake architecture settings do not constrain Qt SDK files copied by macdeployqt, so recursively inspect every Mach-O in the app, including frameworks, dynamic libraries, plugins, and QML native plugins. Trim dual-architecture files to `arm64` only in the newly created packaging copy. Missing `arm64`, unrecognized architectures, or remaining extra slices fail packaging; main-executable architecture does not replace nested checks. Trim before RPATH rewriting and recursive ad-hoc signing.

The product uses Qt Quick Controls `Basic`. Deployment removes only explicitly listed non-Basic styles and dedicated imports/implementation plugins, retaining Basic, shared controls, Templates, and actual QML dependencies. Do not shrink by deleting unknown modules. Unexpected style layouts, missing retained dependencies, or unused-style remnants fail auditing. Product-style or Qt SDK changes require synchronized trimming rules and tests. Size reduction does not omit Qt licensing, corresponding-source materials, system translations, or necessary runtime plugins.

Optimize only the newly staged app under bounded `build/`, not the input primary app, Qt SDK, or source. Receipts retain architecture/style checks, trimmed binaries and removal lists, and actual logical app bytes before/after optimization with framework, plugin, QML, and other categories. Bytes are counted from actual file content without double-counting symlinks; they are not Finder disk usage or UDZO image size. Final DMG size is recorded separately. Version-specific comparisons, verification conclusions, and remaining work go in issues; raw measurements remain local artifacts.

`scripts/test-macos-bundle.py` uses small project-owned fixtures in cross-platform CI to verify architecture recognition, copy-only operation, symlink boundaries, style retention/removal, and failures without macOS deployment tools. It does not replace final package verification. DMG checks must recursively confirm only `arm64` in all native files and contract-compliant styles after copying from the mounted image, and execute SDK search isolation and Cocoa/Metal flows below.

## Image and runtime verification

The UDZO-compressed image contains xdzip.app, an Applications link, and installation instructions; run hdiutil verify. Quit old versions normally before dragging into Applications. There is no updater or task migration during installation in this flow. Settings and recent items remain in user directories and are not automatically deleted when removing the app.

Keep deployment-verification JSON and a CycloneDX file-level SBOM beside the image, bound to the same-commit source inventory, final native-file SHA-256 values, and import relationships. The SBOM remains incomplete; file enumeration is not exact component attribution or license approval. Keep the inventory outside the image to avoid recursive self-hashing.

The new staged `xdzip.app` exists only during deployment, signing, image creation, and verification and is automatically reclaimed on both success and failure. DMG, JSON, SBOM, and instructions remain; the input primary app is unchanged. No loose staged bundle remains for the system to rediscover.

Modified ARM64 Mach-O files need valid code directories, so use local ad-hoc signing after copying/rewriting and verify nested code. **This is not Developer ID signing or notarization and does not imply Gatekeeper trust.** Under the [personal open-source release policy](release-runbook.md), formal signing and notarization are not mandatory release conditions; dependency, license, security, and actual-package verification remain required. Do not disable SIP or delete global user security policies.

The verification script mounts the image read-only, checks the Applications link, copies to an isolated path with Chinese characters and spaces, and recursively checks arm64, Basic styles, and strict signatures on every native file and the bundle. It launches and closes using Cocoa/Metal after clearing SDK/QML/plugin search variables. The actual product from the image opens a self-generated ZIP, performs query, sort, extraction, and exit, independently verifies Unicode, empty files/directories, and every output byte, and checks build identity, architecture, and Metal backend at the actual first frame. Then, only in the verification copy, modify QtCore Mach-O library identity and re-sign ad-hoc, then recheck architecture, style, signatures, and startup. This proves deployed modification/replacement is allowed; it does not prove Qt was rebuilt from corresponding source, arbitrary ABI compatibility, or legal compliance approval.

app.zip verification also uses the product's default same-name folder option to extract the entire app, compares file bytes, link targets, and safe permissions item by item against `ditto`, then runs deep strict signature checks and Cocoa/Metal startup on the extracted real app. The outer directory must not end in `.app`. Temporary apps and registrations newly created during the check are cleaned afterward. This does not change signature-trust or notarization boundaries.

The local machine still has an SDK; import checks and environment clearing do not replace a separate clean Mac. Finder drag-in, system Open With, default selection, Gatekeeper, real restart, and signing/notarization remain subject to the [manual checklist](../qa/private-test-manual.md).

Native menu edit commands and Qt standard controls need system-locale qtbase translations. In addition to copying `Contents/Resources/translations/qtbase_*.qm`, packaging sets `Translations = Resources/translations` in `[Paths]` of `Contents/Resources/qt.conf`, retains existing plugin/QML paths, and writes it before integrity signing. Relocated startup gates check both native edit responders and application menu roles; SDK development-directory checks alone are insufficient.

## Local Finder Sync signing and enablement

Finder Sync uses a native appex and independent relay inside the same app; the service helper remains. Complete signing order is native images, frameworks and dedicated helpers, appex with exact sandbox entitlements, relay bundle, and outer app, followed by strict verification of the nested bundle. Do not overwrite a signed appex afterward with generic signing lacking entitlements. Ordinary CI builds do not implicitly sign or enable the extension.

For local Finder verification, complete the default native build, then explicitly run:

```sh
python3 scripts/sign-macos-finder.py --app build/desktop/apps/desktop/xdzip.app --output build/review/finder-signing.json
build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip --register-shell-integration
```

The signing script accepts only project-owned apps and output under repository `build/`; it does not deploy Qt, create a DMG, or read release credentials. Its receipt establishes ad-hoc integrity and strict nested verification only, not Developer ID, notarization, Gatekeeper trust, or release licensing. The app must remain at its original location. Enable xdzip's Finder extension in system Settings under login items and extensions. Settings paths and menu layouts may vary by OS version; successful registration does not establish menu/action acceptance.

Before ending temporary verification, use the same app's `--remove-shell-integration` to revoke owned service and extension registrations. After checking the initial state, clean only configuration and temporary LaunchServices registrations created by this work. Do not alter other apps or default associations. If Finder has loaded the app, disable and re-enable the owned extension in system Settings after updates rather than restarting Finder. See [file-manager actions](shell-actions.md) for handoff and rollback boundaries.

## Open With registration for development and verification copies

LaunchServices remembers app bundles opened previously; multiple copies of the same bundle ID in a build tree may appear in system or Codex Open With menus. Do not delete the system database, restart Finder, delete verification packages, or change user defaults for this reason.

New development bundles default to `LSHandlerRank=None` in generated Info.plist; only the chosen primary app and delivered package retain `Alternate`. CMake option `XDZIP_MACOS_HANDLER_RANK` strictly allows these two values. Offline source rebuilds, source Qt product verification, and Mac product CI explicitly pass `None` so future default changes cannot reintroduce test copies as candidates. [Apple's definition of `None`](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html) retains file drag-in without selecting the bundle as an open handler for the type. It does not affect in-app file selection or explicit application opening and does not set defaults.

Mac product CI runs `scripts/check-macos-registration.py` to query file candidates for the new bundle through native `NSWorkspace`. Candidate count and registrations enumerated by bundle ID are evaluated separately; the latter does not replace actual user-facing Open With candidates. Existing default handlers must remain during temporary registration. Types with no prior handler may resolve temporarily to a new candidate; all tested types must regain original handlers and candidate states after deregistration. Current-user `LSHandlers` preferences are compared only as in-memory snapshots before, during, and after registration; their contents are not output. Other preferences and system-level rules are outside this comparison.

`scripts/macos_launch_services.py` recognizes only `org.xdzip.desktop`. DMG relocation, offline source rebuild, source Qt product verification, QtCore replacement, and system-file-open verification record pre-run registrations within bounded `build/` and, on completion or failure, deregister only newly added test copies. Existing primary apps and other products remain. Cleanup failure fails verification rather than being silently reported as success.

DMG relocation and QtCore replacement retain delivered bundles' original document declarations and verification but copy only to new, nonexistent `.app` paths under `build/`. Automatically reclaim the newly created bundle after verification or failure, freezing necessary hashes inside its scope. JSON, logs, the pre-replacement primary bundle, DMG, and source Qt remain. Cleanup does not accept existing apps or symlinks and does not retrospectively delete historical verification bundles. Old `Alternate` bundles that were merely deregistered may be rediscovered and registered by the system or other apps; permanent nonrecurrence cannot be guaranteed. If old bundles must be retained, they may be placed into verifiable, restorable backups before handling loose copies under explicit authorization.

Preview historical leftovers with the following tool, then explicitly apply. `--keep` must name an existing correctly identified primary app, either the current development build or an installation in Applications:

```sh
python3 scripts/cleanup-macos-open-with.py --keep build/ui-release/apps/desktop/xdzip.app
python3 scripts/cleanup-macos-open-with.py --keep build/ui-release/apps/desktop/xdzip.app --apply
```

The tool deregisters only other xdzip copies within this repository's `build/` and invalid paths with the same identity. It does not delete bundles, alter signatures, or set the default ZIP app. Different products, other Applications installations, and paths outside the build tree are not handled automatically. Paths resolving outside scope through symlinks are not cleaned. Recheck registrations afterward, retaining only the specified primary development bundle. If multiple installations exist outside the build tree, list them separately as unhandled; do not claim global deduplication.

Record cleanup progress, counts, and run conclusions in the relevant issue, such as [development-copy maintenance #7](https://github.com/xiaodaozhi/xdzip/issues/7). Generated JSON and native probe results remain local/CI artifacts only.
