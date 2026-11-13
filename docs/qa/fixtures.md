# Test sample specification

Samples must be self-generated, explicitly authorized or drawn from a test set with clear licensing. Never commit users' real archives, passwords, personal paths or malware of unknown origin. Record source URL, license and redistribution basis for third-party samples.

Each fixture records an ID, generation method/tool version, SHA-256, format characteristics, expected entries/bytes/errors and associated acceptance IDs. Prefer small generatable samples. For large workloads record deterministic seeds, counts, size distributions and generation commands rather than committing gigabyte files to Git.

The minimum set includes empty archives, single files, Chinese/emoji/RTL, duplicate names, case and Unicode normalization collisions, long paths, ZIP64, CRC errors, truncation, unknown sizes, special TAR entries, absolute/parent paths, ADS, symlinks and reparse points. Simulate archive bombs with controlled small quotas and limit test process CPU, memory and temporary storage.

Reference tools must include at least one independent implementation and record its version. A product generator and reader must not self-certify compatibility by sharing the same error. File-system races must run natively on the target OS; simulated path-string tests do not replace them.

`scripts/generate-extract-fixtures.py` generates ZIP link and permission fixtures. `unix_bundle.zip` contains a synthetic Framework with multi-level links, ordinary resources and an executable. An app-bundle-style parent-relative link separately covers `../` normalization. `link_*.zip` covers escapes, backslashes, empty content, NUL, device names, dangling and cyclic links, writes through link parents, overlong content and bad CRC. All are small synthetic fixtures; application contents are not executed. Real third-party archives and extracted outputs are used only for local verification and are not committed.

Password retries use [pinned owned inputs](../../tests/fixtures/extract/README.md), retaining generator source, exact digests and expected verification results. Fixed salts only prevent random collisions in wrong-password assertions; product random creation and two-way interoperability must be tested separately. Large encrypted workloads, corrupted authentication and source changes are generated during tests into local temporary directories or `build/`; generated evidence and user passwords are not committed.
