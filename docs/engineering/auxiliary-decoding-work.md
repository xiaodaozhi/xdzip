# Auxiliary algorithm work accounting

Extraction and checking separately accumulate file-decoded bytes, actual output, and auxiliary algorithm work. PPM model initialization, capacity restarts, and control/parameter decoding are not file content. They must not increase `decoded_bytes`, `written_bytes`, file progress, or compression expansion ratio, or produce prepaid delivery credit. Task snapshots do not add a user-facing auxiliary counter.

`DecodedSink::charge_auxiliary(bytes)` reports only auxiliary work already performed; its default implementation changes no file counters. Algorithms first admit future work against their own remaining budgets, then account for actual work after execution, including work completed before failure or cancellation. Unperformed reservations are not reported as actual work. Completed work is not refunded on error, model reset, or password retry. RAR's private total-work budget continues to bound file decoding plus auxiliary work.

## Isolation boundary

- Worker protocol `AUX_WORK=24` carries one little-endian `u64` count and no content. Each packet must contain 1..65,536, and the frame must end exactly. The host replies with an empty `ACK` after successful accounting. Reject zero, truncation, trailing fields, excess counts, and invalid operation phases.
- This message is permitted only for RAR payload decoding, not metadata scanning, creation, modification, or other formats. CXX passes counts synchronously and retains no Rust pointers; any source/output slices are borrowed only for the call. C++ exceptions are mapped to stable errors inside callbacks, Rust unwinding does not cross CXX, and abort or process exit fails unfinished output.
- A native payload instance separately accumulates at most 20 GiB of auxiliary work, retained across entries, worker restarts, and password changes. It records completed work before calling the host sink; sink rejection does not refund a later retry. `AUX_WORK` does not increase `WorkerCredit`.
- Runtime extraction and check tasks independently accumulate at most 20 GiB of auxiliary work, with the accounting object outside entry and password-retry loops. This budget is checked separately from actual file bytes and output limits. Exceeding limits or cancellation returns failure without publishing an unfinished file; public counters cannot be changed to evade limits.
- `WORK=17` continues to represent decoded file bytes. Only its successful admission grants one-time delivery credit for `PRECHARGED_DATA=23` to the same valid worker. Credit is deducted before delivery and cleared on restart. RAR, 7z, and CAB use this path; scanning and creation cannot. Auxiliary work cannot pay for such file output.

High-compression admission uses frozen declared sizes and packed/whole-archive sizes, not auxiliary accounting. Runtime file decoding and output limits remain independently enforced. If an 87-byte file performs several MiB of model initialization, file counters still reflect only its actual 87 bytes.

## Required acceptance conditions

- A normal small file can finish extraction and checking after more than 64 KiB of auxiliary work and model initialization; file counts remain exact and final content verification succeeds.
- Native and runtime each verify rejection when repeated actual auxiliary packets reach 20 GiB. Password retries and worker replacement do not reset cumulative costs; unfinished files are not committed.
- Fake-worker requests for other formats, metadata phases, empty/oversized packets, trailing fields, and prepaid output funded only with auxiliary work all fail.
- Ordinary `WORK` can fund RAR prepaid output for the same worker; credit is consumed once and cannot be reused after an error or restart.
- Protocol fixtures verify only state machine and accounting. Complete Rust RAR worker, runtime business path, three-platform, and desktop acceptance retain separate actual evidence and are not inferred from a passing protocol fixture.

The protocol fixture compiles only the project's own `tools/native-codec/main.cpp` and its stub, with no compression algorithm:

```sh
cmake -S crates/xdzip-codec-native/tests -B build/auxiliary-protocol
cmake --build build/auxiliary-protocol --config Release
```

Point `XDZIP_NATIVE_CODEC_WORKER` to the fixture, set `XDZIP_AUXILIARY_PROTOCOL_FIXTURE=1`, and explicitly run `cargo test -p xdzip-codec-native auxiliary_protocol -- --ignored`. Ordinary targeted regression uses `cargo test -p xdzip-codec-native -p xdzip-runtime auxiliary`; codec API tests verify default sink compatibility. Generated evidence stays in build or CI. Task results and remaining items are recorded in [migration task #51](https://github.com/xiaodaozhi/xdzip/issues/51).
