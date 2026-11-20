# AES Crypt 只读封装

[English](../../engineering/aescrypt-envelopes.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/engineering/aescrypt-envelopes.md) | [日本語](../../ja-JP/engineering/aescrypt-envelopes.md) | [한국어](../../ko-KR/engineering/aescrypt-envelopes.md) | [Français](../../fr-FR/engineering/aescrypt-envelopes.md) | [Deutsch](../../de-DE/engineering/aescrypt-envelopes.md) | [Español](../../es-ES/engineering/aescrypt-envelopes.md) | [Português (Brasil)](../../pt-BR/engineering/aescrypt-envelopes.md) | [Русский](../../ru-RU/engineering/aescrypt-envelopes.md)

独立 Rust 后端 `xdzip-codec-aescrypt` 读取 AES Crypt 流版本 0、1、2、3，应用程序版本与流版本分开。每个封装呈现一个普通虚拟文件，不提供创建、修改、分卷或外部扩展获取能力。内容仅加密而不压缩，因此条目方法显示 Store，同时标记已加密。压缩大小为完整物理封装长度。识别依据内容，不依赖 `.aes` 后缀。

## 准入与认证

四个版本都必须在打开时提供密码，包括空文件。发布精确大小的元数据前，必须认证整个密文。不可变归档会话通过现有秘密槽保留有界的打开密码，供检查、预览、提取使用，不另设逐条目密码模型。任务读取器再次完整认证封装后才准入。导出过程对实际解密的同一批密文计算认证，最终认证码与大小检查成功后才返回成功。主机只有在格式解码和源检查成功后才提交暂存文件，未验证的数据块不授予最终发布权限。

0–2 版使用指定的 8192 轮 SHA-256 ACKDF，密码编码为 UTF-16LE。0 版直接派生数据密钥，无法区分错误密码与密文损坏，返回 `password_or_integrity`。1–2 版先对封装的会话 IV/密钥验证 HMAC-SHA-256，再进行 CBC 解密，验证失败返回 `wrong_password`，该错误也可能来自封装密钥损坏。3 版采用 UTF-8 密码与 PBKDF2-HMAC-SHA-512，对封装会话字节及版本字节 `03` 认证，数据采用 AES-256-CBC 与 PKCS#7 填充。元数据准入和导出都先完成密文认证，再检查填充，不能以填充作为密码验证器。认证码比较使用已准入的固定宽度常量时间原语。

所有版本都验证密文的 HMAC-SHA-256。但是 0–2 版把最后一块的明文长度模 16 放在认证范围之外，修改该字段可能改变报告及导出的长度，却不使认证码失效。2–3 版的扩展字段同样未加密、未认证。因此 `format_declared_checks` 仅表示格式声明的检查通过，不表示全部封装字段、发送者身份或来源已认证。3 版的加密填充可以认证明文长度，旧版检查成功不能描述为明文长度获得完整认证。

## 名称、限制与失败

虚拟名称仅从主机授权的源文件基名移除大小写不敏感的 `.aes` 得到，不安全、空、点或双点名称回退为 `content`。未认证的 `FILENAME`、`URI`、`CREATED_BY` 及空容器扩展值不授予路径、网络、进程或执行权限，不恢复时间和执行位。首条成员恰好名为 `AES` 的普通 TAR 通过有界结构消歧保留为 TAR，扩展内含 TAR 标记的合法加密封装仍为 AES Crypt。进入 AES Crypt 后，认证失败不会回退为 TAR、ZIP 或其他格式。

1–3 版要求保留字节为零，非空扩展记录必须包含以 NUL 分隔的标识。准入限制为最多 256 条扩展、1 MiB 头部，3 版迭代数为 1 至 2,000,000。零迭代、结构损坏、截断或非整块密文在执行 KDF 前拒绝。未知版本返回不支持。旧版长度模值须为 0–15，空密文只能声明零；3 版至少有一个填充块。不提供损坏修复。

读取器分别限制累计实际输入、实际哈希压缩块工作、实际 AES 块工作和交付明文，每项硬上限为 24 GiB。计算包括元数据认证、KDF 和后续重复读取，失败不退还预算。调用者的元数据输入及名称限制只能收紧准入。哈希计算包含 KDF、HMAC 内外垫与结束块，CBC 计算包含全部实际处理块及封装会话材料。附加准入和 KDF 工作在交付载荷前计入主机接收端。正常解码继续受主机源、输出、任务限制约束，明文单文件上限 8 GiB、单任务上限 20 GiB。通用格式接口最多接收 4096 个 UTF-8 密码字节，产品共享密码入口仍为 1024 字节。迭代上限是应用资源策略，不是格式定义的最大值。

KDF、每次读取和有界 CBC 批次都检查取消。密码派生哈希、中间 KDF 块、密钥、IV 链和明文缓冲使用可擦除状态，已准入 AES 原语启用清零功能。格式读取器保留派生密钥，不再复制保存明文密码。协作式取消不能中断阻塞的操作系统读取。源、取消、配额、输出错误保留类型，仅重试无附加错误的中断读取。读取器出错后终止使用。主机物理身份、大小和时间检查是尽力检测源变化，不承诺原子内容快照。

## 资料与回归输入

结构和密码学规则依据作者的 [AES Crypt 流规范](https://www.aescrypt.com/aes_stream_format.html)及[引擎流说明](https://github.com/terrapane/aescrypt_engine/blob/master/stream_format.md)。后端使用已准入的通用 AES 与 SHA-2 原语，不引入 AES Crypt 运行时或官方应用程序，不执行解密文件，也不绕过应用许可。共享秘密 SHA-512/HMAC 状态遵循 RFC 2104 与 [RFC 4231](https://www.rfc-editor.org/rfc/rfc4231)。

[自有样本集](../../../tests/fixtures/aescrypt/README.md)包含独立编码的 0–3 版输入，并由独立 Python 密码学解码器交叉核对。固定公开测试密钥与 IV 只用于确定性样本。参考工具只留在 `build/toolchain/`，生成证据只留在 `build/` 或 CI。验收条目为[验收要求](../../../developers/zh-CN/qa/acceptance.md)中的 `FORMAT-AESCRYPT`。另见[密码处理](password-extraction.md)、[归档检查](archive-checking.md)及[安全提取](zip-safe-extraction.md)。
