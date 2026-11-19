# ARC containers and compressed stream profiles

English | [简体中文](../zh-CN/engineering/arc-streams.md) | [繁體中文（台灣）](../zh-TW/engineering/arc-streams.md)

`ArcReader` implements bounded raw decoding for ARC methods 3–9. Methods 1 and 2 are container Store profiles. `xdzip-codec-arc` and `ArcCodec` connect complete framing and required CRC-16 to runtime, CLI and desktop browsing, checking and extraction. The [format registry](../../specs/engineering/archive-formats.json) declares this read-only scope; creation, modification, passwords and volumes are unavailable.

## Container and host admission

The admitted profile uses ordinary ARC 5.x framing; ARC 6.x additional metadata and Spark extensions are not claimed. Every member starts with `0x1a` and its original method byte. Method 1 uses 25 bytes; all other nonzero methods use the ordinary 29-byte header. The thirteen-byte filename field must contain a nonempty NUL-terminated name. All integers are little endian: packed length at 15, DOS date/time at 19/21, required CRC-16 at 23 and ordinary decoded length at 25. Old Store inherits packed length. Both Store methods require equal lengths. Packed spans must fit, the next marker is mandatory, and the exact two-byte `1a00` terminator must coincide with physical EOF. No resynchronization, prefix scanning, padding or neighboring files are admitted.

The container preserves every physical duplicate and unknown method. Metadata reads have a 64 MiB hard ceiling, at most 100000 records, caller-bounded raw names and a 24 GiB physical input ceiling. SHA-256 binds every header byte and terminator, including ignored name padding, when reopening an exact `Index`. Header hashes do not authenticate payload. Each reader has cumulative 24 GiB actual-input and decoded-byte ledgers and 96 GiB work, reserves table-CRC byte work before decoding, and terminates on any failure without budget refund or partial-index publication.

The host retains strong-container and ZIP/SFX precedence, and fully validates weak ARC proposals under its shared probe ledger. Names are decoded as complete fields before DOS backslash conversion, using the shared bounded archive encoding decision. Original bytes and method IDs remain available. Suspicious/unresolved names and unknown methods remain visible without payload authority. ARC has no admitted directory, link, mode-restoration or external object profile. Legacy XOR has no encryption flag; stream, size and CRC validation remain mandatory, without password guessing or fallback. No dedicated Linux MIME registration is claimed by this profile.

Payload opening reprojects and compares frozen entries; selected IDs identify physical rows and must match exactly. Runtime preflight, frozen host source identity, private staging, cancellation, old-file protection and final publication remain authoritative. Whole-archive `supported_content` checking actually decodes every admitted regular member, validates exact size and CRC-16, and fails on unsupported rows. A supported physical selection can still be extracted beside an unsupported neighbor.

## Sources and references

The original ARC Unix port maintained by Howard Chu and Anibal Monsalve Salazar is pinned to [commit `70f0b93516d0ee222db5cf1241aad0db1b721c0d`](https://github.com/ani6al/arc/tree/70f0b93516d0ee222db5cf1241aad0db1b721c0d). Its `Arcinfo` records Robert Freed's ARC 5.12 framing description. Limited wire observations use `arcunp.c`, `arcusq.c` and `arclzw.c`. The authors' `LICENSE` and `COPYING` record GPL licensing. Reference algorithms are compiled unchanged only under build/toolchain, with a project-owned test adapter. They are not compiled into or deployed with the product.

Independent comparison uses Russell Marks's nomarch 1.4, GPL version 2 or later. Its original source package is available from the [Debian source pool](https://deb.debian.org/debian/pool/main/n/nomarch/), SHA-256 `fe20da34e0d3ba0cf6388701f44ac22224cf65130ddbc5fcbc27bc4949a6e1ad`, 22411 bytes, matching the maintainer's source descriptor. No strict clean-room claim is made. Product code expresses the wire fields with project-owned bit, input, work and allocation primitives. The [owned fixture record](../../tests/fixtures/arc/README.md) distinguishes native encoder output, authored wire inputs, independent complete-byte comparisons and reference limitations.

## Method scope

| Method | Wire profile | Subsequent stage |
| --- | --- | --- |
| 1 | Obsolete 25-byte framing with implicit original length | Store |
| 2 | Ordinary 29-byte framing | Store |
| 3 | Bytes with DLE `0x90` runs and escaped DLE | None |
| 4 | Explicit signed-child squeeze tree, LSB-first codes and EOF symbol 256 | Method-3 RLE |
| 5 | Fixed 12-bit MSB-first codes, original hashed dictionary | None |
| 6 | Same original hashed dictionary | Method-3 RLE |
| 7 | Fixed 12-bit codes, improved hash | Method-3 RLE |
| 8 | Initial maximum-width byte 12, 9–12-bit grouped LSB LZW | Method-3 RLE |
| 9 | No width header, 9–13-bit grouped LSB LZW | None |

Unknown methods, Spark extensions, passwords and SFX framing require separate container admission. A shared algorithm name does not authorize another method's bit order, dictionary, alignment, reset or RLE stage.

## RLE and squeeze trees

A normal byte is emitted and becomes the repeat history. DLE followed by zero emits a literal DLE without changing that history. A nonzero count emits `count - 1` copies of the previous normal byte; a count without history is invalid. The escape must be complete, and the exact decoded size bounds every repeat. RLE state survives dictionary strings and caller output chunks.

Squeeze begins with a little-endian node count, at most 256, followed by two signed 16-bit children per node. Nonnegative children name internal nodes; negative values encode symbol `-child - 1`, from 0 through 256. The root is zero; the legacy zero-node form has implicit EOF children. The complete graph must be a rooted tree with no cycle, shared internal child, unreachable internal node or out-of-range edge, and EOF must be reachable. Traverse least-significant bits first until EOF. Packed input must end in the same byte as that terminator; unused final bits are ignored. Decoding to the declared size alone does not replace the EOF check.

## Fixed dictionaries

The 4096-slot dictionary initially inserts byte strings 0–255 with predecessor `0xffff`. Codes identify physical hash slots. The old hash uses the middle twelve bits of the square of `((prefix + suffix) mod 65536) | 0x0800`. Method 7 instead multiplies the 16-bit sum by 15073 and takes twelve low bits. These constants define historical wire identifiers.

Collisions follow the existing slot chain, start probing 101 slots beyond its final element, then search linearly with wrap. Every probe and prefix expansion is bounded and charged. Each new string is the preceding string plus the current string's first byte. A code not yet present is admitted only for the exact newly inserted KwKwK slot. Full dictionaries stop growing. Twelve-bit codes are high-nibble first across bytes, distinct from the later LSB profiles. The final unused nibble is ignored; missing code bytes and extra complete bytes are invalid.

## Dynamic dictionaries

The first code must be a byte. Code 256 is CLEAR and 257 starts sequential dictionary additions. Width grows at dictionary thresholds to the method's maximum. Codes are packed into groups of eight; a width change or CLEAR ends the old group, consumes its unused alignment bits and starts a new group. Alignment bits are unspecified. CLEAR resets the next dictionary slot and width, requires a following literal, and cannot borrow a pre-reset dictionary string. No synthetic EOF code or bytes are supplied.

Dictionary growth, KwKwK, maximum-width operation and adaptive resets are separate from the RLE stage. Native forced-LZW reference inputs establish every width and actual resets. Authored CLEAR inputs independently cover each of the eight old-group positions.

## Budgets and failures

Packed and decoded lengths each have a hard 24 GiB ceiling and work a hard 96 GiB ceiling. Smaller caller limits prevail. Full private memory admission is 16 KiB for method 3, 32 KiB for method 4, 64 KiB for methods 5–7 and 96 KiB for methods 8–9. These bounds include the 8192-byte input buffer, dictionaries, reverse expansion, graph validation and state.

Dictionary initialization, consumed bits, tree edges, hash probes, prefix expansion, stack delivery and output/repeat work are charged. `work_remaining()` includes constructor work for a container-owned cumulative ledger. Cooperative polling covers both input-dependent and source-independent work. Output reads are at most 8192 bytes. Short input, typed host errors, quota exhaustion, cancellation, invalid codes/trees, trailing data and decoded-size disagreement terminate the instance; the first error retains its payload and subsequent reads cannot resume. Completed readers return stable EOF without spending more work.

ARC's required payload CRC-16 and complete physical framing belong to the container. Raw success alone does not authenticate content or grant filesystem, path, process, neighboring-file or publication authority.
