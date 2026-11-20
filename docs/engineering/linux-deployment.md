# Linux standalone runtime directory and verification

English | [简体中文](../zh-CN/engineering/linux-deployment.md) | [繁體中文](../zh-TW/engineering/linux-deployment.md) | [日本語](../ja-JP/engineering/linux-deployment.md) | [한국어](../ko-KR/engineering/linux-deployment.md) | [Français](../fr-FR/engineering/linux-deployment.md) | [Deutsch](../de-DE/engineering/linux-deployment.md) | [Español](../es-ES/engineering/linux-deployment.md) | [Português (Brasil)](../pt-BR/engineering/linux-deployment.md) | [Русский](../ru-RU/engineering/linux-deployment.md)

The current development deployment baseline is Ubuntu 24.04, x86_64, and X11. The goal is running the current product without the development Qt SDK; this does not promise other distributions, older glibc, Wayland, or complete installer support. macOS CI retains native build, test, and startup verification. Local development DMGs follow the [Mac deployment guide](macos-deployment.md) and are unaffected by this flow.

## Layout and generation

`scripts/deploy_linux.py` accepts only trusted project-owned ELF files and an explicitly specified Qt 6.8.2 SDK, creating a new build/ or dist/ directory:

- `xdzip` is the launcher, using local libraries, plugins, and QML without inheriting development SDK search paths.
- `bin/xdzip` is the product; adjacent `qt.conf` uses relative Qt paths.
- `lib/` contains actually imported Qt/SDK shared libraries; `qml/` retains the Basic Controls style and required implementation submodules. Other Controls styles, NativeStyle, Effects, style-specific dialog variants, designer files and QML type metadata are removed before collecting native dependencies, so their exclusive libraries are not deployed.
- `plugins/` contains xcb, offscreen verification, image, icon, platform input, xcb graphics integration, and required `tls/libqopensslbackend.so`; deployment fails without the OpenSSL backend. EGLFS, framebuffer, and Wayland platform plugins are not described as verified delivery.
- `translations/` retains Qt standard-control translations; `share/notices/` delivers current notices and source dependency materials.
- `share/deployment.json` records QML, system runtime dependencies, copied SDK libraries, and boundaries; README.md provides usage.

The scanner uses pinned-Qt [qmlimportscanner](https://doc.qt.io/qt-6.8/qtqml-tooling-qmlimportscanner.html). Xdzip.Settings/Xdzip.Native are built-in registered modules; development static libraries are not copied. Missing ordinary dependencies, QML paths escaping the SDK, same-name library conflicts, and unknown external libraries fail. Recursive native-import collection copies only SDK libraries, not host glibc or graphics drivers; system dependencies are listed by actual Debian package ownership.

Pruning validates the copied QML tree, required Basic plugins, retained strong imports and known style layouts before deletion. Unexpected links, unknown styles or dialog selectors, and strong dependencies on omitted modules fail. Default dialogs, input methods, accessibility, image plugins, TLS and system translations retain their existing rules. `share/deployment.json` records removed QML files and byte counts. The private `cli/` shares `bin/xdzip-native-codec` through its fixed context path; standalone CLI packages remain self-contained.

HTTPS additionally requires Ubuntu `libssl3t64` and `ca-certificates`. Verify system-package ownership of `libssl.so.3` and `libcrypto.so.3` and a nonempty system CA trust store. The Qt OpenSSL backend may load these libraries at runtime, so ldd alone cannot establish complete dependencies. The deployment manifest records installed binary package versions, source packages, and source-package versions. Installers declare both dependencies; the actual-file SBOM records them as required OS packages without bundling system libraries in the app.

patchelf sets relative `$ORIGIN` RPATH on every ELF, calculating each plugin's relative distance to lib/. Qt path redirection follows [official qt.conf rules](https://doc.qt.io/qt-6.8/qt-conf.html), allowing relocation of the entire directory. Libraries remain dynamically linked and replaceable; compatible Qt replacement is not restricted. See [Qt Linux deployment](https://doc.qt.io/qt-6.8/linux-deployment.html) for general boundaries.

After relative RPATH rewriting, deployment strips unneeded native symbol tables and verifies that every DT_NEEDED and RPATH/RUNPATH entry is unchanged. It records tool version, before/after hashes and byte counts for each ELF in `symbol_stripping`; SDK inputs and original build binaries remain untouched. For a diagnostic copy, the underlying deploy script accepts `--keep-symbols`. Final SDK attribution and application hashes describe the transformed files. This reduces package bytes, not a measured memory or throughput improvement.

Use the [unified build entry point](developer-build.md) in stages for routine generation and verification. Complete same-configuration build/test and packaging preparation first, then run:

```sh
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

The default build directory is build/desktop, delivery output is build/package, and Qt SDK is `build/toolchain/Qt/6.8.2/gcc_64`; pass shared options at each stage for other locations. `package --kind runtime` generates the runtime directory and audit materials; then `package --kind installer` with the same output-dir generates tar.gz and .deb. Default `all` performs both in order. Underlying [deploy_linux.py](../../scripts/deploy_linux.py), [package_linux.py](../../scripts/package_linux.py), and [verify_linux_runtime.py](../../scripts/verify_linux_runtime.py) retain platform logic; consult their `--help` for troubleshooting.

Deployment runs only on native Linux x86_64 and does not overwrite existing directories. Packaging preflight checks qmlimportscanner, ldd, patchelf, dpkg-query, and installer-required dpkg-deb; these build tools do not enter the product. Container runtime verification additionally requires Docker and an accessible daemon, with builds limited to the pinned Ubuntu image and recorded OS packages. Missing Docker does not block ordinary build/test, but verify fails explicitly; an unexecuted isolation check is not recorded as passed. Ordinary verify does not install .deb on the host or change file associations; installation lifecycle remains a disposable CI check.

## Automated evidence without the SDK

The container uses Ubuntu 24.04 pinned by digest and installs actual OS runtime dependencies plus Xvfb/font/locale test support, but not Qt, Rust, or a compiler, and checks their absence again. It mounts neither the development SDK nor the complete source tree, only the read-only product directory, a separate project-owned 100,000-entry ZIP, and verification scripts. Archive runtime verification disables networking and privilege escalation; only the temporary configuration directory is writable.

Mount the application under a new path containing spaces. After clearing development search variables, use the launcher to check:

1. Actual `--version` runs.
2. A real QML window starts and closes safely under X11.
3. The native product probe completes first frame, 100,000-entry reading, active-request cancellation, and exit; counts and cancellation outcomes satisfy the contract.
4. After clearing host development variables, verify all packaged ELFs in deployed mode with no SDK fallback. The actual-file SBOM retains the same version/commit identity and passes the official schema.
5. Enable networking in a new process from the same SDK-free image, run product `--update-probe` with isolated `XDZIP_SETTINGS_PATH`, and actually request GitHub HTTPS. It must report an available OpenSSL backend and complete the update check; retain separate `.https.json` and `.https.log`. Network, certificate, or TLS failures block verification.

The HTTPS probe separately records OpenSSL backend, system CA file, and actual GitHub request results. A successful real request alone does not establish signature verification or native update installation. Signature parsing and SDK-free update workflows are verified by the [separate update regression workflow](../../.github/workflows/update-regression.yml); these evidence types cannot replace each other.

Retain raw logs and structured reports for each probe attempt as `.https.attempt-N.log` and `.https.attempt-N.json`, plus `.https.log` for the final attempt. Missing or invalid reports, timeouts, and process exits retain failure status. Retry only when the product controller confirms anonymous GitHub release-list rate limiting and status, request phase, response headers, and diagnostics are complete. Wait until the server deadline or use bounded backoff, with at most three attempts and 3,700 seconds of total waiting, matching the macOS acceptance probe. A deadline exceeding the remaining budget, an interrupted wait, or a TLS, network, or other error fails immediately. The probe must complete a real update request to pass. Record each wait decision and actual duration in `.https-waits.json`.

Before reusing an output path, move the previous report and logs to a `.history` directory named for the report in the same parent, with a separate subdirectory per run. If a new run fails midway, the old success report does not remain at the current output path, avoiding confusion; historical artifacts remain independently inspectable.

linux-runtime.json records actual events and executable SHA-256. Xvfb/Qt Quick software rendering establishes only the runtime flow, not physical GPU, Chinese candidate windows, real screen readers, or complete desktop installation.

## Download delivery and remaining gates

CI generates xdzip-linux-x86_64.tar.gz and records SHA-256 and executable hashes. tar.gz preserves executable permissions, avoiding launcher permission loss from Actions ZIP extraction. Archive paths, UID/GID, and timestamps are normalized without private directories. After download, extract and run `./xdzip` inside the directory; do not move only bin/xdzip. The raw linux/ directory is not uploaded again.

The `build/package` runtime directory, tar.gz, and .deb generated here are experimental development deliverables and do not automatically install or change default associations. Official Release Ubuntu tar.gz and .deb are generated by the dedicated Release workflow and supplied under the [release protocol](release-protocol.md). Platform scope follows the [support policy](support-policy.md); other distributions/desktops, Wayland, and related expansions need separate issues. Existing desktop acceptance remains in [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). Reverify affected real IME/screen-reader and clean install/upgrade/uninstall checks for later changes; rebuilding a directory alone does not require repeating existing acceptance. Historical `v0.1.0` component review is in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53); successful directory execution does not automatically complete future release gates.

See [installation and associations](installers-associations.md) for Ubuntu .deb install/upgrade/rollback/uninstall, runtime locks, and desktop MIME automation. Passing the runtime container does not replace real desktop checks or release license review.
