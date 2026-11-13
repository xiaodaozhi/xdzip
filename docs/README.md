# Documentation index

[English](README.md) | [简体中文](zh-CN/README.md)

These documents describe the current source tree's general design, implementation contracts, and reusable methods. `docs/` contains only Markdown and necessary images. Machine-readable inputs for generators, product resources, and checks are organized under `specs/` by visual design, terminology, and engineering dependencies. See the root [README](../README.md) for installation and version distinctions, and the [releases](https://github.com/xiaodaozhi/xdzip/releases) for individual versions. Progress and acceptance conclusions are tracked in [issues](https://github.com/xiaodaozhi/xdzip/issues); evidence is kept in [Actions](https://github.com/xiaodaozhi/xdzip/actions) or local artifacts rather than accumulated in docs.

## Where to start

- **Install and use:** Read the [README](../README.md), [installation and file associations](engineering/installers-associations.md), and [support policy](engineering/support-policy.md). The [command-line tool](engineering/command-line.md) has a separate release package starting with `v0.2.0`; earlier release assets remain unchanged.
- **Privacy and data handling:** See the user-facing [Privacy Policy](../PRIVACY.en.md) and its [Chinese version](../PRIVACY.md); engineering boundaries are in [data lifecycle and diagnostics](engineering/privacy.md).
- **Build and contribute:** Prepare your environment with the [developer build guide](engineering/developer-build.md), find code using the [repository layout](engineering/repository-layout.md), and select checks from [verification](qa/README.md).
- **Change product behavior:** Consult the browsing, task, format, and interaction contracts below, distinguishing current implementation from design and unverified behavior.
- **Change the interface:** Read the [visual standard](visual/design-standard.md) and [design workflow](visual/design-workflow.md), then consult the high-fidelity sources and relevant interaction contracts.
- **Prepare a release:** Read the [release protocol](engineering/release-protocol.md) and [release review process](qa/release-review.md). Evidence and decisions for each candidate belong in its issue.

## Topic index

| Area | Entry points |
| --- | --- |
| Command line | [Standalone CLI, scripting interface, installation and updates](engineering/command-line.md) |
| Product and architecture | [Design overview](design/index.md), [architecture decisions](design/architecture-decisions.md), [requirements traceability](design/requirements-traceability.md) |
| Interface standards | [Strict design standard](visual/design-standard.md), [design workflow](visual/design-workflow.md), [logical dimensions](../specs/visual/layout-scale.json), [visual parameters](../specs/visual/tokens.json) |
| High-fidelity design and assets | [Design sources](../design/hifi/README.md), [interactive HTML design](../design/html/README.md), [asset catalog](../design/assets/index.html) |
| Interaction contracts | [Main window](ux/main-window-components.md), [task windows](ux/task-windows.md), [password extraction](ux/password-extraction.md), [desktop integration](ux/desktop-integration.md), [file-manager actions](engineering/shell-actions.md), [settings and localization](ux/settings-localization.md), [Chinese terminology](ux/chinese-terminology.md) |
| Browsing and tasks | [Runtime](engineering/m1-runtime.md), [read-only index](engineering/m2-readonly.md), [large archive index](engineering/large-index.md), [preview](engineering/content-preview.md), [browser contract](engineering/readonly-browser.md), [extraction preflight](engineering/extraction-preflight.md) |
| Files and formats | [Filename encoding](engineering/filename-encoding.md), [split archives and temporary opening](engineering/archive-opening.md), [drag-and-drop and modification](engineering/archive-transfers.md), [integrity checks](engineering/archive-checking.md), [safe extraction](engineering/m3-extraction.md), [password extraction](engineering/password-extraction.md), [ZIP creation](engineering/m4-creation.md), [TAR/GZIP](engineering/m4-tar-gzip.md), [CAB sources and implementation basis](engineering/cab-format-sources.md) |
| Toolchains and dependencies | [Toolchain policy](engineering/toolchain.md), [dependency admission](engineering/dependencies.md), [cxx bridge rationale](engineering/cxx-admission.md), [licenses](engineering/licensing.md), [Windows licenses](engineering/windows-licensing.md) |
| Build and verification | [Unified build entry point](engineering/developer-build.md), [development guide](engineering/development.md), [Qt application modules](../apps/desktop/README.md), [CI policy](engineering/ci-policy.md), [verification](qa/README.md) |
| Deployment and provenance | [Deployment audit](engineering/deployment-audit.md), [runtime attribution](engineering/runtime-attribution.md), [native library materials](engineering/native-runtime-materials.md), [Linux deployment](engineering/linux-deployment.md), [Mac DMG](engineering/macos-deployment.md), [installation and associations](engineering/installers-associations.md) |
| Source and candidates | [Corresponding source](engineering/source-delivery.md), [SDK correspondence](engineering/qt-sdk-correspondence.md), [automated evidence collection](engineering/wp12-automation.md), [candidate materials](engineering/release-candidate.md) |
| Versions and releases | [Versioning](engineering/versioning.md), [updates](engineering/software-update.md), [release protocol](engineering/release-protocol.md), [application sessions](engineering/application-sessions.md), [support policy](engineering/support-policy.md), [release review](qa/release-review.md), [release runbook](engineering/release-runbook.md) |
| Execution and risks | [Work package roadmap](planning/backlog.md), [risk controls](planning/risks.md), [readiness review method](planning/readiness.md) |
| Repository management | [Content and history management](engineering/repository-hygiene.md), [layout and dependencies](engineering/repository-layout.md), [contributing guide](../CONTRIBUTING.md), [changelog](../CHANGELOG.md) |
| Open-source collaboration | [Governance](../GOVERNANCE.md), [code of conduct](../CODE_OF_CONDUCT.md), [security policy](../SECURITY.md) |

## Reading by task

- **Use and build:** Start with the root [README](../README.md) and [developer build guide](engineering/developer-build.md). Use the same entry point for preparation, building, testing, packaging, and verification. Platform deployment documents describe differences; the Qt application document explains modules and low-level debugging.
- **Change the core:** Find the relevant code in the [repository layout](engineering/repository-layout.md), then read the appropriate browsing, preflight, creation, or extraction contracts. The large archive index and preview scopes are defined in their respective engineering contracts. The target protocol and host design for the external SDK are described in the format extension specification.
- **Isolate format workers:** See the [format backend contract](engineering/native-codecs.md) for controlled streams, worker processes, and licensing boundaries for Rust 7z/RAR/CAB/ISO/UDF. The product does not compile official UnRAR/7-Zip sources; a format name does not imply that every feature has passed acceptance.
- **External extensions:** The standalone [read-only SDK trial](engineering/sdk-trial.md) includes two independent protocol implementations, controlled I/O, and a sandbox gate. The desktop application and CLI do not automatically enable third-party packages.
- **In-house Rust backends:** See the [in-house formats and algorithms contract](engineering/independent-rust-codecs.md) for sources, per-format replacement, and criteria for removing production dependencies.
- **Host calendar and encoding:** See [host platform services](engineering/host-platform-services.md) for optional format-independent `ReadSource` services, OS callbacks, borrowing, and secret-buffer boundaries.
- **7z backend:** See the [read/write specification](engineering/independent-rust-codecs.md#7z读写规格) for streaming method graphs, shared algorithms, creation, rewriting, and password wrapping, and the [7z implementation basis](engineering/sevenzip-format-sources.md) for sources and reference analysis. The product uses Rust; official 7-Zip sources are not compiled into or deployed with it.
- **CAB backend:** See the [in-house specification](engineering/independent-rust-codecs.md#cab只读规格) for the read-only container, shared algorithms, and full-frame work accounting, and the [source record](engineering/cab-format-sources.md) for specification versions and reference-analysis boundaries. The product uses Rust; official CAB handlers and CAB-specific LZX/Quantum are not built into it, and there is no fallback on failure.
- **Change the interface:** Read the visual standard and workflow first. High-fidelity sources are in `design/hifi`; behavior is described in the UX documents. Verify HTML examples and actual Qt capabilities separately.
- **Prepare delivery:** Recheck against deployment and installation contracts and the release runbook, keeping development acceptance separate from release licensing review. Official releases do not generate or include a corresponding-source archive. Routine release acceptance does not run the source delivery collector; it is used for local review or rebuild verification only when the user explicitly requests source collection. Supported scope follows the [approved policy](engineering/support-policy.md), not CI runner names.

Check for duplicates and track new defects and tasks in issues. Completed [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5) does not absorb new follow-up work. Historical release license reviews for `v0.1.0` are in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53); file-input and macOS ACL fix scope and validation are in [#6](https://github.com/xiaodaozhi/xdzip/issues/6). Consult those issues for actual status. Future candidates require new release issues; this document does not duplicate execution reports.

## Documentation authority

Requirements are defined in design/01; domain and tasks in design/03 and 08; the bridge in design/05; format extensions in design/06; and visuals in the approved artwork and tokens. Engineering documents supplement implementation boundaries and do not define conflicting protocols. Candidate, unimplemented, and unverified content must remain clearly labeled; a document's existence does not make a feature implemented.

The Chinese design Markdown under `docs/zh-CN/design/` is the source for the Chinese HTML reading edition. Run `python3 scripts/build-design-html.py` to generate multipage and single-file HTML under ignored `build/docs-html/`; generated output is not committed to Git.
