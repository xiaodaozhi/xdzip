# xdzip: 10 Coding conventions, comments, and quality engineering

## 1. General principles

Prioritize explicit data ownership, small meaningful interfaces, predictable resource use, and verifiable behavior. Readability does not mean abstracting all code into a framework; performance does not mean unsafe by default. Optimizations require measurements, and complex boundaries require tests and comments.

Decide business state in one place, do not swallow errors, and do not handle external input with panic. Cross-language, file, concurrency, cryptography, and parsing boundaries require stricter review than ordinary UI text. Adding a format adapter cannot change default security policy.

## 2. Rust conventions

- Unify edition, toolchain, lints, and dependencies across the workspace; releases lock Cargo.lock. New crates have explicit responsibilities and dependency directions.
- Use rustfmt; Clippy has no warnings for release verification targets. `allow` must be local and explained; do not globally disable indexing/error lints.
- Public APIs have rustdoc describing parameters, returns, errors, resource ownership, threading requirements, and important complexity. Do not repeat obvious internal getters in comments.
- Use checked arithmetic and conversion for external counts/offsets. IDs use newtypes so SessionId and TaskId cannot be mixed.
- Avoid unjustified `unwrap/expect` on production critical paths. A local `expect` may express an invariant, but its source must be explained and tested. Tests may use unwrap to make failures directly readable.
- `Option` means unknown and `Result` means failure; empty lists, zero, or empty strings do not hide corruption. Domain errors do not depend on UI text.
- Unsafe is forbidden by default. Necessary unsafe for platform I/O/FFI belongs in small modules, with a `SAFETY` explanation for each block covering pointer, length, aliasing, lifetime, and thread prerequisites.
- Do not hold ordinary locks across await/blocking I/O; keep critical sections short and ordered consistently. Do not blindly implement Clone/Send/Sync for public types.
- Reduce copies using borrowing, immutable Arc data, and batching. Ordinary loops are usually preferable to unreadable nested iterator combinations. Consider layout and cache optimization only after data sizes are known.

Comment example (illustrative interface, not yet implemented):

```rust
/// Publish a validated staging file to the planned destination.
///
/// The caller must hold a commit permit; cancellation takes effect before or after publication.
/// A changed destination returns Conflict without deleting it or retrying with a truncating write.
/// On success ownership transfers to the committed item; cleanup must not delete it.
fn publish_verified_file(/* typed parameters */) -> Result<CommitReceipt, CommitError>;
```

Meaningful internal comments explain why a commit permit is needed or why buffers must not be allocated from an archive-declared size; they do not restate “increment the counter here.”

## 3. C++/Qt conventions

Use C++17, RAII, clang-format, and explicit ownership. Choose a consistent ownership scheme for Qt QObject parent-child management and smart pointers; do not let both a parent and unique_ptr delete an object. Model objects are updated only on the GUI thread; worker results cross threads as value objects.

Qt connect specifies the receiving context and connection type to avoid capturing a potentially invalid raw `this`. QPointer assists lifetime checks but is not a universal lock against cross-thread races. Destructors do not perform unbounded joins; the shutdown protocol wakes the receiver first.

Do not duplicate Rust path, quota, or format logic in C++. Do not squeeze every request into one giant function handling arbitrary JSON. Validate lengths before narrowing to `int/qsizetype`; do not lossily convert raw bytes through QString and then use them as filenames.

Use a selected clang-tidy ruleset and exclude generated FFI code from automatic fixes. Exception boundaries, Qt non-exception paths, and Rust Result are unified at the bridge; exceptions do not cross into Rust.

## 4. QML conventions

Name components by responsibility and semantics. Use required property for mandatory properties and specify role types. Centralize visual tokens and use declarative state bindings. Do not form property-write cycles in onChanged or call business interfaces every frame.

List delegates do not read disks, parse archives, assemble HTML, or start a timer per row on component completion. Business actions go through controllers; long computations leave the GUI thread. Use qmllint, formatting tools, and QML tests. Different Qt versions may affect lint, so lock tools together with the runtime.

All visible strings go through translation, including plural rules for quantities. Fields, times, and bytes are displayed according to user locale; raw protocol values do not change with locale.

## 5. Test layers

| Layer | Content | Failure concerns |
|---|---|---|
| Domain unit/property tests | IDs, selection ranges, snapshots, state machine, budgets | Invariants that must not be violated, not mirrors of the implementation |
| Codec interoperability | Reference tool creates → product reads; product creates → reference reads | Unicode, ZIP64, duplicates, unknown sizes, truncated streams |
| Native filesystem integration | Platform handles, atomic publication, links, collisions, permissions | Nothing outside the destination changes, old target remains intact, partial files are not published |
| Concurrency/fault injection | cancel/commit, full queues, app close, crashes | No deadlocks or duplicate tasks; terminal state once; resources released |
| Bridge boundary | Unicode, lengths, invalidated objects, cross-thread use, shutdown | Memory ownership, integer narrowing, stale events |
| GUI scenarios | Keyboard/mouse, IME, focus, screen readers, scaling | Users can complete actions, not just screenshot comparisons |
| Performance | Reference-machine data and full process tree | Reproducible workload and uncertainty explanation |
| Distribution | Install, first run, update, uninstall, Qt replacement | Complete dependencies, valid signatures, no implicit developer-machine paths |

Properties include: selection identities are not replaced by row numbers under any query transformation; repeated Cancel does not add writes; failed items never become committed; output mappings within a plan do not conflict; actual accounting is at least the successfully written bytes; an old generation cannot change the new model.

## 6. Security and fuzzing

Fuzz magic detection, ZIP/TAR metadata, name parsing, CBOR frames, manifests, recovery logs, output mappings, and state messages. Limit fuzz-process CPU/memory/temporary writes; retain minimized crash samples. Use sanitizers for native backend and bridge tests. Rust Miri covers only applicable small modules.

Seeds include truncation, overflow, duplicate key fields, unknown types, oversized strings, layer recursion, CRC errors, ZIP64 boundaries, directory/file name collisions, and special links. Regression samples must be self-generated or authorized; avoid redistributing malware of unclear origin.

Do not request confidential archives in public security issues. The project provides SECURITY.md and a private GitHub Security Advisory disclosure entry; SECURITY.md governs reporting and supported versions. Fixes first strengthen verification boundaries and regressions rather than blocking only one sample hash.

## 7. CI and merge gates

Every PR: documentation/links, Rust fmt/Clippy/core tests, C++ format/selected tidy, qmllint, target-platform builds, and relevant integration tests. Test core logic on three platforms and the GUI on at least the primary reference platform. Prose-only changes do not run the entire codec performance suite.

Nightly: long fuzzing, fault tests, real filesystem/GUI integration, reference-machine benchmarks, dependency vulnerability checks, and license inventory. Before release: three-platform packages, required ad-hoc integrity and disclosure of lack of formal signing, full interoperability, upgrades/uninstall, leak/security checks, complete size, and SBOM.

Assess vulnerability-tool reports against actual reachability; a dependency compiling successfully does not exempt it from security fixes. Changes to unsafe, file commits, encryption, plugin protocols, and platform sandboxes must explain ownership, prerequisites, and failure behavior and run corresponding boundary and regression tests.

## 8. Contribution and documentation maintenance

PR descriptions identify the trigger, resulting behavior, tests, and boundaries; complex formats include capability changes and samples. Code records mechanisms and invariants, design documents record cross-module decisions, and ADRs record selection rationale. Do not copy the same implementation text into three places where it can quickly become stale.

Check public API documentation coverage and necessary security comments; do not score comment line counts. Do not add tests that mirror implementation line by line or large numbers of snapshots for color-only changes. Performance complexity requires demonstrated benefit and must let maintainers understand resource and error paths from local code.
