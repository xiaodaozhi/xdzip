# Candidate automated verification and evidence aggregation contract

Automated verification and external admission are registered separately in the [gate list](../../scripts/wp12-gates.json). Scripts check candidate binding and evidence completeness; completing automated items does not approve a formal release. Work-package progress and acceptance status belong in [Issues](https://github.com/xiaodaozhi/xdzip/issues); historical WP-12 completion is in [#5](https://github.com/xiaodaozhi/xdzip/issues/5).

<a id="qt-源码与兼容替换"></a>

## Qt source and compatible replacement

The [pinned source manifest](../../scripts/corresponding-sources.json) contains QtBase, QtDeclarative, QtSvg, QtTranslations, build-required ShaderTools and QtTools, and pinned Rust standard-library source components. Archives and source patches have SHA-256 values. Reverify regular files before restoring source trees; accept only original texts or recorded patches and do not carry unknown modifications into the next build.

```sh
python3 scripts/build-qt-source.py --output build/wp12 --resume \
  --apple-sdk /Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk --parallel 6
python3 scripts/verify-qt-source-product.py --qt-build build/wp12 \
  --output build/wp12/source-product
```

The source build installs under an isolated build/ directory without replacing system or existing SDKs. QtTools retains Linguist and lrelease but explicitly disables the optional Clang parser and WrapLibClang lookup to avoid incomplete runner LLVM development configurations. A compatibility patch explicitly includes ARM instruction headers and adds a read-only proof export to QtCore without changing existing public functions. The real application locates this fixed C ABI marker in its in-process loaded QtCore and fails on absence or mismatch. Full product compilation, six CTest groups and 100,000-entry browsing/search/sort/cancel verification run together; a string present in a file is not proof of loading.

<a id="修改-qt-区域数据"></a>

### Modifying Qt locale data

This example assumes a completed `build/wp12` source build above and obtained CLDR 46 data and ISO 639 input table. Qt's locale generator requires the source directory to be named `qtbase`; the delivery script extracts a differently named directory, so first copy the source for editing. The example CLDR path must contain `common/main/root.xml`. The table can use limited derived inputs as described in source delivery, or independently obtained and converted iso-codes data. Check each input's provenance and license first.

For a specific new language, the [source delivery guide](source-delivery.md) also permits checking its subtag against the delivered IANA registry and explicitly adding reviewed code fields to the limited table. IANA registration alone does not establish ISO 639-2/3 field correspondence. Upstream rules still require updating Qt enums and CLDR data; adding a table row alone is insufficient.

```sh
python3 -c "from pathlib import Path; import shutil; shutil.copytree(Path('build/wp12/qt-src/qtbase-everywhere-src-6.8.2'), Path('build/qt-locale-edit/qtbase'))"
python3 build/qt-locale-edit/qtbase/util/locale_database/cldr2qlocalexml.py path/to/cldr-root build/qt-locale-edit/locale.xml
python3 build/qt-locale-edit/qtbase/util/locale_database/qlocalexml2cpp.py build/qt-locale-edit/locale.xml path/to/qt-locale-codes.tsv build/qt-locale-edit/qtbase
```

Regenerate after changing CLDR or the table. After checking output, copy the following eight relative files from `build/qt-locale-edit/qtbase` to corresponding locations in `build/wp12/qt-src/qtbase-everywhere-src-6.8.2`, then run the commands below. The original Qt source archive is unaffected; both directories are local build copies.

- `src/corelib/text/qlocale_data_p.h`, `src/corelib/text/qlocale.h`, `src/corelib/text/qlocale.qdoc`.
- `src/corelib/time/qhijricalendar_data_p.h`, `src/corelib/time/qjalalicalendar_data_p.h`, `src/corelib/time/qromancalendar_data_p.h`, `src/corelib/time/qtimezoneprivate_data_p.h`.
- `util/locale_database/testlocales/localemodel.cpp`.

```sh
cmake --build build/wp12/qt-build/qtbase --parallel 3
cmake --install build/wp12/qt-build/qtbase
```

Do not run `build-qt-source.py --resume` again on modified source: the automated entry intentionally rejects unrecorded source changes. These CMake commands use the existing build configuration and install into the original isolated prefix. Check interface compatibility and application replacement according to the changes. The limited table does not cover every new language; alternative data does not prove upstream historical inputs or complete license status.

The source build requires Qt Network SSL. Linux explicitly enables system OpenSSL and requires development libraries; Windows and macOS use their native TLS backends. Check the installed Qt Network generated configuration and fail if SSL is missing or disabled. Configuration checks do not replace actual network update verification.

The Linux source build also explicitly enables GLib and Zstandard, requires both development libraries and checks Qt Core `qtcore-config.h` and `qconfig.h` respectively. Original alpha.8 Qt Gui depends on private GLib event-dispatcher symbols and the original application uses Zstandard resources. Disabling either prevents the original application starting after replacing Qt Core alone. Configuration checks do not replace actual replacement and loading verification.

The current local Command Line Tools do not provide the upstream reference Xcode version; SDK 27 libc++ is incompatible with old Qt 6.8.2 iterators at compile time. Existing SDK 26.5 can be explicitly used locally. The script records actual SDK settings hash, configuration and compiler without claiming the combination is in Qt's official support matrix or byte-for-byte reproduction of the original prebuilt SDK. See official [macOS source builds](https://doc.qt.io/qt-6.8/macos-building.html) and [Qt module builds](https://wiki.qt.io/Building_Qt_6_from_Git).

The [independent source-verification workflow](../../.github/workflows/qt-source.yml) is manually dispatched only from trusted main and runs three-platform source builds and real-product verification. Windows/Linux may reuse dependency caches keyed by runner image, configuration and build-script fingerprints; partial Windows/Linux compilation state after failure may also be cached. Caches do not mean verification passed: source trees and configuration are reverified and real-product checks rerun. External PR caches are not used. Mac uploads only bounded logs and JSON, without caching/deploying/signing/compressing/uploading apps or SDKs. Source verification on all three platforms isolates rustup toolchains in runner temporary directories and installs pinned version, rust-src, rustfmt and clippy once, avoiding incomplete host installations.

Locally, also run scripts/verify-qt-replacement.py: copy an isolated app from the development package, replace only QtCore with the source build, rewrite development RPATH and apply ad-hoc integrity signatures if needed. Verify recursive imports without an SDK, obtain the source marker from the real product process, complete 100,000-entry interactions and real 32 MiB extraction, and independently check every byte. This differs from the earlier experiment changing only Mach-O identity and still does not guarantee arbitrary ABI changes are compatible.

The trusted-main Windows and Linux source workflows additionally verify a pinned alpha.8 portable package in isolated copies. The original package and source-built QtCore are SHA-256 pinned. The original library must reject the source marker; after replacement the original application process confirms it is loaded, browses and cancels a 100,000-entry fixture, and extracts 32 MiB with independent byte comparison. Windows uses its bundled native platform plugin; Linux uses its bundled offscreen plugin. Only records are uploaded; published assets and system installations remain unchanged. Automated checks do not replace manual acceptance or prove arbitrary ABI compatibility or upstream SDK reproducibility.

A local Apple Silicon Mac may additionally run `scripts/verify-alpha8-macos-qt-replacement.py --qt-build build/wp12 --output build/alpha8-mac-qt-replacement` against the pinned original alpha.8 app.zip. It verifies the original package and contained links, replaces verified source-built QtCore in a temporary copy, applies necessary local ad-hoc integrity signatures, checks original app code sections and imports, in-process marker, browsing/cancellation and real extraction, and cleans its temporary app registration and copy. It refuses CI and does not change installed or published files. Controlled native verification is not manual acceptance, Developer ID signing or notarization.

<a id="性能与资源证据"></a>

## Performance and resource evidence

scripts/measure-desktop.py uses read-only OS counters from tools/process-metrics. macOS reads physical footprint, CPU and disk I/O, converts CPU Mach ticks to seconds using host timebase and cross-checks an owned process's independent CPU clock. Linux uses PSS, CPU ticks and /proc I/O; Windows uses private commit, CPU and process I/O transfer. Windows transfer is not physical disk bytes; memory metrics across platforms are not combined as a single physical-memory ranking. Process start time distinguishes PID reuse. Even 100 ms sampling may miss peaks, short-lived descendants and final I/O before exit.

```sh
python3 scripts/run-performance-suite.py --app /path/to/xdzip \
  --metrics build/ui-release/tools/process-metrics/xdzip-process-metrics \
  --core target/release/xdzip-m3-probe --output build/review/performance
```

The script generates owned fixtures and actually runs 30 new-process first frames, 100,000-entry opening, post-search/sort frame notification and cancellation. After ten seconds of stability, observe at least sixty seconds idle, then thirty seconds browsing memory. The GUI product and the same Rust core each perform real extraction five times with default verification, synchronization and publication; independently check all bytes. Statistics include sample count, median, P95, minimum/maximum, standard deviation and bootstrap intervals. Record GUI-before-core ordering and uncontrolled caches; do not directly attribute timing ratios to GUI overhead.

Collect complete local performance after heavy workloads such as source compilation finish; record build, backend, device and power observations. Observations on the current host are not a release manager-approved stable reference machine. Do not clear caches, change global power saving or disable security checks, and do not automatically approve performance budgets.

Actual embedded resources are exported through an independent opt-in event enumerating only :/xdzip. Downstream checks each against the corresponding qrc source hash, retaining font and project license references; this does not automatically approve generative-asset rights. Linker maps, static library members and retained-object records provide actual linker inputs, not byte-by-byte license attribution after inlining or generic instantiation.

## Aggregation, materials and remaining responsibilities

When reusing the historical material-collection process, `scripts/collect-wp12-ci.py --run <Qt run ID> --foundation-run <Foundation run ID> --ci-run <lightweight CI run ID> --output build/wp12/ci` explicitly selects three successful runs matching the current commit; Qt and Foundation may be manual. The collector checks actual Qt jobs and three-platform artifacts without relying on a full matrix automatically running on every push. Optional `--source-run` points to same-commit Qt source verification and `--packages` additionally collects Windows/Linux development installers. Tokens are sent only to the GitHub API, not storage redirects; large artifacts use bounded range reads and downloaded binaries are not run. scripts/prepare-wp12-materials.py checks SDK declarations, imports, installation/association and SBOM, then organizes each upstream provenance/license gap into concrete review questions. Original differences and incomplete status remain; a hash cannot establish missing publisher build/signing history.

scripts/assemble-wp12-evidence.py automatically assembles CI, source, Qt replacement, performance and three-platform package records and invokes strict gates without manual JSON assembly. scripts/wp12-readiness.py validates each evidence path, hash, commit, platform coverage and type, rejecting dirty builds, mixed identities, missing source-loading markers and insufficient sampling. Only complete automated items produce automatic_complete=true; release_ready remains false and --require-release-pass rejects incomplete external gates.

These aggregation tools remain voluntary for full historical scope, not required forms for every commit or candidate. Automated completion and formal release approval are separate. WP-12's eight manual checks and support policy were closed under the owner's conclusion; see [#5](https://github.com/xiaodaozhi/xdzip/issues/5) and the [support policy](support-policy.md). Rebuilding evidence does not require repeating them. `v0.1.0` provenance and licensing review is in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53); future candidates need new release issues. Select desktop and performance rechecks for new candidates or platform expansions according to actual changes. Scripts do not replace the owner's release decision or licensing conclusions.
