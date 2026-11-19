# Release runbook

English | [简体中文](../zh-CN/engineering/release-runbook.md)

Cross-platform stable releases and prereleases use [release protocol v1](../../docs/engineering/release-protocol.md), freezing source by complete tag and assembling six platform package types, update materials, notices, licenses, SBOMs, and deployment/verification materials in a draft. Releases do not generate or attach corresponding-source archives. Routine releases and acceptance do not run the source-delivery collector; it may be used for local review only when the user separately and explicitly requests source collection. Formal release uses one manual Release workflow run: three-platform native artifacts are built once and assembly directly reuses those artifacts for publication. Do not run a validation round and then push a tag for a second build. Complete local verification and release preflight before creating the tag. After all checks pass, stable is a normal Release marked Latest; alpha/beta/rc are Pre-releases. Redownload retained attachments and verify hashes after publication. Explain unperformed manual acceptance within the candidate's actual scope; automation does not replace its conclusions.

Development acceptance, [release license review](../qa/release-review.md) and release execution are recorded separately. License conclusions bind the actual candidate. Before publication, resolve concrete gaps in provenance, notices and applicable source and replacement obligations.

## Dispatch and verify the release

Before creating the version tag, run the relevant local checks with the candidate's pinned inputs. Validate synchronized version files and the reviewed notes from the repository root:

```sh
python3 scripts/sync-version.py --check
python3 scripts/publish-release.py --check-notes
```

For an explicitly authorized release, push the existing version tag for the intended source commit. Dispatch [Release](../../.github/workflows/release.yml) from `main`. Replace `vX.Y.Z` with the complete tag, including the prerelease suffix when applicable:

```sh
gh workflow run release.yml --ref main -f tag=vX.Y.Z
```

The workflow takes product code from the tag and reviewed notes from the dispatch revision. Record both SHAs and the run ID. Verify the draft, native builds and deployment receipts, and the release contract checks for that run. Confirm the six desktop assets and three standalone CLI packages before final assembly. The protected `release-signing` environment supplies the update-signing authority. Its signatures and macOS ad-hoc package integrity serve different purposes. See [candidate materials](release-candidate.md) and the [signing contract](../../docs/engineering/release-protocol.md#trusted-update-attachments).

After publication, check the public release state, notes, asset inventory and verified re-downloads against `SHA256SUMS`. The [publisher](../../scripts/publish-release.py) performs a separate download after publication. For an already public version, preserve its original assets. Corrected packages use a new version. Record only versions actually published in the [changelog](../../CHANGELOG.md).

## Candidate review

GitHub packages use the project's distribution policy. The Windows installer is unsigned. macOS packages use the necessary Apple Silicon ad-hoc integrity signature. Developer ID, notarization, and paid signing accounts are outside the current policy. Release still requires a specific authorization and checks on versions, commits, package hashes, actual licensing obligations and security.

1. Update VERSION.json and derived versions under [versioning](../../docs/engineering/versioning.md); confirm scope and capability matrix, disabling or labeling features that have not met gates. Prepare `release-notes/<full-version>.md` with consistent version/build metadata and Chinese/English notes citing actual platform/commit acceptance. Run `python3 scripts/publish-release.py --check-notes`; missing notes cannot inherit old content.
2. Build each platform's unsigned packages, debug symbols, SBOM, NOTICE, and dependency licenses from the specified commit. Do not generate or upload corresponding-source archives as release attachments.
3. Record toolchains, hashes, features, targets, and benchmark environments; verify source can rebuild unsigned portions.
4. Verify delivery without formal signatures: check macOS ad-hoc integrity, explicitly state the Windows installer is unsigned, and accurately describe potential system warnings and verified installation limits in notes. Do not require users to disable system protection or claim platform publisher trust.
5. Check installation, first open, associations, upgrade/uninstall, and dependency checks actually executed by the release workflow. Select additional desktop or clean-environment checks based on the change and record uncovered areas.
6. Review security, interoperability, and defect regression results, recording executed scope and known limitations.
7. Review version-specific notes, known limitations, complete installed/download sizes, and checksums, then upload after release-owner confirmation. Stop when draft and notes differ; explicitly align reviewed inputs first rather than automatically overwriting manual draft edits.
8. After release, confirm downloaded artifacts match verified hashes. If problems occur, withdraw the affected channel, explain impact, and retain diagnostic evidence.

Desktop GitHub builds check for updates from About. Eligible installations download and hand off a verified candidate. Portable builds and builds without the required installation identity or trust use manual installation. The standalone CLI and bundled CLI have distinct update targets. Store releases use Microsoft Store for publication and updates. When active tasks delay installation, users can cancel the wait. Before rollback, verify settings-schema compatibility and keep an older runnable version. See the [desktop update contract](../../docs/engineering/software-update.md) and [CLI update contract](../../docs/engineering/command-line.md).

Choose desktop checks for the actual candidate under [desktop verification](../qa/private-test-manual.md). Link prior evidence only when the identity and scope still apply. Review the [deployment SBOM and file inventory](../../docs/engineering/deployment-audit.md) against the final packages and investigate concrete component, notice and redistribution gaps. Check installed and portable behavior against the applicable platform requirements. On macOS, verify the delivered app and DMG deployment and ad-hoc integrity separately from SDK-mode validation.

A local Mac can produce a [development DMG](../../docs/engineering/macos-deployment.md) at the user's explicit request. Run the [source-delivery collector](../../docs/engineering/source-delivery.md) only at the owner's separate explicit request for local review. Before publishing affected binaries, resolve the actual component obligations through the [release review](../qa/release-review.md). The project uses ad-hoc integrity signing on the Mac. Developer ID and notarization are outside the current GitHub distribution policy.
