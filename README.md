<p align="center">
  <img src="design/assets/brand/xdzip-128.png" width="80" height="80" alt="xdzip application icon">
</p>

<h1 align="center">xdzip</h1>

<p align="center"><strong>Make packing and unpacking a little simpler.</strong></p>

<p align="center">xdzip is an open-source, cross-platform archive manager. Its Rust format implementations work without external extraction programs. Browse and search archives, extract selected files, work with encrypted and split archives, and modify entries in eligible formats.<br>It supports ZIP, 7z, RAR, TAR, CAB, ISO/UDF and more on Windows, macOS, and Linux.</p>

<p align="center">
  <a href="https://github.com/xiaodaozhi/xdzip/actions/workflows/docs.yml"><img src="https://github.com/xiaodaozhi/xdzip/actions/workflows/docs.yml/badge.svg?branch=main" alt="Daily checks"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="Apache License 2.0"></a>
</p>

<p align="center">English | <a href="README.zh-CN.md">简体中文</a></p>

<p align="center">
  <a href="#download-and-install">Download</a> / <a href="#using-xdzip">Using xdzip</a> / <a href="#format-support">Format support</a> / <a href="#build-from-source">Build from source</a> / <a href="docs/README.md">Documentation</a> / <a href="https://github.com/xiaodaozhi/xdzip/issues">Feedback</a>
</p>

![xdzip welcome page](docs/images/main-window.png)

## Download and install

The current stable release is [xdzip 0.2.0, build 27](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.2.0). Choose the file for your system on the release page:

| System | Supported scope | Download |
| --- | --- | --- |
| Windows | Maintained versions of Windows 11, x86_64 | `setup.exe` per-user installer or portable `.zip` |
| macOS | macOS 14 and later, Apple Silicon | `.dmg` or `.app.zip` application bundle |
| Linux | Ubuntu 24.04 LTS, x86_64, X11, glibc 2.39 or later | `.deb` package or `.tar.gz` runtime directory |

Desktop packages start with `xdzip-0.2.0-`; packages for the standalone [command-line client](#command-line) start with `xdzip-cli-0.2.0-`. The release page also provides a checksum manifest and third-party component materials. The Windows installer is not signed with a publisher certificate. The macOS package uses an ad-hoc integrity signature and is neither Developer ID signed nor notarized, so the system may display security prompts. Installation does not change default applications. See the [installation guide](docs/engineering/installers-associations.md) for file associations and manual installation.

You can check for updates from “About xdzip.” Native updates for installed macOS builds support system proxies, custom HTTP CONNECT/SOCKS5 proxies, and direct connections. Automatic installation depends on a valid release signature and installation identity; portable packages and development builds must be installed manually. See [software updates](docs/engineering/software-update.md). For the first upgrade from an older client, see the [release notes](release-notes/0.2.0.md). The [support policy](docs/engineering/support-policy.md) describes the scope for other systems, architectures, and desktop environments.

**Version note:** Features on this page describe the current source tree. Consult the release notes for the corresponding release package. Builds from source are experimental development versions.

## Using xdzip

Open an archive to browse its directories, search and sort entries, and select files to extract. The interface offers list and grid views, file properties, Simplified Chinese and English (US), light and dark themes, and font size and density settings. By default, the application detects the system UI language and falls back to English (US) for unsupported languages; you can also choose a language manually. A regular single-file ZIP/ZIP64 archive can index up to one million physical records; other formats and split archive paths retain a limit of 100,000. See [large archive indexing](docs/engineering/large-index.md) for task and modification limits.

- **Content preview:** Explicitly preview bounded plain text, common raster images, and static SVG in the properties card; macOS also supports HEIC/HEIF. Preview appears after the content is fully read and any checks provided by the format are complete, and it can be canceled. See [preview](docs/engineering/content-preview.md) for formats, sizes, and decoding budgets.
- **Extract and open:** Choose a destination and name-conflict policy, view progress, and request cancellation. Double-clicking a regular file extracts a copy to a private temporary directory before opening it with the system. Changes to the copy are not written back to the archive; cached copies are cleaned up during a later open operation after seven days.
- **Create and modify:** Create ZIP, TAR, TAR.GZ, GZIP, and 7z archives. Add, replace, or move entries in eligible regular single-file archives. You can also drag files out of the list or drag files in and confirm adding them. See the table below for formats and limits.
- **Check:** Test an entire archive in the background without writing files. For TAR, ISO, and UDF, only structure and readability are checked; per-file content verification is not guaranteed. See [archive integrity checks](docs/engineering/archive-checking.md).
- **Multiple archives:** Open archives in new tabs by default or configure them to open in new windows. A login session shares the application process and tray; background tasks remain accessible from the task entry point after browser pages close.
- **File-manager actions:** Create or extract archives directly from the Windows classic context menu, Ubuntu Files submenu, or macOS Finder Sync dynamic submenu. Finder services remain available as a fallback on macOS. Actions use separate windows and do not open the main window or destination directory by default. Per-item ZIP creation, smart extraction directories, and batch cancellation are supported. See [file-manager actions](docs/engineering/shell-actions.md) for entry points, system enablement, and installation scope.

`.DS_Store` files and `__MACOSX` and `_MACOSX` directories are excluded when creating archives or adding files. Extraction checks paths and destination authorization, handles name conflicts, and stages output; supported formats also verify content. Handling of special objects and links varies by format and platform. See the [safe extraction contract](docs/engineering/m3-extraction.md).

## Format support

“Modify” means adding, replacing, or moving entries in an existing archive. Support does not imply support for every algorithm or extension of a format; see the notes below.

| Format | Browse, extract, and check | Create | Modify |
| --- | --- | --- | --- |
| ZIP | Store, Deflate, AES-256, and split archives | Store, Deflate, AES-256, and numbered volumes | Eligible regular single-file archives |
| TAR, TAR.GZ (including `.tgz`) | Regular files and directories | Supported; TAR only packs files | Eligible regular single-file archives |
| GZIP (`.gz`) | One regular file, including concatenated members | One regular file, Deflate 0/1/6/9 | Not supported |
| 7z | Passwords, encrypted filenames, solid archives, and numbered volumes | Encryption, solid archives, and numbered volumes | Eligible unencrypted, non-split archives |
| RAR | Legacy, RAR5, and RAR 7 format extensions, including passwords, solid and split archives | Not supported | Not supported |
| CAB | Store, MSZip, Quantum, LZX, and volumes in the same directory | Not supported | Not supported |
| ISO, UDF | ISO9660, Joliet, Rock Ridge, and UDF | Not supported | Not supported |

- ZIP AES-256 encrypts content only; filenames remain visible. AES-128/192 and traditional ZipCrypto are not supported. Split archives can be read using `.zip.001`, PKZIP `.z01`, and related naming conventions; creation uses numbered volumes. See [password extraction](docs/engineering/password-extraction.md) and [opening split archives](docs/engineering/archive-opening.md).
- Modifying 7z rebuilds the archive, with a 512 MiB working-set limit for retained and added content. New archives use streaming input and are not subject to this rebuild limit. Encrypted and split ZIP and 7z archives cannot be modified. See [archive modification](docs/engineering/archive-transfers.md).
- RAR, CAB, ISO, and UDF are read-only. ISO/UDF images are read as files; they are not mounted or executed. See the [RAR contract](docs/engineering/rar-format-sources.md) and [format backend contract](docs/engineering/native-codecs.md) for format coverage and resource limits.
- Large ZIP archives using the disk-backed index do not currently support whole-archive testing or modification. Selected regular files can still be extracted within task limits; see [large archive indexing](docs/engineering/large-index.md).
- Sparse TAR and third-party extensions are not currently supported. Ordinary GZIP and ZIP-derived XPI/APK/IPA/CBZ candidates follow the [single-file stream contract](docs/engineering/single-file-streams.md). Legacy filename encodings are detected where possible, but short names and mixed encodings cannot be identified uniquely in every case; see [filename encoding](docs/engineering/filename-encoding.md).

## Command line

The stable release includes a standalone Rust CLI that does not require Qt or a graphical session. It lists, extracts, creates, adds to, and tests archives, and supports JSON output. Download the `xdzip-cli-0.2.0-` package for your system and run `bin/xdzip --help` after extraction; on Windows use `bin/xdzip.exe`. You can also build it from source:

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
python3 scripts/cli.py build
python3 scripts/cli.py test
build/cli/bin/xdzip --help
build/cli/bin/xdzip list archive.zip --json
build/cli/bin/xdzip extract archive.zip -C output
```

On Windows, use `python` and add `.exe` to the executable name. Running on Linux requires system OpenSSL 3 and certificates; building also requires `libssl-dev` and `pkg-config`. See the [CLI guide](docs/engineering/command-line.md) for passwords, non-interactive use, exit codes, command installation, and update limits.

## Build from source

The desktop interface uses Qt Quick/QML, connected to the Rust core through a thin C++ layer. Format, query, and task logic are independent of Qt. The 7z, RAR, CAB, ISO, and UDF implementations use isolated worker processes.

Project documentation is in `docs/`; visual parameters, terminology, and dependency lists used by generators and checks are in `specs/`; high-fidelity designs are in `design/hifi/`.

Install Python 3.10+, Git, CMake 3.24+, Ninja, rustup, and your platform compiler/SDK, then obtain the source and build it. If you already cloned it for the CLI steps above, start with `prepare`:

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor
python3 scripts/desktop.py build
python3 scripts/desktop.py test
```

On Windows, use an MSVC x64 development environment. On macOS, use a native Apple Silicon terminal and Xcode Command Line Tools. The default Qt 6.8.2 toolchain is in `build/toolchain/`; pass `--qt-prefix` if you already have Qt. See the [developer build guide](docs/engineering/developer-build.md) for dependencies, launching, packaging, and verification. The CLI and desktop builds use separate entry points. The desktop build also includes a private CLI whose command entry point must be explicitly enabled.

Development packages are not official releases. Windows and Linux development artifacts may also be downloaded from successful manual runs of the [Qt components workflow](https://github.com/xiaodaozhi/xdzip/actions/workflows/qt-components.yml). Daily CI does not automatically produce development packages.

## Documentation and contributing

| Topic | Start here |
| --- | --- |
| Project documentation, architecture, and format contracts | [Documentation index](docs/README.md) |
| Privacy and data handling | [Privacy Policy](PRIVACY.en.md), [Chinese version](PRIVACY.md) |
| Build, repository layout, and testing | [Development guide](docs/engineering/development.md), [repository layout](docs/engineering/repository-layout.md), [verification](docs/qa/README.md) |
| Changes and planned work | [Changelog](CHANGELOG.md), [work package roadmap](docs/planning/backlog.md), [issues](https://github.com/xiaodaozhi/xdzip/issues) |
| Contributing | [Contributing guide](CONTRIBUTING.md), [code of conduct](CODE_OF_CONDUCT.md) |

Feedback and contributions are welcome. When reporting a bug, include the version or commit, system and architecture, reproduction steps, and actual results. Generate your own samples and avoid including sensitive information. Report security issues privately through [Report a vulnerability](https://github.com/xiaodaozhi/xdzip/security/advisories/new).

## License

Original project content is licensed under the [Apache License 2.0](LICENSE); see [COPYRIGHT](COPYRIGHT). Qt, codecs, fonts, and other dependencies are subject to their own licenses; see the [license documentation](docs/engineering/licensing.md) for sources and notices. Official releases do not provide a separate corresponding-source archive; see [source delivery](docs/engineering/source-delivery.md). This policy does not change applicable source, relinking, or notice obligations. Each release candidate still requires its own review; historical records are in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53).
