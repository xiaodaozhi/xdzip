# WIM, MSI, and installer format sources and licenses

English | [简体中文](../zh-CN/engineering/wim-msi-installer-format-sources.md) | [繁體中文](../zh-TW/engineering/wim-msi-installer-format-sources.md) | [日本語](../ja-JP/engineering/wim-msi-installer-format-sources.md) | [한국어](../ko-KR/engineering/wim-msi-installer-format-sources.md) | [Français](../fr-FR/engineering/wim-msi-installer-format-sources.md) | [Deutsch](../de-DE/engineering/wim-msi-installer-format-sources.md) | [Español](../es-ES/engineering/wim-msi-installer-format-sources.md) | [Português (Brasil)](../pt-BR/engineering/wim-msi-installer-format-sources.md) | [Русский](../ru-RU/engineering/wim-msi-installer-format-sources.md)

This page describes technical references, licensing restrictions, and requirements for supporting WIM, MSI, and installer formats. References stay in `build/toolchain/references` or link to public specifications; none is a product dependency.

## WIM

- Reference implementation: `ebiggers/wimlib`, commit `cd5e231c348c255ae5088873b5a66ee0eb96fa07`.
- The repository offers GPLv3+ for the project and a conditional LGPL-2.1+ choice for the library portion. The LGPL choice is conditional on third-party dependencies such as `libntfs-3g` not forcing GPL terms.
- WIM is an imaging container rather than an ordinary archive. A bounded reader must handle resource tables, metadata streams, chunked resources, Xpress/LZX/LZMS compression, integrity tables, hard-link metadata and image selection without mounting or restoring security authority.

The reference implementation is useful for wire and corpus research but cannot be linked into xdzip under the current no-third-party-runtime policy. WIM is **excluded by product decision**: the project will not take on the imaging-specific reader, decompression-amplification model, integrity resources, hard-link semantics and complete corpus needed to support it. No registry, MIME, runtime or CLI entry is permitted unless this product decision is explicitly revisited.

## Compound File and MSI

- Microsoft [MS-CFB] is the primary public specification: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b . The page publishes PDF/DOCX revisions and an Intellectual Property Rights notice that permits implementation of the described technology, while patent licenses remain governed by the applicable Microsoft programs.
- A CFB reader needs sector/FAT/DIFAT allocation, directory red-black tree traversal, mini-stream allocation, stream-size bounds and cycle detection before any MSI-specific interpretation.

A standalone CFB metadata reader is technically feasible and is now implemented as the `xdzip-codec-cfb` read-only slice. It validates CFB 3.0/4.0 headers, including the 512-byte and 4096-byte sector bases, FAT/DIFAT chains, directory tree, mini-stream allocation, stream-size bounds and cycles before exposing storage and stream rows. Raw stream copying is available only after the immutable metadata snapshot matches. MSI tables, embedded cabinets, custom actions and transforms remain outside the contract: there is no installer execution, custom-action interpretation, package installation or write-back, and the `.msi` suffix grants no execution authority. The fixture and runtime evidence are automated in the codec and runtime tests.

## NSIS and other SFX

- NSIS reference source: `kichik/nsis`, commit `a96303eae3115221471bb127fa916a13d8bf9ecd`.
- Its `COPYRIGHT` records zlib/libpng for most source and the zlib compressor, bzip2 for the bzip2 module, and Common Public License 1.0 for the LZMA module.
- NSIS files are executable installers with script and embedded payload semantics. A parser must stay below the execution boundary, bound header and payload spans, and keep compression methods separate from installer behavior.
- The public source defines the fixed `firstheader` record: `flags`, `0xDEADBEEF`, `NullsoftInstallSystem` signature words, `length_of_header` and `length_of_all_following_data`. A static reader can locate and validate this record without opening the PE stub, but it still needs an independent model for compressed header blocks and complete owned installers.

NSIS is now **admitted as a static-metadata-only profile**. The codec scans only 512-byte-aligned records in a bounded PE overlay, accepts the two recorded `firstheader` magic values, limits flags and lengths, and requires the declared following span to fit the physical input. It then publishes one `NSIS/firstheader` metadata row. It does not decompress the installer header or expose any payload bytes. The `.exe` registry entry is intentionally neither a drag/drop candidate nor a shell association; content detection is authoritative. The parser remains independent of Qt and installer execution APIs. It never runs the PE stub, script, custom page, plug-in or extracted payload. Other SFX formats remain aliases only when their inner archive passes content validation.

## Admission gates

1. Pin source revisions and license text hashes before using a reference tool.
2. Create complete, redistributable fixtures for each admitted container and compression method; a header-only sample is insufficient.
3. Keep CFB/WIM/NSIS parsing independent from Qt and from installer, mount, registry or script execution APIs.
4. Add runtime, CLI and desktop entries only after exact physical identity, duplicate entries, cycle bounds, cumulative input/work/output limits, cancellation, source mutation and terminal failure tests pass.
5. Keep GPL/LGPL/CPL reference components in build/toolchain unless a separate dependency and license decision explicitly changes the product policy.
