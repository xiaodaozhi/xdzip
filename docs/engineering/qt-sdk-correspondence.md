# Qt SDK and source correspondence verification

WP-12 source materials use actual official archive bytes; matching version numbers do not establish SDK build equivalence. `scripts/qt_sdk_correspondence.py` associates pinned source archives, source SPDX supplied by the SDK, build declarations, installed configuration, and known consumer patches, writing output to build/. It does not modify the SDK or approve license branches.

```sh
python3 scripts/qt_sdk_correspondence.py \
  --qt-prefix build/toolchain/Qt/6.8.2/macos \
  --output build/review/qt-sdk-correspondence.json
```

By default, read build/source-cache offline and fail if missing. Only explicit --fetch downloads and verifies official archives from the [pinned manifest](../../scripts/corresponding-sources.json). On-demand three-platform Qt workflows run --fetch after installing the pinned SDK and retain JSON with other verification records. Mac still uploads records only, without deploying, signing, or packaging apps. When the source-delivery collector receives --qt-prefix, it includes this record, patches, and pinned manifest in delivery hashes; see [source delivery](source-delivery.md).

## Verification layers

1. Archive SHA-256 must match the pin; archive root, member paths, and fixed `.tag` marker must match. Record marker Git object type separately. For a tree marker, first recompute the fixed original commit-object hash and verify its tree field. Read files individually without extracting upstream source.
2. If the SDK provides a module's .source.spdx, compare each declared SHA-1 against archive content. Record matching counts, both hashes for differences, SDK-declared files missing from the archive, and uncovered archive counts. SHA-1 checks upstream's original declaration only; the complete archive is separately bound by pinned SHA-256. Missing declarations, differences, and uncovered files are not filled in as passes.
3. Retain SDK module-build SPDX hashes, compiler declarations, and configure arguments. Also read installed modules/*.json, qconfig*.pri, and QT_FEATURE configuration headers, retaining content and hashes. Replace upstream builder absolute directories in output with placeholders; file hashes bind original evidence. Configuration is not the complete CMake cache, build environment, or toolchain materials. SDK default architecture fields do not represent every architecture in actual libraries and cannot be used to misclassify universal libraries as x86_64.
4. The [AGL patch](../../scripts/qt-patches/macos-no-agl.patch) changes only SDK lib/cmake/Qt6/FindWrapOpenGL.cmake to avoid old AGL linkage when this project consumes Qt. The tool reads original content from the pinned QtBase archive, replays the unified diff, verifies patch, original, and result SHA-256, then checks the actual SDK file. Accept only original or known patched versions. For Windows SDK CRLF, compare only after line-ending normalization while retaining actual-byte SHA-256, LF SHA-256, and a conversion marker. Other unknown changes fail. This does not change Qt runtime machine code or prove Qt was rebuilt. Complete SDK modification history still needs verification.

Source correspondence records use schema 2, separately retaining pinned archive_commit, actual archive_marker, archive_marker_type, and commit_tree_verified. Qt Declarative 6.8.2 `.tag` is Git tree `0a8139c574fe150aba92d5f2484a8f3fdb774977`; the release tag maps to commit `75534f3e7fff24ed7ccb364e2ed9950a73da879f`. A pinned base64 original commit object in the repository permits offline recomputation of the commit hash and validation of its tree field. Other currently pinned module markers are commits. The tree relationship establishes only the relationship between these two Git objects, not that the archive contains the entire tree or that the SDK is build-equivalent to it.

source_commits in qt-license-sources.json identify pinned retrieval commits for license texts only and are stored separately from archive markers and release commits. They cannot be represented as SDK build commits. Historical schema 1 materials placed the Qt Declarative tree marker in archive_commit; published assets remain unchanged. Later explanations should correct object type while continuing to verify license texts using original SHA-256.

## Interpreting results and release boundaries

The QtNetwork security derivative separately records `network_security`, with receipts binding patches, builder, original QtCore and actual QtNetwork digests, features, and real transport verification. When reading receipts, the tool rechecks actual SDK bytes. This evidence applies only to the modified library; it does not rewrite upstream SPDX as proof of the derivative or change uncovered facts for other modules. See the [Qt patch notes](../../patches/qt/README.md) for inputs and maintenance.

Whether modules supply source SPDX, match/missing/uncovered counts, configuration, and patch states follow the JSON generated for the current run. Do not generalize one SDK's observations to another. Without per-file declarations, do not infer correspondence passed; retain specific gaps.

Report status is always evidence-collected-not-build-equivalence. [Actual Qt source build and loading/replacement](wp12-automation.md#qt-源码与兼容替换) has a separate flow, but it does not establish byte-for-byte reproduction of the original SDK. Until complete toolchain/system SDK materials, uncovered/generated files, runtime provenance, and license fulfillment are fully clarified, do not claim all corresponding-source obligations are met or byte-reproducible builds. Existing [runtime attribution](runtime-attribution.md) incomplete status, license choices, and platform runtime reviews remain.
