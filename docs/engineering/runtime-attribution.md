# Actual component attribution and license materials

This flow supplements file-level SBOMs by associating actual Qt libraries/plugins with SDK SPDX declarations. Its output is reviewable evidence, not automatic license selection or compliance approval. Formal release completeness remains incomplete.

## Inputs and sources

Qt 6.8 onward provides build-corresponding [SPDX inventories](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html). Tools read only runtime declarations such as qtbase, qtdeclarative, qtsvg, and qttranslations from this pinned Qt 6.8.2 SDK's sbom/, excluding qttools build/test tools. Validate SPDX version, module name, and Qt version; retain original document SHA-256, namespace, and stable package references. Output keeps only necessary fields rather than copying upstream build comments containing developer-machine paths.

`scripts/qt-license-sources.json` pins official Qt source and SPDX license-list-data commits, license-text URLs, and SHA-256. Downloads are cached under build/; any cache/download hash mismatch fails. Arbitrary SPDX URLs are not followed for material downloads. In addition to standard LICENSES/ texts, include original notices for Qt GUI smooth-scaling algorithms and SVG, plus LGPL-2.1 texts referenced by system dependency declarations. Upstream LicenseRef texts and copyrights remain in declaration JSON.

Qt runtime notices also include pinned editable Public Suffix List source used by Qt Network, MPL-2.0 text, and retrieval instructions. The collector verifies upstream blob, SHA-256, and license bytes. Source delivery separately verifies that this input reproduces two generated data forms in the Qt archive; see [corresponding source materials](source-delivery.md). This does not replace source and replacement checks for other Qt modules.

Qt Core Tika MIME notices retain relevant Apache Software Foundation and UK National Archives PRONOM attribution, Apache-2.0 and OGL-UK-3.0 license texts, and the complete pinned upstream NOTICE. The complete NOTICE also describes other upstream Tika components; it does not imply they ship with this product. See [corresponding source materials](source-delivery.md) for original MIME data and Qt conversion verification.

## File and component association

1. Recalculate file hashes inside the verification root and reject files modified after deployment-audit.json.
2. macOS SDK mode uses actual resolved Qt SDK relative paths and byte identity, without claiming standalone deployment.
3. Linux deployment records original SDK path/SHA-256 and post-patchelf artifact path/SHA-256. Later checks verify records in both directions, rejecting stale or altered copy evidence.
4. Windows prefers SDK file byte identity. Unique filename candidates with different bytes are marked only filename-only-unverified-rewrite and registered as gaps, not verified sources.
5. Use SPDX CONTAINS to find file owners, then recursively resolve DEPENDS_ON, including cross-Qt-repository references and embedded third parties. Missing package references remain gaps. Do not incorrectly include all tools in runtime through a repository-wide CONTAINS.
6. Non-native QML/translations without per-file declarations are associated only with the relevant source repository scope; their individual licenses are not inferred. SDK native libraries lacking SPDX file records, native files outside Qt scope, NOASSERTION licenses, and missing texts are separately recorded.

Actual SDK file bytes may differ from SHA-1 in upstream build SPDX, for example due to later installation processing or signing. Tools retain expected values, comparisons, and gaps without inferring causes. SDK byte association, deployment-rewrite records, and upstream original hashes are three distinct evidence types.

Dedicated macOS packaging generates deployed associations while the final signed app exists, matching SDK inputs by exact resource bytes or arm64 sections, UUID, and import versions and adding the report to the actual-file SBOM. Comparison excludes signatures, load-command layout, and LINKEDIT. See [native runtime provenance and notices](native-runtime-materials.md) for scope and dedicated notices. Ordinary SDK mode does not thereby establish deployed conclusions.

For plugins moved from QML directories into `Contents/PlugIns/quick/`, deployment association may use audited in-package QML symlinks to locate a unique SDK input. Recheck actual links, audited targets, and file hashes both when generating and consuming records, then perform the same byte or Mach-O comparison. Missing provenance remains unverified; multiple different sources or changed links are rejected. Filename matches alone cannot strengthen attribution or remove upstream SDK hash differences and license-review gaps.

Rust retains the complete host build graph and generates runtime candidates by following ordinary dependencies from desktop `xdzip-qt-bridge`, format-worker `xdzip-codec-bridge`, and Windows/Linux actual `xdzip-update-helper`; macOS has no separate update-helper root. Build, development, and proc-macro tools and their dependencies are excluded. These are potential runtime inputs, not proof every package's machine code survived linking; linker trimming, native C/C++, and compiler runtimes need further review.

Simple legacy Cargo license alternatives separated by slashes become SPDX `OR` in source CycloneDX, with upstream original preserved in `xdzip:cargo_license_raw`. Slash expressions with ambiguous precedence are rejected. Original build inventories/notices are not rewritten; standard `AND`, `OR`, and `WITH` remain unchanged. This is format normalization, not license-branch selection or release approval.

## Execution and delivery

The known patched QtNetwork derivative adds `network_security_variant` to file records; reports also retain complete, reverified build receipts. Upstream SDK digest differences remain visible; release review uses pinned source, full patches, build receipts, and deployment transformation evidence to explain them. Release staging requires the final Network attribution to link to those bytes; a filename match does not pass. Source and artifact SBOMs separately mark the derivative identity without converting global incomplete status into license approval.

```sh
python3 scripts/prepare-runtime-notices.py --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --output build/review/notices/Qt-runtime
python3 scripts/runtime_attribution.py --audit build/review/deployment-audit.json \
  --root build/review/linux --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --deployment build/review/linux-deployment.json --output build/review/runtime-attribution.json
python3 scripts/sbom.py --audit build/review/deployment-audit.json \
  --source build/review/source-build.cdx.json --attribution build/review/runtime-attribution.json \
  --output build/review/artifact-files.cdx.json
```

Linux passes deployment-rewrite records; other platforms omit --deployment. Write reports outside the verification root to avoid self-inventory recursion. On-demand Qt and Release workflows prepare license materials before deployment/final hashing. Windows and Linux put notices in actual runtimes; Linux tar.gz also delivers them. On-demand Qt macOS reads the SDK and records logs/JSON only. An explicitly authorized dedicated Release workflow separately deploys, ad-hoc signs, packages, and checks actual release materials under the [release protocol](release-protocol.md), retaining separate deployed scope and hashes. SDK records do not replace final app-bundle attribution.

runtime-attribution.json includes files, components, relationships, upstream declarations, copyrights, text sources, and gaps. artifact-files.cdx.json adds the same components and known owner relationships, binding the report and same actual audit hashes/build identity. Official CycloneDX schema and reference checks continue; compositions remain incomplete.

Default checks allow explicitly recorded development-review gaps but never silently mark them passed. Identity, path, byte, version, copy-record, or license-text hash errors fail immediately. --require-complete is a release-blocking check and currently fails on unfinished attribution or release materials. Tests cover rewritten files, stale copy records, path escape, wrong versions, cross-document omissions, partial SBOMs, and Rust tool exclusion.

## Unresolved release gates

A [source-delivery collector](source-delivery.md) retains actual project source, Rust vendor, four runtime Qt module archives, two build-module archives, and rebuilding/dynamic-replacement instructions. It does not prove per-item SDK/upstream build equivalence. Each release must still review exact static-link attribution, OS/redistributable runtimes, individual licenses for all embedded resources, open-source license-branch choices, and fulfillment. Metadata modification/replacement in [local Mac deployment verification](macos-deployment.md) and [actual source-Qt build/loading/replacement](wp12-automation.md#qt-源码与兼容替换) are distinct flows with separate evidence; neither establishes byte-level build equivalence for arbitrary SDKs. Source inventories and SPDX fields do not replace license obligations. Historical `v0.1.0` candidate provenance and licensing are in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53); later candidates need separate review, without paid signing identity as a gate. Existing desktop acceptance remains in [#5](https://github.com/xiaodaozhi/xdzip/issues/5).

[Qt SDK correspondence verification](qt-sdk-correspondence.md) further records pinned archive commits, per-file hashes in SDK source declarations, installed configuration, and known AGL consumer patch. Matches within declared scope do not establish complete SDK build equivalence; existing release blockers remain.

Automated closeout adds actual linker maps/static-library members, runtime-resource hashes, trusted native dependency versions, and precisely reversible binary-difference analysis; see the [complete contract](wp12-automation.md). Mark a transformation explained only when reconstructed byte hashes exactly match upstream declarations. Do not delete original SDK differences or guess undocumented publisher history.
