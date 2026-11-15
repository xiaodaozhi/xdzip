# Bandizip format inventory and xdzip feasibility

This assessment records the Bandizip format inventory used for the format-expansion work and the capability boundary that xdzip can defend with source, fixtures and automated tests. The inventory is taken from the Bandizip product page and its format-help pages, checked on 2026-11-15:

- [Bandizip product page](https://www.bandisoft.com/bandizip/) lists: `7Z`, `ACE`, `AES`, `ALZ`, `ARJ`, `BH`, `BIN`, `BR`, `BZ`, `BZ2`, `CAB`, `Compound(MSI)`, `DAA(1.0)`, `DEB`, `EGG`, `GZ`, `IMG`, `ISO`, `ISZ`, `LHA`, `LZ`, `LZH`, `LZMA`, `PMA`, `RAR`, `RAR5`, `SFX(EXE)`, `TAR`, `TBZ/TBZ2`, `TGZ`, `TLZ`, `TXZ`, `UDF`, `WIM`, `XPI`, `XZ`, `Z`, `ZIP`, `ZIPX`, `ZPAQ`, `PEA`, `UU`, `UUE`, `XXE`, `ASAR`, `ZSTD`, `NSIS`, and `LZ4`.
- The same page states that Bandizip creates `ZIP`, `7Z`, `ZIPX`, `EXE (SFX)`, `TAR`, `TGZ`, `LZH`, `ISO`, `GZ`, and `XZ`; the longer list is primarily an extraction list.
- [Bandizip ALZ and EGG help](https://www.bandisoft.com/bandizip/help/how-to-open-alz-and-egg-files/) describes ALZ Deflate/obfuscated Deflate/BZip2 and ZipCrypto, and EGG Deflate/BZip2/LZMA/AZO with AES/LEA encryption, solid blocks and split archives.
- [Bandizip ZPAQ help](https://www.bandisoft.com/bandizip/help/zpaq/) describes ZPAQ as a separate, solid, journaled format. It is not a ZIP method alias.

## Current xdzip disposition

The table uses the following meanings. **Admitted** means the current registry, runtime, CLI and desktop path have automated evidence for the stated profile. **Alias/profile** means the extension is admitted through an existing container only after content validation. **Candidate** means the suffix is useful as a future entry point but no decoder is claimed. **Blocked** means that the format cannot be admitted responsibly until a public, independently testable wire specification and compatible implementation or owned algorithm are available.

| Bandizip name | xdzip status | Defensible current scope or reason |
| --- | --- | --- |
| 7Z | Admitted | Independent Rust 7z backend, including the declared password, solid and split profiles. |
| ACE | Blocked | Proprietary legacy format and old `unacev2` implementations need a licensing and isolation decision; no product decoder is present. |
| AES | Admitted | AES Crypt v0–v3, password and HMAC profile; this is the `.aes` envelope, not arbitrary encrypted archives. |
| ALZ | Admitted | Read-only ordinary single-volume Store and raw Deflate with exact sizes and CRC-32; modified BZip2, ZipCrypto and split volumes remain outside the profile. |
| ARJ | Admitted | Read-only Store and methods 1–4 with complete headers, sizes and member CRC-32. |
| BH | Admitted | Read-only `BH 05 07` ordinary members, method 8 raw Deflate, exact sizes and CRC-32. Store, encryption, volumes and other methods are excluded. |
| BIN | Candidate | Extension is ambiguous across disk, executable and data formats. Detect only a validated inner format; never register generic BIN as an archive decoder. |
| BR | Admitted | Bounded Brotli stream and TAR.BR profiles. |
| BZ / BZ2 | Admitted | BZip2 single-file streams and the declared TAR.BZ2 aliases. |
| CAB | Admitted | Independent Rust CAB backend with Store, MSZip, Quantum, LZX and authorized cabinet sets. |
| Compound(MSI) | Candidate | Structured Storage/CFB parsing and MSI streams need a separate container contract; extension-only admission is unsafe. |
| DAA(1.0) | Blocked | Proprietary disk/archive format without a maintained public implementation in the product dependency policy. |
| DEB | Admitted | Common `ar` envelope, control/data views and admitted compressed members. |
| EGG | Admitted | Read-only non-encrypted, non-solid single-volume Store and raw Deflate with exact block sizes and CRC-32; AZO, BZip2, LZMA, encryption, split and other profiles remain outside the scope. |
| GZ | Admitted | GZIP members, including concatenated streams and TAR.GZ. |
| IMG | Candidate | Ambiguous disk-image suffix. ISO/UDF admission is content based and does not imply generic IMG support. |
| ISO | Admitted | ISO9660, Joliet and Rock Ridge read-only images. |
| ISZ | Blocked | Encrypted/compressed ISO container; needs a dedicated format and key-handling contract. |
| LHA / LZH | Admitted | LHA levels 0–3, Store, LH1–LH7, LZS/LZ5, plus LH5 creation. |
| LZ | Admitted | Lzip v1 and TAR.LZ. |
| LZMA | Admitted | LZMA Alone and TAR.LZMA. |
| PMA | Admitted | PM0/PM1/PM2 read-only profiles with CP/M framing and CRC checks. |
| RAR / RAR5 | Admitted | Independent Rust RAR backend, including legacy, RAR5, RAR7 extensions, passwords, solid and split profiles within the declared scope. |
| SFX(EXE) | Alias/profile | ZIP SFX prefixes are validated by the ZIP reader. Generic executable extraction or execution is never granted. SFX creation remains outside the current writer set. |
| TAR | Admitted | TAR and filtered TAR.GZ/TAR.BZ2/TAR.LZMA/TAR.LZ/TAR.XZ/TAR.Z/TAR.BR/TAR.LZ4/TAR.ZST profiles as listed by the registry. |
| TBZ / TBZ2 | Alias/profile | BZip2-filtered TAR aliases. |
| TGZ | Alias/profile | GZIP-filtered TAR alias. |
| TLZ | Alias/profile | LZMA-filtered TAR alias. |
| TXZ | Alias/profile | XZ-filtered TAR alias. |
| UDF | Admitted | UDF read-only images, including hybrid-image namespace rules. |
| WIM | Candidate | Microsoft imaging container needs a bounded resource model and a separate implementation decision. |
| XPI | Alias/profile | ZIP content under the XPI suffix; content validation remains authoritative. |
| XZ | Admitted | XZ and TAR.XZ with LZMA2, declared checks and admitted filters. |
| Z | Admitted | Unix compress LZW and TAR.Z/TAZ aliases. |
| ZIP | Admitted | ZIP/ZIP64, AES-256 and supported volumes, methods and rewrite profiles. |
| ZIPX | Alias/profile | ZIPX is admitted for validated ZIP methods such as XZ; it is not a separate container implementation. ZIPX creation is not currently exposed. |
| ZPAQ | Blocked | Solid, journaled and highly stateful format; requires a dedicated implementation and large corpus before admission. |
| PEA | Blocked | Independent container and encryption design, no current owned implementation. |
| UU / UUE | Admitted | Bounded UU/UUE envelopes with physical member and line validation. |
| XXE | Admitted | Bounded XXE envelope profile. |
| ASAR | Admitted | Ordered metadata, stored ranges and declared SHA-256 checks. |
| ZSTD | Admitted | Standard Zstandard frames and TAR.ZST/TZST profiles. |
| NSIS | Candidate | Installer container with executable/script semantics; browsing requires a dedicated parser and must never grant execution authority. |
| LZ4 | Admitted | Standard LZ4 frames and TAR.LZ4. |

The inventory therefore does not justify adding generic handlers for `BIN`, `IMG`, `NSIS`, `Compound(MSI)` or `SFX(EXE)`. These names can describe executable, disk or installer data, and a suffix alone cannot establish an archive capability.

## Delivery plan

1. **Inventory and registry gate.** Keep one machine-readable entry per admitted profile in `specs/engineering/archive-formats.json`. Generate the Rust, C++, JavaScript and association consumers, then run both registry checks. Every new format gets a format contract and a corresponding acceptance paragraph.
2. **Wire contract before code.** Pin public source revisions and SHA-256 values, document header fields, name encoding, physical termination, methods, checksums, duplicate identity and resource limits. A reference tool is evidence only when it produces complete owned samples; a crash or a single Store sample is recorded as a limitation.
3. **Raw algorithm tests.** Implement the bounded reader independently of Qt. Cover empty and boundary payloads, every admitted method, exact input/output/work/memory budgets, cancellation, typed I/O, source mutation, truncation, trailing bytes, CRC and terminal poisoning. Unknown methods stay visible without payload authority.
4. **Container and product admission.** Add the codec crate, runtime weak/strong discovery, CLI list/test/extract, desktop labels and association fixtures. Verify frozen metadata and physical duplicate selection, then run the relevant automatic Qt and layout checks. No manual visual acceptance is needed for this research/format slice.
5. **Priority queue.** ALZ's conservative Store/raw-Deflate slice and EGG's non-encrypted, non-solid Store/raw-Deflate slice are admitted. The next feasible candidates are WIM only after a public bounded parser is selected, then Compound(MSI)/NSIS as metadata-only read paths. ACE, DAA, ISZ, ZPAQ and PEA remain blocked until specification and licensing gates are cleared. ZIPX creation and SFX creation are separate writer projects and do not follow from read alias support.
6. **Release boundary.** Keep all candidate formats out of the public capability table until the codec, runtime, CLI and desktop evidence is complete. Record unsupported methods and reference-tool limitations in the issue rather than silently falling back to Store or a different container.

The current BH slice follows this plan. It is read only, has no password or volume path, and uses an independently bounded Deflate reader with mandatory CRC-32 and exact physical termination.
