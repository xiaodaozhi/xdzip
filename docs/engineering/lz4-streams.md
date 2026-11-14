# LZ4 stream contract

The standard-frame profile follows the author's [Frame format 1.6.5](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Frame_format.md), [Block format, 2026-10-26](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Block_format.md), and [XXH32 specification](https://github.com/Cyan4973/xxHash/blob/v0.8.3/doc/xxhash_spec.md). Product algorithms are independent Rust implementations. Reference tools and generated execution evidence remain under local `build/`; only owned reproducible input patterns are versioned. See [single-file streams](single-file-streams.md) for host authority and publication.

## Admission and framing

Standard version-1 frames validate reserved bits, BD values 4–7, header XXH32, optional content size, every declared block/content XXH32, and the end marker. Block data lengths cannot exceed the declared 64 KiB–4 MiB maximum. Independent blocks reset history; linked blocks retain only the preceding 64 KiB. Raw blocks and zero-length raw blocks follow the same checks and accounting. External dictionary profiles and legacy frames are unsupported. Dictionary declarations never authorize reading another file.

Concatenated standard frames and intervening skippable frames are bounded by physical input, task budgets, cancellation and a 100,000-frame/block policy. A successful standalone file contains at least one standard data frame; arbitrary tails and skippable-only inputs do not become successful files. Discovery can inspect at most 8 MiB of actual compressed input before producing a 512-byte content prefix. Strong identification or identified TAR damage cannot downgrade to another format on failure. Full scanning validates all frames before publishing the observed size and virtual row.

## Block processing and creation

Lengths use checked arithmetic. Matches require a nonzero in-history offset, copy overlapping bytes in increasing order, and stay within the declared output bound. The last sequence is literal-only; its unused match nibble is ignored. A stream containing matches retains at least five final literals and starts its last match at least twelve decoded bytes before the block end. Reads produce bounded chunks and do not resume after failure. History transfers only after exact validated block EOF. Host I/O, cancellation, quota failures and unsupported profiles retain distinct error categories.

An admitted writer must emit real LZ4 matches, use raw blocks only when compression does not reduce size, and include block and content checksums. Offered levels select bounded matching work, with independently decoded multi-block content required at each level. Frame and match buffers are admitted before allocation; creation still uses exact frozen input sizes, host staging, complete verification and final source/output checks. Ordinary LZ4 has one regular input file. A TAR composition may be declared only after the shared TAR parser/writer and corresponding automatic checks cover it. Neither profile gains rewriting, encryption or volumes from a suffix registration.

Checking uses `format_declared_checks`, because input frames may omit content checksums. Do not describe a checksum-free frame as content-verified. Resource limits and actual platform acceptance remain separate from format validity and formal release approval.
