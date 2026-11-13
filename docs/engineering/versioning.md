# Product version and build identity

The optional `prerelease_number` separates prerelease sequence from build number. For example, alpha.1 can use build 14 while later alpha.2 continues incrementing build. When absent, the historical `channel.build` interpretation remains. Release manifests and asset names are specified in the [release protocol](release-protocol.md).

[VERSION.json](../../VERSION.json) is the sole maintained source for product version, channel and build number. The build reads its current values; this document does not duplicate them. See the [changelog](../../CHANGELOG.md) for version changes and [project issues](https://github.com/xiaodaozhi/xdzip/issues) for deliveries and verification. Incrementing a version alone does not approve a release.

Historical compatibility checks use fixed source baselines together with the current build to verify installation upgrades, rollback and settings compatibility. The [previous product preparation script](../../scripts/prepare-previous-product.py) maintains the full-product baseline; the [settings compatibility script](../../scripts/check-settings-compat.py) maintains the settings baseline. Changing commits and current versions are not copied here. Settings schema and product version are managed independently.

- `version` is `major.minor.patch` without leading zeros. Incompatible changes, compatible features and fixes use their respective version ranges; compatibility boundaries must still be explained during 0.x.
- `channel` supports development, alpha, beta, rc and stable. Non-stable versions display as `version-channel-abbreviation.build`, with development abbreviated to dev; stable displays the base version.
- `build` is an integer from 1 to 65535. Increment it for a new external delivery build, not automatically for every local compilation; it does not automatically create a tag or release. Each numeric version component is also at most 65535 for Windows resources.
- CMake reads JSON directly. `python3 scripts/sync-version.py` updates derived Cargo workspace and HTML versions; `--check` is used in CI. All internal Rust packages inherit the workspace version. Update Cargo.lock after synchronization and run `cargo check --workspace --locked`.
- The Windows four-part numeric resource is the base version plus build; its text version retains the channel. macOS `CFBundleShortVersionString` is the base version, `CFBundleVersion` the build, and the long version retains the channel. Follow [Apple's bundle version documentation](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleversion). Use each platform's permitted numeric format; do not insert Git commits into numeric system fields.
- Windows `FileDescription` is `xdzip Archive Manager` for system locations such as Task Manager. `ProductName` remains `xdzip`; executable filename and association identity do not change with the display name. Build checks read and verify the actual EXE resources.
- Each desktop build generates a short Git commit and dirty flag; without Git it shows unknown. No machine path, username or unstable timestamp is included. The header updates only when contents change; rebuilding after committing yields the correct clean commit identity.

QML module, bridge protocol and settings schema versions are independent of the product version and do not automatically change with it.

`xdzip --version` and Settings → About xdzip read the same build information, as does the preview application. CLI version arguments are handled before creating the GUI. About displays version, build, commit, actual Qt, system and architecture, explicitly marking development channels and uncommitted modifications. Copied JSON contains no personal files, paths, machine names or credentials. The project-home button opens a browser only on user action.

The large About icon uses the platform's corresponding 512 px PNG; HTML uses the vector from the same source rather than enlarging a 72 px UI icon.

About uses the settings window's logical parameters without a footer, and supports Simplified Chinese, English (US), light/dark themes and 100%/125%/150% font sizes. See the current [About settings design](../../design/hifi/sources/settings.html?tab=about). HTML version information is generated from the same JSON; unknown native Qt or commit values are not fabricated.

Verification entries are `scripts/check-build-version.py` against the actual CLI and native version resources, `scripts/test-version.py`, `scripts/check-html-about.py` and `BrowserTest::versionAboutWindow`. Screenshots and measurements stay under `build/review/about/`. Source, system metadata and actual running versions must agree; a version number itself does not establish signing, manual acceptance or compatibility.
