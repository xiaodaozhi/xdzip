# xdzip documentation

English | [简体中文](zh-CN/README.md)

Find guides for using xdzip, building the project, and understanding its archive formats. If you are new to xdzip, start with the [project README](../README.md) for its features, downloads, and supported systems.

## Use xdzip

| I want to... | Read |
| --- | --- |
| Install xdzip or set file associations | [Installation and file associations](engineering/installers-associations.md), [supported systems](engineering/support-policy.md) |
| Use the command line | [Command-line guide](engineering/command-line.md) |
| Drag files or modify an archive | [File transfers and archive modification](engineering/archive-transfers.md) |
| Understand passwords, split archives, and filename encoding | [Password extraction](engineering/password-extraction.md), [split archives and temporary opening](engineering/archive-opening.md), [filename encoding](engineering/filename-encoding.md) |
| Check an archive or preview a file | [Integrity checks](engineering/archive-checking.md), [content preview](engineering/content-preview.md) |
| Understand limits for large archives | [Large archive indexing](engineering/large-index.md) |
| Compress or extract from a file manager | [File-manager actions](engineering/shell-actions.md) |
| Learn how data is handled | [Privacy policy](../PRIVACY.en.md), [data lifecycle and diagnostics](engineering/privacy.md) |
| Check for updates | [User-initiated updates](engineering/software-update.md), [releases](https://github.com/xiaodaozhi/xdzip/releases) |

The [format support table](../README.md#format-support) describes the formats, operations, and limits in the current source tree. Consult the release notes for the capabilities of a specific package.

## Develop and contribute

- [Developer build guide](engineering/developer-build.md): prepare your environment, build, run, and test the desktop application and command-line tool.
- [Developer preparation](engineering/development.md) and [repository layout](engineering/repository-layout.md): understand the development workflow, modules, and verification entry points.
- [Contributing guide](../CONTRIBUTING.md), [code of conduct](../CODE_OF_CONDUCT.md), and [security policy](../SECURITY.md): contribute changes, participate, and report security issues.
- [Verification](qa/README.md) and [CI policy](engineering/ci-policy.md): select checks for a change and distinguish automated results from desktop verification.
- [System architecture](design/02-system-architecture.md), [domain and data](design/03-domain-data.md), and the [Qt/Rust bridge](design/05-qt-rust-bridge.md): understand the main modules and their boundaries.
- [UI design standard](visual/design-standard.md), [design workflow](visual/design-workflow.md), and [interaction prototype](ux/prototype-spec.md): explore interface, component, and interaction guidelines.

## Formats and implementation

These documents explain format capabilities, limits, and implementation sources. For current support, start with the [format support table](../README.md#format-support).

- **Reading, extraction, and creation:** [Browsing and indexing](engineering/readonly-browser.md), [safe extraction](engineering/m3-extraction.md), [ZIP creation](engineering/m4-creation.md), [ISO creation](engineering/iso-creation.md), [Windows self-extracting archives](engineering/sfx-creation.md).
- **Common compression streams:** [TAR and GZIP](engineering/m4-tar-gzip.md), [single-file compression streams](engineering/single-file-streams.md), [LZ4](engineering/lz4-streams.md), [Zstandard](engineering/zstandard-streams.md), [Brotli](engineering/brotli-streams.md), [Unix compress](engineering/compress-streams.md).
- **Archives and envelopes:** [7z implementation basis](engineering/sevenzip-format-sources.md), [RAR implementation basis](engineering/rar-format-sources.md), [CAB implementation basis](engineering/cab-format-sources.md), [DEB](engineering/deb-containers.md), [ASAR](engineering/asar-containers.md), [AES Crypt](engineering/aescrypt-envelopes.md), [UU and XXE](engineering/text-envelopes.md).
- **Other formats:** [LHA/LZH](engineering/lha-containers.md), [PMA](engineering/pma-streams.md), [ARJ](engineering/arj-streams.md), [ARC](engineering/arc-streams.md), [BH](engineering/bh-containers.md), [ACE, ALZ, and EGG](engineering/ace-alz-egg-format-sources.md).
- **Design and dependencies:** [Format backends](engineering/native-codecs.md), [in-house formats and algorithms](engineering/independent-rust-codecs.md), [format extension design](design/06-codec-extension-sdk.md), [dependencies and licenses](engineering/dependencies.md).

## Releases and collaboration

Maintainers can consult [versioning](engineering/versioning.md), the [release protocol](engineering/release-protocol.md), the [release runbook](engineering/release-runbook.md), and [release review](qa/release-review.md). Project plans, issues, and feedback are on [GitHub Issues](https://github.com/xiaodaozhi/xdzip/issues); version changes are in the [changelog](../CHANGELOG.md).
