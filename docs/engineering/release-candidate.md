# Release candidate materials contract

Each candidate binds a clean source commit, product version, and actual package hashes. Generating materials does not automatically create GitHub Releases or tags, upload packages, or approve license choices; actual status is recorded in the candidate issue. Formal signing and notarization are not gates. Under the owner's 2026-11-05 policy, release attachments do not provide corresponding-source archives; local source collection and review tools remain available.

## Deliverables

After pinning a clean commit, produce Windows x86_64 ZIP/installer, macOS arm64 app ZIP/DMG, and Ubuntu 24.04 x86_64 tar.gz/deb, collecting notices, actual-file SBOMs, build receipts, and lifecycle receipts. A corresponding-source package may be reviewed separately locally and is not supplied as a candidate Release or Actions artifact. The candidate manifest records relative file paths, SHA-256, sizes, platforms, build versions, and source commit; do not combine packages from different commits into one candidate.

The on-demand Qt workflow's macOS job retains logs and JSON only. With explicit owner authorization, the dedicated Release workflow may deploy Qt on Apple Silicon runners, perform necessary ad-hoc signing, and export app ZIP and DMG from the same app, verifying integrity and relocated startup without SDK search paths. Local development DMGs retain existing authorization requirements. Windows/Linux development packages come from actual successful on-demand Qt runs; QA-only installers are not product packages. Historical complete source-Qt/performance records are cited as historical materials without rewriting commits or requiring manual acceptance to be redone.

Offline rebuild and actual startup from local source delivery are separately verified; this does not mean completely reproducible upstream SDK binaries or approved licensing. Removing a source package does not eliminate component license or relinking obligations. If actual license delivery conditions cannot be met otherwise, affected binaries must not be released. Deployed macOS DMG bytes differ from SDK-mode CI bytes; their SBOMs retain distinct audit scope and are not treated as one artifact.

<a id="许可与来源核对"></a>

## License and provenance checks

Existing inputs include runtime-attribution.json, qt-sdk-correspondence.json, qt-binary-differences.json, link inputs, and embedded-resource records. Review each into build/, distinguishing precisely explained binary transformations, upstream undeclared build origins, system wrapper dependencies, actually redistributed libraries, composite main-program code, and license branches to select. Retain original gaps and incomplete status; a technical explanation is not automatically legal approval.

Binary-difference records retain both attribution-report `attribution_path` and `sdk_path`. Multiple distributed paths can refer to one SDK file, so record counts are not automatically distinct SDK-file counts. Upstream digest differences still require their own actual evidence.

The 110 differences in published alpha.8 macOS materials correspond to 61 SDK files. The [official Qt PySide6 6.8.2 macOS wheel](https://download.qt.io/official_releases/QtForPython/pyside6-essentials/PySide6_Essentials-6.8.2-cp39-abi3-macosx_12_0_universal2.whl) contains files matching all 61 original SPDX SHA-1 values; its pinned SHA-256 is `abe8446367413b7b56626352e05adc7b15a180956ac9d90e4ee6419c16eab23d`. Provide the original public macOS materials, wheel, and corresponding Qt SDK to reverify complete SDK byte relationships:

```sh
python3 scripts/verify-alpha8-macos-sdk-signatures.py \
  --materials path/to/xdzip-0.1.0-alpha.8-macos-materials.tar.gz \
  --reference-wheel path/to/PySide6_Essentials-6.8.2-cp39-abi3-macosx_12_0_universal2.whl \
  --qt-prefix path/to/Qt/6.8.2/macos \
  --output build/alpha8-macos-signature-review.json
```

The tool verifies pinned material and wheel digests, original difference records, and each SDK-file digest. It reconstructs SDK files in memory using existing SDK signature bytes and required signature-command, link-edit-size, and alignment changes, requiring all 61 outputs to match the SDK byte for byte. It does not sign files, alter original packages, verify Qt signature trust policy, prove historical signing commands, or approve licensing. Original release SBOMs and historical materials remain unchanged; see [#53](https://github.com/xiaodaozhi/xdzip/issues/53#issuecomment-6571328466).

The review checklist checks actual availability and scope per platform: final file SBOM, deployed component attribution, and SDK references are three inputs. With only file hashes or SDK-mode attribution, record deployed component association as missing. An app after architecture trimming, relocation, and signing cannot directly reuse SDK file identity. Compare historical and current candidate observation counts only after scope, files, and components match; count changes alone are not resolutions.

Dedicated macOS Release captures a Cocoa/Metal resource probe, SDK-source correspondence, and difference analysis while the final deployed app exists, with conventional source/file SBOM names. Link graphs and static libraries bind the compiled app; resources and audits bind the deployed app. `release-deployment-evidence.json` reuses the Mach-O comparator to check exact section bytes, addresses, flags, zero-fill sizes, UUID, and normalized import versions while retaining different whole-file hashes. Load-command layout, LINKEDIT, and signature metadata are outside this equivalence conclusion. Invalid bounds, mixed identities, or changed sections/UUID/imports fail immediately. One probe does not approve reference-machine performance, human visual acceptance, or licensing; original attribution gaps and incomplete status remain.

Qt dynamic linking, notices, and locally retained source/replacement materials are review inputs and do not automatically remove other obligations; see [Qt's official obligations](https://www.qt.io/development/open-source-lgpl-obligations). Check Windows redistributables file by file against the [official Visual Studio 2022 redistribution list](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution), retaining toolchain/DLL provenance. “From the Qt SDK” is not specific authorization. Record fonts under existing OFL materials, original project content under Apache-2.0, and generative icons under their provenance separately. Corresponding-source archives are not release attachments; if actual license conditions cannot be met another way, block distribution of affected binaries.

Remaining issues must identify component/file, platform, original declaration, technical findings, acceptable supplementary evidence, and responsible role. Do not generically require commercial Qt, developer accounts, or certificates. Keep unapproved license choices pending rather than accepting terms for the owner.

## Policy and review materials

- [System support and maintenance policy](support-policy.md): on 2026-10-19 the owner approved initial macOS 14+ arm64, maintained Windows 11 x86_64, and Ubuntu 24.04 X11 x86_64 scope and best-effort personal-project maintenance. That policy governs scope and actual verification boundaries; draft status in frozen materials remains historical.

## Material completion criteria

Actual candidate packages and source identities on three platforms are verified, source and notices are complete, offline rebuilding succeeds, license issues are organized into actionable review items, and materials can be read independently. Continue tracking unresolved provenance and licensing under the [release review process](../qa/release-review.md); preparation does not replace approval.

Actual packages, execution reports, and checksums stay in local/CI artifacts. Official delivery materials accompany approved Releases; generated reports are not committed to the source repository.
