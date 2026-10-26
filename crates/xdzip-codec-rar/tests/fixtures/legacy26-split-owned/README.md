# 自有 RAR26 十七卷夹具

这里仅保存官方 RAR 2.90 对既有项目自有稀疏数学原文生成的未修改压缩卷。顺序是 `split26.rar`、`split26.r00` 至 `split26.r15`，共 1,063,842 字节，一个逻辑文件为 2,147,483,649 字节。真实版本为 26、压缩强度为 1，LARGE 与各片 split flags 均保留。逐卷身份、作者参数和工具摘要见 sources.json。

原文与相邻 legacy26-owned 单卷夹具相同：前 4,096 字节为 `(31*i + 7*(i>>5) + 13) mod 256`，末 4,096 字节为 `(i*i + 5*i + (i>>3)) mod 256`，两者间全零。完整 SHA-256 为 `d1ea9e8a1dd77c140253106db1be9331430622ee141e57a1ccdf31bab1d5a13a`。作者阶段只读复用了既有稀疏数学源，组件测试不生成大明文，夹具目录不保存原文。

作者原样保存的包内名称是一个以 `GitHub/` 开头的相对路径；它只是归档元数据，不授予主机源路径或输出权限。显式 release 组件测试复用有界 sink，完整累计大小、SHA 与 CRC，检查实际 WORK、辅助工作及预付信用归零。两份 2 GiB 加 1 字节夹具代表对同一原文的两次解码，不是一份 4 GiB 文件或两份不同原文。

只覆盖一个无加密、非固实的版本26压缩分卷正例，不扩展为其他分卷、固实、Audio 或 Store 混排已经验证。程序、安装器、稀疏原文和执行记录均不进入此目录。

English: These seventeen unchanged official RAR 2.90 volumes contain the same owned 2,147,483,649-byte mathematical original as the neighboring single-volume fixture. Authorized order is `.rar`, `.r00` through `.r15`. Actual version26/method1, LARGE and split flags are pinned in sources.json. The first and last 4,096-byte formulas above surround a zero middle. Authoring reused the existing sparse mathematical source read-only; component tests create no large plaintext, and the fixture directory contains no plaintext. The archived `GitHub/` relative path is metadata, never host filesystem authority. The explicit release test shares the bounded sink and verifies the complete count, SHA, CRC, actual work and zero remaining prepaid credit. Two decodes of this same original are neither a single 4 GiB file nor two distinct originals. This one unencrypted nonsolid example does not establish other volume, solid, Audio or Store-mixing combinations; author tools and execution evidence remain outside the fixture directory.
