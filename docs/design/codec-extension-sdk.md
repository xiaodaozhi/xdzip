# Format adapters and the extension trial

English | [简体中文](../zh-CN/design/codec-extension-sdk.md)

## Three execution paths

| Path | Implementation and availability |
| --- | --- |
| Built-in adapters in the host | Desktop and CLI use the same fixed format registry and runtime dispatch. ZIP, TAR, stream filters and supported containers such as DEB and ASAR have Rust implementations called in the host process. |
| Bundled archive worker | `xdzip-codec-native` in the host starts `xdzip-native-codec`. Its C++ protocol layer calls the Rust 7z, RAR, CAB and optical backends through `xdzip-codec-bridge`. The same route handles applicable 7z creation and rewriting. ISO creation uses its in-process optical writer. |
| External SDK trial | `xdzip-plugin-host` and `xdzip-plugin-protocol` form a separate, explicit XDP1 experiment. Desktop and CLI product builds do not discover, install or run third-party extensions. |

The first two paths are built-in product components. The bundled worker is a separate process with a private protocol and resource supervision. The external trial has its own package validation, version negotiation and platform permission gates. [System architecture](system-architecture.md), [Archive worker contract](../engineering/native-codecs.md), [SDK trial contract](../engineering/sdk-trial.md)

## Product routing and operation capabilities

The [format registry source](https://github.com/xiaodaozhi/xdzip/blob/main/specs/engineering/archive-formats.json) declares extensions, container and filter types, supported operations, methods and creation options. The generator keeps the Rust registry, desktop lists and design artifacts in sync. At read time, the runtime probes source bytes and validates the format before choosing a backend. At creation time, the requested output name selects a creation profile, which the runtime checks against the options. The extension list supports discovery and UI hints. Successful probing and scanning establish the opened format.

Read, preview, extraction, whole-archive checks, creation and rewriting have separate constraints. A listed entry may be visible but lack a supported payload method. Passwords, volumes and special records affect admission, and the runtime handles format-specific distinctions such as ISO/UDF structural checks and unsupported rewrite profiles. [Runtime routing](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/formats.rs), [Format registry contract](../engineering/format-registry.md), [Archive checking](../engineering/archive-checking.md)

## Internal Rust adapter interfaces

| Interface | Responsibility |
| --- | --- |
| `ReadSource` and `MetadataCodec` | Read or seek a host-authorized source, including applicable volume metadata, and produce bounded format records. |
| `PayloadCodec`, `PayloadReader` and `DecodedSink` | Resolve content and decode an eligible physical record into a host sink. Distinguish ordinary bytes, skipped decode work and auxiliary work where the backend reports them. |
| `CreationCodec`, `EncodeInput` and `EncodeOutput` | Encode from frozen input metadata and indexed streams. Write to a host-owned sink, with bounded random writes when the writer requires them. |
| `RewriteCodec` | Rewrite supported archives from the retained original entries, changes and authorized additions into a new staged archive. |

The host owns paths, source revalidation, cancellation, passwords, quotas and all final publication. A codec receives capabilities through the interfaces needed for the operation. The archive worker requests allowed source ranges and delivers bounded records or output blocks. The host validates protocol state and byte quotas before acting. A worker fault or invalid response cannot authorize publication. The worker boundary provides fault and resource isolation. Kernel permission restrictions for arbitrary external code are a separate requirement of the trial. [Metadata contracts](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/lib.rs), [Payload contracts](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/payload.rs), [Encode and rewrite contracts](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/encode.rs), [File authorization](file-security.md)

These Rust traits are internal source interfaces. A product adapter is integrated into the workspace and explicit runtime routing. Its allowed operations need separate tests for valid files, unsupported features, resource limits, corruption and output behavior. Use the [independent Rust codec contracts](../engineering/independent-rust-codecs.md) and [development guide](../contributing/README.md).

## Implemented external SDK trial

The trial accepts an explicitly selected native package manifest and a host-selected source. It supports scanning an owned XDP1 fixture and reading one ordinary entry. Package admission validates manifest version 1, the executable target and hash, in-package paths, the fixed permission profile and limits. The host executes a verified private copy. The declared hash establishes package consistency. Publisher authentication would require its own trust policy.

The version-1 trial protocol uses length-prefixed bounded CBOR and checks operation identity and sequence. The host grants source reads and one output sink through opaque tokens. The peer receives byte ranges and one-block output credit. The host meters and verifies content, checks protocol completion and source state, and publishes without overwriting only after the peer exits cleanly. The sample does not expose archive passwords, split volumes, creation, updates, links or host paths to the peer.

The trial has explicit native permission gates for macOS and Linux, with platform requirements and fail-closed behavior defined in the trial contract. Windows refuses execution. Native interoperability and denial tests exercise these boundaries independently of the desktop. Use the [trial execution and wire contract](../engineering/sdk-trial.md) for the actual schema, budgets and test entry points. [Trial host](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-host/src/lib.rs), [Framing implementation](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-protocol/src/lib.rs)

## Design options outside the current implementation

A product-facing extension manager, publisher trust and upgrade policy, compatibility beyond the trial protocol, and external write or password capabilities remain design work. Product admission would require verified permission restrictions, interoperability and fault behavior for each supported platform and capability. The bundled worker and the XDP1 experiment do not activate this feature. [Architecture decisions](architecture-decisions.md)
