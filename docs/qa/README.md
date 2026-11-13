# Verification and evidence management

This directory contains reusable acceptance requirements, test methods, fixture rules, review processes and blank templates. It does not contain acceptance results for specific versions, stage reports, screenshots, measurement JSON, runtime logs or handoff progress.

- [Product acceptance requirements](acceptance.md)
- [Samples and fixtures](fixtures.md)
- [On-demand desktop verification](private-test-manual.md)
- [Release review process](release-review.md)
- [CI policy](../engineering/ci-policy.md)
- [Deployment dependencies and SBOM contract](../engineering/deployment-audit.md)

<a id="验收范围证据和收口"></a>

## Acceptance scope, evidence and closure

At the start, determine the minimum sufficient checks, pass criteria and platforms that verify actual behavior based on explicit user delivery requirements, mandatory applicable specifications and the risks of this change. Reuse existing evidence applicable to the current commit; reverify affected parts when source or environment changes affect results. An executor may add diagnostic checks but must not automatically elevate self-imposed recordings, screenshot counts, manual matrices, repeat runs or workflows into conditions for merging, closing an issue or releasing. Suggestions in historical progress comments are not automatically mandatory either. User requirements and applicable specification conditions remain binding and cannot be removed or lowered for closure.

Before building, downloading artifacts or rerunning workflows, check the commit or affected content, environment, coverage and validity of existing results. Reuse them when suitable rather than repeating work merely because a branch or device changed. Reverify affected scope when results are inapplicable or relevant code or environment changes. After failure, prioritize retrying the failed job or affected check according to the cause; explain when a complete matrix rerun is necessary. Reuse does not change evidence scope: automated, native and manual checks remain separate, and required independent verification cannot be omitted.

| Acceptance type | Necessary checks and evidence boundaries |
| --- | --- |
| Documentation and static resources | Check affected links, JSON and generated outputs; documentation changes do not automatically require real-device product acceptance. |
| Product, domain and bridge | Build and verify affected paths and corresponding failure, cancellation and protection behavior under the [CI policy](../engineering/ci-policy.md). Automated tests establish only their actual coverage. |
| UI and native desktop | Run applicable window, shared-control and bilingual multiple-font-size checks. Operate actual windows and observe relevant states when required by the user or applicable specifications. Functional tests cannot replace required visual checks; static screenshots cannot establish transient behavior not observed. Recording is not a default requirement and is used only when explicitly requested or other suitable evidence cannot verify the issue. |
| System interactions, installation and updates | Check affected system operations, identities, failures and recovery against the actual changes and contracts. Record isolated fixtures, test keys and controlled services separately from production. |
| Cross-platform and release | Check only platforms needed by user requirements, support commitments or affected contracts. Release still verifies version, signatures, assets and release permission under the [release protocol](../engineering/release-protocol.md); ordinary CI does not replace it. |

Record commit, environment, checks performed, results and limitations relevant to conclusions. On failure, diagnose, fix and reverify affected scope while retaining historical failures. After passing, do not repeatedly expand scope for optional checks. Once required conditions are met, continue merging and cleanup; pause only when necessary conditions cannot be met or a user-required external condition is actually unavailable, without repeatedly seeking confirmation at internal checkpoints. Report unexecuted checks honestly, neither marking them passed nor creating new blockers when inapplicable.

## UI comparison regressions

First-frame menu layout regressions locate sequence and status tags by object name and verify actual text metrics and shared padding scaled with font size. Cover Chinese and English, light and dark themes, 100%/125%/150% font scales, all eight combinations of sequence, status and shortcuts, and submenu arrows. Assert the first rendered frame after popup and font changes directly, without extra waits or wider tolerances to eliminate layout failures.

HTML body comparisons retain frozen artwork and hashes. Task-entry icons use the approved 16 logical-pixel revision, strictly measuring actual SVG size, vertical center and position between button and text; old artwork ink is retained as review history without rewriting the target. Qt logical dimensions and DPR matrices check main-window/menu proportions; actual browser zoom may optionally be checked locally for HTML.

On Windows, after building Qt browser tests, optionally run `scripts/check-windows-update-uia.ps1` with `-TestExecutable`, `-QtBin` and a nonexistent `-OutputDirectory`. It checks whether Windows UI Automation can read bilingual update text in an isolated test process with simulated update states; output stays under `build/`. It does not connect to a real update service or replace real screen-reader or other-platform checks.

The native Windows settings-sidebar hover regression first confirms the target font size and sidebar geometry are in effect and waits for pending Qt layout polish, then checks whether its test window receives the target position. If obscured, it temporarily raises only its own window to topmost, without operating other applications. It still verifies actual system pointer position, window hit, and hover states on entry, switching and exit. Topmost is test setup and does not change product window policy. An owned obstruction window separately verifies the fallback. On failure, `XDZIP_SETTINGS_HOVER_FAILURE_AUDIT` can retain screenshots and diagnostics of the test window itself.

## Status and evidence locations

Record specific work, acceptance conclusions, uncovered items and blockers in [GitHub Issues](https://github.com/xiaodaozhi/xdzip/issues). Track work packages, defects and release permission reviews separately; reuse existing issues for the same problem rather than accumulating batch-completion documents. Historical WP-12 acceptance is in [#5](https://github.com/xiaodaozhi/xdzip/issues/5), and historical `v0.1.0` release permission reviews are in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53). Directory reorganization does not require repeating user acceptance; future candidates require a new release issue.

Screenshots, logs, individual measurements and machine reports go to `build/` or `dist/` and may be uploaded as [Actions artifacts](https://github.com/xiaodaozhi/xdzip/actions) under CI retention policy. Issues cite actual runs, commits, artifact summaries and limitations, without pasting entire generated reports or presenting expired artifacts or successful configuration as passed verification. Long-term release materials accompany the actual Release; publication still requires explicit authorization.

Choose desktop checks based on the changes and available environment; maintainers need not fill out a fixed manual acceptance form for every commit. If an unobstructed local foreground is temporarily unavailable, manually dispatch the [Windows native visual diagnostic](../../.github/workflows/windows-native-visual.yml) to collect menu and owned-window images on a disposable hosted runner. See the [CI policy](../engineering/ci-policy.md) for method and limitations. It records an independent automation source and retains native capture safeguards. Do not report unattended results as human clicks or virtual screens as physical multi-monitor acceptance.

`design/hifi/` holds original designs, `design/assets/` product assets, and `tests/fixtures/` owned redistributable regression inputs. These are durable inputs, distinct from screenshots and measurements generated by actual runs. See [repository hygiene](../engineering/repository-hygiene.md) for content boundaries and historical cleanup.
