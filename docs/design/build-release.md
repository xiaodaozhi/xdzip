# xdzip: 11 Build, release, licensing, and updates

English | [简体中文](../zh-CN/design/build-release.md)

## 1. Support matrix

The application display name and executable basename are consistently `xdzip`; platform artifacts are `xdzip.app`, `xdzip.exe`, and `xdzip`, with `xdzip.desktop` as the Linux desktop entry. Production Rust modules use the `xdzip-` prefix. Releases use the project's existing bundle ID. A paid signing identity is not required for personal open-source GitHub distribution; see the [release runbook](../../developers/engineering/release-runbook.md). The project owner approved the initial support scope and maintenance approach on 2026-10-19. The authoritative description is the [support policy](../engineering/support-policy.md) and [ADR-011](architecture-decisions.md#adr-011首批支持范围与个人项目维护政策).

| Platform | Initial target | Minimum system and scope | Release verification |
|---|---|---|---|
| macOS | Apple Silicon arm64 | macOS 14 and later; no promise for every future OS version | Native GPU/IME/VoiceOver, ad-hoc integrity, operation without a development environment |
| Windows | x86_64 MSVC | Maintained versions of Windows 11 | Unicode/DPI/long paths/NTFS, unsigned package, clean environment |
| Linux | x86_64 | Ubuntu 24.04 LTS, X11, glibc 2.39 and later | X11 platform plugin, isolated execution, native file APIs on the target distribution |

Initial releases do not provide Intel Mac, Universal, Windows ARM64, or Linux ARM64 packages and do not promise Windows 10, other Linux distributions, or Wayland. Supported scope does not mean every OS version has been individually covered; expanding scope requires actual verification. This personal project prioritizes maintenance of the latest public version and handles feedback on a best-effort basis without a fixed response-time promise. Development previews remain experimental.

## 2. Toolchain and build entry points

CMake organizes Qt/C++/QML and Cargo organizes Rust. Top-level scripts/presets handle targets, dependency locations, and order. The Rust static core is linked into the application; Qt is dynamically deployed by default to facilitate license compliance and replacement. Native platform codecs/helpers are explicit, auditable separate artifacts.

The current Rust toolchain is locked to 1.99.0 by rust-toolchain.toml, and the build configuration locks the Qt SDK to 6.8.2; see [toolchain policy](../engineering/toolchain.md) for sources and hashes. Upgrades require corresponding review and reverification on three platforms. Prototype versions or a broad ≥6.8 must not be treated as interchangeable build inputs. The Qt LTS label does not mean every open-source distribution channel provides identical long-term patches.

`cxx`, Cargo, CMake, compiler, Qt package SHA, features, and target architecture are recorded in a versioned build manifest. Build-generated files go under build. Whether the QML compilation cache is enabled is also recorded in benchmark metadata. Cache keys include toolchain/Qt/feature to avoid incorrect reuse across architectures.

Provide development Debug, diagnostic ReleaseWithSymbols, and release Release configurations. Store debug symbols separately and archive them before signing. Do not enable high-risk aggressive LTO/custom allocators without measurements. Use understandable, reversible size optimizations while keeping panics/crashes diagnosable.

## 3. Qt modules and deployment

The desktop uses Qt Core, Gui, Qml, Quick and Quick Controls, along with the QML modules it imports. The private image preview worker links Qt SVG for supported static SVG previews. Avoid WebEngine, Multimedia, full Qt development tools, and unused SQL drivers.

Do not delete every transitive DLL/QML plugin based only on directly linked libraries. Deployment tools scan QML imports and dynamic dependencies, followed by clean-machine verification. QML resources and platform/imageformat plugins use explicit allowlists; do not load unknown Qt plugins from the user's current directory.

On macOS, deployment tools produce an app bundle and configure relative load paths. Windows deploys Qt DLLs/QML and required MSVC runtime. Linux uses an Ubuntu 24.04 runtime-directory tar.gz and .deb, collecting Qt and OS dependencies according to the [deployment contract](../engineering/linux-deployment.md). Review deployment-tool output; do not assume it automatically satisfies licensing, security, or minimum size.

Product installers contain only the Qt product core and selected backends, not Slint/Tauri prototypes, Python benchmark environments, Cargo target, headers, or font experiments. A source package may retain technology-selection experiments but must be separate from product binary packages.

## 4. Platform releases

### macOS

GitHub releases offer a `.dmg` for installation and an `.app.zip` application bundle. Under the personal open-source distribution policy, use only necessary ad-hoc integrity signing; Developer ID, notarization, and paid accounts are not required. Explain system warnings for packages without formal signatures; do not require users to disable system security protections. Keep permissions minimal and do not enable broadly protection-disabling entitlements for plugin convenience.

The Mac App Store is a separate distribution route requiring reevaluation of App Sandbox, bookmark authorization, downloaded-plugin restrictions, and licensing. Do not assume that the directly downloaded third-party executable extension model automatically works in a store build.

### Windows

GitHub Releases provides the unsigned per-user installer and portable ZIP. Microsoft Store distributes and updates a separately built MSIX through Partner Center. The EXE installs for the current user without administrator rights. The Store edition uses its package identity for associations and Explorer menus. See the [installation guide](../engineering/installers-associations.md) and [Store build guide](../engineering/developer-build.md#microsoft-store-msix).

### Linux

Current delivery provides an Ubuntu 24.04 runtime directory and .deb, not AppImage or Flatpak. Expanding channels requires additional installation, file-access, and provenance verification. Build against the baseline system; do not build against a newer glibc and claim compatibility with older systems. Test Wayland/X11, portal, Qt plugins, and sandbox permissions separately. AppImage/Flatpak/native packages impose different file and worker restrictions; expose actual capabilities per release configuration.

## 5. Licensing and supply chain

Original project content uses Apache-2.0; see the [licensing decision](../engineering/licensing.md) for the frozen record and root LICENSE for full terms. Review Rust crates, Qt modules, format reference tools and materials, icons, and packaging tools individually; the project LICENSE does not override dependency terms. Dynamic Qt deployment alone is not complete compliance proof.

Dependency license review includes at least distribution permissions, copyrights/notices, modification notices, source-provision obligations, user library replacement and relinking conditions, installation information, and compatibility with anti-debugging/signing policies. LGPL/GPL/commercial choices depend on actual modules and distribution methods; one Qt module's conclusion cannot be generalized to all Qt.

Assess the specific terms of 7-Zip/UnRAR or other RAR implementations separately, especially RAR creation and reverse-engineering restrictions; do not simply label them “MIT compatible.” If a capability does not satisfy target licensing, do not bundle it. A user-installed compliant backend may be considered, subject to review of the host distribution relationship.

GitHub release materials record dependency versions and hashes, file inventories, SBOMs, and the required license notices. Review source and build instructions under the [source delivery policy](../engineering/source-delivery.md). Release update manifests use signing keys separate from platform code signing. See the [release protocol](../engineering/release-protocol.md) for the signature and publication requirements.

## 6. Reproducible and offline builds

Dependency locks, source hashes, optional vendor caches, and toolchain manifests support offline builds. Restrict unreviewed build-script downloads; do not assume cargo build should access arbitrary sites by default.

Platform signing/notarization timestamps normally change final package bytes. Compare reproducibility of unsigned artifacts separately from verification of signed artifacts. Do not claim all signed packages are byte-for-byte identical.

## 7. Update policy

GitHub desktop builds check for updates on request from About. Eligible installations can install an update after verifying the signed release metadata, with a separate Sparkle feed and archive signature on macOS. Portable and development builds use manual installation. Microsoft Store manages updates for its MSIX edition. See [software updates](../engineering/software-update.md) for the selection, privacy and installation rules.

Installation waits for active tasks to finish and follows the platform's recovery procedure if it fails. Manual updates use a complete package from the selected release channel. Core archive operations work offline.

## 8. Release checklist

- Build platform packages from the frozen release commit and verify the required manifests, checksums, SBOMs and license materials. Follow the [release protocol](../engineering/release-protocol.md) for the attachment inventory.
- Verify the format capability inventory; unverified encryption/split volumes cannot be advertised as “all formats supported.”
- Pass clean-machine install, first open, system associations, exit, upgrade, and uninstall on three platforms.
- Verify source changes, output quotas, cancellation, recovery, safe paths, and sandboxing; third parties remain disabled by default if they do not meet the gate.
- Record complete compressed download/installed sizes, not just the main executable.
- Verify required ad-hoc integrity, warnings for lack of formal signing, Qt replacement policy, and all notices; publish known limitations.

The release owner may reject features that do not meet security/licensing gates and must not implicitly reduce default protection for a release date.
