# Build, deployment and release architecture

English | [简体中文](../zh-CN/design/build-release.md) | [繁體中文（台灣）](../zh-TW/design/build-release.md) | [日本語](../ja-JP/design/build-release.md) | [한국어](../ko-KR/design/build-release.md) | [Français](../fr-FR/design/build-release.md) | [Deutsch](../de-DE/design/build-release.md) | [Español](../es-ES/design/build-release.md) | [Português (Brasil)](../pt-BR/design/build-release.md) | [Русский](../ru-RU/design/build-release.md)

## Build and package inputs

CMake builds the Qt desktop, C++ adapters and native helpers. Cargo builds the Rust runtime and generates the linked CXX bridge. The standalone CLI uses its own build entry point and does not require Qt. `scripts/desktop.py` and `scripts/cli.py` coordinate these build and test steps. The source locks Rust 1.99.0 and the Qt 6.8.2 SDK. Keep build identity, target architecture, lockfiles and toolchain provenance together when comparing results. [Toolchain](../engineering/toolchain.md), [Developer build guide](../engineering/developer-build.md)

The desktop deploys the Qt libraries, QML modules, platform resources and bundled helpers that its package requires. The private image worker uses Qt for image decoding and static SVG. The CLI distribution includes its required archive worker. Build profiles, package allowlists and deployment checks live with the scripts and platform contracts. An executable alone does not establish the installed dependency set or total package size. [Deployment and dependency audit](../engineering/deployment-audit.md), [CLI distribution](../engineering/command-line.md)

## Current distribution channels

| Target | Package and trust boundary |
| --- | --- |
| macOS Apple Silicon | GitHub provides a DMG and an app bundle ZIP. The target baseline is macOS 14. The package uses ad-hoc integrity signing. See platform instructions for install and update behavior. |
| Windows 11 x86_64 | GitHub provides the unsigned per-user installer and portable ZIP. Microsoft Store handles its own MSIX identity, signing and update channel. |
| Ubuntu 24.04 LTS x86_64 | GitHub provides a runtime-directory archive and a DEB. The support baseline includes X11 and glibc 2.39 or later. |
| Standalone CLI | Published with separate package and update identity. The CLI included with a desktop installation follows its bundle's integration and handoff rules. |

The [support policy](../engineering/support-policy.md) defines the operating system and architecture scope. See the [project README](../../README.md) and matching release notes for actual assets, and the [installer guide](../engineering/installers-associations.md) for association and integration limits.

## Routine checks, native builds and release

The routine [CI workflow](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml) runs lightweight checks. Native Foundation and Qt jobs are separate manual workflows. Ordinary Apple Silicon CI builds, tests and runs smoke checks without deployment of a downloadable application. The dedicated release workflow has its own authorized deployment, signing, packaging and publication steps. The [release protocol](../engineering/release-protocol.md) and [CI policy](../../developers/engineering/ci-policy.md) define gates and evidence.

Release manifests, checksums, inventories, SBOM and license materials describe the delivered artifacts. Project source ZIPs are not generated or uploaded by the release workflow. GitHub's own tag snapshots follow its separate mechanism. Dependency obligations follow the actual shipped components and the project's [licensing record](../engineering/licensing.md) and [source delivery policy](../engineering/source-delivery.md).

## Update and future distribution

Update selection and verification are separate from archive tasks. Eligible installed builds follow the update contract for trust metadata, verified assets, installation identity and active-task handoff. Windows/Linux use the applicable helper route, macOS uses Sparkle, and the Store edition is serviced by Microsoft Store. Portable or development configurations follow their own limits. [Software updates](../engineering/software-update.md)

Expanding to other architectures, operating-system baselines, package formats or stores requires separate build and acceptance evidence. The historical distribution decisions and their review triggers remain in the [architecture decision records](architecture-decisions.md).
