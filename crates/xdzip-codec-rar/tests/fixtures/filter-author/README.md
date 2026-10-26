# Independent RAR filter input fixtures

The three small archives in this directory contain original finite test data,
compressed by fixed official RAR executables. The project implements no RAR
compressor. The generator creates ISA-like bytes as data and never executes them.
No corresponding third-party filter decoder or encoder implementation was read.

- `rar29-e8e9-auto.rar`: original 2048-byte branch recipe, official RAR5.91
  automatic x86 selection. Complete original equality and one successful E8E9
  transform are required. These branch-like input bytes are never executed.
- `rar29-itanium.rar`: original 524288-byte bundle recipe, official RAR5.91.
  Its decoded original SHA is fixed and eight successful Itanium transforms
  are required. Official6.00 change notes explain that Itanium creation was
  removed in that version; passing `-mci+` to a newer author proves nothing.
- `modern-delta4.rar`: original 524288-byte byte-channel recipe, official RAR7.23.
  Complete codec and direct-decoder output must match the recipe, and eight
  Delta descriptors must accompany complete successful transforms/output.
- `arm-sources.json`: fixed public ARM archive and an independently published
  90808-byte original input from a second data repository. Its GPL-3.0-or-later
  original notice is explicit. Those binary inputs are **not vendored** here;
  the bounded preparer places them and the exact notice only under `build/`.
  Actual complete output equality accompanies four ARM and two Delta descriptors.

`sources.json` records exact archive identities, original recipes, official author
hashes, commands and coverage limits. Recreating the fixed-mtime inputs in the
same author timezone reproduced the archive bytes. Other timezone/attribute
settings can change RAR4 metadata; every independently generated archive still
requires exact known bytes and actual candidate coverage.

From the repository root, use fixed official test tools:

```powershell
python crates/xdzip-codec-rar/tests/generate-independent-filter-fixtures.py --author-591 <fixed-Rar-5.91.exe> --author-723 <fixed-Rar-7.23.exe> --output build/filter-author
$env:XDZIP_RAR_FILTER_AUTHOR_FIXTURES = (Resolve-Path build/filter-author)
cargo test -p xdzip-codec-rar independent_author -- --ignored --nocapture --test-threads=1

python crates/xdzip-codec-rar/tests/prepare-public-filter-fixtures.py --output build/filter-public-arm
$env:XDZIP_RAR_FILTER_AUTHOR_FIXTURES = (Resolve-Path build/filter-public-arm)
cargo test -p xdzip-codec-rar modern_independent_author -- --ignored --nocapture --test-threads=1
```

The three fixed original-recipe tests also run without external tools in the normal
crate suite (`fixed_official_`). The optional manifest reader caps the manifest
at64KiB, each file at8MiB and retained input bytes at64MiB. Preparers require empty
build subdirectories, preserve prior evidence and enforce fixed source hashes.
These codec/component checks are separate from worker/service/GUI acceptance.

The automatic E8E9 case establishes one finite known-input success, alongside
Itanium, Delta and the separately prepared public ARM case. Author switches alone
never establish coverage. No broad instruction, encrypted, solid, split or
platform coverage is inferred from these four representative inputs. This does
not validate the product worker, service or GUI.

中文：三份小归档由固定官方作者压缩自有有限原文，强制检查完整原文及实际 E8E9、Itanium、Delta 成功。公共 ARM 输入另有独立原文与固定 SHA，GPL 二进制只在 build 下载并保留原始许可，仓库仅保存来源清单与准备脚本。参数不提升覆盖结论，这些检查不代替 worker、服务链或 GUI 验收。
