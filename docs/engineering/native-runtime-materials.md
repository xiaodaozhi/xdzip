# Native runtime provenance and notices

This contract covers non-Qt native runtimes in actual packages and is used with [runtime attribution](runtime-attribution.md) and [corresponding source](source-delivery.md). Source version, file identity, license conditions, and actual fulfillment are checked separately; supplier file hashes are not license approval. Historical candidate conclusions and supplementary evidence for `v0.1.0` are in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53). Future candidates use new release issues; evidence is generated only under build/ or CI artifacts.

| Component | Original basis and delivery materials | Review boundary |
| --- | --- | --- |
| Qt 6.8.2 | SDK SPDX, original module sources and patches, Qt notices, LGPLv3 and GPLv3 texts, app rebuild and compatible replacement instructions | Runtime uses the LGPL branch; tool and third-party licenses remain separate. SDK file correspondence is not complete upstream build equivalence |
| Rust runtime code | Cargo.lock, actual link inputs, runtime dependency candidates, complete vendor and original notices, Rust standard-library source | Conservatively deliver build-graph notices without misreporting build/dev/proc-macro tools as linked runtime code |
| Noto Sans SC derived fonts | Font manifest, original OFL, renamed font, generation entry point | Project Apache-2.0 does not apply to fonts; retain modification and naming explanations |
| ICU 73.2 | Actual library version, complete upstream ICU LICENSE including third-party notices, provenance records | Generic MIT text does not replace ICU's own notices; system wrappers are not mistaken for actual packaged files |
| Microsoft VC++ runtime | Pinned official VC_redist package and DLL hashes, original license, version and distribution inventory | Files remain unmodified; maintainer eligibility and redistribution conditions require separate review |
| Windows SDK graphics compiler libraries | Excluded from new packages; historical packages retain original SDK license, per-DLL hashes, and provenance | Historical distribution conditions remain separately reviewed and are unchanged by removal from new packages |
| Inno Setup 6.7.3 installer engine | Pinned compiler package and source commit, original Inno and RemObjects Pascal Script licenses, bilingual Installer-CREDITS.txt | Retain existing installer-engine copyright UI. Pascal Script separately requires its name and source URL in About or documentation; the general Inno license alone is insufficient |

Microsoft's [runtime redistribution guidance](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files) and [VS 2022 redistributable list](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution) tie redistribution rights to applicable software licenses. Installer license text does not prove the maintainer meets eligibility. Qt's [open-source obligations](https://www.qt.io/development/open-source-lgpl-obligations) require preserving users' rights to obtain corresponding source, modify, relink, and run modified libraries; “dynamic linking” does not replace other obligations.

See [Windows licensing](windows-licensing.md) for the free Community path, separate hosted Enterprise authority, and redistribution conditions. Deployment includes original CRT/SDK licenses, provenance, and bilingual Windows-runtime-NOTICE.txt in both runtime and materials. Before compiling, Setup checks the original CRT agreement hash and shows its license page. Microsoft DLLs are not relicensed under Apache-2.0.

## Generation and verification

`scripts/native-runtime-sources.json` pins the ICU license version, official URL, and SHA-256. `collect-notices.py` collects the complete ICU license on Linux and rechecks hashes for both caches and downloads. Current Windows development packages omit `opengl32sw.dll` and no longer collect Mesa/LLVM notices for it. If the notice output directory contains old dedicated materials, collection refuses reuse rather than incorrectly adding them to a new candidate.

`prepare-source-delivery.py` includes the ICU license and pinned source inventory. Materials retain original bytes; the delivery manifest recalculates sizes and hashes. Historical packages containing Mesa software-rendering libraries still require corresponding historical source, notices, and obligations; the new policy cannot rewrite historical delivery.

Platform materials attachments must include directories referenced by attribution report `license_materials_base`. Recheck each original license-text hash before packaging. Another identical text elsewhere in the runtime directory does not justify a report pointing to a missing relative directory.

Windows deployment disables windeployqt environment searches for software OpenGL, D3D compilers, and DXC and no longer copies the three graphics compiler libraries from the Windows SDK. Reject staging containing these names. Remove a leftover root software OpenGL DLL in controlled staging and reject same-name leftovers elsewhere.

macOS packaging emits `deployment-audit.json`, `macos-deployment.json`, and `runtime-attribution.json` while the final signed app still exists. The file SBOM uses the same audit and attribution report. Compare Qt native files by arm64 slices, all section bytes, UUID, and import versions, permitting Qt path relocation needed for deployment. Signatures, load-command layout, and LINKEDIT are outside that comparison's conclusion. Associate non-native resources by exact bytes. Unassociated files and original SPDX gaps remain; this process is not complete binary equivalence.

When missing notices are found after publication, supplementary materials cannot be described as already included in the original package. Original version assets and checksums remain immutable; corrected packages require a new version and reverification. Explicitly record supplementary materials and what they bind to.
