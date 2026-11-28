# Changelog

English | [简体中文](CHANGELOG.zh-CN.md) | [繁體中文](CHANGELOG.zh-TW.md) | [日本語](CHANGELOG.ja-JP.md) | [한국어](CHANGELOG.ko-KR.md) | [Français](CHANGELOG.fr-FR.md) | [Deutsch](CHANGELOG.de-DE.md) | [Español](CHANGELOG.es-ES.md) | [Português (Brasil)](CHANGELOG.pt-BR.md) | [Русский](CHANGELOG.ru-RU.md)

## Unreleased

- Add a standalone read-only SDK trial with bounded CBOR, controlled I/O, two independent protocol implementations and a sandbox gate that refuses unsafe execution. Third-party extensions remain disabled in the product.

## 0.3.3

- View archive comments and edit comments in ordinary writable ZIP, ZIP64 and ZIPX archives. Display additional entry comments and attributes in the file preview.
- Improve filename and path handling when creating, editing and extracting archives, including editing and extraction in shared Windows directories.
- Add recent archives, quick extraction and an action to open the archive location. Update application and document icons.

See the [release notes](release-notes/0.3.3.md) and [GitHub release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.3) for platform support, verification scope and known limitations.

## 0.3.2

- Publish native macOS Intel desktop and standalone CLI packages for macOS 14 or later, with authenticated updates for both Mac architectures.
- Fix dropped minimize requests during macOS fullscreen transitions.
- Align the format suffix labels on the start page and correct the size and dropdown styling of the Windows extraction window.
- Publish the Ubuntu amd64 Snap to the stable channel.

See the [release notes](release-notes/0.3.2.md) and [GitHub release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.2) for platform support, verification scope and known limitations.

## 0.3.1

- Place Simplified and Traditional Chinese together at the top of the manual language choices, labeled “中文 (简体)” and “中文 (繁体)”.
- Improve start-page format categories and paging, and fix toolbar and operation-window layout.
- Fix premature reporting of completion for paged archive queries and preserve Windows origin information when extracting files.
- Update image-preview dependencies.

See the [release notes](release-notes/0.3.1.md) and [GitHub release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.1) for platform support, verification scope and known limitations.

## 0.3.0

- Support ten languages in the desktop interface, system menus and user documentation, with automatic system-language selection and a manual setting.
- Extend archive editing to more supported formats, including adding, replacing, moving, renaming and deleting eligible entries.
- Load file previews automatically with an on-demand refresh, and improve archive actions, macOS file associations, and Ubuntu window and file handoff behavior.

See the [release notes](release-notes/0.3.0.md) and [GitHub release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.0) for platform support, verification scope and known limitations.

## 0.2.1

- Extend browsing and extraction for formats including LHA/LZH, ARJ, ARC, DEB, ASAR, Zstandard and LZ4. Add creation for LZH, ISO9660/Joliet data images and Windows x64 ZIP self-extracting files.
- Extend preview to common raster images and a restricted static SVG profile. Add system HEIC/HEIF on macOS and generate thumbnails through a bounded private process. The preview contract describes format limits.
- Index up to one million records in ordinary single-file ZIP/ZIP64 with a streaming disk index. Load large query results on demand and retain selection across pages.
- Add user-requested, bounded plain-text and PNG/JPEG previews to the properties card, sharing password, integrity and cancellation handling.

See the [release notes](release-notes/0.2.1.md) and [GitHub release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.2.1) for platform support and verification scope.

## 0.2.0

- Support native updates in installed macOS builds over custom HTTP CONNECT or SOCKS5 proxies and direct connections.
- Provide standalone CLI packages for Windows, macOS and Linux, with archive commands, JSON results, explicit command registration and updates verified against signed manifests.
- Apply the official QtNetwork patches for CVE-2026-116337 and CVE-2026-105946. Retain the complete patches, provenance and replacement instructions.
- Move the bundled CLI metadata on macOS into ordinary app resources covered by bundle integrity signing.

See the [release notes](release-notes/0.2.0.md) and [GitHub release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.2.0) for platform support and verification scope.

## 0.1.3

- Prevent Finder from treating the default archive-named extraction folder for an archive ending in `.app` as a damaged application.

See the [release notes](release-notes/0.1.3.md) and [GitHub release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.3) for platform support and verification scope.

## 0.1.2

- Preserve the file list during directory changes, searching and sorting to reduce flicker.
- Improve the explanation and manual installation steps when automatic installation is unavailable on macOS.
- Fix extraction of safe parent-relative symlinks from ZIP archives on macOS and correct Qt runtime notices.

See the [release notes](release-notes/0.1.2.md) and [GitHub release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.2) for platform support and verification scope.

## 0.1.0

- Publish the first stable release with native installers or portable packages for Windows, macOS and Linux, and user-initiated updates for eligible installations.
- Improve Settings scrolling, update-state accessibility and system proxy support. Fix native hover behavior.
- Fix third-party notice collection and normalize license expressions in the source SBOM and the Rust runtime candidate dependency graph.

See the [release notes](release-notes/0.1.0.md) and [GitHub release](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.0) for platform support, known limits and release materials.

## 0.1.0-alpha.9

- Remove three Windows SDK graphics compiler DLLs from the installer and portable package. Upgrades clean up old bundled copies.
- Improve Settings and update entry points. Windows file-type descriptions follow the application language.
- Fix minimization during macOS fullscreen transitions and several file associations. Improve startup and font loading.
- Fix archive-close cleanup, extraction destination checks and task prompts.
- Streamline routine CI and run full application builds and release verification on demand. Expand source and third-party licensing materials.

See the [release notes](release-notes/0.1.0-alpha.9.md) for platform and licensing limits of this experimental prerelease.

## 0.1.0-alpha.8

- Add user-initiated update checks to About. Eligible installations can upgrade after signature, version and platform checks. The previous public alpha.6 requires a manual upgrade.
- Use system fonts and platform fallbacks for archive names, filenames and paths. Fixed UI copy keeps the product font. Show distinct messages for empty folders, empty archives and searches with no matches.
- Remove the software OpenGL library from Windows development bundles. Keep Direct3D 11 as the default and WARP available for software rendering.
- Fix double-counted active tasks when snapshots arrive before task acceptance. Preserve acceptance, cancellation and exit cleanup.
- Detect common legacy filename encodings consistently across browsing, navigation, extraction, temporary opening, export and archive modification. Prefer Unicode declarations, retain raw-name validation and write rewritten names as UTF-8. Short names and mixed encodings can remain ambiguous.
- Remove retired release gates and report templates. Align contribution rules, readiness aggregation and material generators, and track provenance and licensing in the release checklist.
- Use reviewed version-specific notes with version and build checks. Stop when the draft notes differ from the reviewed file so retries cannot overwrite recorded acceptance limits.
- Update documentation for dedicated macOS release, split-volume reading and the changelog.

## 0.1.0-alpha.6

- Clear stale hover state after windows hide or minimize, including the red close-button flash when restoring from the Windows tray. Apply shared pointer cleanup to browsing and operation windows.
- Browse and extract numbered ZIP and PKZIP volumes. Cross-volume reads, ZIP64 offsets, passwords and integrity checks use the shared extraction path, with source-change and overwrite protection for every volume.
- Open regular archive entries through the system from a private temporary copy. Hide progress after the system accepts the request and retain failed results for retry. Copies follow a seven-day retention policy and edits to them stay outside the archive.
- Fix persistence for the archive opening mode. Add tab overflow scrolling, a tab list and drag reordering. Use regular weight for inactive tabs.
- Support GNU long filenames in Git source TAR archives. Anchor welcome-page error messages to its top layout. Closing a window clears its tabs and retains background tasks. Update tray task-count copy accordingly.

## 0.1.0-alpha.1

- Introduce Debian epoch 1 so alpha packages upgrade earlier dev packages in system version order. Keep historical fixture identities.
- Preserve normal, maximized and fullscreen states through dialogs, tray restoration and external file opens.
- Share one process and tray within a login session. Forward activation and batch open requests during startup, reject timeouts and recover from stale state after a crash.
- Add optional tabs or separate windows with independent archive controllers and browsing state. Retain tasks and results after page close, route them by session and task identity, and serialize heavy admissions.
- Separate prerelease number from build number. Prepare six cross-platform package types, immutable manifests and checksums, source and licensing materials, and publication procedures. At this stage, automatic updates were still pending, and release licensing was tracked separately.
- Retain all explicitly selected regular creation inputs, including an existing target or older volume. Stage the output privately and apply conflicts at final publication. Preserve a target that is also an input and offer a new name or cancellation.
- Update task menu entries in place by task ID to prevent flicker as progress changes. Preserve hover, highlighting and keyboard focus, and update only entries added to or removed from history.
- Trim macOS development DMGs to arm64 Qt binaries, the Basic Qt Quick style and required shared dependencies. Record architecture, style and size and verify the final package. Keep the original app and SDK intact, using local ad-hoc integrity signing.
- Retain only the pinned Qt Quick Basic style in Windows deployment, remove QML debug plugins and development type metadata, and keep native style and graphics fallbacks. Fix Qt system translation loading and bundle the same pinned VC++ runtime in the portable and per-user installer packages. Deployment audits reject borrowing host runtimes.
- Standardize the main executable name as xdzip across Windows, macOS and Linux. Align launcher entries, resources, CI, rebuilds and documentation while retaining upgrade test compatibility with earlier builds.
- Add single-volume AES-256 ZIP extraction for AE-1/AE-2 and Store/Deflate, with task password prompts, retry and cancellation. Publish files after authentication. Update designs, UI copy, protocol coverage and interoperability checks.
- Add targeted Open With cleanup for macOS development copies. Remove new registrations after verification or failure and preserve the primary app, files and system defaults. Add boundary and failure-cleanup regressions.
- Close the development and acceptance work package with owner approval, recording ten automated gates, eight manual checks and the support policy for the frozen candidate. Track provenance and licensing in the release checklist and issues. Formal release was still pending.
- Add the macOS native menu bar and tray menus on all three platforms. Share commands, edit focus and task state. Default to tray residency when closing the last window. Settings can disable it, and quit and restart retain task confirmation and cleanup. Add bilingual copy, small template icons, designs and lifecycle tests.
- Introduce unified versioning in VERSION.json, native metadata and About with bilingual copy and build-info copying. Continue full-process performance and licensing evidence while keeping desktop, clean-machine and licensing checks separate.
- Align Qt main-window dimensions, panes, lists, text, icons, tabs and preview to frozen HTML and artwork measurements. Share font sources and add independent CI measurements and real ZIP screenshots. Support closing tabs and selecting all in the current page, with disabled states and unknown dates retained.
- Complete shared main-window menus, hints, button states and breadcrumbs. HTML adds context, dropdown and submenus, sample tabs, grid, sort, selection, touch and demo check tasks. Qt adds real read-only selection, copy, view and directory menus. Verify 22 new interactions, 20 existing flows and five local Qt groups. Keep unavailable actions disabled and demos labeled.
- Add directory-tree connector lines, fixed counts, collapsed-ancestor indicators and shared icon slots to HTML and Qt. Preserve the earlier main artwork. Track boundary hints, skeletons and manual accessibility separately.
- Deliver three directory-tree artboards, 26 original-pixel slices, light and dark indicators, layout and interaction specifications, and single-file HTML and assets. HTML and Qt integration was pending at this point. Preserve original artwork and photos.
- Move high-fidelity artwork to design/hifi with main and operation window overviews, historical proposals and hashes. Align asset sources and links. The matching dark main artwork was still pending.
- Add a bounded directory tree with implicit folders, sorting over all matches and keyboard column headings. Page after sorting and preserve duplicate record IDs. Add node, depth and byte limits, fallback messaging, interaction tests and 54 visual checks.
- Add the product runtime foundation with two cxx handles, bounded commands and events, independent cancel and shutdown control, Qt mailbox and shutdown barriers. Verify three-platform product CI, ten lifecycle repeats per platform and deployed native launch and exit on macOS and Windows, with an Apple Silicon development build and a one-time verification guide.
- Retry brief admission-lock contention asynchronously within bounds. Disable disconnected commands, report invalid arguments and guard destruction during event handling.
- Add bounded, metadata-only extraction preflight with directory expansion, path, conflict and size checks and a native result window. Output authority remains with extraction.
- Refine the three-pane browser with light and dark themes, details, directory scope, search and multi-selection by physical ID. Add mouse and filtering tests.
- Add real read-only ZIP browsing with independent Rust adaptation, bounded scanning, duplicate and Chinese names, 128-record pages and asynchronous cancellation and close. Record rawzip admission and Qt screenshots.
- Add Qt task progress path cards, three-column metrics, progress gradient and shared footer from the artwork. Preserve large-text reflow and demo labels. Capture light, dark and 200% text states.
- Establish domain identity and paging invariants, standalone Qt task-window previews, shared hints and generated theme parameters. Add layout, close and offscreen CI checks.
- Add a standalone Rust/cxx bridge probe, unified CMake and Cargo builds, pinned dependencies and native CI. Verify Linux ownership, error, UTF-8 and shutdown behavior.
- Add a standalone Qt Quick toolchain probe and Linux offscreen and path build evidence with Chinese characters and spaces. Three-platform and cxx checks were still pending at that point.
- Add eight high-fidelity state artboards and assets for creation, conflicts, progress, results, Settings and platform comparisons, with save, cancel and recovery flows.
- Deliver 24 original-pixel slices, 27 light and dark vector icon types and transparent PNGs at multiple densities, with source and precision notes.
- Align file-type icons, list geometry and the type column. Define standalone windows, title bars and close behavior on all three platforms.
- Add an offline HTML interaction prototype covering browsing, selection, extraction, creation, tasks and failure states, linked to requirements and verification.
- Adopt Apache-2.0, add the full license and update contribution and release documents.

### Pre-development baseline (2026-10-11)

- Record the software design, requirements traceability and architecture decisions.
- Choose Qt Quick with a thin C++ layer and Rust.
- Set the visual baseline and theme parameters for light and dark views and extraction.
- Add contribution, governance, security, engineering, acceptance, risk and work-package documentation.
- Product code and binary releases were still pending.
