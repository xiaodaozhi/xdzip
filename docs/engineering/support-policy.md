# System support and maintenance policy

The project owner approved the initial support policy on 2026-10-19, and `v0.1.0` was released publicly on 2026-11-06. See the [releases page](https://github.com/xiaodaozhi/xdzip/releases) for current versions and downloads. This document defines support scope; it does not change minimum-version checks or extend CI host results or user acceptance without environment details into guarantees for all system versions. The manual review was completed according to the user's conclusion and need not be repeated.

## Initial support scope

| Platform | Initial support | Engineering evidence and limits |
| --- | --- | --- |
| macOS | macOS 14 or later, Apple Silicon arm64 | Deployment target 14.0, native CI and local development DMG; no Intel or Universal package and no promise for every future system version |
| Windows | Windows 11 x86_64 versions still maintained by Microsoft | MSVC build, per-user installer, actual installation/upgrade/rollback/uninstall CI; Server CI does not establish support for every Windows 11 version; no ARM64 package |
| Linux | Ubuntu 24.04 LTS x86_64, X11, glibc 2.39 or later | Current .deb and isolated runtime verification without the Qt SDK; no claim for arbitrary distributions, older glibc, Wayland or ARM64 |

The Mac minimum follows the current deployment target, not the unverified version 13 in older designs. Standard support for Windows 10 ended 2025-10-14 and is not in the initial commitment; LTSC and ESU are not automatically included. See [Microsoft's lifecycle announcement](https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support). Qt's own support matrix defines a dependency boundary, not actual xdzip verification; see [Qt 6.8 supported platforms](https://doc.qt.io/qt-6.8/supported-platforms.html).

## Maintenance

This personal project is maintained on a best-effort basis, without paid service, a fixed response SLA or unperformed security commitments. The latest public stable release takes priority; development previews may introduce compatibility changes. Release notes list known issues and rollback options, and affected downloads may be withdrawn when security problems arise. The desktop application lets users initiate update checks and handoff from About, without background polling or a silent updater. The current source also provides a CLI update command; see the [command-line contract](command-line.md) for eligible identities and distinctions from released assets.

Candidates remain pinned to Qt 6.8.2 and the existing locked Rust/Cargo versions. Freezing supports repeatable verification; it does not declare older patches vulnerability-free or promise permanent free LTS updates. Before every public release the maintainer checks official Qt security advisories and available open-source updates, prioritizing evaluation of maintained public versions. If the current dependency has an unpatched security issue affecting this project, the affected release is paused. A Qt upgrade requires revalidation of source, module licensing, Windows private interfaces, dynamic replacement, three-platform builds and actual menu/window behavior; purchasing commercial maintenance is not a prerequisite.

## Approved scope and changes

Approval covers the initial platforms, best-effort maintenance of only the latest public release, no fixed response time and experimental labeling of development builds. Actual `v0.1.0` release results are recorded separately in the Release and historical review issues; this policy does not extend one release into performance guarantees for all system versions. Expansion to architectures, systems or distribution channels requires implementation and verification before updating this policy; see [ADR-011](../design/architecture-decisions.md#adr-011首批支持范围与个人项目维护政策).

The user has cancelled formal signing, notarization and paid-account requirements; do not request them again. Report vulnerabilities according to the [security policy](../../SECURITY.md).
