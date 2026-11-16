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
- **Create and modify:** Create ZIP, TAR, TAR.GZ, GZIP, XZ, TAR.XZ, LZ4, TAR.LZ4, Zstandard, TAR.ZST, LZH and 7z archives. Add, replace, or move entries in eligible regular single-file archives. You can also drag files out of the list or drag files in and confirm adding them. See the table below for formats and limits.
- **Check:** Test an entire archive in the background without writing files. For TAR, LZMA, TAR.LZMA, Z, TAR.Z, UU/UUE, XXE, Brotli, TAR.BR, ISO, and UDF, only structure and readability are checked; per-file content verification is not guaranteed. See [archive integrity checks](docs/engineering/archive-checking.md).
- **Multiple archives:** Open archives in new tabs by default or configure them to open in new windows. A login session shares the application process and tray; background tasks remain accessible from the task entry point after browser pages close.
- **File-manager actions:** Create or extract archives directly from the Windows classic context menu, Ubuntu Files submenu, or macOS Finder Sync dynamic submenu. Finder services remain available as a fallback on macOS. Actions use separate windows and do not open the main window or destination directory by default. Per-item ZIP creation, smart extraction directories, and batch cancellation are supported. See [file-manager actions](docs/engineering/shell-actions.md) for entry points, system enablement, and installation scope.

`.DS_Store` files and `__MACOSX` and `_MACOSX` directories are excluded when creating archives or adding files. Extraction checks paths and destination authorization, handles name conflicts, and stages output; supported formats also verify content. Handling of special objects and links varies by format and platform. See the [safe extraction contract](docs/engineering/m3-extraction.md).

## Format support

“Modify” means adding, replacing, or moving entries in an existing archive. Support does not imply support for every algorithm or extension of a format; see the notes below.

| Format | Browse, extract, and check | Create | Modify |
| --- | --- | --- | --- |
| ZIP and ZIPX/XPI/APK/IPA/CBZ candidates | Store, Deflate, Deflate64, BZip2, LZMA, XZ; admitted AES-256 and split profiles | ZIP with Store, Deflate, AES-256 and numbered volumes | Eligible Store/Deflate ZIP archives |
| TAR, TAR.GZ (including `.tgz`) | Regular files and directories | Supported; TAR only packs files | Eligible regular single-file archives |
| GZIP (`.gz`) | One regular file, including concatenated members | One regular file, Deflate 0/1/6/9 | Not supported |
| BZip2, LZMA Alone, Lzip v1 | One regular file, within stream and memory limits | Not supported | Not supported |
| TAR.BZ2, TAR.LZMA, TAR.LZ | Regular files and directories | Not supported | Not supported |
| XZ, TAR.XZ (including `.txz`) | Declared checks, LZMA2, Delta/BCJ and concatenated streams | LZMA2 levels 1/6/9, XZ with one regular file | Not supported |
| LZ4, TAR.LZ4 | Standard frames, independent/linked blocks, concatenation and declared checks | LZ4 levels 1/6/9, ordinary LZ4 with one regular file | Not supported |
| Zstandard, TAR.ZST (including `.tzst`) | Standard frames, admitted windows, Huffman/FSE, repeat offsets and declared checks | Zstandard levels 1/6/9, ordinary Zstandard with one regular file | Not supported |
| 7z | Passwords, encrypted filenames, solid archives, and numbered volumes | Encryption, solid archives, and numbered volumes | Eligible unencrypted, non-split archives |
| DEB | Common ar members, control/data TAR views, admitted GZIP/XZ/Zstandard/BZip2/LZMA profiles | Not supported | Not supported |
| LHA/LZH | Headers 0–3, Store, LH1–LH7, LZS/LZ5 and declared CRCs | LH5 levels 1/6/9, level-2 headers, regular files/directories | Not supported |
| [PMA](docs/engineering/pma-streams.md) | Level-0 PM0/PM1/PM2, exact CP/M framing and declared CRCs | Not supported | Not supported |
| [ARJ](docs/engineering/arj-streams.md) | Store and methods 1–4, validated headers and full member CRC-32 | Not supported | Not supported |
| [ARC](docs/engineering/arc-streams.md) | Old/new Store and methods 3–9, exact framing and full member CRC-16 | Not supported | Not supported |
| [BH](docs/engineering/bh-containers.md) | `BH 05 07` ordinary members, method 8 raw Deflate, exact sizes and CRC-32 | Not supported | Not supported |
| [ALZ](docs/engineering/ace-alz-egg-format-sources.md) | Ordinary single-volume members using Store or raw Deflate, exact sizes and CRC-32 | Not supported | Not supported |
| [EGG](docs/engineering/ace-alz-egg-format-sources.md) | Non-encrypted, non-solid single-volume members using Store or raw Deflate, exact sizes and CRC-32 | Not supported | Not supported |
| [PEA](docs/engineering/m10-candidate-source-gates.md) | PEA 1.x single-volume, UTF-8 names, unencrypted PCOMPRESS0 stored files/directories; no object, stream or volume checks | Not supported | Not supported |
| NSIS (`.exe`) | Static `firstheader` metadata only; content-validated PE overlay | Not supported | Not supported |
| ASAR | Stored files/directories, shared payload ranges and declared file/block SHA-256 | Not supported | Not supported |
| Z, TAR.Z (including `.taz`) | Unix compress LZW, 9–16-bit dictionaries and admitted CLEAR/alignment profiles | Not supported | Not supported |
| Brotli, TAR.BR | Ordinary RFC 7932 streams, 10–24-bit windows and all static dictionary transforms | Not supported | Not supported |
| AES Crypt (`.aes`) | Versions 0–3, opening password, full ciphertext HMAC and version-specific length rules | Not supported | Not supported |
| UU/UUE, XXE | Traditional counted lines and admitted begin-base64 envelopes, including multiple members | Not supported | Not supported |
| RAR | Legacy, RAR5, and RAR 7 format extensions, including passwords, solid and split archives | Not supported | Not supported |
| CAB | Store, MSZip, Quantum, LZX, and volumes in the same directory | Not supported | Not supported |
| ISO | ISO9660, Joliet and Rock Ridge | ISO9660 + Joliet data image | Not supported |
| UDF | UDF | Not supported | Not supported |
| ZIP SFX | Validated ZIP self-extracting archives | Windows x64, Store/Deflate, `.sfx.exe` | Not supported |

- ZIP AES-256 encrypts content only; filenames remain visible. AES-128/192 and traditional ZipCrypto are not supported. Split archives can be read using `.zip.001`, PKZIP `.z01`, and related naming conventions; creation uses numbered volumes. See [password extraction](docs/engineering/password-extraction.md) and [opening split archives](docs/engineering/archive-opening.md).
- BZip2, LZMA Alone, Lzip v1 and their TAR compositions are currently read-only. LZMA has no content checksum; testing establishes structure, sizes and readability. See the [stream contract](docs/engineering/single-file-streams.md) for suffixes, members, memory limits and admission.
- XZ verifies each declared content check; streams declaring None provide structure/readability checks only. ZIPX reading supports the listed ZIP methods; PPMd, JPEG and WavPack variants remain unsupported. Additional ZIP methods are read-only and do not extend AES profiles or enable ZIPX creation.
- Modifying 7z rebuilds the archive, with a 512 MiB working-set limit for retained and added content. New archives use streaming input and are not subject to this rebuild limit. Encrypted and split ZIP and 7z archives cannot be modified. See [archive modification](docs/engineering/archive-transfers.md).
- RAR, CAB and UDF are read-only. [ISO creation](docs/engineering/iso-creation.md) supports ordinary files and directories, single volume, files smaller than 4 GiB and Joliet names of at most 64 UCS-2 characters per component. No boot images, UDF or Rock Ridge writing or image updates are provided. Reading an image never mounts or executes it. See the [RAR contract](docs/engineering/rar-format-sources.md) and [format backend contract](docs/engineering/native-codecs.md).
- [Owned SFX creation](docs/engineering/sfx-creation.md) produces a standalone Windows x64 ZIP extractor without Qt or installed xdzip. It selects a destination, handles conflicts and supports cancellation, and never runs extracted files. Passwords, volumes and automatic launch commands are unavailable. Generic `.exe` files remain outside file associations.
- DEB exposes `members/`, `control/` and `data/` as export views. Extracting all includes raw members and expanded content; no installation or script execution occurs. Control accepts none/GZIP/XZ/Zstandard, data additionally BZip2/LZMA. Checks validate container readability and declared checks, without package authentication. See the [DEB contract](docs/engineering/deb-containers.md).
- Z and TAR.Z checks cover structure and readability without checksums. No writer or damaged legacy-stream repair is provided; see the [Unix compress contract](docs/engineering/compress-streams.md).
- UU/UUE and XXE validate structure and readability without checksums. Header paths and modes grant no output or execution authority; GNU encoded filenames and multipart repair are outside the admitted [text envelope profile](docs/engineering/text-envelopes.md).
- Opening or extracting read-only ASAR does not launch its application. External `unpacked` data and declared links are visible but unsupported; no adjacent directory access or executable-mode restoration is granted. Optional hashes detect corruption, and hashless records provide structure/readability only. See the [ASAR contract](docs/engineering/asar-containers.md).
- NSIS is recognized only when its PE overlay contains a bounded, valid `firstheader` record. xdzip publishes that static metadata row and never executes the PE stub, installer script, plug-in or payload, decompresses the installer header, or exposes payload extraction. The `.exe` entry is hidden from drag-and-drop and shell associations; this is not a general executable handler.
- LHA/LZH creation uses LH5 and UTF-8 names with level-2 headers. Legacy Windows tools may display these names using their old code page. Existing archives cannot be modified. Legacy names are decoded before separators, and present CRCs are verified during actual decoding; ancient members without CRC provide structure/readability only. Passwords, volumes and repair profiles are not admitted. PMA has a separate read-only profile. See the [LHA contract](docs/engineering/lha-containers.md).
- ARJ is read only. Binary and legacy text members deliver the archive's CRC-covered bytes without platform newline conversion. Passwords, volumes, recovery/security profiles, links and special objects are not admitted. Unknown methods remain visible and prevent whole-archive check success.
- ARC is read only, with sequential regular files and exact termination. Password decoding, volumes, SFX and Spark extensions are not admitted. Unknown methods remain visible without ZIP method aliases and prevent whole-archive check success. Old XOR encryption has no header flag; decoding still requires a valid stream, exact size and CRC, without password or repair fallback.
- BH is read only and limited to the independently confirmed `BH 05 07` method-8 raw-Deflate profile. Unknown methods, encryption, directories, volumes, trailers and SFX prefixes remain outside the admitted scope; the official IZArc command-line reference crashed before producing output in this environment.

- Large ZIP archives using the disk-backed index do not currently support whole-archive testing or modification. Selected regular files can still be extracted within task limits; see [large archive indexing](docs/engineering/large-index.md).
- Sparse TAR and third-party extensions are not currently supported. Ordinary GZIP and ZIP-derived XPI/APK/IPA/CBZ candidates follow the [single-file stream contract](docs/engineering/single-file-streams.md). Legacy filename encodings are detected where possible, but short names and mixed encodings cannot be identified uniquely in every case; see [filename encoding](docs/engineering/filename-encoding.md).

- AES Crypt requires an opening password before metadata is published. All ciphertext MACs must pass; versions 0–2 leave the final length modulo unauthenticated, and extensions grant no authority. See the [AES Crypt contract](docs/engineering/aescrypt-envelopes.md).

- Brotli and TAR.BR are read-only, require complete content validation during weak discovery and have no checksum. Shared dictionaries and large-window extensions are outside the admitted [Brotli profile](docs/engineering/brotli-streams.md).

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

The desktop interface uses Qt Quick/QML, connected to the Rust core through a thin C++ layer. Format, query, and task logic are independent of Qt. The 7z, RAR, CAB, ISO and UDF readers and the 7z writer use isolated worker processes; ISO creation runs in the Rust host. The standalone Windows SFX uses system dialogs without Qt.

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
