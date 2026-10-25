# xdzip-crypto

格式无关的固定内存哈希工具，供 Rust 候选后端复用。此 crate 不接收路径，不派生归档密钥，不解释密码正确性，不决定文件发布。

`SecretSha256` 在已准入的 SHA-2 0.10.9 压缩原语上管理可归零的增量状态，来自本项目此前原创的 7z `ErasedHash`。`HmacSha256` 依据 [RFC 2104 第 2 节](https://www.rfc-editor.org/rfc/rfc2104.html#section-2) 独立组织 pad、长密钥归一化及内外消息。`Blake2s` 与 `Blake2sp` 依据 [RFC 7693 正文第 2、3 节](https://www.rfc-editor.org/rfc/rfc7693.html#section-2) 的数学定义和 [BLAKE2 规范第 2.8、2.10、2.11 节](https://www.blake2.net/blake2.pdf) 的树参数独立实现。固定 IV、轮置换属于规范常量。实现者未读取 RFC 附录的参考 C 实现或第三方 BLAKE2 算法源码。

公开接口均为 `new`、`update(&[u8])`、消耗实例的 `finish`。SHA/HMAC 输出为 `Zeroizing<[u8;32]>`；BLAKE 输出为普通 `[u8;32]`，调用方若用于秘密派生结果须自行归零。SHA/HMAC 状态支持显式克隆，副本独立归零。BLAKE2sp 固定八叶、每块 64 字节轮转，不受调用方 update 分块影响，不引入线程或按输入增长的分配。调用方须在输入准入处限制 SHA 消息小于 2^64 bit、BLAKE 消息小于 2^64 byte，并负责工作预算和取消。

普通测试使用独立 Python hashlib/hmac 结果，覆盖 SHA 填充边界、HMAC 64 字节密钥边界、逐字节输入和 BLAKE2sp 0/1/63/64/65/511/512/513/1024/65537 字节。BLAKE2sp 的四个长度还由冻结官方 RAR Store 档案的已知内容哈希交叉确认。独立工具仅用于测试，没有进入产品依赖。具体运行日志与来源哈希保留于 build 工件；这些低层向量不代表宿主服务链已通过。
