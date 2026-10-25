# RAR crypto-version-zero 候选

本模块仅处理 RAR5、RAR7 共用的 crypto version 0。依据 `RAR5-CRYPTO-FREEZE-2026-10-26-v1` 纯数学交接，复用已准入 AES 0.9.3，以及本项目共享 `xdzip-crypto`。实现者没有读取 UnRAR 密码算法源码，不实现任何 RAR 压缩编码器。这不代替独立来源及许可复核。

同一 PBKDF2-HMAC-SHA256 链在 `2^L`、再加 16、再加 16 次取得 AES key、hash key、折叠 check。`L<=24` 是本实现资源政策，不是整个字段的规范最大值。任务累计限制默认 2^26 次 HMAC 和 4 GiB 哈希输入，只有 cache miss 派生。预算先整体准入，计数按实际执行累计。取消不会把尚未执行的轮次记为完成。缓存固定 16 槽，16 KiB 保守预检包括缓存及派生 scratch；密码变化清空旧秘密，IV 变化不触发另一条 KDF。密码、派生中间状态、密钥和解密块均归零，绝不进入 Debug。

密码外层准入保持 4096 UTF8 字节。之后复现原 native 平台兼容 profile，Windows 先截前 127 个 UTF16 单元，再生成包括边界孤立 surrogate 在内的底层派生字节；Unix 取前 127 个 Unicode scalar 后 UTF8 编码。该差异不是格式的统一密码规范，边界字节不作为公开 str 或文件名编码。长密码边界目前只有数学单测，未宣称独立归档互操作。

后续独立 host 边界核对明确首 U+0000 终止，因此投影前先取 NUL 前缀，终止符本身不参与派生；4096 字节准入仍针对完整原始 str。NUL 后缀或 127 原生单元之外后缀等价时复用同一缓存，前缀变化必须重新计费。`Some("")` 是本 API 显式的空候选，与 `None` 缺少密码不同；旧 `SecPassword` host 对空串或开头 NUL 标 unset，不能把数学空候选单测说成那个 host 曾完成空串 KDF。v1 冻结报告保留修正前的来源 SHA，NUL 修正另有 v2 报告。

仅当 check8 自身 SHA256 前四字节校验可信，且不是历史服务记录的全零 check，check mismatch 才证明密码验证失败。其他情况下应由容器将解密后的结构或内容错误表达为密码或完整性歧义。CRC tweak 是 HMAC 的八个 LE32 word 异或，BLAKE2sp tweak 使用完整 HMAC32；覆盖普通明文还是分片密文由容器选择。

`CbcReader` 只解密调用方授权的密文区间，跨物理片保持 CBC，不按片重新补齐，不向区间外探测。它输出全部解密块，每次 Read 至多 16 字节。`finish` 要求全部数据已读；只有调用方确认 Store 的声明内容长度或 LZ 正确终止后，才可调用 `finish_padding(15)` 显式消费剩余至多 15 字节，不要求零、不解释 PKCS7。超过一个对齐尾部的额外整块被拒是本实现明确政策。头的 true length、CRC、同物理卷完整跨度均由容器校验。typed Interrupted 保留错误对象并转成 Other，避免标准 Read combinator 自动重试吞掉；裸 OS Interrupted 可重试，其余失败均 poison。

普通低层验证包括 Python hashlib PBKDF2/HMAC 三检查点、短 Unicode 数学向量、NIST AES256 CBC、跨非对齐片段、六个固定独立归档密文与 .NET AES 明文 SHA256、可信/不可信 check、累计预算、cache miss/eviction、非零 padding、截断、取消和带类型源错误。真实加密 split 归档、长非 BMP 密码归档、所有 legacy 加密族仍不在本批低层结论内。宿主提取、旧目标保护及 GUI 交互须另记真实服务链证据。
