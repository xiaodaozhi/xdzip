# Qt Public Suffix List 对应输入

`public_suffix_list.dat` 是 Qt 6.8.2 的 qtbase 源码中 `psl-data` 记录所指向的固定版本，来自 [Public Suffix List](https://github.com/publicsuffix/list) 提交 `903a83ff7bfc3148e3692e09396f9f3bdc9462ef`，Git blob 为 `6ba26da001a9a094a6dfab0ae7cca5e31f7e8e2e`。`PSL-LICENSE.txt` 保留相同 Qt 源码归档中的 MPL-2.0 原文。两个文件均保留原始字节；具体 SHA-256 见 `scripts/qt-psl-source.json`。

源码交付会从校验过的 Qt 源码归档读取 `psl-make-dafsa`，用此列表分别生成 C++ 数据和二进制测试数据，均与该归档中对应文件逐字节核对后才收集。此验证说明生成输入与归档内容对应，不证明整个 Qt SDK 的构建等价性，也不代替候选发行的许可复核。

English: The list is the fixed `psl-data` revision identified by Qt 6.8.2. Its Git blob and SHA-256, together with the exact MPL-2.0 text from the Qt archive, are pinned in `scripts/qt-psl-source.json`. Source delivery regenerates both Qt's C++ data and binary test data with the generator from the verified archive and compares the bytes. This does not establish full SDK build equivalence or approve a release's license compliance.
