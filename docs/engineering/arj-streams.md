# ARJ compressed streams and container admission

English | [简体中文](../zh-CN/engineering/arj-streams.md)

`ArjReader` provides bounded raw decoding of ARJ methods 1–4. Store belongs to the container, not this reader. Raw tests do not establish product opening, checking, extraction or creation; the [format registry](../../specs/engineering/archive-formats.json) remains authoritative. Container and host admission require their own complete evidence.

## Sources and reference boundary

The [open-source ARJ project](https://arj.sourceforge.net/) supplies [source 3.10.22](https://arj.sourceforge.net/files/arj-3.10.22.tar.gz), SHA-256 `589e4c9bccc8669e7b6d8d6fcd64e01f6a2c21fe10aad56a83304ecc3b96a7db`, and its [Win32 3.10 package](https://arj.sourceforge.net/files/arjw_310.exe), SHA-256 `e5cc5d8f82c5f08246e1c0a8c5be77e5f796c2dd30f987e6bbb27a2c1df1e371`. The source includes GNU GPL version 2 text. Limited wire observations use `defines.h`, `decode.c` and `arj_arcv.c`; the native reference remains unmodified and under its original terms, solely in build/toolchain.

Independent comparison uses official [7-Zip](https://www.7-zip.org/), with bounded source observations from its [ARJ handler](https://github.com/ip7z/7zip/blob/main/CPP/7zip/Archive/ArjHandler.cpp), Git blob `da890f279d35c00bc61d332ce55f4d96ecd6d373`. No strict clean-room claim is made. The Rust implementation reuses project-owned bit, prefix and LZ primitives, and directly expresses the fast unary fields. Reference source, binaries and compression engines do not enter product dependency closures. Exact component and owned-input identities are recorded with the [fixtures](../../tests/fixtures/arj/README.md).

## Static methods 1–3

Methods 1–3 share the static Huffman/LZ wire model while native encoders may choose different searches. Their 16-bit block command counts, 19-symbol length alphabet, 510-symbol literal/length alphabet, 17-symbol position alphabet and five-bit position count reuse the shared bounded LZH prefix implementation through an explicit ARJ profile. They do not acquire LH3's legacy tree fields or a public LHA method identifier. Zero block counts are not admitted.

Literal symbols are 0–255; other symbols yield lengths `symbol - 253`, from 3–256. Position symbol zero gives offset zero; otherwise offset is `2^(symbol - 1)` plus that many low bits. Actual distance is offset plus one, at most 26624, and must refer to already emitted bytes. A 32 KiB ring implements the bounded logical history; its larger allocation does not widen admission. LHA's initial spaces remain separate and cannot initialize ARJ output. Dictionary wrap and overlapping copies retain sequential byte semantics. Complete canonical trees, singleton fields, zero runs and 16-bit code depth keep the shared prefix validation and work charging.

## Fast method 4

The stream is MSB-first. A length field counts 0–7 leading one bits, with a zero stop except at the maximum. Read that many low bits and add `2^count - 1`. Zero denotes an eight-bit literal; nonzero denotes a copy of value plus two, from 3–256 bytes. The pointer similarly counts 0–4 leading one bits, with low width `9 + count` and base `512 * (2^count - 1)`. Offset plus one gives a maximum distance of 15872. Every reference must name emitted history, using a 16 KiB ring with overlap and wrap; no initial-history repair is permitted.

## Limits and failures

Static profiles admit 32 KiB history plus 64 KiB for input/tables/scratch. Fast decoding admits 16 KiB history plus 32 KiB for input/state/scratch. Packed and decoded sizes each have a hard 24 GiB ceiling, work a hard 96 GiB ceiling; smaller caller budgets remain authoritative. Output reads are at most 8192 bytes. Prefix operations, history work and source-independent copies preserve cooperative polling. `work_remaining()` supports the container's cumulative actual-work ledger.

Declared lengths cannot be exceeded. Logical packed bytes must end exactly; final subbyte fill is ignored, complete extra bytes or missing input fail, and no zero EOF bytes are fabricated. Only bare `Interrupted` retries. Typed errors retain their first payload and terminate; any source, malformed data, quota or cancellation error poisons the reader. Raw readers have no path, credential, execution, filesystem publication or CRC authority.

## Required container and host admission

ARJ main and file headers, their bounded first-header layouts, CRC-32, extended-header chains, exact packed spans and zero-length terminal marker must all validate before publishing a complete index. Physical IDs retain duplicate records. Names, code pages, separators, directories, legacy text/binary types, special objects and every unknown method require explicit projection and payload admission; a recognized marker does not authorize content.

Each declared file CRC must pass after complete decoding. Headers and optional archive security fields do not imply cryptographic authenticity. Passwords, encrypted methods, volumes, continued entries, chapters, recovery/security tails and self-extracting wrappers require separate profiles and evidence. No script, automatic command, external attribute, neighboring path or executable flag grants authority. Host source freezing, selected extraction, cumulative input/work/output budgets, private staging, cancellation, old-file protection and final publication follow the [extraction contract](m3-extraction.md).

The fixture generator creates only owned binary inputs, records requested and actual methods, and independently reads complete output through native ARJ and 7-Zip. Native Store fallback remains Store. It normalizes only unencrypted timestamps, the unused time-derived modifier and corresponding header CRCs; compressed bytes and content CRCs remain untouched. Empty native inputs produce Store entries, not evidence of an encoded empty compressed stream. Product admission also requires actual runtime/CLI/Qt and applicable candidate/layout automation under the [QA scope](../qa/README.md).
