# M10 candidate source and license gates

This record closes the next Bandizip inventory candidates after M9. It separates a public description of a wire format from permission to ship a decoder, and separates a reference implementation from a redistributable product dependency. None of the references below is linked into xdzip. A candidate cannot enter the format registry until its source revision, license text, complete fixtures, resource model and failure behavior are frozen.

## DAA 1.0

- **Public specification:** No vendor-published DAA 1.0 specification was found. Public descriptions identify DAA as PowerISO's proprietary disk/archive image and describe a header, a chunk-length table and independently compressed ISO chunks. The public record covers zlib and LZMA variants and later internal versions such as 110, but it does not define a stable 1.0 contract.
- **Reference implementation:** `daa2iso` 0.1.7e, the Debian/Fedora GPL-2.0-or-later package. The source is useful for historical research only. No upstream Git commit or vendor specification can currently be pinned from an authoritative public repository.
- **License and redistribution:** The implementation is GPL-2.0-or-later. The implementation license does not grant rights to the proprietary DAA technology, and the project has no independent permission record for shipping a decoder in xdzip.
- **Complete samples:** Public samples are commonly disk images or multipart archives whose provenance and version are not fixed. No redistributable corpus covering old headers, version 110, multipart ordering, zlib, LZMA, obfuscation, checksums and malformed chunks is currently admitted.
- **Clean-room feasibility:** A bounded ISO reconstruction may be technically possible after a stable wire record and legal decision. Reverse engineering from a GPL decoder alone is insufficient evidence for a product implementation.
- **Capability boundary:** A future slice could expose one verified image as a read-only virtual ISO, with exact chunk ranges, decompressed-size and output budgets. It must reject password, obfuscated or multipart variants until their own contracts and fixtures exist. No image mounting, booting, writing or execution would be allowed.
- **Conclusion:** **Blocked.** Missing public 1.0 specification, missing fixed source revision and missing complete redistributable corpus prevent a defensible admission.

## ISZ

- **Public specification:** EZB Systems, *ISZ File Format Specification*, version 1.00, revised 2006-07-03, documents the `IsZ!` signature, fixed header, optional segment definition table, chunk definition table and data blocks. Header fields include sector size, total sectors, password mode, segment size, block count, block size, pointer length, segment number and table/data offsets. Chunk flags identify zero, stored, zlib and BZip2 blocks. The specification is reproduced at [the ImgBurn forum copy](https://forum.imgburn.com/topic/14260-isz-images-supporting/#comment-115553) and indexed by [FileFormat.ArchiveTeam](http://fileformats.archiveteam.org/wiki/ISZ).
- **Reference implementations:** CDEmu `libmirage` ISZ filter, commit `39fcffe09739e1dc40bef4781646f9ffb8b7dc4e`, and Play- `IszImageStream`, commit `5bb010f27d8e2309f37b1f8eba0f74ecd546b29b`. Both are reference-analysis material only. `Helias/isz-tool` is a Python reader, but an authoritative immutable commit and license record were not retrievable in this environment.
- **License and redistribution:** Section 8 of the EZB specification says that incorporating ISZ technology into a product requires contacting EZB for a free license subject to conditions. The existence of a public text or a permissively licensed independent reader does not waive that product-license gate. CDEmu and Play- are not xdzip dependencies.
- **Complete samples:** A valid Store, zlib and BZip2 block can be generated from public tools, but a complete redistributable matrix still needs single-volume and segmented images, zero blocks, all pointer lengths, exact tail blocks, password modes, table obfuscation, truncation, offset and source-mutation cases. No such owned corpus is currently recorded in the repository.
- **Clean-room feasibility:** The unencrypted, single-volume, read-only subset is technically separable from ISO browsing. A first implementation could decode the four documented block kinds into one virtual `image.iso` output, with no mounting or boot execution. However, legal clearance, an immutable reference pin and a complete fixture set must precede coding.
- **Capability boundary:** Until those gates pass, do not register `.isz`, do not infer support from `IsZ!`, and do not expose password, AES, segmentation, writing or direct optical browsing. If later admitted, the decoder must validate header/version, pointer and segment tables, XOR/deobfuscation rules, block/data bounds, logical-size arithmetic, complete block termination, cancellation and input/work/output limits.
- **Conclusion:** **Blocked pending legal and corpus gates.** The wire format is sufficiently described for a future bounded reader, but current evidence does not authorize product implementation.

## ZPAQ

- **Public specification:** Matt Mahoney, *The ZPAQ Open Standard Format for Highly Compressed Data*, versions 2.03 and 2.06. The format stores a decompression program in ZPAQL bytecode, supports solid blocks, journaling, deduplication, SHA-1 records, encryption in later levels and multipart archives.
- **Reference implementation:** `zpaq/zpaq`, tag `6.22`, and the corresponding `libzpaq` sources. The tag is a fixed research reference for this assessment; a commit hash and license-text hash must be captured before any implementation work.
- **License and redistribution:** The reference implementation and configuration files are GPLv3. They cannot be linked into the current product under the no-third-party archive-runtime policy. The public standard supports clean-room analysis, but the decoder VM and configuration corpus still require a separate implementation and review.
- **Complete samples:** A useful corpus must cover level 1 and 2 blocks, arbitrary ZPAQL programs, journaling updates, deduplication, encrypted and multipart records, SHA-1 verification and malformed VM or block boundaries. The repository has no such owned corpus.
- **Clean-room feasibility:** Possible in theory, but the implementation is a virtual machine and resource-accounting project rather than a small container parser. Safe limits must charge bytecode interpretation, model memory, decompressed output, history and deduplication state.
- **Capability boundary:** A future first slice would be read-only, unencrypted, non-multipart and non-journaled, with a declared VM subset. No creation, update, password handling or unbounded solid extraction is implied.
- **Conclusion:** **Blocked.** Public specification exists, but the GPL reference, missing complete corpus and high-risk decompression VM prevent admission in M10.

## PEA

- **Public specification:** PeaZip, *Pack, Encrypt, Authenticate* specification, covering PEA 1.x and the still-proposed PEA 2.0 streams, volume and object controls, end markers and PCOMPRESS levels. The specification is published as [`pea_help.pdf`](https://peazip.github.io/pea_help.pdf) and summarized at [pea-file-format.html](https://peazip.github.io/pea-file-format.html). The current official page explicitly releases the PEA and PCOMPRESS specifications to the public domain.
- **Reference implementation:** `peazip/PeaZip`, commit `81b31f7e487b462980a52ac0f2e0b3cf700fe799`, including `peazip-sources/dev/pea_utils.pas` and `unit_pea.pas`. The implementation is LGPLv3 and the bundled Wolfgang Ehrhardt cryptographic units use the zlib/libpng license. The source release URL is [peazip-11.4.0.src.zip](https://github.com/peazip/PeaZip/releases/download/11.4.0/peazip-11.4.0.src.zip). The pinned source is reference-analysis material only.
- **License and redistribution:** The format specification itself has no product-license blocker, but the LGPLv3 engine and cryptographic dependencies cannot be copied or linked into the current product by default. A clean-room Rust implementation must use the public-domain format record and independently selected algorithms, while the reference binary remains test-only.
- **Complete samples:** The admitted profile has self-authored complete Store fixtures covering files, directories, UTF-8-valid names, empty and nonempty data, truncation, trailing bytes, source mutation and limits. A broader corpus must still include PEA 1.x and 2.0, multiple streams, volume boundaries, PCOMP levels, authentication failures, encrypted headers, malformed end markers and large declared sizes; those profiles remain outside the product boundary.
- **Clean-room feasibility:** A non-encrypted, single-stream, Store/Deflate reader is technically plausible, but authentication and encryption must be treated as separate security work. The format's integrity design means that structural parsing alone cannot be advertised as content verification.
- **Capability boundary:** Future admission would be read-only first, with no password, encryption, multi-volume, creation or modification. Every extracted stream would require its declared digest and terminal authentication checks before publication.
- **Conclusion:** **Admitted as a conservative read-only profile.** xdzip now admits PEA 1.x, one physical volume, UTF-8 names, PCOMPRESS0 stored files/directories and no object, stream or volume checks. Passwords, authenticated/encrypted streams, checksum algorithms, PCOMPRESS1–3, split volumes, PEA 2.0, creation and modification remain unsupported. The LGPL reference remains test-only and is not linked into the product.

## BOND

- **Public specification:** No Bandizip BOND archive specification, signature, version or compression contract was found. Recent Bandizip release notes mention vulnerabilities in “some BOND files”, but that is not a wire-format definition. Public `.bond` extension databases primarily describe Microsoft Bond schematized data, which is a different serialization format.
- **Reference implementation:** None with an authoritative archive-format identity, fixed revision and redistributable license was found.
- **License and redistribution:** Unknown. No product or decoder may be admitted while the format owner, technology rights and reference terms are unknown.
- **Complete samples:** None from a trustworthy, versioned source. A filename extension alone is not an acceptable fixture identity.
- **Clean-room feasibility:** Cannot be assessed without a signature and enough complete samples to distinguish an archive from Microsoft Bond data or another binary payload.
- **Capability boundary:** No detection, suffix registration, browsing, extraction, checking, creation, password or volume support.
- **Conclusion:** **Blocked and unconfirmed as a Bandizip archive format.** Reopen only after an authoritative Bandizip source or vendor disclosure supplies a format identity and complete corpus.

## J2J

- **Public specification:** No J2J archive specification, signature, version or developer record was found. Historical Bandizip feature lists mention J2J as extraction-only, while modern official format pages do not provide a technical definition. Other J2J search results describe unrelated job-to-job datasets or JPEG-to-JPEG tools.
- **Reference implementation:** None with a fixed revision and license that can be tied to a Bandizip archive reader.
- **License and redistribution:** Unknown. Third-party extension pages and old software lists are not a license grant.
- **Complete samples:** No trustworthy, redistributable sample matrix exists in the repository. A `.j2j` name cannot establish a container identity.
- **Clean-room feasibility:** Not assessable until the signature, record model and compression/encryption behavior are documented and a complete corpus is available.
- **Capability boundary:** No detection, suffix registration, browsing, extraction, checking, creation, password or volume support.
- **Conclusion:** **Blocked and unconfirmed as a Bandizip archive format.** Treat old third-party lists as historical leads only.

## M10 admission gates

1. Record an authoritative specification or a legally reviewed clean-room protocol. A blog, extension database or decoder source alone is not enough.
2. Pin every research source to an immutable commit or content hash and record the applicable license text.
3. Build a complete, redistributable fixture matrix for every claimed method, version, volume mode, encryption state and terminal condition. Header-only and Store-only samples cannot establish interoperability.
4. Define a bounded resource model for input reads, metadata, decompression work, memory, logical output, history, cryptographic state and cancellation. Reject malformed offsets, cycles, overflow and source mutation before publication.
5. Keep every decoder independent of Qt, mounting, boot, installer, script, registry and arbitrary process execution APIs.
6. Connect runtime, CLI, associations and user-facing documentation only after the codec, snapshot identity, exact payload behavior and failure taxonomy pass targeted automation.
7. A blocked candidate remains visible in the assessment as blocked. It must not receive a suffix, MIME entry or product capability merely because Bandizip or a third-party list names it.

The repository guard `python scripts/test-m10-source-gates.py` checks that all six bilingual sections retain the required evidence fields, each conclusion remains explicitly blocked or unconfirmed, and no blocked candidate has entered `specs/engineering/archive-formats.json`.
