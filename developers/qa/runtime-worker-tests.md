# Runtime tests through the real worker

English | [简体中文](../zh-CN/qa/runtime-worker-tests.md)

The [runtime integration inventory](../../specs/engineering/runtime-worker-tests.json) declares every environment-dependent runtime library test and its prerequisites. Ordinary `cargo test` leaves these tests opt-in because they need a separately built isolated worker, fixed external data or substantial temporary storage. An ignore marker is not verification evidence.

Use an existing project-built `xdzip-native-codec` worker. The following entry point prepares authorized inputs, builds only the runtime test binary and executes the inventory serially. It does not build Qt, invoke author tools, install the product or grant release permission.

```powershell
python scripts/check-runtime-worker-tests.py --worker build/desktop/tools/native-codec/xdzip-native-codec.exe --output build/runtime-worker-tests
```

Omit `.exe` for Linux/macOS. The default selects the complete inventory. `--profile worker` selects the 24 ordinary worker cases, while `--profile large` selects four cases: version-26 RAR above 2 GiB, 513 MiB 7z creation, a genuine 4.25 GiB RAR7 stream, and the million-record index. These cases retain their complete sizes and assertions; they are not replaced with small stand-ins. Reserve sufficient temporary storage for the large 7z output and index. The large RAR checks do not write their decoded multi-GiB plaintext to disk.

`--include-regular` runs the entire runtime library with the integration tests enabled. The compiled test inventory and every actual outcome must agree, with zero ignored results. A successful process running zero tests, an incomplete selection or an ignored result fails verification. `--case <full-test-name>` can be repeated to retry specific cases without rerunning successful unrelated tests; it cannot be combined with a profile or `--include-regular`.

The runner reuses [fixed RAR input preparation](../../docs/engineering/native-codecs.md), the pinned RAR7 data/notice checker and the project-owned ZIP64 generator. Public inputs keep their size, hash, response and provenance restrictions; no third-party decoder source or author executable is downloaded. `--rar-cache` accepts an existing prepared public-data directory or its original cache layout, and `--rar7-cache` accepts the prepared RAR7 `inputs` directory. `--offline` requires these verified inputs to be available and fails rather than silently skipping missing data. Mathematical unknown-size headers test rejection, not RAR algorithm compatibility.

Each attempt has its own directory below the specified `build/` output. TEMP/TMP/TMPDIR point to a private directory for that attempt, including on Windows. Reports retain the worker and test-binary hashes, source identities, exact case outcomes and logs. Timeouts stop the owned test process tree. Failed and unexecuted cases remain distinct from passes. The parent process's environment is not changed. Clean temporary attempts with [the repository cleanup script](../engineering/repository-hygiene.md#研发周期工件收尾) after preserving necessary receipts.

The desktop test entry and Release contract job call this same complete inventory after ordinary Rust checks. These automated results cover their actual worker/task paths, independently of GUI operation, other platforms or formal release acceptance.
