# CI and merge policy

## Routine checks

[CI](../../.github/workflows/docs.yml) runs one Linux job on pushes to main, pull requests, and manual dispatch. It checks documentation links and JSON, versions and UI text, generated-resource consistency, window/shared-control contracts, Rust and C/C++ formatting, workflow structure, and whitespace errors. It does not build the desktop product or run complete Rust, three-platform Qt, archive-format interoperability, packaging, or installation lifecycle checks. The target is usually under five minutes; queue and download times follow actual run records.

Each PR update can cancel older runs for the same PR; main and manual runs use independent run identities. Failures and cancellations do not count as passes. Workflows use read-only repository permissions and pinned dependencies, without release keys. Routine CI does not start Playwright, Chromium, or another web browser; static HTML design-generation checks still run.

## Local verification before commit

Before committing and pushing, complete the build or verification appropriate to the proposed changes locally and record what actually ran. Git hooks are not required. Reverify after source or configuration changes that affect results; do not describe tests including other uncommitted changes as results for a clean commit.

| Change | Local verification |
| --- | --- |
| Documentation and static assets | Check links, JSON, and affected generated outputs. |
| Rust core and formats | Check formatting; build or test affected crates and business paths. Changes to secure I/O, formats, or task semantics cover relevant failure, cancellation, and protection behavior. |
| C++, Qt, and bridge | Build affected targets; run relevant model, bridge, and interaction tests. |
| QML windows | Check window contracts, shared controls, and bilingual multi-font-size layouts. For radio or checkbox behavior changes, exercise actual accessibility actions and inspect the selection model. |
| Packaging, installation, and updates | Run affected local tests; use appropriate workflows as needed for isolated systems, other platforms, or real installations. |

See the [developer build guide](developer-build.md) for commands and toolchain prerequisites. Prefer the environment available on the current machine. When required tools are missing, report unexecuted checks; do not report preflight or another platform's results as a local pass. Routine checks do not require a full IDE installed locally and do not automatically install the product or change file associations.

## Full checks on demand

[Foundation probes](../../.github/workflows/foundation.yml) and [Qt components](../../.github/workflows/qt-components.yml) run only through manual `workflow_dispatch`; ordinary pushes, PRs, and tags do not trigger them. Both retain full matrices on Windows, Linux, and Apple Silicon macOS. Foundation covers the Rust workspace, Clippy, bridge probes, isolated workers, and format interoperability. Qt covers product builds, component and session tests, native platform checks, deployment, and applicable installation lifecycles. On-demand Qt runs retain the first and ten complete browser-suite rounds; routine local verification does not repeat those ten rounds.

Qt on-demand `test --skip-rust` marks the Rust workspace as not executed. When a Foundation conclusion is needed, run and verify it separately for the same commit; Qt results alone do not establish a Foundation pass. Running failures, cancellation, missing platforms, or missing original logs still fail the corresponding full check. Use only runs and artifacts actually executed for this work.

Foundation and Qt use pinned-input caches containing only Cargo downloads, pip downloads, and pinned Qt corresponding-source archives, not compiled products, installed Qt SDKs, test conclusions, or credentials. Trusted manual main runs may save hash-sealed caches; other branches only restore and verify them. Cache hits do not replace rerunning checks.

Foundation natively builds and runs CLI regressions without Qt on all three platforms, clearing display-session and Qt search paths and retaining JSON receipts. Qt builds also verify private CLI layout and same-user update handoff. Standalone package generation requires passing receipts bound to the same executable, worker hash, and build identity. Three-platform CLI release packages and independent signing manifests are produced only by an authorized Release workflow. Ordinary macOS CI still uploads only logs and JSON.

Foundation CLI checks use the unified build entry point and real binaries and also verify creating, running, overwrite protection, and removal of user-level command entry points. Installation fixtures copy only project-owned test binaries, do not generate release archives or deploy the desktop application, and upload only JSON receipts. Private update fixtures are also exercised.

Manual Qt runs on Windows and Linux upload experimental development packages and verification artifacts, normally retained for 14 days. Ordinary macOS checks retain only logs and JSON and do not deploy, sign, or upload app packages. Routine CI does not produce these development packages. See [Mac deployment](macos-deployment.md) and [release protocol](release-protocol.md) for on-demand development DMG and dedicated Release exceptions.

Ordinary development and release acceptance do not automatically generate project-source archives or invoke the corresponding-source collector. Source collection requires a separate explicit owner request. GitHub's tag-provided source snapshots are described separately from project pipeline attachments.

[SDK trial](../../.github/workflows/sdk-trial.yml) is also manual only. It first runs independent read-only protocol and native-kernel gates without downloading Qt or deploying the application and uploads only logs and JSON. Foundation's manual `sdk_only=true` option invokes the same SDK workflow and skips the full Foundation jobs and gate; it cannot establish a full Foundation pass. Use this entry point first to narrow SDK platform failures. Reuse other still-applicable Foundation and Qt evidence according to the verification method.

Foundation `image_only=true` uses system sips/ImageIO on native macOS to convert a fixed project-owned HEIC, installs pinned Qt through the same shared toolchain preparation method, and separately builds the real image worker and test controls. It compares an unrestricted Qt reader and real worker code with one limit omitted at a time to distinguish system/plugin capabilities from resource budgets; test interceptors do not enter product builds. `--verify-worker` additionally requires correct dimensions/pixels for two inputs through the real parent and default worker, growth within the runtime file budget, and EFBIG beyond it; otherwise this check fails. This entry point does not build the desktop application, deploy, or sign, and saves only JSON and logs. Observed success or affected-component regression does not establish a full product/Foundation pass. The two preflight options are mutually exclusive.

## Visual, desktop, and release

Run automation appropriate to the change first, and inspect the actual application or screenshots where practical. Check input methods, real screen readers, system dialogs, window decorations, physical multiple displays, and real GPUs as needed. Record limitations when checks or environments are unavailable. Maintainers need not complete a fixed manual checklist every time, and offscreen, synthetic-input, or virtual-display results are not recorded as human or physical-device passes. See [desktop verification](../qa/private-test-manual.md).

[Windows native visual diagnostics](../../.github/workflows/windows-native-visual.yml) can manually capture project-owned windows and menus. It is a one-off hosted-runner automated diagnostic, not human clicking or physical-display acceptance. [Qt source verification](../../.github/workflows/qt-source.yml) and update-specific workflows likewise use their respective manual entry points.

[Windows MSIX lifecycle](../../.github/workflows/windows-msix.yml) builds an experimental package on a disposable GitHub-hosted Windows Server, signs a test copy with a temporary self-signed certificate, checks sideload installation, default associations, update entry points in Chinese and English About pages, settings retention, portable coexistence, file activation, classic-menu registration rejection, and uninstallation, then removes the certificate. The upgrade fixture increments only the MSIX manifest version for a dedicated CI identity and verifies every other product file remains unchanged, then checks upgraded settings, update-entry isolation, and default associations. It is not a new product version or Store update acceptance. This workflow is manual only and does not use actual Store signing or modify the developer's computer. Server sideload results do not establish Windows 11 desktop, Store provenance, ordinary installed-edition coexistence, or manual visual acceptance.

When only test scripts or that workflow change, [MSIX lifecycle recheck](../../.github/workflows/windows-msix-recheck.yml) can be run manually with the original build run ID and complete commit. It checks package digests, production receipts, and commit ancestry and permits only explicitly listed test and documentation changes. Product or packaging-input changes require rebuilding. Results identify build and test commits separately; the old package is not called a new product build.

The recheck can optionally use GitHub-hosted `windows-11-arm` to run the same x64 package under Windows 11 x64 emulation. Results must identify system and architecture; this is not native x64, a native ARM64 build, or an expanded support commitment.

[MSIX and installed-edition coexistence](../../.github/workflows/windows-msix-coexistence.yml) reuses builds and runtime audits subject to the same provenance constraints, generates a per-user installer on a disposable hosted machine, and checks concurrent operation, mutual uninstallation, and default associations. It can use Windows Server or Windows 11 ARM with x64 emulation. It does not modify the maintainer's computer or replace Windows 11 x64 native and Store acceptance.

The [Release workflow](../../.github/workflows/release.yml) separately handles three-platform release builds, package verification, signing, and immutable assets under the [release protocol](release-protocol.md). A release requires owner authorization for the specific result; unexecuted checks and license-review limitations are recorded accurately. Routine CI success, manual workflow success, and formal release approval are distinct conclusions.

Logs, screenshots, copies of test inputs, and generated reports remain only in `build/`, `dist/`, or Actions artifacts. Project source, design specifications, and test code remain versioned; actual work progress and results are recorded in GitHub issues.
