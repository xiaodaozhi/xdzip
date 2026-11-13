# xdzip: 11 Build, release, licensing, and updates

## 1. Support matrix

The application display name and executable basename are consistently `xdzip`; platform artifacts are `xdzip.app`, `xdzip.exe`, and `xdzip`, with `xdzip.desktop` as the Linux desktop entry. Production Rust modules use the `xdzip-` prefix. Releases use the project's existing bundle ID. A paid signing identity is not required for personal open-source GitHub distribution; see the [release runbook](../engineering/release-runbook.md). The project owner approved the initial support scope and maintenance approach on 2026-10-19. The authoritative description is the [support policy](../engineering/support-policy.md) and [ADR-011](architecture-decisions.md#adr-011首批支持范围与个人项目维护政策).

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

Initially required modules are Core, Gui, Qml, Quick, QuickControls2, QML Layouts, and actual transitive dependencies such as corresponding Templates and Models. Icons may require Svg; add it only when selected. Avoid WebEngine, Multimedia, full Qt development tools, and unused SQL drivers.

Do not delete every transitive DLL/QML plugin based only on directly linked libraries. Deployment tools scan QML imports and dynamic dependencies, followed by clean-machine verification. QML resources and platform/imageformat plugins use explicit allowlists; do not load unknown Qt plugins from the user's current directory.

On macOS, deployment tools produce an app bundle and configure relative load paths. Windows deploys Qt DLLs/QML and required MSVC runtime. Linux uses an Ubuntu 24.04 runtime-directory tar.gz and .deb, collecting Qt and OS dependencies according to the [deployment contract](../engineering/linux-deployment.md). Review deployment-tool output; do not assume it automatically satisfies licensing, security, or minimum size.

Product installers contain only the Qt product core and selected backends, not Slint/Tauri prototypes, Python benchmark environments, Cargo target, headers, or font experiments. A source package may retain technology-selection experiments but must be separate from product binary packages.

## 4. Platform releases

### macOS

Direct downloads use a local `.dmg`. Under the personal open-source distribution policy, use only necessary ad-hoc integrity signing; Developer ID, notarization, and paid accounts are not required. Explain system warnings for packages without formal signatures; do not require users to disable system security protections. Keep permissions minimal and do not enable broadly protection-disabling entitlements for plugin convenience.

The Mac App Store is a separate distribution route requiring reevaluation of App Sandbox, bookmark authorization, downloaded-plugin restrictions, and licensing. Do not assume that the directly downloaded third-party executable extension model automatically works in a store build.

### Windows

Distribute the unsigned per-user installer through GitHub Releases, stating whether administrator rights are needed; default per-user installation must be viable. Workers reside in fixed application/versioned extension directories. DLL search paths exclude untrusted current directories. Installation/upgrades do not overwrite plugin packages still used by tasks.

### Linux

Current delivery provides an Ubuntu 24.04 runtime directory and .deb, not AppImage or Flatpak. Expanding channels requires additional installation, file-access, and provenance verification. Build against the baseline system; do not build against a newer glibc and claim compatibility with older systems. Test Wayland/X11, portal, Qt plugins, and sandbox permissions separately. AppImage/Flatpak/native packages impose different file and worker restrictions; expose actual capabilities per release configuration.

## 5. Licensing and supply chain

Original project content uses Apache-2.0; see the [licensing decision](../engineering/licensing.md) for the frozen record and root LICENSE for full terms. Review Rust crates, Qt modules, 7z/RAR native libraries, icons, and packaging tools individually; the project LICENSE does not override dependency terms. Dynamic Qt deployment alone is not complete compliance proof.

Dependency license review includes at least distribution permissions, copyrights/notices, modification notices, source-provision obligations, user library replacement and relinking conditions, installation information, and compatibility with anti-debugging/signing policies. LGPL/GPL/commercial choices depend on actual modules and distribution methods; one Qt module's conclusion cannot be generalized to all Qt.

Assess the specific terms of 7-Zip/UnRAR or other RAR implementations separately, especially RAR creation and reverse-engineering restrictions; do not simply label them “MIT compatible.” If a capability does not satisfy target licensing, do not bundle it. A user-installed compliant backend may be considered, subject to review of the host distribution relationship.

Releases provide an SBOM (such as SPDX/CycloneDX), NOTICE, dependency versions/hashes, source and build instructions, required license materials, and file inventory. Static Rust third-party code is also included in the SBOM. No formal signing key is currently used. If another signing channel is chosen later, credentials must not enter the repository or ordinary PRs.

## 6. Reproducible and offline builds

Dependency locks, source hashes, optional vendor caches, and toolchain manifests support offline builds. Restrict unreviewed build-script downloads; do not assume cargo build should access arbitrary sites by default.

Platform signing/notarization timestamps normally change final package bytes. Compare reproducibility of unsigned artifacts separately from verification of signed artifacts. Do not claim all signed packages are byte-for-byte identical.

## 7. Update policy

The v0.1 plan uses manually downloaded updates without background automatic installation. v1.0 may offer an optional version check, but networking requires clear settings and minimal data; user archive information is not uploaded. Update-manifest signature verification does not rely on TLS alone and covers version, platform/architecture, minimum OS, hashes, and origin. Reject rollback attacks; necessary security rollbacks require trusted policy authorization.

Updates apply only after tasks end safely; failure preserves the current version. Extension updates independently verify protocol, origin, and permission changes; old packages in use are reclaimed later. Offline users can import a complete package after checking origin and checksums; missing network access does not make extraction unavailable.

## 8. Release checklist

- Generate source package, platform packages, symbols, SBOM, license materials, and benchmark reports from the same commit.
- Verify the format capability inventory; unverified encryption/split volumes cannot be advertised as “all formats supported.”
- Pass clean-machine install, first open, system associations, exit, upgrade, and uninstall on three platforms.
- Verify source changes, output quotas, cancellation, recovery, safe paths, and sandboxing; third parties remain disabled by default if they do not meet the gate.
- Record complete compressed download/installed sizes, not just the main executable.
- Verify required ad-hoc integrity, warnings for lack of formal signing, Qt replacement policy, and all notices; publish known limitations.

The release owner may reject features that do not meet security/licensing gates and must not implicitly reduce default protection for a release date.
