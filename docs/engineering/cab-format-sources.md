# CAB format sources and implementation basis

This document records sources, scope, and independent-implementation constraints for the CAB container, MSZIP, LZX, and Quantum. Product capabilities, resource governance, and per-format replacement rules are defined by the [isolated format worker contract](native-codecs.md), [in-house Rust contract](independent-rust-codecs.md), and [split-volume contract](archive-opening.md), respectively. Source records do not replace interoperability acceptance or release license review.

## Microsoft technical documents

Prefer Microsoft's format documentation. The table pins the copies actually read. Dates identify document versions and do not mean a URL will always return identical bytes. Verify downloaded copies by SHA-256; matching titles alone do not establish identical versions.

| Document and entry point | Pinned version | Scope and limitations |
| --- | --- | --- |
| [MS-CAB official archive entry](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprotlp/5641c1b2-35ed-4542-9058-a07ccab5b0a1), [PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/Archive_Exchange/%5BMS-CAB%5D.pdf) | Footer `v20110304`, Release 2011-03-04 | §2 CFHEADER, CFFOLDER, CFFILE, CFDATA; §3.1 checksum method. Microsoft lists this as archived and explicitly warns against relying on archived material. Do not call it a current maintained specification. It does not define internal compression algorithm bitstreams. |
| [MS-MCI](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-mci/27f0a9bf-9567-4e40-ad66-6ae9ab9d2786), [PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-MCI/%5bMS-MCI%5d.pdf) | 2016-10-17, revision 7.3 | §2 MSZIP structures, CK marker, complete Deflate streams, and block sizes. The Deflate bitstream also follows [RFC 1951](https://www.rfc-editor.org/rfc/rfc1951.html). |
| [MS-PATCH](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-patch/cc78752a-b4af-4eee-88cb-01f4d8a4c2bf), [PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-PATCH/%5bMS-PATCH%5d.pdf) | 2025-08-19, revision 14.1 | §2.1 bitstream, windows, and distances; §2.2 header and E8; §2.3 blocks; §2.4 Huffman trees, solely to compare shared mechanisms and differences. It describes LZX Delta, not a replacement specification for CAB LZX. |

| Pinned PDF | SHA-256 |
| --- | --- |
| MS-CAB | `09f796a493547697ed7d4bc36222ac132bf3ca1bd6b1da76fbdc3b38b5307709` |
| MS-MCI | `cb097c434e1cfccb368f0d7e9084ab4d47a65809a2eea94bbf41f9722c1a0343` |
| MS-PATCH | `ea17466b97c9a9d4e2c815538387bf442fb2393ed29632c6f58b2c47ef1f29fe` |

Microsoft Open Specifications documents have their own copyright and intellectual-property terms, including documentation licenses needed for implementation and patent notices. They do not thereby become this project's Apache-2.0 material, and download availability does not establish that all patent or redistribution questions are resolved.

## 1997 CAB SDK documents

The original titles are Microsoft Cabinet File Format, Microsoft MSZIP Data Compression Format, and Microsoft LZX Data Compression Format. The copies read contain Microsoft's 1997 copyright notice and are hosted by a third party:

| Mirrored original | Scope read | SHA-256 |
| --- | --- | --- |
| [CABFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/CABFMT.DOC) | §2 container layout and cross-volume semantics, §3 fixed container examples, §4.1 checksum definition | `5a220658b76f8a66e9013e342b4eb236f057b597a8e6f3939c86f05efc51e83a` |
| [MSZIPFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/MSZIPFMT.DOC) | Introduction and Implementation Details, especially independent Deflate blocks and retained history | `046d283fa3b697e46c35b6a329bb90c86cbc9f4dedbed24c3ae43bd9b8ccf239` |
| [LZXFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/LZXFMT.DOC) | Concepts, LZX compressed data format, tree coding, distance/length, and E8 preprocessing sections | `68ba4df86e08bece3259574133f1c61606d143d85f749b388f5f1e8ea46f8bef` |

These `.DOC` files are actually RTF. Their titles, copyright notices, and CAB positioning support using them as copies of Microsoft-authored documents, but the host is not Microsoft and they have not been compared byte-for-byte with a Microsoft-published CAB SDK archive. Microsoft's [old SDK download URL](https://download.microsoft.com/download/platformsdk/cab/2.0/w98nt42kmexp/en-us/cabsdk.exe) did not provide a verifiable archive; mirror authenticity cannot be claimed on that basis. The reading scope is format documentation, not the SDK's FCI/FDI implementation or example code.

Public TCOMP parameters were also checked against constant declarations in Windows SDK 10.0.19041.0 `um/fdi_fci_types.h` and the [TCOMPfromLZXWindow documentation](https://learn.microsoft.com/en-us/windows/win32/api/fdi_fci_types/nf-fdi_fci_types-tcompfromlzxwindow). The header SHA-256 is `27a1f6f7728f5db64d11e434d101819d646ca49270930606bb76c7989ebe0254`. Only public parameter ranges and bit definitions were used; FCI/FDI algorithms were not called or analyzed on that basis.

Parts of the LZXFMT overview, tables, and text conflict. MS-PATCH helps identify common mechanisms, but its reference data, extended long matches, 128 KiB to 32 MiB windows, and additional block headers cannot be directly applied to CAB. CAB LZX windows are 15 through 21 bits and require independent checks against the original text, reference behavior, and distinguishing vectors.

## Quantum material scope

The original algorithm vendor material obtained is a [mirror of the Cinematronics QUANTUM user manual](https://carrero.es/wp-content/uploads/2025/07/QUANTUM.pdf), copyrighted Cinematronics 1993, 1994, 1995, all rights reserved. PDF SHA-256: `a2d6ed51e47db407ea5849b42004750f2f36c82a380f263b983c5282838a8212`. Section 6 describes an independent `.Q` archive container but does not define the arithmetic bitstream, probability-model updates, or frame termination required by CAB Quantum. Do not transfer `.Q` whole-stream state or file checks to CAB as rules.

[Matthew Russotto's Quantum research page](http://www.russotto.net/quantumcomp.html) and historical archives did not yield verifiable original text. Its content was therefore not treated as read evidence. It is not possible to determine whether the original fully defines the algorithm or infer its licensing terms.

Only [David Carrero's UnQuantum README](https://github.com/dcarrero/unquantum/blob/main/README.md) was additionally read, with copy SHA-256 `187af81f8231cfe4cc31aec0c76476b1e33179fe517a365c820fc49c1b8106e0`; that project's decoder source was not read. It is an unofficial overview of the `.Q` project and states that it references Russotto, libmspack, and binary research. It is not Russotto's original text and cannot fully replace a CAB bitstream specification. The project's MIT label does not establish that all its references and algorithm sources are MIT licensed.

These gaps are addressed by behavioral analysis of a pinned official reference implementation. Unofficial material is not categorically excluded, and the reference implementation is not called a Microsoft format specification.

## Pinned reference source and analysis scope

The reference is [official 7-Zip 26.05](https://github.com/ip7z/7zip/releases/tag/26.05), specifically [7z2605-src.tar.xz](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz), archive SHA-256 `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e`. See [native-codec-sources.json](../../scripts/native-codec-sources.json) for the build-source inventory. That inventory describes complete official sources; the records below identify only files actually used for CAB reference analysis.

Under the archive's `DOC/License.txt`, the following decoder and common files use default LGPL-2.1-or-later; no alternative license notices were found in these files. The archive also contains BSD terms for other files. Do not simplify the entire archive to one exception-free license or incorrectly label these decoder files BSD.

| File in archive | Actual analysis scope | SHA-256 |
| --- | --- | --- |
| `CPP/7zip/Compress/QuantumDecoder.cpp` | Arithmetic bitstream, models, tokens, frame termination, dictionary state | `1a22429d792493573f542782a45a7f702b6e636e5aee1d47bdb3d1cf75ef11e2` |
| `CPP/7zip/Compress/QuantumDecoder.h` | Model alphabet and selector constants | `7571ee0360a7abe8f297f4cd4e34df1863003f252c6fab66c61f8d1bd25a21b4` |
| `CPP/7zip/Compress/Lzx.h` | LZX bitstream constants | `779b32ecaecfc5b8c135e0051394001786d680fca4cfda57a480884342500e99` |
| `CPP/7zip/Compress/LzxDecoder.h` | Cross-frame retained-state boundaries | `18141528bc8587ade4f7b1e330ee35dc8e31d6c6ab34d86a2dff59d362d0d6c9` |
| `CPP/7zip/Compress/LzxDecoder.cpp` | CAB-mode trees, raw blocks, padding, repeated distances, E8, history, and termination; optimized code and memory layout are not implementation designs | `e78bf50f27f21e04966785fafe7f94fddc533a9e6103f79b81c4e7d7613d1aab` |
| `CPP/7zip/Compress/HuffmanDecoder.h` | Only full-tree/empty-tree admission, approximately lines 109–164 and 430–497 of the pinned copy | `22f93e29b2408ce7e1c1bca5f5f807c1383a8d1634fe3ae6cfcab15f084109c0` |
| `CPP/7zip/Archive/Cab/CabHandler.cpp` | Only compression methods, CFDATA frames, and history continuation, approximately lines 1020–1252 of the pinned copy | `77b82387eea40eda02929b90d3006877c08765089d6b96c5c12b9cfd3f56b8f7` |
| `C/7zVersion.h` | Pinned version identity | `f7dc58b3a3c01569e061eda9cfe633ad8983667eb0fa10ab94a6c364ddd7e699` |
| `DOC/License.txt` | Default file licensing rules and exceptions | `9ac2b4a97ab5d523965534d8b2d5868e511b39096d51fff458ab72c38b80fccc` |
| `DOC/copying.txt` | Identity digest of LGPL text, not interpreted as legal advice | `1e7e6bae5a5bde32f1ae5a7c37a082d1ab03cf89354f7f936ac40be9e39a6531` |

Line numbers only delimit reading scope for that pinned copy; file identity is established by digest. When upgrading the reference version, record differences again; reused line numbers do not establish unchanged behavior.

## Division between analysis and independent implementation

Initial preparation read only format documentation and project-owned interfaces. A designated reference analyst then explicitly read the source files listed above and turned missing or contradictory bitstream, mathematical update, reset, termination, and error conditions into behavioral specifications without code expression, handing them to LZX and Quantum implementers who had not read those sources.

Each algorithm's Rust implementation was independently organized by its implementer without reading the reference source, including state, types, control flow, and tests. The analyst did not implement either algorithm in Rust and separately implemented the CAB container. This does not mean nobody involved in CAB work saw reference source and cannot be called a strict clean room. Behavioral specifications must not contain reference code, line-by-line rewrites, transplanted structure, or translated comments; specification names, constants, and mathematical relationships retain their sources.

Reference reading, independent coding, and black-box interoperability answer different questions. An in-house encoder and decoder succeeding together do not establish independent compatibility. Reference-tool success does not prove every permissive behavior fits this project's integrity contract. Reference source, documents, tools, and samples retain their respective licenses. A Rust rewrite does not establish that legal risk has been eliminated. See the [license contract](licensing.md) and [corresponding-source contract](source-delivery.md) for licensing and release requirements.

## Technical distinctions that must be preserved

The following behavior boundaries are jointly constrained by format documentation and scoped reference analysis; they do not prescribe Rust internal structures or optimization methods.

- A CAB folder is a compressed-history unit; files are output ranges within it. File boundaries, CFDATA boundaries, and cross-volume continuation do not automatically reset history; only a new logical folder does. Cross-volume compressed fragments with `cbUncomp=0` are concatenated as one logical frame, while each physical CFDATA checksum is verified independently. That checksum is byte/word XOR, includes reserved data, and cannot be replaced with CRC or a single checksum after concatenation. Metadata, lengths, volume relationships, and payload bounds are checked separately.
- Each MSZIP frame contains CK and a complete raw Deflate stream. Trees and bitstreams reset per frame; at most 32 KiB history persists per folder. Frames are limited to 32,780 compressed bytes and 32,768 output bytes. Reuse the project's shared Deflate preset-history interface; do not create a second Deflate implementation. Zero-padding or forcibly padding truncated streams cannot replace strict termination checks.
- CAB LZX uses little-endian 16-bit words with most-significant-bit-first ordering within each word. Position-slot counts for 15- through 21-bit windows are 30, 32, 34, 36, 38, 42, and 50, respectively. Large-window counts in the old overview must be corrected against the complete distance table. Tree-length deltas subtract modulo 17; the aligned tree precedes main/length trees. Main, pretree, and aligned trees must be complete; the length tree may be entirely empty, but requesting that empty tree must fail.
- LZX raw blocks still have the common block length. Skip 1–16 zero bits before raw data, including one whole word when already word-aligned. An odd raw block's zero pad byte may be consumed at the beginning of the next frame; omission is tolerated only at final folder end with no next block. Ordinary compressed frames permit only 0–15 zero padding bits, not an extra complete word or arbitrary trailing bytes. LZX blocks may span CFDATA frames. Final `finish` must reject a block with declared output still unfinished; do not retain the old CAB integration's permissive missing end check.
- LZX LZ history stores bytes before E8 reversal; a new folder has no valid zero history to reference. E8 scans before the last ten bytes of each frame. Interpret translation size as a full unsigned 32-bit value and accumulate position across the folder. The older description of the last six bytes and comparisons valid only for positive signed sizes cannot be directly applied.
- Quantum resets its 16-bit closed-interval arithmetic-code state for every frame. Nine probability models and actual dictionary history persist across the folder. Seven selector values do not imply only seven models. Symbol frequencies increase by 8 each time; rescale before the next use only when total frequency is strictly greater than 3800. Per-model cumulative-frequency halving, reordering on the fourth and then every fiftieth time, and same-frequency order are bitstream behavior and cannot be replaced arbitrarily with language-default sorting. Dictionary windows are 10–21 bits and levels 1–7; level does not change decoding semantics.
- Quantum has no separate end token; the container supplies exact output length, and the same compressed bytes may satisfy different legal declared lengths. After reaching expected output, consume at least two zero bits and the minimum needed to reach a byte boundary, 2–9 bits in total, then exhaust compressed input exactly. Appended zero bytes, nonzero terminal bits, truncation, and cross-frame matches cannot succeed. A new folder cannot reference window content that has not yet been generated.

The host remains responsible for freezing the volume set and source identities, name parsing, entry identities, path preflight, task budgets, cancellation, staging, and commit. The format layer reads only host-authorized streams and does not independently open files by CAB-internal names. Preserve original names and UTF-8 declarations separately. Ordinary duplicate-name records are not merged; only continued files explicitly defined by the specification are combined. Algorithm errors must not mask host cancellation, quota, or I/O errors. Shared contracts govern interfaces and resource limits; do not derive new product permissions from reference buffer sizes or HRESULT categories.
