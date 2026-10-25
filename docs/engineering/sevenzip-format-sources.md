# 7z 格式来源与独立实现依据

本文件记录可复用的来源和实现边界。迁移验收与剩余能力在[任务 #50](https://github.com/xiaodaozhi/xdzip/issues/50)维护，固定 Rust 路由及官方产品依赖退出不自动构成跨平台、人工或发行验收。通用替代规则见[自研契约](independent-rust-codecs.md)。

## 规范与参考身份

容器字段、变长整数、方法图、属性及文件映射依据官方 `DOC/7zFormat.txt`、`DOC/Methods.txt`。固定参照为 [7-Zip 26.05 官方源码归档](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz)，SHA-256 为 `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e`。其中两个文档的 SHA-256 分别为 `8ec976b73ca88710286be1e991e49dbab485d9c0bf2410a80fa87e657850f82b`、`b16ea5fdc50345fa2cb031f06fe410cea0b089f74a4138ac8141b9e2d816ffc7`。

LZMA 位流另参考 Igor Pavlov 2015-06-14 的[官方规格包](https://www.7-zip.org/a/lzma-specification.7z)，归档 SHA-256 为 `4529aca392d6b9598f72086e7d136b1e05d2863bf7a757836d3fd4f49e623b58`，其中规格文本 SHA-256 为 `6a4d441c2fa1d8d37b0189c7147a4b5424d5cdb122a7fc1fbebd022864ed54fb`。仅阅读格式文字、规范数学与伪码，不将包内参考解码器作为 Rust 移植输入；文档及样本不因下载而获得新的再分发许可。

通用 Deflate 依据 RFC 1951，与 ZIP、GZIP 和 MSZIP 共用一套实现。Deflate64 在该实现中显式选择 64 KiB 历史、距离码 30/31 和长度码 285 的扩展规则，普通 Deflate/MSZIP 不改变。[PKWARE APPNOTE](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT) 的增强 Deflate 段只提供概述，具体位流差异通过固定参考的事实分析和独立读取验证补足，不能将概述冒充完整规格。AES-CBC 的数学和独立向量依据 [NIST SP 800-38A](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf)，7z 的属性、UTF-16LE 密码派生和零填充是额外格式规则，不能从 ZIP AES 或 PKCS#7 推导。BZip2 手册及 PPMd 的格式名称没有完整规定历史位流、模型重启和结束行为，不能将这些实现描述成只读手册即可完成。

## 参考分析与原创编码

分析者在上述固定原始归档中定向阅读下列组，补充文档省略的字段、数学状态、容量重启、结束及错误关系。交接只含事实和辨识要求，不含参考代码、逐行改写、类/数组布局、函数划分、控制流程搬运或注释翻译。对应 Rust 算法由未阅读这些参考实现的实现者独立组织；参考分析者不再实现其已分析的算法。容器作者也未阅读官方容器实现，依据文档和事实交接独立编码。

| 分析主题 | 固定归档中的参考范围 | 许可区分 |
| --- | --- | --- |
| LZMA1/LZMA2 | `C/LzmaDec.c/.h`、`C/Lzma2Dec.c/.h`，编码器中 range、chunk/reset 与属性相关部分，C++ Lzma/Lzma2 包装 | 这些 C 文件声明 Public domain；C++ 包装适用归档默认条款 |
| 容器细节 | `CPP/7zip/Archive/7z/7zIn.cpp` 的外部属性、子流校验继承与文件属性相关部分 | 归档默认 LGPL-2.1-or-later |
| 7z AES | `CPP/7zip/Crypto/7zAes.cpp/.h`，MyAes、FilterCoder 及 7z 编解码包装中的块长、密码、填充和完成相关部分 | 归档默认 LGPL-2.1-or-later，不读取或移植底层 AES 实现 |
| BZip2 | BZip2Decoder、Const、Crc，区分普通 7z 和独立 bz2 包装的错误处理 | 归档默认 LGPL-2.1-or-later，不据格式名称套用其他实现的许可 |
| PPMd7 | `C/Ppmd7.c/.h`、`Ppmd7Dec.c`、`Ppmd7Enc.c` 的数学状态、虚拟容量和重启事实，以及 C++ Ppmd 包装的属性与结束约束 | 这些 C 文件声明 Public domain；C++ 包装适用归档默认条款，不复制上下文布局、分配器实现或控制流程 |
| Deflate64 | `CPP/7zip/Compress/DeflateConst.h`、DeflateDecoder 的 64 位模式相关范围、Deflate64Register 及 7z 属性准入 | 归档默认 LGPL-2.1-or-later，仅交接与普通 Deflate 的格式差异 |
| ISA、Delta、Swap | `C/Bra.c`、`Bra86.c`、`Bra.h`、`Delta.c`、`SwapBytes.c` 的标量语义及 C++ 方法注册/属性/尾部包装 | 这些 C 文件声明 Public domain；C++ 包装单独适用归档默认条款，SIMD 策略不作为格式 |
| BCJ2 | `C/Bcj2.c/.h` 的四流和数学状态，C++ Bcj2Coder 的解码完成与尺寸包装 | C 文件声明 Public domain；C++ 包装适用归档默认条款，不分析或搬用编码器主体 |

固定归档及具体文件的身份、实际阅读范围和事实交接摘要应随开发证据保留，升级参照时重新核对，不能沿用旧行号证明新版本行为。规范常量、标识和数学关系可用于格式实现，内部优化及历史未定义行为不作为产品权限。Public domain 声明不扩展到整个归档、文档、CPP 包装或其他样本。

这不是严格洁净室或法律风险清零的声明。第三方来源、历史交付、仍在过渡的官方后端及测试工具继续遵守各自条款，见[许可](licensing.md)与[对应源码](source-delivery.md)。

## 互操作边界

自有 writer 和 reader 相互成功不能替代独立参照。测试分别验证独立压缩器的已知输出、自有压缩流由官方读取、所有输入消费、结束状态、CRC、损坏、配额、取消及来源错误；组件验证与宿主/进程/平台验收分别记录。

历史兼容宽容必须逐项确认。例如 BZip2 的 7z 节点可接受最终字节的非零未使用位，以及块末四个重复字节恰无额外计数字节，但仍需正确 CRC、声明大小及完整流结束。额外尾字节和拼接流不能据此放行。末字节位与额外字节不能混为一种填充。

PPMd7 区分外层已知长度与未知长度的结束 profile。7z 已知长度节点在精确输出后核对 range 终态和 packed 范围，不额外消费 END 符号；未知长度的共享 reader 要求 END。格式规定的模型容量重启与宿主内存准入是两回事，内存不足不能触发伪造的格式重启。独立对照输入包含固定官方 C 编码器产生的容量重启流，生成器和原始参考源码仅作为本地测试工具，不进入生产依赖。

English: The implementation uses the pinned official 7z grammar and LZMA specification, supplemented by code-free factual handoffs from a designated reference analyst. Independent Rust implementers do not read or translate the corresponding reference implementations. C files declaring public domain and LGPL C++ wrappers retain distinct provenance; no strict clean-room or legal-clearance claim is made. Generic cryptographic primitives remain admitted dependencies, while container and compression logic is original. Independent known-byte interoperability, exact termination, corruption, resource, host and platform checks remain separate migration gates.
