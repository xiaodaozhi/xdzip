# Owned LHA reference inputs

The payloads and `Reference.java` are project-authored and distributed under the repository's Apache-2.0 license. They contain no imported upstream test archives or user data. Compressed bytes are generated through public reference APIs, with additional independently authored initial-history streams. Tool source and binaries remain outside Git under `build/toolchain/` and are not runtime dependencies.

## Fixed references

- jLHA 0.0.11, original Michel Ishizuka BSD-2-Clause license, [source commit](https://github.com/umjammer/jlha/tree/0a6348d96e9bbb1ac5ee8b3f6ff1c2c75b17d0d9) `0a6348d96e9bbb1ac5ee8b3f6ff1c2c75b17d0d9`. The downloaded source zip SHA-256 is `ef9feb1c59f55aff69b00c0d69dee5edaefb45e86950a99a6728beb35e70ff3a`. Compile only its main sources/resources and the owned API harness, without importing upstream test inputs or Maven dependencies.
- Temurin OpenJDK 17.0.21+9, [official Windows x64 release](https://github.com/adoptium/temurin17-binaries/releases/tag/jdk-17.0.21%2B9). Zip SHA-256 `98518ecf554465057ee5509462631deeee374bc368c12a411d225cd29381f84b`; every extracted regular file is compared with the verified zip before compilation and execution. This is a portable test tool, without installation or system configuration changes.
- Lhasa 0.6.0, ISC, [author release](https://github.com/fragglet/lhasa/releases/tag/v0.6.0). Windows zip SHA-256 `5925e6a7b615d784e381e18240a1537d55a42cddad91d21e13c604a40974fe28`, executable SHA-256 `93cf8100eb520e49708c5076694253d52caf241d6013be4e37c11ec8e89bbe85`. Extract only owned `hello.bin` members inside an owned temporary child directory. Windows stdout can translate line endings, so binary file contents, rather than redirected text output, are compared.
- 7-Zip 26.06, [author download](https://www.7-zip.org/download.html). The existing verified full reference executable SHA-256 is `dc0493429fe28136fcb177a0e77c9875cbb3b0bff5f2f8eeba4df302e98cef76`, and its matching `7z.dll` is `500db9eb17f3c21b729ba41dd02ecfe1823da05028c7e4ba129084cc74cc364e`. Compare complete binary stdout only for supported methods.

The generator verifies these identities, recompiles the public API harness and compares reference output with authored payload bytes. It does not install reference code or add product dependencies. The [wire contract](../../../docs/engineering/lha-containers.md) records primary format sources and source-observation limits.

## Input definitions

`{method}-h{level}.lzh` combines methods `lh0`, `lh1`, `lh2`, `lh3`, `lh4`, `lh5`, `lh6`, `lh7`, `lzs`, `lz4`, `lz5` with header levels 0, 1 and 2. Each contains `hello.bin`, the bytes `abracadabra abracadabra\n` repeated six times, timestamp 1700000000 UTC, and a declared payload CRC-16. The harness forces the named raw encoder without a Store fallback.

For the nine compressed methods, `empty`, `tail1`, `spaces`, `random` and `long` use respectively no bytes, `A`, 4096 spaces, 65000 deterministic xorshift32 high bytes, and the phrase repeated 6 × 4096 times. Xorshift starts at `0x6d2b79f5` and applies shifts 13 left, 17 right and 5 left in 32-bit arithmetic. All these cases use header level 2. Empty compressed members have zero packed bytes; they do not establish acceptance of a coded empty block.

Three wire-authored level-0 cases test initial history directly. `lh5-initial.lzh` contains one length-three distance-one command and produces three spaces. `lzs-initial.lzh` references initial absolute position zero for seventeen spaces. `lz5-initial.lzh` references position zero for eighteen bytes, thirteen zeroes and five ones. Their headers and CRCs are constructed independently of Rust.

jLHA generates and exactly rereads the 78 ordinary cases and exactly reads the LZ5 initial case. Its zero-filled LH5/LZS defaults disagree with the original LHa space-filled profile, so those two cases are explicitly recorded as reference disagreements. Lhasa exactly reads 65 cases and does not support LH2/LH3. 7-Zip exactly reads 38 ordinary cases using LH0/LH4/LH5/LH6/LH7/LZ4 and rejects the other methods and the initial LH5 profile. Unsupported exit codes and initial-history disagreements are never counted as successful decoding.

These are reference capabilities, separate from Rust method admission or product support. The decoded payload and CRC must match; merely listing a method is insufficient. No claim is made for passwords, volumes, special objects, other header levels or historical repair variants.

## Reproduction and integrity

Run `python scripts/generate-lha-fixtures.py --check` with the verified references under the documented `build/toolchain/references/` subdirectories. Without `--check`, it regenerates the owned archives and compile-time fixture expectations. Actual reference results and temporary extraction files go under `build/review/format-expansion/`; the source tree contains only durable owned inputs, the harness and expectations.

For all 81 archive files sorted by UTF-8 filename, SHA-256 of the concatenation of each filename, one zero byte and its complete archive bytes is `8866cb95bfac04a90895c5e426dc8689bba7a5e508d8d294803a7f79849ee20b`. The generator's check mode compares every archive and the exact Rust fixture expectations. Rust primitive tests independently compare complete decoded bytes and declared CRCs; self-roundtrips are not the independent reference.
