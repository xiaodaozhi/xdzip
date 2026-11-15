# PMA framing and PM2 primitive

English | [简体中文](../zh-CN/engineering/pma-streams.md)

PMarc uses LHA-like sequential headers with `-pm0-`, `-pm1-` and `-pm2-` methods. These identify different algorithms, not aliases for the similarly named LH methods. The read-only PMA profile admits level-0 PM0 and PM2 members for browsing, checking and selected extraction. PM1 remains visible with an unsupported-content warning. Creation, modification, encryption, volumes and repair are not admitted; the format registry remains authoritative.

## Sources and reference boundary

Yoshihiko Mino's [original software page](https://www.asahi-net.or.jp/~am9y-mn/fswlist.htm), [PMarc 2.00 manual](https://www.asahi-net.or.jp/~am9y-mn/doc/pmarc2.doc) and [original tool package](https://www.asahi-net.or.jp/~am9y-mn/arc/pmarc200.pma) identify PM2, binary/text modes and CP/M's 128-byte file records. The manual is plain CP932 text despite its suffix. Tool and document identities, compiler, emulator and independently authored inputs are recorded with the [fixtures](../../tests/fixtures/pma/README.md). Original tools retain their own terms and are not redistributed or included in product dependency closures.

The manual is not a complete compressed-bitstream specification. Limited wire observations use Simon Howard's [PM2 decoder](https://github.com/fragglet/lhasa/blob/v0.6.0/lib/pm2_decoder.c), Git blob `b711988f2ac8a21ea4081e06020ecbd3fb83c49c`, and [common PMA facts](https://github.com/fragglet/lhasa/blob/v0.6.0/lib/pma_common.c), blob `7433c3b6fcbd742963a7a63a03d21d858e798631`, under Lhasa's ISC license. No strict clean-room claim is made. Rust uses the project's bounded bit reader, generic complete canonical prefix implementation and an independently expressed flat byte permutation, without copying the reference's linked-list or tree structures. No reference algorithm source or binary is compiled into the product.

Original PMarc 2.00 runs inside hash-pinned, source-verified CP/M emulation, with every source/output confined to owned test directories and binary I/O explicitly selected. Lhasa 0.6.0 and the original PMext 2.22 in its documented IBM-PC port independently compare complete output bytes. A zero-length input is skipped by the original encoder and produces an empty archive; it is not evidence of an encoded empty member. Independent wire-authored cases separately cover a 28-bit literal prefix and the accepted leading-bit variant. Source observations, generated data and actual reference comparisons have distinct provenance.

## Container requirements

PMA headers and payload ranges require the same arithmetic, checksum, name, duplicate, frozen-source and special-object rules as [LHA](lha-containers.md). PM0 is Store with equal packed and original sizes. PM1 and PM2 must not route to LH decoders. Present payload CRC-16 values must pass after complete decoding; an opened or listed method does not establish content validity.

An original CP/M archive ends with a zero byte and enough `0x1a` fill to reach the next 128-byte record boundary. The admitted padded form must be exactly aligned, with at most 127 fill bytes, all of that value. An exact zero terminator may also delimit an unpadded container. These are specific framing requirements, not permission to ignore arbitrary trailing bytes, synthesize a missing terminator or append decoded EOF bytes. Container padding never belongs to a member's packed span. Source freezing must include the entire physical file, including padding.

An eight-byte proposal never publishes a format by itself. The complete metadata chain, exact zero terminator and optional physical padding must validate before the host publishes rows. All weak-family probes share one input ledger. A CP/M empty archive identifies PMA by its bounded fill; an indistinguishable one-byte zero terminator retains the earlier LZH choice. Strong containers retain priority over weak discovery, and raw TAR-marker collisions require complete container admission. Source I/O failures remain terminal, including a typed malformed-input value originating from the host.

Physical row IDs, raw/decoded names, flags, sizes and methods are compared again before payload output. PMA method 2 routes directly to PM2, without LHA's LH2 variant trials. CRC failures, unsupported methods and source changes retain the host's private staging and old-file protection. `.pma` is a shell/drop candidate on applicable platforms. No dedicated MIME type is declared, so this does not add a Linux PMA default-association claim.

## PM2 wire model

The stream is MSB-first and starts with a leading bit whose two values are independently exercised without changing the decoded result. A code table declares a five-bit count of 1–29 and a three-bit minimum length. A zero minimum selects the constant symbol `count - 1`; otherwise a three-bit width encodes each length, zero for absent or `minimum + value - 1` for present. Complete canonical trees are required. At most 29 leaves permit at most 28 meaningful levels; `Prefix<29>` supports those without enlarging existing Brotli/LZH `Canonical = Prefix<17>` instances. Invalid counts, oversubscribed/incomplete trees and excessive depths fail before use.

Codes 0–7 select a byte's recency rank, with bases 0/8/16/32/64/96/128/192 and additional widths 3/3/4/5/5/5/6/6. Initial rank order is `20–7f, 00–1f, a0–df, 80–9f, e0–ff` in hexadecimal. Every output byte, including a history copy, moves to the front. The Rust implementation finds and rotates a bounded flat permutation, charging actual comparisons and movements.

Codes 8–28 represent copies. Classes 0–14 produce lengths 2–16. Classes 15–20 use bases 17/25/33/65/129/256 and widths 3/3/5/6/7/0. Class 0 directly reads a six-bit offset; classes 1–19 use a position table. Position zero reads six low bits, otherwise position 1–7 reads `position + 5` low bits plus that power-of-two base. Class 20 always uses offset zero. The actual distance is offset plus one. History is an 8192-byte space-filled ring; overlapping and initial-history references retain byte-by-byte semantics.

When needed, the position table initially has five three-bit lengths, then six at 1024 output bytes, seven at 2048 and eight at 4096. At 4096 an explicit bit controls code-table replacement, followed by the eight-position table. From 8192 onwards, every 4096 bytes an explicit bit controls replacement of both tables. Events occur at exact emitted-byte positions, including inside copies and at the exact end of output. Original boundary samples contain final control fields. They cannot be omitted or replaced by successful partial output.

A position table with one present symbol is constant. An all-zero table is represented as absent and may remain unused, including at a final control event; any command that actually requires it fails. No zero-filled table is repaired into a fabricated symbol. Code-table selection determines whether a position table is needed. The final subbyte fill is ignored, but logical packed bytes must end exactly; a complete extra byte is rejected.

## Limits and failure

PM2 admits its 8192-byte history plus 32 KiB for buffered input, prefix tables, recency state and bounded scratch. Packed and decoded sizes each have a hard 24 GiB ceiling, algorithm work 96 GiB. Smaller caller budgets remain binding. Construction, table reads, recency operations, copies and bits are metered; `work_remaining()` preserves actual work across members. Reads return at most 8192 bytes, with cooperative polling even without another source read.

Only bare `Interrupted` I/O retries. Typed interruption keeps its payload and terminates; source, malformed data, cancellation and resource errors poison the instance. A host must drain successfully, verify its declared size and CRC, and keep output private until success. No filenames, processes, permissions or filesystem publication authority enter the raw reader. Raw tests, complete container tests, runtime/CLI/Qt tasks, another platform and release permission remain separate evidence.
