# Release runbook

English | [简体中文](../zh-CN/engineering/release-runbook.md)

Cross-platform stable releases and prereleases use [release protocol v1](../../docs/engineering/release-protocol.md), freezing source by complete tag and assembling six platform package types, update materials, notices, licenses, SBOMs, and deployment/verification materials in a draft. Releases do not generate or attach corresponding-source archives. Routine releases and acceptance do not run the source-delivery collector; it may be used for local review only when the user separately and explicitly requests source collection. Each platform builds once in a manual Release run. Package verification, signing, publication, and public download checks use artifacts from that run. Complete the matching preflight before creating the tag. After all checks pass, stable is a normal Release marked Latest; alpha/beta/rc are Pre-releases. Redownload retained attachments and verify hashes after publication. Explain unperformed manual acceptance within the candidate's actual scope; automation does not replace its conclusions.

Development acceptance, [release license review](../qa/release-review.md) and release execution are recorded separately. License conclusions bind the actual candidate. Before publication, resolve concrete gaps in provenance, notices and applicable source and replacement obligations.

## Choose the minimum release runs

Before dispatching a long release or pre-release workflow, check the intended outcome, the actual entry points and their default inputs, and existing runs, source revisions, package identities, environments and evidence. Record the necessary checks and any remaining risk in the release issue. Reuse evidence whose identity, environment and coverage still apply; do not turn optional diagnostics, previous recommendations or an executor's preference into release gates.

For an authorized ordinary release, complete the applicable local preflight and reviewed notes, freeze the version and dispatch **one formal Release run directly**. Do not first run the complete `candidate=true` rehearsal, a separate three-platform packaging workflow or another equivalent full build merely for confidence, habit, or to confirm that the formal run is likely to pass. The formal run itself builds and verifies all required platforms before signing and publication. Investigate known risks first with the smallest checks that cover the actual entry point and inputs.

A separate candidate rehearsal is allowed only when the user explicitly requests it, an applicable specification requires it, or a concrete change or failure creates a risk that the required targeted checks cannot resolve. Before starting it, record the specific condition, its scope, why smaller checks are insufficient and whether a formal release is intended. A rehearsal is not release approval and must not be added as a new default prerequisite.

The current workflow cannot promote another run's candidate packages into a formal release: formal verification, signing and publication accept artifacts from their own run. Check this limitation **before** selecting a rehearsal. A successful rehearsal can supply applicable evidence, but it cannot replace verification of the actual formal packages or eliminate the formal run's builds. Do not start a rehearsal first and then discover that this rule forces a second full build. Cross-run promotion requires a separately implemented and verified workflow; do not bypass same-run identity, package hashes, signatures, licensing review or release authorization to avoid repetition.

When a job fails, preserve its logs and passing artifacts, diagnose the affected boundary, and first rerun the failed job within the original run when its frozen source and inputs remain valid and the workflow supports it. If changed source, configuration, environment, missing artifacts or another actual constraint invalidates that retry, explain the reason in the issue and rerun the affected checks using a suitable new run. Never mix artifacts from different commits or runs in violation of the handoff contract, and never treat a failed or unperformed check as passed.

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

For a stable release, configure the [Snap Store publishing credential](../market/linux/snap-store.md#prepare-and-publish-a-revision) in `release-signing` before dispatch. The approved publication job also sends the verified same-run amd64 Snap to `latest/stable` and records the Store revision. If only the Snap step fails after GitHub publication, rerun the failed job in the same run. It verifies the public GitHub release before retrying the Snap upload.

After publication, check the public release state, notes, asset inventory and verified re-downloads against `SHA256SUMS`. The workflow's `verify-public` job calls the [publisher](../../scripts/publish-release.py) to perform the separate download. Keep the original assets for each published version and use a new version for corrected packages. Record the published version in the [changelog](../../CHANGELOG.md), keep pending changes under [Unreleased](../../CHANGELOG.md#unreleased), and update all changelog languages together.

## Candidate review

GitHub packages use the project's distribution policy. The Windows installer is unsigned. macOS packages use the necessary Apple Silicon ad-hoc integrity signature. Developer ID, notarization, and paid signing accounts are outside the current policy. Release still requires a specific authorization and checks on versions, commits, package hashes, actual licensing obligations and security.

1. Update VERSION.json and derived versions under [versioning](../../docs/engineering/versioning.md); confirm scope and capability matrix, disabling or labeling features that have not met gates. Prepare `release-notes/<full-version>.md` with consistent version/build metadata and Chinese/English notes citing actual platform/commit acceptance. Run `python3 scripts/publish-release.py --check-notes`; missing notes cannot inherit old content.
2. Verify that the run supplies desktop and standalone CLI packages, SBOMs, notices, dependency licenses and verification records from the specified commit. The `sign` job on Linux adds the update signatures after package verification. Do not generate or upload corresponding-source archives as release attachments.
3. Record toolchains, hashes, features, targets, and benchmark environments; verify source can rebuild unsigned portions.
4. Verify delivery without formal signatures: check macOS ad-hoc integrity, explicitly state the Windows installer is unsigned, and accurately describe potential system warnings and verified installation limits in notes. Do not require users to disable system protection or claim platform publisher trust.
5. Check installation, first open, associations, upgrade/uninstall, and dependency checks actually executed by the release workflow. Select additional desktop or clean-environment checks based on the change and record uncovered areas.
6. Review security, interoperability, and defect regression results, recording executed scope and known limitations.
7. Review version-specific notes, known limitations, complete installed/download sizes, and checksums, then upload after release-owner confirmation. Stop when draft and notes differ; explicitly align reviewed inputs first rather than automatically overwriting manual draft edits.
8. After release, confirm downloaded artifacts match verified hashes. If problems occur, withdraw the affected channel, explain impact, and retain diagnostic evidence.

Desktop GitHub builds check for updates from About. Eligible installations download and hand off a verified candidate. Portable builds and builds without the required installation identity or trust use manual installation. The standalone CLI and bundled CLI have distinct update targets. Windows and Linux store builds use their respective stores for publication and updates. Follow the [Microsoft Store](../market/windows/build.md) or [Snap Store](../market/linux/snap-store.md) procedure. When active tasks delay installation, users can cancel the wait. Before rollback, verify settings-schema compatibility and keep an older runnable version. See the [desktop update contract](../../docs/engineering/software-update.md) and [CLI update contract](../../docs/engineering/command-line.md).

Choose desktop checks for the actual candidate under [desktop verification](../qa/private-test-manual.md). Link prior evidence only when the identity and scope still apply. Review the [deployment SBOM and file inventory](../../docs/engineering/deployment-audit.md) against the final packages and investigate concrete component, notice and redistribution gaps. Check installed and portable behavior against the applicable platform requirements. On macOS, verify the delivered app and DMG deployment and ad-hoc integrity separately from SDK-mode validation.

A local Mac can produce a [development DMG](../../docs/engineering/macos-deployment.md) at the user's explicit request. Run the [source-delivery collector](../../docs/engineering/source-delivery.md) only at the owner's separate explicit request for local review. Before publishing affected binaries, resolve the actual component obligations through the [release review](../qa/release-review.md). The project uses ad-hoc integrity signing on the Mac. Developer ID and notarization are outside the current GitHub distribution policy.

## Rehearse and resume the release workflow

To check Qt Network preparation alone, dispatch the unsigned candidate probe from the branch being tested:

```sh
gh workflow run release.yml --ref BRANCH -f candidate=true -f qt_network_probe=true
```

The probe runs on Windows x64 and both native Mac architectures. It uses the update-regression Qt Network lookup and stops after Qt Network validation. `qt-network-prepare.json` records whether the matching dependency was restored or built from source. Use `-f dependencies=cold` to force a source build.

Candidates and formal builds use the same native preparation on Windows x64, both Mac architectures and Ubuntu x64. Seed Qt Network, Qt ImageFormats and the Mac Sparkle SDK without building the application, then check cross-run hits on fresh runners after the producer commit is on `main`.

```sh
gh workflow run release.yml --ref BRANCH -f candidate=true -f native_dependencies=seed
gh workflow run release.yml --ref main -f candidate=true -f native_dependencies=hit
```

The default `dependencies=auto` admits exact dependency keys from successful native producer jobs at a commit contained in `main`. The archive digest, stock Qt SDK, ABI and native architecture are checked before use. Sparkle also requires its pinned framework receipt and strict code signature. Ubuntu release jobs use a native-runner artifact. Missing or rejected bundles fall back to the pinned source build. `dependencies=cold` forces the source path. `dependencies.json` records each result. The probe runs the PNG and TIFF pixel checks, while normal candidates and formal packages still build, deploy and verify their own same-run application and CLI packages.

To benchmark Qt ImageFormats reuse on Windows x64 and native macOS Intel, dispatch the image module probe:

```sh
gh workflow run release.yml --ref BRANCH -f candidate=true -f qt_image_probe=true
```

Each platform builds the patched QtGui and TIFF plugin from source, verifies them with the preview worker, then restores the two modules and their licenses on a fresh runner in the same run. The restore requires matching Qt SDK binaries, image build inputs and toolchain. The worker checks PNG and TIFF pixels and confirms that TIFF loaded from the selected SDK. Compare `image-timing.json` for module preparation and the download step for artifact transfer. This probe ends before desktop and CLI packaging. Reuse is enabled for the probe.

Only when the selection criteria above require a rehearsal, run the dedicated Release workflow from the branch containing the proposed workflow change or other candidate under investigation:

```sh
gh workflow run release.yml --ref BRANCH -f candidate=true
```

For a resumption test, add `-f retry_probe=true`. If the first workflow attempt reaches Windows verification, it interrupts that job after restoring the candidate. Wait for the workflow to finish, then use `gh run rerun RUN_ID --job JOB_ID` with the failed verify job. A verify retry reads the existing build artifact and checks its digest. Signing, publication, and public download verification are also separate jobs. Rerun the failed job within the original run so GitHub can retain completed jobs and their artifacts.

The assembled rehearsal is available as `release-candidate` in the Actions run. The `candidate-download` job checks every file after download. The candidate has no release update signatures, and this rehearsal does not create a draft or public release. A formal run takes the authorized tag on `main` and uses its own verified assets.
