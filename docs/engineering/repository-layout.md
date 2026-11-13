# Repository layout and dependencies

Current product modules are defined by root Cargo.toml and CMakeLists.txt. This table describes actual directories; future splits in architecture designs do not imply existing independent crates.

| Directory | Ownership and dependency rules |
| --- | --- |
| apps/desktop | Qt/QML, thin C++ controllers, models, settings, and system interaction; accesses the core through the bridge |
| crates/xdzip-cli | Qt-free arguments, terminal, JSON, managed installation, and update host; directly depends on runtime, secure I/O, and update core without interpreting formats |
| crates/xdzip-domain | Pure types and invariants; must not depend on Qt or concrete codecs |
| crates/xdzip-codec-api | Capability, borrowed input, encoding output, and cancellation contracts; depends on domain |
| crates/xdzip-compression | Shared Deflate/Deflate64/GZIP, LZMA/LZMA2, BZip2, PPMd7, MSZIP, LZX, and Quantum algorithms/wrapping without third-party production dependencies or path/publication authority; see the [contract](independent-rust-codecs.md) |
| crates/xdzip-crypto | Format-independent fixed-memory SHA/HMAC and BLAKE hashes with zeroizable sensitive state, reusing admitted primitives; does not interpret archive passwords or decide publication |
| crates/xdzip-codec-zip, xdzip-codec-tar | ZIP and TAR/GZIP implementations; no arbitrary output paths |
| crates/xdzip-codec-7z | In-house 7z container, method graphs, filters, AES composition, streaming creation, and bounded rewriting without file-path ownership |
| crates/xdzip-codec-cab | In-house read-only CAB container using host-authorized volumes and shared algorithms; preserves raw names, UTF-8 declarations, and ordinary duplicate-name records without discovering volumes or publishing output. See the [contract](independent-rust-codecs.md#cab只读规格) and [sources](cab-format-sources.md) |
| crates/xdzip-codec-optical | In-house read-only ISO9660/Joliet/Rock Ridge/UDF parsing and content streams without file-path ownership |
| crates/xdzip-codec-bridge | Shared thin CXX boundary from isolated worker to Rust 7z, CAB, and optical backends, reusing authorized streams, cancellation, bounded input, output, and staging protocol without file paths |
| crates/xdzip-codec-native, tools/native-codec | Isolated worker for Rust 7z/RAR/optical/CAB backends; thin C++ supplies protocol and system calendar/encoding services only. Native independently checks prepaid output and auxiliary work; see the [I/O contract](native-codecs.md) |
| crates/xdzip-extract-plan | Metadata preflight and bounded reports without destination authorization or write capability |
| crates/xdzip-io | Input trees, native authorization and identity, secure staging, and file commits |
| crates/xdzip-runtime | Sessions, bounded memory/disk indexes, queries, selection, bounded preview, creation/extraction tasks, and scheduling |
| crates/xdzip-qt-bridge | cxx type bridge, dual handles, and lifecycle; depends on runtime |
| crates/xdzip-plugin-protocol | std-only bounded CBOR/framing without path, process, or Qt authority |
| crates/xdzip-plugin-host, examples/sdk | Independent read-only SDK trial, controlled I/O, native sandbox gates, and independent Python peer; outside product format discovery and runtime closure |
| tools | Independent bridge, format, I/O, and process-count probes, not GUI product entry points |
| scripts, .github | Generation, checking, deployment tools, CI, and collaboration templates |
| release-notes | Reviewable version-specific release notes and version/build metadata; no execution reports or generated packages |
| design/hifi, design/html, design/assets | High-fidelity originals, interactive designs, and runtime asset inputs |
| docs | Durable Markdown design, engineering contracts, reusable verification methods, and necessary images; no accumulated execution reports |
| specs | Machine-readable visual, terminology, and dependency inventories for product resources, generators, and checks |
| third_party | Third-party sources and license materials retained with code |

Independent xdzip-index, xdzip-platform, and complete external-format conformance tools are future architecture. Current indexing is in runtime, and platform file operations are implemented in io and desktop adapters. Existing plugin-host is only a [read-only SDK trial](sdk-trial.md), not a complete plugin ecosystem. Tests are primarily maintained with Rust modules, apps/desktop/tests, and scripts checks; empty unimplemented shells are not created.

CAB uses fixed Rust product routing. Official Cab handlers and CAB-specific LZX/Quantum do not enter compilation. Calls continue through the same isolated worker, without new product executables, test switches, or failure fallback. Complete-frame prepaid work is defined by codec-api; containers hold per-frame credit, and Native receivers and runtime independently verify their allowances and output limits. See the [in-house contract](independent-rust-codecs.md#cab只读规格).

See [system architecture](../design/02-system-architecture.md) for dependency design. Avoid domain reverse dependencies on runtime, codecs notifying QML directly, and platform layers deciding UI wording. Generated bindings, samples, reports, and build output go under build/, target/, or dist/, not source directories. See [installation and associations](installers-associations.md#兼容基线的长期输入) for durable historical-compatibility inputs.

`tools/image-preview/` is a private QtGui/QtSvg static-image worker. Input is bounded verified content and output bounded RGBA; it holds no archive or destination paths. See the [preview contract](content-preview.md) for platform support and failure behavior.
