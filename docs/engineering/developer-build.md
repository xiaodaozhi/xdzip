# Development builds and platform verification

This is the unified entry point for preparing, building, testing, packaging, and verifying environments on three platforms. Shared orchestration is implemented by [scripts/desktop.py](../../scripts/desktop.py). Platform documents supplement deployment layouts, tool behavior, and system limits rather than maintaining separate routine commands.

The standalone CLI uses `python3 scripts/cli.py build`, `test`, and `package` without Qt, X11, or a graphical session. Linux compilation also requires `libssl-dev pkg-config`; runtime uses system OpenSSL and certificates. Desktop builds automatically create a private CLI directory. See the [CLI contract](command-line.md) for commands, installation, and trusted-update boundaries.

Development builds remain experimental and are not official Release installers. See the root [README](../../README.md) for capabilities and limits and the [support policy](support-policy.md) for supported scope. Historical `v0.1.0` release review is in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53); future versions require new review records for their specific candidates.

## Environment and entry points

First install Python 3.10+, Git, CMake 3.24+, Ninja, rustup, and the platform compiler/SDK. The Rust version comes from root rust-toolchain.toml; Qt is pinned to 6.8.2. See [toolchain policy](toolchain.md) for dependency and update rules. Run commands from the repository root. On Windows, replace `python3` in examples with an available `python` and use an x64 MSVC development environment.

## Local toolchain directories

`prepare` builds and verifies a security-patched QtNetwork in the existing SDK for CVE-2026-116337 and CVE-2026-105946; other runtime modules are not replaced by the rebuilt library. Perl, make (nmake on Windows), and the OpenSSL command-line tool are also required. Ubuntu requires `zlib1g-dev libpcre2-dev libicu-dev libglib2.0-dev libzstd-dev libkrb5-dev`. Source, generated declarations, and build caches stay under `build/`, defaulting to reuse of `build/toolchain/qt-network-security/<platform>`. Product build and packaging fail immediately if input and actual library digests do not match; rerun `prepare`. See [patch maintenance notes](../../patches/qt/README.md) for provenance, ABI, TLS functionality, and failure recovery.

Local dependencies for the unified entry point use only `build/toolchain/`, which Git ignores and does not commit:

| Content | Fixed directory | Notes |
| --- | --- | --- |
| Qt SDK 6.8.2 | `build/toolchain/Qt/6.8.2/<platform-subdirectory>` | `<platform-subdirectory>` is `macos`, `msvc2022_64`, or `gcc_64`; qmllint, qmake, CMake configuration, and Qt runtimes come from here |
| Python tool environment | `build/toolchain/desktop-tools` | Fixed virtual environment for aqtinstall, packaging, and verification scripts |
| Qt download staging | `build/toolchain/qt-install-tool` | Qt installer downloads and temporary inputs only |

`scripts/desktop.py prepare --install-qt`, `doctor`, `build`, and `test` share these locations by default. Do not create separate `.cache/Qt`, `.cache/desktop-tools`, or similar copies for each development pipeline. Pass `--qt-prefix`, `--qt-root`, and `--tools-dir` explicitly only when isolating different Qt versions, platforms, or experimental builds. CI may explicitly point these to runner temporary directories, but CI fallback paths do not change local defaults. CMake, Ninja, rustup, Rust, and the platform C++ compiler belong to the host toolchain and are installed/invoked under environment preflight requirements, not copied by the Qt tool directory.

After an R&D cycle and cleanup of branches and worktrees, remove temporary directories and files created under `build/` for that cycle. Retain reusable shared toolchains such as `build/toolchain/`; remove cycle-specific builds, diagnostics, and download staging. See [repository content and history management](repository-hygiene.md#研发周期工件收尾) for rules and manual fallback.

| Platform | System environment to prepare first | Qt SDK subdirectory |
| --- | --- | --- |
| macOS | Native Apple Silicon terminal, Xcode Command Line Tools, and valid macOS SDK | macos |
| Windows | MSVC 2022 x64 C++ tools and Windows SDK, x64 Developer PowerShell/command line | msvc2022_64 |
| Ubuntu 24.04 | x86_64 C++ compiler, OpenGL/EGL/xkbcommon development packages, and X11 runtime libraries | gcc_64 |

On macOS, `xcode-select --install` prepares command-line tools. Ubuntu base development packages include `libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev libnautilus-extension-dev pkg-config`; packaging also needs patchelf and Debian package tools, and isolated runtime verification needs accessible Docker. Merely installing Visual Studio on Windows without entering a development environment may leave cl, link, rc, or dumpbin unavailable; preflight identifies the actual missing tools. The shared entry point does not automatically install system compilers, SDKs, or system packages requiring administrator privileges.

Windows developers may prepare their own tools under the [Community free-license path](windows-licensing.md), reading and accepting that version's terms. Build Tools or VC_redist alone does not establish eligibility. Enterprise use and associated redistribution on GitHub standard hosted runners have separate public authority; maintainers need not buy Enterprise for hosted builds or install a Community IDE locally, and that authority does not apply to self-hosted machines. Existing 2019 tools need not be uninstalled for license review; compatibility of local alternatives with the current 2022 release SDK must be verified separately.

All commands share these options:

| Option | Default and purpose |
| --- | --- |
| `--qt-prefix` | First `XDZIP_QT_PREFIX`, otherwise `build/toolchain/Qt/6.8.2/<platform-subdirectory>`; points to the SDK prefix, not bin |
| `--build-dir` | `build/desktop`, CMake build and test directory |
| `--output-dir` | `build/package`, deliverable artifact and verification output directory |
| `--tools-dir` | `build/toolchain/desktop-tools`, isolated Python tools and preparation materials |
| `--configuration` | `Release`; another development configuration may be selected explicitly |

build-dir and output-dir must be independent subdirectories inside repository build/ to match underlying packaging and verification output boundaries. Keep the same options across stages of one run. Explicit `--qt-prefix` takes precedence over the environment variable; global CMAKE_PREFIX_PATH changes are unnecessary. The entry point prepares Qt and tool search environments for subprocesses. Ordinary desktop launches should not inherit offscreen-test variables. Generated output goes only into ignored build/, target/, dist/, or tool caches and is not committed.

## Staged execution

Stages can run independently and do not implicitly rerun previous preparation, build, or test stages. Missing prerequisites produce errors; run the corresponding stage and retry. Each stage checks its required environment before work; `doctor` previews problems without building or packaging.

Protected Windows builds coordinate processes within their existing console. Separate consoles may run in parallel; builds sharing one existing console run sequentially. During a build, other code must not detach or reattach protected processes from that console. A temporary console created by a process without one is for that process and its children; outside processes must not attach. Older scripts use a common fallback lock for consoles without window handles and cannot coordinate with newer scripts; do not start a new-version build in the same windowless console until an older build finishes. If process lists cannot be determined, stop without changing the code page.

For noninteractive builds, an independent-console wrapper may be used explicitly with stdin, stdout, and stderr fully redirected. For example, in PowerShell:

```powershell
New-Item -ItemType Directory -Force build\isolated-logs | Out-Null
cmd /c "python scripts\windows_detached_console.py --timeout 7200 -- python scripts\desktop.py build <NUL >build\isolated-logs\desktop.log 2>&1"
$LASTEXITCODE
```

The wrapper creates an independent hidden console per command and terminates children on timeout or supervisor exit; missing completion signals fail. Parallel builds still require separate `--build-dir` values; the wrapper does not lock build directories. Existing desktop-build and `windows_ninja_safety.py` dependency checks still run, but wrapping other commands does not add them automatically. This entry is only for noninteractive commands; prompts, password input, and pseudoconsole interaction are unsupported.

`prepare` verifies the current product format-source inventory. ZIP, TAR/GZIP, 7z, RAR, CAB, and optical containers use independent Rust; the official product source list is empty and official UnRAR/7-Zip engines are not downloaded or compiled. Independent test references are prepared from their own pinned sources and separated from product runtime dependencies. Original source notices for limited RAR mappings remain in packaging and corresponding-source delivery; see the [license contract](licensing.md). `build` uses prepared inputs only and does not download floating versions. For Rust-only testing, first build `xdzip-native-codec`, point `XDZIP_NATIVE_CODEC` to that worker, then run workspace tests. The native backend task flow additionally runs `cargo test -p xdzip-runtime --locked native_backend_tests -- --ignored`; both unified `test` and Foundation CI require it. An ignored status is not a passed format check. See the [native backend contract](native-codecs.md) for source, stream authorization, and licensing boundaries.

| Command | Purpose and prerequisites |
| --- | --- |
| `doctor --for-stage build` | Check PATH, pinned Rust, Qt, compiler, SDK, and versions; checks build environment by default |
| `prepare` | Prepare shared tools and dependencies; downloading/installing Qt, Rust toolchains, and packaging materials requires explicit switches below |
| `build` | Configure and build real product, tests, and diagnostics; environment must be prepared, uncommitted source changes permitted |
| `test` | Run Rust workspace tests, qmllint, and CTest by default; requires a completed same-configuration build, uncommitted changes permitted |
| `package` | Deploy runtime, collect licensing/provenance materials, audit deployment, and produce selected artifacts; requires clean source, same-commit build/test receipts, and packaging tools |
| `verify` | Verify an existing runtime or package without rebuilding, installing the product, or changing default associations; requires the artifact and platform verification environment |

Preflight distinguishes required missing tools, version mismatches, and tools unnecessary for the current stage; missing Docker or Inno Setup does not block ordinary development builds. Packaging preflight actually checks Rust/Cargo invocation and locked dependency metadata so PATH or dependency-cache problems are not discovered only during license collection. Failed commands and verification limits remain in local/CI output; partial completion is not reported as a pass.

First prepare the pinned Qt and Rust toolchains:

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt --qt-root build/toolchain/Qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

`--install-rust` uses installed rustup to prepare the repository-pinned toolchain; it does not install rustup. `--install-qt` explicitly downloads the pinned Qt SDK to `build/toolchain/Qt` by default. An existing SDK does not need to be downloaded again; pass its path to each stage:

```sh
python3 scripts/desktop.py doctor --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
python3 scripts/desktop.py build --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
```

After building, explicitly prepare public test inputs for the complete `native_backend_tests` batch. CAB, ISO/UDF, and basic RAR fixtures are retained in project source under `tests/fixtures/cab-iso/` and `tests/fixtures/native-archives/`. 7z tests create their own archives with the freshly built Rust worker. This runtime batch does not require official 7-Zip/UnRAR tools or mounted images; other Foundation interoperability references follow the [native backend contract](native-codecs.md).

For the first acquisition of pinned public RAR PPM, legacy-password, and other inputs, explicitly run:

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs
```

With an existing pinned cache verified by size and SHA-256, explicitly use offline mode, replacing the cache path and retaining the manifest's `junrar/` subdirectory layout:

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs --archives /path/to/fixed-rar-cache --offline
```

Pinned sources and notices are in `scripts/rar-rust-fixtures.json` and its public compatibility inventory. Output `sources.json.environment` uses paths relative to the source root. Current corresponding-source delivery does not contain these seven public cache files; Cargo `--offline` does not supply them. The preparer does not export shell environment variables. Before complete `test`, start a child process using the explicit handoff below:

```sh
python3 -c "
import json, os, pathlib, subprocess, sys
root = pathlib.Path.cwd()
worker = pathlib.Path(sys.argv[1]).resolve(strict=True)
prepared = json.loads((root/'build/native-test-inputs/sources.json').read_text(encoding='utf-8'))
environment = os.environ.copy()
environment.update({name: str(root/value) for name, value in prepared['environment'].items()})
environment.update(XDZIP_NATIVE_CODEC=str(worker), XDZIP_NATIVE_CODEC_WORKER=str(worker), XDZIP_CAB_BACKEND='rust')
environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'
subprocess.run([sys.executable, 'scripts/desktop.py', 'test', *sys.argv[2:]], env=environment, check=True)
" build/desktop/tools/native-codec/xdzip-native-codec
```

On Windows, add `.exe` to the worker path. With a custom SDK, build-dir, or tools-dir, append the same options after the worker path, for example `--qt-prefix /path/to/Qt/6.8.2/gcc_64`, and adjust the worker path to the actual build directory. `XDZIP_CAB_BACKEND=rust` selects the fixed Rust CAB check; `password` is an existing public fixture test value passed only to the test child. This handoff retains default workspace tests and the complete ignored batch; it neither downloads inputs nor implicitly builds the worker.

`test` enables offscreen/software only for children and does not contaminate later desktop commands. For routine changes, rebuild and verify according to impact. Rust-core-only verification can run `cargo test --locked` for the affected crate without Qt. `test --skip-rust` runs qmllint and Qt CTest but skips the Rust workspace and complete native backend tests; receipts mark Rust `not-run`. When a Rust conclusion is needed, execute it separately and record actual results.

Browser tests may run directly or in groups through the existing suite entry point. Each suite process creates a unique short temporary root under project `build/test-tmp/`. On exit, after test objects are destroyed and before Qt application destruction, restore TEMP, TMP, and TMPDIR and clean up. Default settings files are also isolated. Unix checks the byte length of the temporary root plus existing IPC name against actual sun_path capacity. A path that is too long explicitly fails the fixture precondition; shorten the checkout path rather than moving outside the project or changing product IPC. Auxiliary processes in a group inherit the parent temporary root to preserve single-instance IPC directory identity, use private preflight probes, and finish before parent cleanup. Different groups do not share roots. Language-restart and open-mode children explicitly passed test configuration continue sharing it rather than using a new empty configuration. Before running, actually verify child-file creation, locks, staging, atomic commits, and settings readback. Failure reports `Fixture precondition` and a nonzero exit; do not skip tests or weaken window-hiding assertions. Directory existence and QTemporaryDir::isValid alone do not prove writability, and system permissions are not modified to make fixtures pass.

Test configuration paths use `tests/test_environment` scoped save/restore. Windows retains wide-character values and distinguishes unset from explicitly empty entries in the system environment block; Unix retains original environment bytes. The MSVC CRT setter deletes empty values, so the native environment block is restored separately. Check empty-value presence through the helper's processValue or QProcessEnvironment rather than promising Qt's CRT empty/null distinction. This helper is restricted to the serial test main thread: set before relevant engines/workers are created, restore after destruction/join. It is not for product code or concurrent environment changes. ASCII, Chinese, and ANSI-unrepresentable characters require checking actual files at the original path, language-restoration children, and window/draft preservation on failure. Automated results remain separate from real-person settings operations.

The real product uses the name `xdzip` on every platform. Default build entry points:

| Platform | Entry point |
| --- | --- |
| macOS | `build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip` |
| Windows | `build/desktop/apps/desktop/xdzip.exe` |
| Linux | `build/desktop/apps/desktop/xdzip` |

The `xdzip-ui-preview` component example is not for real-file acceptance. Adjust paths after changing build-dir. Actual build identity records version, channel, commit, and dirty state. See the [Qt development application](../../apps/desktop/README.md) for modules and low-level CMake debugging.

## Packaging and verification

`test` and `package` check build receipts, source fingerprints, configuration, SDK identity, and actual product files. Rebuild first after source or configuration changes. Commit deliverable source before packaging, then build and test from that clean commit. package must not bypass checks by ignoring dirty state, changing artifact identity, or reusing old reports. License materials, Rust metadata, and actual CLI version must match this source. Platform tools check use and overwrite restrictions for existing package directories. Prefer a new output-dir for rebuilt candidates to avoid old-file mixing.

On Windows/Linux, prepare packaging tools, then run the unified stages:

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py build
python3 scripts/desktop.py test
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

`prepare --packaging` prepares isolated Python validation dependencies, pinned license texts and schema caches, and platform packaging materials; Windows includes pinned CRT and Inno Setup. Developers still install system tools first. `package --kind runtime` prepares only the deployed runtime directory and audit materials. The same output-dir can then run `package --kind installer` to create an installer from the verified runtime; Linux also creates a tar.gz preserving executable permissions. `--kind all` is the default and performs both in one command. Windows packaging uses Release or RelWithDebInfo, without deploying Debug Qt libraries. Windows provides a deployed directory and per-user installer; Linux provides a runtime directory, tar.gz, and .deb. Windows verification requires a same-configuration browser test executable and Qt Test/offscreen test inputs. Linux verification requires m2-flat.zip generated by build tests. Test files do not enter the product directory. verify `--kind runtime` verifies the runtime; default `--kind all` verifies selected deliverables. Ordinary verification does not install, upgrade/roll back, or change user file associations.

A macOS DMG requires a separately selected primary build permitted to register as a document-handler candidate, preventing ordinary development/test copies from entering “Open With”:

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

Ordinary macOS build handler rank defaults to `None` and does not change user defaults; select `Alternate` only for a long-lived primary app or DMG input. Local macOS package supports only `all`, producing an arm64 development DMG with necessary ad-hoc integrity signing. verify checks relocated Cocoa/Metal and real extraction. Ordinary CI must not run Mac package or deploy, sign, compress, or upload an app package; see the [release protocol](release-protocol.md) for the dedicated Release exception. See the [Mac deployment guide](macos-deployment.md) for full limits and copy-registration maintenance.

See [Linux deployment](linux-deployment.md), [Mac DMG](macos-deployment.md), and [installation and file associations](installers-associations.md) for layouts and underlying tools. Windows/Linux installers are not formally signed; macOS has neither Developer ID signing nor notarization. Diagnose installation/runtime issues using the actual product and relevant platform verification; running under the SDK does not establish standalone-package verification.

## Relationship to CI

Before committing or pushing, run local builds or verification appropriate to the proposed content. Documentation changes run `python3 scripts/check-docs.py` and check affected generated output. Rust changes check formatting and at least build or test affected crates. Tests involving isolated format backends prepare the actual worker and required fixtures as described above. C++, bridge, and QML changes build the affected product with `python3 scripts/desktop.py build`, then run relevant tests. Every change does not require all format interoperability or ten repeated complete browser-suite rounds.

When adding or changing a QML window, run `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py`, and affected bilingual multi-font-size layout regressions. After a build, `python3 scripts/desktop.py test --skip-rust` runs full Qt component and CTest checks. Alternatively, after configuring Qt SDK paths and offscreen environment, run relevant cases through existing entry points such as `ctest --test-dir build/desktop -R qt_task_components --output-on-failure`. Radio/checkbox changes also execute QAccessibleActionInterface regressions appropriate to the change, such as `accessibleRadioActionsCommitSelection` and `accessibleCheckboxActionsCommitIntent`.

[Routine CI](../../.github/workflows/docs.yml) runs lightweight static checks only and does not replace local product builds. [Foundation](../../.github/workflows/foundation.yml) and the [Qt workflow](../../.github/workflows/qt-components.yml) run manually on demand. Qt uses the same desktop.py for builds, Qt tests, and Windows/Linux packaging and runtime verification. [ci-desktop.ps1](../../scripts/ci-desktop.ps1) retains on-demand repetition, visual, interoperability, and installation-compatibility checks and is not mandatory for routine local verification.

Historical full-product, real install upgrade/rollback, and association lifecycles run only on disposable Windows/Linux CI hosts or isolated containers; the unified entry point does not make them default ordinary local verify behavior. The macOS job in the on-demand Qt workflow uses SDK-mode deployment audit and Cocoa startup/file-open checks, retaining only logs and JSON. See [CI policy](ci-policy.md) for full checks and evidence boundaries.

## File and task boundaries

Opening browses only; it does not automatically extract. Regular same-name files support Ask, Skip, Rename, and explicit Replace. Publish a whole file only after content verification and synchronization. Cancellation before commit or write failure preserves the old file; previously committed extracted entries remain. Closing a task window only hides details. Exit uses the active-task decision and asynchronous shutdown barrier.

ZIP can restore relative symbolic links and executable bits under restricted POSIX conditions but not privilege bits, extended attributes, or original modification times. This does not establish app-bundle installation or signature verification. Source and destination must satisfy the [safe extraction contract](m3-extraction.md). Crash staging is only reported and retained for inspection; unknown content is not automatically recovered or deleted.

<a id="windows-msix-实验构建"></a>

## Experimental Windows MSIX build

MSIX reuses the Windows Release desktop build, tests, and audited runtime directory from the same commit. Complete `prepare --packaging`, `build`, and `test` as above, then run with the same options:

```powershell
python scripts/desktop.py package --kind runtime
python scripts/desktop.py package --kind msix --msix-name xdzip.Development --msix-publisher 'CN=xdzip development' --msix-publisher-display-name xdzip
python scripts/desktop.py verify --kind msix
```

The example identity is for local development only. A formal Store identity's package name, publisher, and publisher display name must match Partner Center. Output is `build/package/xdzip-windows-x64.msix`, an unsigned experimental package, not automatically a distributable sideload package or Store-certified. Normal sideloading requires a device-trusted signature matching the publisher. Windows 11's restricted unsigned development mode additionally requires a specified publisher OID and, for executable content, typically administrator rights; the example identity does not meet that mode. Scripts do not create certificates, sign, install, or submit to the Store. The technical minimum is Windows 10 22H2 x64 and does not expand supported-platform commitments. Version `1.0.<product build>.0` is separate from display version; Store version policy must be revisited before build numbers are exhausted.

Building requires clean source, matching build/test receipts, a fully audited runtime directory, and an MSIX-capable Windows SDK MakeAppx. Scripts recheck the runtime and MSIX update-capability marker, generate manifest and brand icons in separate staging, compare every packaged file, and record hashes. They do not overwrite existing packages or staging. The manifest declares nine existing extension types and a full-trust desktop entry point, not a context-menu extension. Licensing, real installation, associations, uninstall, Store signing, and actual Windows 10/11 acceptance must be completed separately and are not inferred from successful packaging.

The MSIX manifest explicitly declares `en-US` and `zh-CN` to match built-in English and Simplified Chinese catalogs; receipts record and verify the language list. This MakeAppx flow does not automatically convert Qt translation resources into package language declarations. Partner Center's “Languages supported by the package” reflects the uploaded package; after correcting source, rebuild and replace the draft package to update it. “Additional Store listing languages” are configured independently and cannot substitute for manifest declarations. See [Microsoft's manifest language documentation](https://learn.microsoft.com/windows/apps/design/globalizing/manage-language-and-region).

## Evidence and reverification

Select affected checks under [general verification](../qa/README.md) and [on-demand desktop verification](../qa/private-test-manual.md); routine work does not require a manual acceptance form. Build and test receipts are `desktop-build.json` and `desktop-test.json` under build-dir. Prepare, package, and verify receipts are `<stage>.json` under output-dir and link underlying evidence. Automation receipts record actual stages and sources. Offscreen/software rendering, native Cocoa checks, and performed manual checks are recorded separately and do not substitute for one another. Existing acceptance remains in [#5](https://github.com/xiaodaozhi/xdzip/issues/5) and is not repeated merely because a candidate inventory is regenerated.

Windows/Linux artifact delivery includes same-commit source and license notices; Actions retains artifacts for 14 days. After expiry, rebuild using identities and pinned inputs. deployment-audit.json and source/file-level CycloneDX inventories support provenance verification but may remain incomplete and cannot establish license approval. See [source delivery](source-delivery.md) and the [automated verification contract](wp12-automation.md) for source rebuilding and Qt replacement.

Builds, screenshots, measurements, reports, and packages remain only as ignored local or CI artifacts. Do not describe generated pending forms, offscreen checks, or partial SBOMs as clean-environment and complete release-gate passes.

## macOS update framework

Native builds use `prepare-sparkle.py` to prepare `build/toolchain/Sparkle/2.10.0-xdzip-proxy.2`. The framework's main library is compiled from pinned upstream source and the proxy patch; installation helpers, resources, and signing tools retain official SDK provenance. See the [dependency contract](dependencies.md#macos-更新提供者). Valid build receipts may be reused; an old official SDK cannot replace the derived framework with proxy interfaces. CI may explicitly select a temporary SDK directory.

`python3 scripts/prepare-sparkle.py --output build/toolchain/Sparkle/2.10.0-xdzip-proxy.2` uses native Clang from existing Xcode Command Line Tools or Xcode and needs no separate Qt SDK. Original SDK and source archives are cached in `build/source-cache/`; offline operation may pass `--archive` and `--source-archive` separately. The build does not execute scripts from downloaded source, modify system proxies, access production signing keys, or collect project source-delivery archives.

`check-macos-proxy.py` defaults to project-owned loopback downloader checks only. `--system-conflicts` is restricted to disposable hosted Apple Silicon runners in a dedicated workflow. A constrained helper stages, restores, and verifies system proxy and fixed test-domain hosts settings. Do not add this flag locally.
