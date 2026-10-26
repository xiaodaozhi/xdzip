# 自有 RAR20 七卷夹具

这里仅保存官方 RAR 2.90 对项目自有数学原文生成的未修改压缩卷。顺序是 `split20.rar`、`split20.r00` 至 `split20.r05`，共 200,991 字节，合并为一个 589,826 字节的 `known.bin`。真实版本为 20、压缩强度为 3，无加密、非固实。逐卷大小、SHA-256、原始 split flags、工具身份与作者参数见 sources.json。

原文按以下公式生成，不保存为夹具：

1. `A[i] = (31*i + 7*(i>>5) + 13) mod 256`，`0 <= i < 131071`。
2. 从 `k=0` 开始连接 `SHA256(ASCII("xdzip-rar20-store-middle-v1") || LE32(k))`，取前 196,613 字节为 B。
3. 原文为 `A || B[:98317] || A || B[98317:] || A`，完整 SHA-256 为 `b63fd1d316ba1dcf8a351d0ec895bcf066495cc1ebf1eae1aec7a823c4eb5567`。

普通组件测试逐字节比较完整原文、校验每卷身份和物理片信息，并保留公开 packed_size 为首片 32,707 字节的语义。仅这一个压缩分卷正例，不代表 Store 混排、固实、Audio 或加密分卷覆盖。作者程序、安装器、原文和执行证据均不进入此目录；历史作者的时间戳不保证可重现为相同归档字节。

English: These seven unchanged official RAR 2.90 volumes contain only project-owned mathematical data. Concatenate the authorized physical streams in `.rar`, `.r00` through `.r05` order. The single 589,826-byte file uses actual version20/method3, unencrypted and nonsolid. The formula above fully defines the original bytes. Ordinary component tests compare every byte, pin all volume identities and physical flags, and require public packed_size to retain the first fragment's 32,707 bytes. This one positive case does not cover Store mixing, solid, Audio or encrypted volumes. No author program, installer, plaintext or execution evidence is vendored, and timestamped archive bytes are pinned rather than claimed reproducible.
