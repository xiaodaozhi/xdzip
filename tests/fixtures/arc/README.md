# Owned ARC inputs

All `HELLO.BIN` contents and wire-authored inputs are project-owned, redistributable under the repository license. No third-party archive, source file or executable is included here. `known.rs` and `scripts/generate-arc-fixtures.py` independently express complete known bytes.

The 69 archives cover methods 1–9, old Store framing, complete byte values, long runs, DLE escaping, fixed dictionary fill and dynamic width growth/reset. Six ordinary patterns request each method, except empty method 7, plus sixteen authored CLEAR profiles for methods 8 and 9 at each eight-code group position. Filenames begin with the requested method; native encoder fallback is retained in the actual header and generated descriptor.

`generate-arc-fixtures.py` expresses RLE, explicit squeeze trees, the two fixed dictionary wire assignments and grouped CLEAR fields. Methods 8/9 use unchanged original ARC encoding routines, with forced LZW for incompressible known random content so width growth and CLEAR are tested. This is a test adapter call, not a claim that ARC's automatic selector chooses compression for that content.

The native reference uses original ARC 5.21q, revision `70f0b93516d0ee222db5cf1241aad0db1b721c0d`, source ZIP SHA-256 `d8722a28ed9612f7712efed3836f6fac3253643b7b54f06d15ba759cae849985`, 101449 bytes. `author-reference/reference.c` and `compat.h` are project-owned wrappers, not imported algorithms. The unchanged external files are `arclzw.c`, `arcusq.c`, `arcsq.c`, `arcunp.c`, `arcpack.c`, `arccode.c` and `arcdata.c`, with their own headers and GPL notices. Build with GCC 13.2.0, `-std=gnu89 -fcommon -O2 -DSYSV=1 -Wl,--no-insert-timestamp`, the wrapper compatibility header and an include directory containing the external `source/` folder. The pinned Windows reference executable SHA-256 is `d5d2e5b47154da2766589e8f8007eb0bc8ae320cfae69d2bb6b226c1023171b4`.

The second reference is Russell Marks's nomarch 1.4, original package SHA-256 `fe20da34e0d3ba0cf6388701f44ac22224cf65130ddbc5fcbc27bc4949a6e1ad`, 22411 bytes, checked against the Debian maintainer source descriptor. GCC 13.2.0 with `-std=gnu89 -O2` builds the unmodified `main.c`, `readrle.c`, `readhuff.c` and `readlzw.c`; executable SHA-256 `1382786fc49ed66dffc649a7418f020a121bbeea913ecb56c4d6d3421e397706`.

Every archive is compared byte for byte with at least one independent decoder. Native ARC does not represent an empty first fixed-dictionary code, so empty methods 5/6 use nomarch. Nomarch explicitly does not support method 7, so its five nonempty inputs use the original ARC decoder; no empty method-7 input is claimed. All remaining cases are completely compared through both references. Reference code/binaries and generated logs are confined to build/toolchain and build/review.

The input set SHA-256 is `6e4cda347196139b25849237493d0c392dcad72a2cda1deca321941e55a6e547`, computed over ordered UTF-8 lines `filename sha256\n`. The generator pins it, emits per-case hashes in local JSON, verifies actual methods/CRC and supports `--check` for byte-exact regeneration.

```powershell
python tests/fixtures/arc/author-reference/build.py --cc build/toolchain/strawberry-perl-5.42.3.1/c/bin/gcc.exe
python scripts/generate-arc-fixtures.py --reference build/toolchain/references/arc-5.21q/reference.exe --nomarch build/toolchain/references/nomarch-1.4/nomarch.exe --check
```

Headers use known `HELLO.BIN`, a fixed DOS date/time, declared lengths and required CRC-16; terminators are exact. No passwords, proprietary input, user files, scripts, external paths or execution instructions are carried by these archives. See the [stream contract](../../../docs/engineering/arc-streams.md) for parsing, budgets and the boundary between primitive and product evidence.
