# XDZip product requirements

English | [简体中文](../zh-CN/design/product-requirements.md) | [繁體中文](../zh-TW/design/product-requirements.md) | [日本語](../ja-JP/design/product-requirements.md) | [한국어](../ko-KR/design/product-requirements.md) | [Français](../fr-FR/design/product-requirements.md) | [Deutsch](../de-DE/design/product-requirements.md) | [Español](../es-ES/design/product-requirements.md) | [Português (Brasil)](../pt-BR/design/product-requirements.md) | [Русский](../ru-RU/design/product-requirements.md)

XDZip lets people inspect archives like folders, work with selected content, and create or edit archives when the format supports it. The desktop and [standalone CLI](../engineering/command-line.md) share the archive runtime. The [project README](../../README.md) describes current capabilities and distribution. The requirements below describe the behavior to preserve and verify.

## Requirements and implementation

| ID | Requirement | Current behavior or scope |
| --- | --- | --- |
| P01 | Browse without automatically extracting | Opening probes a source and finishes its bounded scan before publishing the first page. Decoding entry content requires a separate action. |
| P02 | Navigate, search, sort and select | Stable physical IDs, separate navigation identities, paged queries and bounded selection are implemented. Ordinary single-file ZIP/ZIP64 has a larger metadata index with separate task limits. |
| P03 | Extract a selection or the archive | The runtime freezes scope, validates names and authorizes a destination. It supports conflict decisions and reports actual publication, skips, errors and cancellation. |
| P04 | Create archives from files and directories | Validate options against the creation registry, encode to staging and verify before publication. Supported formats and creation options follow the registry. |
| P05 | Offer operations by capability | The UI combines the format descriptor and current session state. Runtime validation still applies to every request. Readability, preview, testing, creation, password and volume profiles have separate bounds. |
| P06 | Run bounded tasks and allow cancellation | Metadata and heavy tasks have separate lanes. Desktop heavy operations are serialized, and an accepted task can continue after its tab closes. |
| P07 | Keep writes within validated authority and limits | Host rules cover names, links, file identity, staging, actual output and commit. Format checks validate the applicable structure and integrity. |
| P08 | Provide consistent, accessible desktop controls | QML and C++ provide shared controls, keyboard navigation, focus and accessibility behavior. Platform-native IME, screen readers and display behavior require their own verification. |
| P09 | Keep resource use measurable and bounded | Code enforces independent budgets. Reference performance targets and measured results are tracked separately. |
| P10 | Keep formats independent of Qt | Rust traits, explicit registry routing and dedicated worker interfaces support built-in implementations. Desktop and CLI use the same core. |
| P11 | Protect passwords and user data | Archive operations work offline. Passwords are kept in scoped, zeroizing storage. Privacy, diagnostic output and explicit update access have their own contracts. |
| P12 | Make release contents and provenance auditable | Use the defined packaging channels, locked inputs, manifests, checksums, license material and platform checks. Results apply to the build and distribution actually verified. |
| P13 | Allow third-party extensions only with validated trust and permissions | The XDP1 read-only trial is implemented separately. Product-side installation and activation remain design work. |
| P14 | Provide bounded preview and temporary opening | When the properties panel is visible, selecting one supported file starts a bounded text or image preview. Explicitly opening an eligible regular entry uses a verified private copy and delegates to the system association. |
| P15 | Report failures and uncertain results accurately | Stable errors and bounded results cover source change, disk failure, skipped entries and cleanup. Automatic restart recovery remains a design option. |

## Workflows and scope

- **Open and inspect.** A file-open intent enters the desktop session coordinator. A successful scan publishes a session; search and sorting make a new query. Double-click enters a directory or opens an eligible file through the temporary-copy workflow. When the properties panel is visible, selecting one supported file starts its preview. [Browsing and preview](../user-guide/browse.md), [Temporary opening](../engineering/archive-opening.md)
- **Extract.** “Extract all” refers to the archive. To extract only matches, select within the query and review the resulting scope. A metadata preflight provides findings before write authorization. Per-file commits preserve already completed files on a later failure or cancellation. [Extraction](../engineering/zip-safe-extraction.md), [Task scheduling](task-scheduling.md)
- **Create, modify or test.** Creation and eligible edits verify staged output before publication. The Test action checks an eligible archive according to its format and obtains no output authorization. Available operations depend on the format, entry and source profile. [Format registry](../engineering/format-registry.md), [Archive transfers](../engineering/archive-transfers.md), [Archive checking](../engineering/archive-checking.md)

## Planned scope and verification

External extension management, persistent job recovery and additional format profiles require their own design, implementation and verification. Version numbers alone do not establish format or platform coverage. Cloud archive synchronization, disk-image mounting and a malware-scanning service are outside the product scope. The [traceability matrix](requirements-traceability.md) maps each requirement to the relevant verification. The [support policy](../engineering/support-policy.md) and release notes describe the distributed builds.
