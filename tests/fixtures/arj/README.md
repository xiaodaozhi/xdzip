# Owned ARJ reference inputs

Plaintexts and generator are project-authored. No upstream archive or user file is imported. Reference source and programs remain under build/toolchain, with their original terms, outside product dependency closures. See the [ARJ contract](../../../docs/engineering/arj-streams.md) for source-observation and admission boundaries.

`scripts/generate-arj-fixtures.py` verifies these fixed inputs before use:

- [Open-source ARJ 3.10.22 source](https://arj.sourceforge.net/files/arj-3.10.22.tar.gz), 431467 bytes, SHA-256 `589e4c9bccc8669e7b6d8d6fcd64e01f6a2c21fe10aad56a83304ecc3b96a7db`.
- [Win32 ARJ 3.10 package](https://arj.sourceforge.net/files/arjw_310.exe), 225396 bytes, SHA-256 `e5cc5d8f82c5f08246e1c0a8c5be77e5f796c2dd30f987e6bbb27a2c1df1e371`. Only arj.exe and license/documentation are extracted as references. The executable is 270443 bytes, SHA-256 `00ac8169236b8f9910c661e6d8b82c2f2e5800338a9c7775d691e622f3521ac9`; banner ARJ32 3.10, 23 June 2005. The package ships GNU GPL version 2 text. No installer, registration, rearranger or display tool is executed.
- Official 7-Zip 26.06 x64 executable, SHA-256 `dc0493429fe28136fcb177a0e77c9875cbb3b0bff5f2f8eeba4df302e98cef76`, independently extracts every complete output.

Forty archives request methods 0–4 over eight patterns: empty; the ASCII phrase `abracadabra abracadabra\n` repeated 128 times; 65536 spaces; byte values 0–255 repeated to 65536 bytes; 32768 deterministic random bytes; a 131072-byte cycle of all 256 values followed by 128 phrases; and two window cases. The 21k case takes 24000 deterministic random bytes followed by its slice 3000:13000. The 14k case takes 15800 deterministic random bytes followed by its slice 1000:13000. Random bytes use 32-bit xorshift with seed `0x12345678`, shifts 13/17/5, masking left shifts to 32 bits and emitting the low byte after each iteration.

All inputs use HELLO.BIN and RESULT.ARJ. A fresh owned directory and cleared ARJ/UNARJ environment prevent inherited switches or configuration. Every process is bounded. Original packed bytes remain unchanged. Only unencrypted header timestamps, the unused time-derived password-modifier byte and their header CRCs are normalized, with fixed DOS time 2023-11-14 22:13:20. Main headers use 34 initial bytes, file headers 46, with no extended blocks. Python independently verifies header and content CRC-32, exact ranges and terminal framing. Both unmodified native ARJ and 7-Zip reproduce the entire plaintext after normalization.

Actual methods are 17 Store, six method 1, six method 2, six method 3 and five method 4. Empty and some incompressible inputs fall back to Store; generated descriptors preserve requested and actual methods. Those fallbacks never count as compressed-method evidence. Native empty inputs create zero-length Store files.

Run `python scripts/generate-arj-fixtures.py --check` with the fixed references installed in the paths used by the script. It rebuilds the inputs independently, compares every archive byte and generated Rust descriptor, and verifies the set identity. For all 40 archives sorted by UTF-8 filename, SHA-256 over each filename, zero byte and complete archive is `9797b39865134962b6caa1a4ae0c2a546e108410f7f589ec14e5f54aa85ef93d`. Generated reports, original tool outputs and diagnostics remain under build/review/format-expansion.

## Authored metadata profiles

Container tests derive bounded metadata profiles from the owned method-1 phrase or empty Store archive, update base-header CRCs and preserve packed content. Canonical complete-input SHA-256 values bind those test constructions to independently read inputs:

| Profile | Input SHA-256 |
| --- | --- |
| Main first width 30, file width 30 | `27287d9d989f6e4b1b9b805febce261fa10186508d86263f6f5e3a5826111b38` |
| Main first width 30, file width 34 | `c64f8b7c6690f8d2939412a7b8e381e4b3e04799629f34080b2f098526fbfc75` |
| Main first width 30, file width 46 | `ea3d4cda03e51d37d91697e2d5bc69847316bb531946d1d01d326d220f41ecf0` |
| CRC-valid opaque Z attribute block | `d3dbabebd5f271c14309e88bd2fefac74a280a811c8034b66a689254fcc4db66` |
| Legacy text type 1 | `0637d05e6d92b7c0efdf5c954f3928c4977fd46cbbb7ff91064edf5566c34e63` |
| Empty directory type 3 with DOS directory attribute | `883ad27bdbb60c959a15fa1b2f27b6f95b1b2f5c4e3f0d4a9ce0010494a41e8f` |

These are authored metadata inputs, separately from original native encoder output. Both references reproduce the binary profiles and directory semantics. The text profile has 3072 bytes under its declared size and CRC, reproduced exactly by 7-Zip. Native Win32 ARJ converts the 128 LF bytes to CRLF and writes 3200 bytes; that representation difference is not counted as an exact byte match. The product profile keeps the archive's CRC-covered bytes. `known.rs` supplies shared test-only plaintext construction, independently of the ARJ decoder and without another expected-byte implementation in each crate.
