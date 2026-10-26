# RAR 有限线格式映射来源与原始通知

本目录保留 RAR 旧格式有限输入输出映射的参考来源与原始许可，不包含官方 UnRAR 引擎，也不将该引擎纳入产品构建或运行依赖。

- RAR 13/15 前缀和 Short 映射的来源、数学转换与数据摘要见 [rar15-codebooks-sources.json](../../crates/xdzip-codec-rar/data/rar15-codebooks-sources.json)。
- 旧 RAR20 密码有限置换的来源及适用范围见 [历史密码说明](../../crates/xdzip-codec-rar/src/legacy_crypto/README.md)。
- 固定参考的上游内部版本为 UnRAR 7.30 beta1，下载归档文件名为 `unrarsrc-7.3.1.tar.gz`。文件名中的 `7.3.1` 不解释为上游人类语义发行版本。归档与许可正文摘要见 [sources.json](sources.json)，原始 [license.txt](UnRAR-7.3.1-license.txt) 按字节保留，通知文件名沿用下载归档标识。

有限映射的事实交接、原创 Rust 表达和官方参考程序是不同材料。这些映射的参考来源不因独立实现、CSV 表达、空的官方产品源码清单或本项目 Apache-2.0 许可证而重新许可。原文保留不等于已裁定条款适用范围，不构成严格洁净室、法律风险清零或发行批准。逐候选仍须结合实际交付审查来源与适用义务，见[许可证契约](../../docs/engineering/licensing.md)和[RAR 来源](../../docs/engineering/rar-format-sources.md)。

打包将本目录的说明、来源记录和精确原文放入 `notices/rar-wire-mappings/`，对应源码交付另保留相同 `rar-wire-mappings/` 材料。交付后的说明链接指向对应源码树，在安装目录中不承诺存在这些相对路径。

English: This directory preserves the pinned provenance and original notice for finite RAR13/15 wire code mappings and the historical RAR20 cipher permutation. It contains no official UnRAR engine and adds none to product builds or runtime dependencies. The upstream internal version is UnRAR 7.30 beta1; the downloaded archive is named unrarsrc-7.3.1.tar.gz. The filename's 7.3.1 is an archive identifier, not a separately inferred human release version. Its complete license is retained byte for byte, with the archive and notice hashes in sources.json. Independent fact handoffs, CSV data and original Rust representations do not relicense the reference-derived mappings as project-original Apache-2.0 material. Preserving the notice does not decide the applicable legal obligations, establish strict clean-room development or approve distribution. Candidate-specific provenance and redistribution review remain required. Packages retain this directory under notices/rar-wire-mappings; corresponding-source delivery also includes the same bounded notice material separately.
