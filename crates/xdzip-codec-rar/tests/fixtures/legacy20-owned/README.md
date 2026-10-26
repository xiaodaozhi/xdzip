# RAR20 自有作者回归数据

三个原始归档由固定官方 RAR 2.90 命令行程序压缩本项目的数学输入生成，未修改归档头。普通、固实与强制 Audio 固实各包含相同的三个成员；普通测试逐字节比较全部九份输出，并从全新 reader 选择每个目标，包含固实前驱重放。每个归档不超过 32 KiB，输入原文合计每档 197,187 字节，未提交大型明文或作者程序。

[sources.json](sources.json) 固定官方来源、工具摘要、归档摘要、命令及原文数学。生成方法见 [可选脚本](../../generate-independent-legacy20-fixtures.py)，须显式传入已核验的官方作者程序，并将输出指定在仓库 `build/` 中。官方安装器只提取 `Rar.exe`，不安装；旧安装器没有 Authenticode 签名，来源由官方 HTTPS URL 和固定 SHA-256 记录。测试本身不调用作者或网络。

这些归档的实际 UnpVer 都是 20，后两个固实成员带真实 solid 标志。它们不代表独立版本 26、大于 2 GiB 实际文件、所有 Audio 通道数或每种旧尾表切换已经验证。自有数学内容及测试归档遵循仓库许可；官方作者与对照工具保留各自许可，未纳入仓库。

English: Three unchanged archives were authored by the pinned official RAR2.90 executable from project-owned mathematical bytes. Ordinary tests compare every byte of all nine members and open a fresh payload reader per target, including solid-prefix replay. Each archive is at most32KiB; each set contains197,187 original bytes. No executable, large plaintext or network dependency is vendored. The optional recipe requires an explicit hash-checked writer and a repository build-directory output. All real unpack versions are20. Explicit version26, actual files beyond2GiB, all Audio channel counts and all tail-table transitions remain separate coverage requirements. Source and tool licenses are not replaced by the repository's test-data license.
