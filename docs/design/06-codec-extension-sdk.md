# xdzip: 06 Format adapters and extension SDK

## 1. Two kinds of extension

**Built-in source-level extensions** implement the Rust SDK, are built with the product, and select backends through Cargo features. The Rust compiler ABI is unstable; they are not made into `.so/.dylib/.dll` files loadable across arbitrary versions.

**Process-level extensions** use a versioned wire protocol handled by `xdzip-plugin-host`. An independent, explicitly started [read-only SDK trial](../engineering/sdk-trial.md) currently provides bounded framing, a broker, Rust/independent Python peers, and platform gates. It is not integrated into product format discovery, installation, or release. The full capabilities below remain future goals and are not automatically enabled by the trial. The wire protocol is independent of Qt's cxx interface; external processes cannot create Qt objects or display their own password windows.

Official native backends may also use the worker protocol to isolate memory errors; being “official” does not exempt them from input validation. Third-party releases are non-executable by default until sandbox verification.

## 2. Format capabilities

| Capability group | Fields/content |
|---|---|
| Detection | Format ID, magic probing, maximum probe bytes required, container/stream layer |
| Read | Metadata scanning, per-entry reading, sequential-only reading, CRC/digest, directory encryption |
| Write | Creation, allowed algorithms, compression-level range, parameter schema, streaming output |
| Encryption | Specific algorithms, read/write, filename encryption; `encryption=true` alone is insufficient |
| Split volumes | Read/write, volume naming and order, missing-volume behavior |
| Execution | Seek requirements, per-session concurrency, internal threads, memory estimate, cancellation safe points |
| Special structures | Solid blocks, links, time precision, encoding, permissions, and extra attributes |

Each open produces `EffectiveCapabilities`, the intersection of backend declarations, actual archive features, host policy, and platform capabilities. QML displays options from this set; the core checks again at start.

Probing may read only a bounded number of bytes; it does not perform arbitrary scans. Multiple candidates are selected by deterministic priority and trusted origin. A low-trust extension cannot displace a built-in backend by claiming higher probe confidence. User selection of an installed compatible backend is future extension design; the current product has no such entry point. Probe errors must not trigger infinite recursive attempts.

## 3. Layers and initial formats

TAR is a container and GZIP a compressed stream. `tar.gz` combines a bounded number of layers: Source → GzipDecoder → TarReader. ZIP/7z use their respective containers. Layer composition follows registered capabilities and magic, with at most four layers; do not strip filename extensions indefinitely. A nested archive is a new session opened by the user from an entry, not automatically recursively expanded during extraction.

| Format | Current source capability | Future | Dependency selection gates |
|---|---|---|---|
| ZIP Store/Deflate | Browse, read, create; AES-256, limited split-volume reading, numbered-volume creation, and regular archive modification | Other encryption and compression methods | Independent Rust container and shared Deflate; retain ZIP64, duplicate names, CRC, cancellation, and host commit contracts |
| TAR | Browse, read, create | Separate capabilities such as sparse entries | Independent Rust container without direct unpack or destination-path authority |
| GZIP/TAR.GZ | Read/write and composition | Verify multi-member and other features individually | Independent Rust GZIP framing and shared Deflate; composition reuses TAR |
| BZIP2/XZ/Zstd layers | Unsupported | Add layers individually after v0.2 | Limit windows/dictionaries, native dependencies, and memory estimates |
| 7z | In-house Rust backend for browsing, reading, and creation; passwords, solid archives, numbered volumes, and limited rewriting each have capability constraints | Expand according to an independent interoperability matrix | Shared algorithms and thin CXX boundary; host controls sources, budgets, output, and commit. See [isolated backend contract](../engineering/native-codecs.md) |
| RAR | In-house read-only Rust backend; legacy, RAR5, and RAR7 declared according to the verified matrix | Add compatibility samples; no creation or modification promise | Container, algorithms, passwords, and physical volume identities reside in Rust; host controls resources and commits; reference sources and historical deliverables retain their licenses |
| CAB | In-house read-only Rust backend; Store, MSZip, Quantum, LZX, and volumes | No creation, modification, or repair promise | Host validates the complete same-directory volume chain, retains original names and declarations, and does not publish a partial index when volumes are missing |
| ISO/UDF | In-house read-only Rust backend in isolated workers handles ISO9660, Joliet, Rock Ridge, and UDF | No promise to mount, write, or handle arbitrary disk images | Hybrid images prefer UDF; corruption after recognition does not trigger fallback. Testing checks structure and readability and does not claim per-file content checksums. See the [independent implementation contract](../engineering/independent-rust-codecs.md#iso与udf只读规格) for extension boundaries |

7z, RAR, CAB, and optical containers run independent Rust through a shared thin CXX boundary. Official 7-Zip and UnRAR sources are excluded from product builds, with no fallback on runtime failure. General format logic is not copied into C++/QML. ZIP/TAR/GZIP use in-process Rust. CAB volumes retain authorized volume boundaries and names rather than being concatenated as a continuous stream. RAR also retains physical part identities and the complete authorized volume set; 7z numbered volumes are frozen by the host into a bounded continuous source. All formats reuse host budgets, secure staging, and final commit. The host exposes only optional OS calendar and character-set primitives; format time and password-byte decisions remain in Rust. Unknown sizes are represented explicitly and cannot be output.

Existing capabilities in the table are governed by each format's implementation contract. Listing a new extension does not grant execution permission. Source, version, feature, build configuration, interoperability results, and license must pass the checklist before being locked into Cargo.lock/SBOM.

## 4. Rust SDK contract

Conceptual interface:

```text
CodecFactory.probe(ProbeSource, ProbeBudget) -> Detection
CodecFactory.open(ReadSource, OpenOptions, OperationContext) -> CodecSession
CodecSession.scan(MetadataSink, OperationContext) -> ScanReport
CodecSession.read_entry(EntryToken, OutputSink, OperationContext) -> ReadReport
CodecFactory.create(InputCatalog, ArchiveSink, WriteOptions, OperationContext) -> WriteReport
```

This illustrates responsibilities, not compiled signatures. Implemented trait return types retain `Result` and explicitly state Send/concurrency requirements. `OperationContext` contains cancellation, budgets, progress, and short-lived secret access; it must not expose global application services or the filesystem.

ReadSource exposes only reading, bounded seek, and authorized volumes. OutputSink writes only the currently approved entry and exposes no destination path. ArchiveSink writes only a new temporary archive object. Adapters check cancellation at each safe point; the host meters actual bytes and returns quota errors. Adapters cannot catch errors and secretly continue writing.

Ordinary extraction OutputSink uses sequential writes. If ArchiveSink needs to revise headers, bounded seek/write may be negotiated. All offsets and additions are checked for overflow; high-water file length and cumulative writes are quota-limited, and seeking must not create over-quota sparse files. Backends that do not need seek should prefer streaming writes.

Scanning sends raw paths, record tokens, sizes, types, and attributes to MetadataSink. The Sink imposes batch and field limits. Backend-declared sizes are not trusted as quota inputs. Completion returns integrity and scan counts, not just a bool.

If a format needs another seek, the same authorized source remains subject to SourceRevision detection. If TAR/solid blocks can only decode sequentially, the plan processes selected items in physical order and skips unrelated content while metering decoding work; it cannot promise O(1) access to arbitrary entries.

## 5. Plugin packages and manifest

The current source encoding slice is compiled in `xdzip-codec-api::encode`. `CreationCodec::encode` receives `EncodeInput`, `EncodeOutput`, `EncodeSettings`, a cancellation token, and a processed-byte callback. Algorithms use backend-defined `store`/`deflate` identifiers; numeric ZIP header values do not become a general format interface. Input provides only immutable entries and read streams opened by ordinal; output provides only sequential `Write`. There are no destination paths, directory handles, or commit authority. The ZIP writer does not depend on Qt. Budgets and per-entry verification after writing follow the [WP-10 host contract](../engineering/m4-creation.md); the external process protocol remains disabled.

An independent package contains a manifest, target-platform executable, licenses, checksums, optional signature, and capability schema. The manifest includes at least `plugin_id`, version, author, supported platforms/architectures, protocol major-version range, format capabilities, entry point, resource requests, and origin.

The entry point is a package-relative path. Forbid `..`, absolute paths, link escapes, and shell command strings. The host launches an absolute, verifiable executable with an argument array, without `sh -c`. Sanitize environment variables; do not pass application credentials, proxy passwords, user PATH extensions, and similar data.

Example draft manifest (hash/signature are supplied by the package inventory; fields are illustrative, not an installable extension):

```json
{
  "manifest_version": 1,
  "plugin_id": "org.example.xdzip.reader",
  "version": "0.1.0",
  "author": "Example Maintainer",
  "protocol_majors": [1],
  "targets": ["macos-aarch64"],
  "entrypoint": "bin/xdzip-reader",
  "formats": [{"id": "example", "scan": true, "read": true, "create": false}],
  "permissions": {"broker_source_read": true, "broker_output_write": true, "network": false},
  "resources": {"memory_bytes": 268435456, "threads": 1},
  "license_files": ["LICENSE"]
}
```

The format capability schema is more complete than this example and must declare algorithms and reading modes. Unknown permissions are not automatically granted. Type definitions for the source SDK and manifest/protocol are released with the conformance tool.

An installation package is an untrusted archive unpacked by the same secure extraction service into staging, then atomically published after manifest/signature/platform checks. A signature identifies the publisher; it does not prove code safety. On first enablement users can see origin, formats, and permissions. Protocol compatibility and permission changes require reevaluation rather than silently inheriting prior authorization.

## 6. Draft wire protocol v1

The [trial contract](../engineering/sdk-trial.md#线协议与状态) defines the exact numeric schema, implemented messages, and stricter single-block credit for the current read-only trial. The remaining messages in this section do not automatically form negotiated capabilities.

Transport uses private inherited IPC pipes; it does not listen on TCP or create public ports. Bidirectional frames are `u32 little-endian length + CBOR object`. Check length before allocation; maximum frame size is 1 MiB and data-message bytes are at most 256 KiB. CBOR depth is at most 16; unbounded lengths, duplicate key fields, and unknown required message kinds are forbidden.

The envelope is `{kind, request_id, operation_id, sequence, payload}`. CBOR uses fixed field keys and types; field schemas ship with the protocol package. Only Hello is allowed as the first frame. The backend declares its version within a two-second handshake budget; the host selects a common major version and negotiated capabilities. Optional fields added within the same major version may be ignored; unknown enum values or required fields may not. Incompatible changes increment the major version.

| Message group | Messages | Contract |
|---|---|---|
| Handshake | Hello, Welcome, Reject | Version, backend identity, resource limits, negotiated capabilities |
| Calls | Probe, Open, Scan, ReadEntry, Create, Close | Unique request/operation IDs; handshake precedes calls |
| Constrained I/O | ReadAt/ReadNext, ReadData, BeginOutput, WriteData, FinishOutput | Handle tokens, offsets, counts, credits; no actual output paths |
| Metadata | EntryBatch, ScanFinished | Host validates fields, unique record tokens, and counts |
| Interaction | NeedSecret, NeedVolume, SupplySecret, SupplyVolume | Host presents UI; secrets are limited to the corresponding operation |
| Control | Cancel, CancelAck, Ping/Pong, Shutdown | Control has priority; an Ack does not mean cleanup is complete |
| Results | Progress, OperationFinished, Error | Monotonic sequence, one terminal state; claimed bytes/CRC are not trusted |

Each data direction defaults to at most four unacknowledged 256 KiB blocks; sending stops when credit is exhausted. Control messages are not blocked by data credit. The host validates operation, handle, range, and current state. Overages, out-of-order messages, duplicate terminal states, or illegal state transitions close the worker and fail the task.

Key payload constraints follow. Integers use bounded unsigned types and byte fields use CBOR bytes instead of forcing raw paths into UTF-8:

| Message | Required fields and constraints |
|---|---|
| Open | source_token, options; token binds the authorized source for the current operation |
| EntryBatch | session_token, entries; records include entry_token/raw_path/type/optional sizes, at most 128 per batch, with frames no larger than 1 MiB |
| ReadEntry | session_token, entry_token, approved_sink_token; entries outside the current plan cannot obtain a Sink |
| ReadAt | source_token, offset:u64, length:u32; length ≤256 KiB, range stays within authorized source, EOF may return a short read |
| WriteData | sink_token, block_sequence, bytes; write only in approved order/seek capability; reject duplicate blocks |
| FinishOutput | sink_token, reported_size, checksum_info; host decides whether to commit from measured bytes and its own accounting |
| OperationFinished | outcome, counts, integrity_info; only one terminal state per operation, and success cannot be claimed with unfinished Sinks |

Tokens cannot be reused across sessions, tasks, or workers; BeginOutput cannot continue after cancellation revokes them. The Create input catalog contains only relative archive names and read tokens for user-authorized inputs. Extensions cannot add files themselves using local paths.

The host needs separate stdout/stdin IPC and bounded stderr diagnostics; plugin logs must not mix with the protocol channel. Diagnostics omit secrets and full paths by default and have total-volume limits. The host cannot assume an unknown plugin will respect log privacy, so enablement information must disclose its access to user content and passwords.

The host independently meters actual writes and computes verifiable checksums. However, if expected CRC/digests and parse results also come entirely from an untrusted plugin, this cannot independently prove that content is faithful to the original archive. The sandbox protects permission boundaries; content correctness still depends on backend trust and interoperability. When stronger assurance is needed, use independent trusted parsing/verification. Do not claim a general broker can automatically verify a malicious plugin's correctness.

## 7. File authority and sandboxing

The broker provides all sources, volumes, and outputs using operation tokens. By default, third parties receive neither raw source paths nor broad directory handles; they are authorized only for bounded read ranges and a current output Sink. If a password algorithm needs a plaintext secret, the plugin inevitably accesses it. The host cannot claim a malicious decryption plugin will never see the password.

On Windows, evaluate restricted tokens/AppContainer, Job objects, and handle allowlists. On Linux, evaluate namespaces, seccomp, Landlock/mount restrictions, and cgroups. On macOS, evaluate sandbox helpers, entitlements, and a broker compatible with supported distribution. Each platform requires evidence that arbitrary writes, networking, descendant escape, and unauthorized file access are blocked. This design does not assume availability of particular APIs.

Builds that cannot implement/verify sandboxing disable third-party execution. Official trusted workers may be allowed separately under release policy, but the UI must not label them “sandbox safe.” A sandbox does not protect the confidentiality of an authorized archive from a malicious format extension the user chooses; origin, permissions, and trust remain necessary.

## 8. Lifecycle and failure

By default, each backend has at most one worker and each worker has one active heavy task. The host supervises memory, protocol size, responsiveness, and CPU/no-progress budgets. A no-progress budget must not simply treat “not finished in 60 seconds” as a crash. When reading complex blocks, consider heartbeat, bytes/progress, and user cancellation together.

After cancellation, give the backend two seconds to cooperate; terminate the worker if it does not exit. Termination releases only uncommitted output and does not revoke already committed entries. A backend crash does not trigger infinite automatic retries. Repeated crashes on the same input isolate the backend and notify the user. Updates wait for all related tasks to finish; the old package remains until its reference count reaches zero.

## 9. Open-source extension development workflow

Implement the reference skeleton → declare actual capabilities → run SDK conformance → submit minimal and adversarial samples → test interoperability with independent reference tools → check cancellation/quotas/corruption → license/SBOM → platform package verification. Provide source-level ZIP/TAR samples and a process-level read-only example; generate documentation from interface definitions.

The Rust SDK uses SemVer; breaking trait changes increment major. The wire protocol has separate major/minor versions. Before freezing the protocol, two independent implementations must interoperate; testing only the host against code generated from the same source is insufficient. The support window covers at least the current and previous supported protocol major versions; the v1.0 release policy determines the specific withdrawal period.
