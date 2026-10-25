# CAB 格式来源与实现依据

本文记录 CAB 容器、MSZIP、LZX 和 Quantum 的来源、适用范围及独立实现约束。产品能力、资源治理和逐格式替代规则分别由[隔离格式工作进程契约](native-codecs.md)、[自研 Rust 契约](independent-rust-codecs.md)及[分卷契约](archive-opening.md)定义。来源记录不代替互操作验收或发行许可复核。

## 微软技术文档

优先使用 Microsoft 的格式文档。下表固定实际阅读的副本，日期是文档版本标识，不表示该 URL 永远返回相同字节。下载副本须核对 SHA-256，不能仅按相同标题判断版本相同。

| 文档与入口 | 固定版本 | 使用范围与限制 |
| --- | --- | --- |
| [MS-CAB 官方归档入口](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprotlp/5641c1b2-35ed-4542-9058-a07ccab5b0a1)，[PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/Archive_Exchange/%5BMS-CAB%5D.pdf) | 页脚 `v20110304`，Release 2011-03-04 | §2 的 CFHEADER、CFFOLDER、CFFILE、CFDATA，§3.1 的校验方法。Microsoft 将其列为归档文档，并明确提示不应依赖归档材料；不能称为现行受维护规范。它不定义各压缩算法的内部位流。 |
| [MS-MCI](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-mci/27f0a9bf-9567-4e40-ad66-6ae9ab9d2786)，[PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-MCI/%5bMS-MCI%5d.pdf) | 2016-10-17，revision 7.3 | §2 的 MSZIP 数据结构、CK 标记、完整 Deflate 流和块大小。Deflate 位流另依据 [RFC 1951](https://www.rfc-editor.org/rfc/rfc1951.html)。 |
| [MS-PATCH](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-patch/cc78752a-b4af-4eee-88cb-01f4d8a4c2bf)，[PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-PATCH/%5bMS-PATCH%5d.pdf) | 2025-08-19，revision 14.1 | §2.1 位流、窗口和距离，§2.2 头与 E8，§2.3 块，§2.4 Huffman 树，仅用于比较共同机制及差异。它描述 LZX Delta，不是 CAB LZX 的替代规范。 |

| 固定 PDF | SHA-256 |
| --- | --- |
| MS-CAB | `09f796a493547697ed7d4bc36222ac132bf3ca1bd6b1da76fbdc3b38b5307709` |
| MS-MCI | `cb097c434e1cfccb368f0d7e9084ab4d47a65809a2eea94bbf41f9722c1a0343` |
| MS-PATCH | `ea17466b97c9a9d4e2c815538387bf442fb2393ed29632c6f58b2c47ef1f29fe` |

Microsoft Open Specifications 文档带有自己的版权及知识产权条款，包括实现所需文档使用许可和专利提示。它们不因此成为本项目 Apache-2.0 材料，也不能从文档可下载推定所有专利或再分发问题均已解决。

## 1997 年 CAB SDK 文档

原始标题为 Microsoft Cabinet File Format、Microsoft MSZIP Data Compression Format 和 Microsoft LZX Data Compression Format。所读副本正文包含 Microsoft 1997 年版权声明，由第三方托管：

| 镜像原件 | 阅读范围 | SHA-256 |
| --- | --- | --- |
| [CABFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/CABFMT.DOC) | §2 容器布局与跨卷语义，§3 固定容器示例，§4.1 校验定义 | `5a220658b76f8a66e9013e342b4eb236f057b597a8e6f3939c86f05efc51e83a` |
| [MSZIPFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/MSZIPFMT.DOC) | Introduction、Implementation Details，特别是独立 Deflate 块与保留历史 | `046d283fa3b697e46c35b6a329bb90c86cbc9f4dedbed24c3ae43bd9b8ccf239` |
| [LZXFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/LZXFMT.DOC) | Concepts、LZX compressed data format、树编码、距离/长度及 E8 预处理各节 | `68ba4df86e08bece3259574133f1c61606d143d85f749b388f5f1e8ea46f8bef` |

这些 `.DOC` 文件实际为 RTF。正文标题、版权和 CAB 定位支持其作为微软作者文档副本使用，但托管站不是 Microsoft，尚未与微软发布的 CAB SDK 归档逐字节比对。Microsoft 的[旧 SDK 下载地址](https://download.microsoft.com/download/platformsdk/cab/2.0/w98nt42kmexp/en-us/cabsdk.exe)未能提供可核验归档，不能据此声称已验证镜像真实性。阅读范围是格式文档，不含该 SDK 的 FCI/FDI 实现或示例代码。

公开 TCOMP 参数另核对 Windows SDK 10.0.19041.0 的 `um/fdi_fci_types.h` 常量声明及 [TCOMPfromLZXWindow 文档](https://learn.microsoft.com/en-us/windows/win32/api/fdi_fci_types/nf-fdi_fci_types-tcompfromlzxwindow)。该头文件 SHA-256 为 `27a1f6f7728f5db64d11e434d101819d646ca49270930606bb76c7989ebe0254`。只使用公共参数范围与位定义，没有据此调用或分析 FCI/FDI 算法实现。

LZXFMT 的部分概览、表格和正文存在矛盾。MS-PATCH 可辅助辨认共同机制，但其参考数据、扩展长匹配、128 KiB 至 32 MiB 窗口及额外块头不能直接套入 CAB。CAB LZX 的窗口范围为 15 至 21 位，须单独核对原文、参考行为和辨识向量。

## Quantum 资料范围

已取得的原算法厂商材料为 [Cinematronics QUANTUM 用户手册镜像](https://carrero.es/wp-content/uploads/2025/07/QUANTUM.pdf)，正文版权为 Cinematronics 1993、1994、1995，保留全部权利；PDF SHA-256 为 `a2d6ed51e47db407ea5849b42004750f2f36c82a380f263b983c5282838a8212`。§6 说明独立 `.Q` 归档容器，没有定义 CAB Quantum 所需的算术位流、概率模型更新和帧终止，不能把 `.Q` 的整流状态或文件校验移作 CAB 规则。

[Matthew Russotto 的 Quantum 格式研究地址](http://www.russotto.net/quantumcomp.html)及历史存档未取得可核验正文。因此没有将其内容作为已读依据，也不能判断原文是否足以完整定义算法，或推定其许可条件。

另只阅读了 [David Carrero 的 UnQuantum README](https://github.com/dcarrero/unquantum/blob/main/README.md)，副本 SHA-256 为 `187af81f8231cfe4cc31aec0c76476b1e33179fe517a365c820fc49c1b8106e0`，未阅读该项目解码源码。它是 `.Q` 项目的非官方算法概览，并声明参考 Russotto、libmspack 和二进制研究。它不是 Russotto 原文，也不能完整替代 CAB 位流规范；项目标注 MIT 不足以推定其参考资料和算法来源均按 MIT 授权。

上述缺口以固定官方参考实现的行为分析补齐，不把非官方材料一概排除，也不把参考实现称为 Microsoft 格式规范。

## 固定参考源码与分析范围

参考来源为 [7-Zip 官方 26.05 发布](https://github.com/ip7z/7zip/releases/tag/26.05)中的 [7z2605-src.tar.xz](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz)，归档 SHA-256 为 `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e`。构建来源清单另见 [native-codec-sources.json](../../scripts/native-codec-sources.json)；该清单描述完整官方来源，下面的记录仅说明 CAB 参考分析实际使用的文件。

按归档中的 `DOC/License.txt`，下列解码器及通用文件适用默认 LGPL-2.1-or-later，未发现这些文件带有替代许可声明。归档还包含适用于其他文件的 BSD 条款，不能把整个归档简化为一个无例外许可，也不能将这些解码文件误标为 BSD。

| 归档内文件 | 实际分析范围 | SHA-256 |
| --- | --- | --- |
| `CPP/7zip/Compress/QuantumDecoder.cpp` | 算术位流、模型、token、帧终止和字典状态 | `1a22429d792493573f542782a45a7f702b6e636e5aee1d47bdb3d1cf75ef11e2` |
| `CPP/7zip/Compress/QuantumDecoder.h` | 模型字母表及 selector 常量 | `7571ee0360a7abe8f297f4cd4e34df1863003f252c6fab66c61f8d1bd25a21b4` |
| `CPP/7zip/Compress/Lzx.h` | LZX 位流常量 | `779b32ecaecfc5b8c135e0051394001786d680fca4cfda57a480884342500e99` |
| `CPP/7zip/Compress/LzxDecoder.h` | 跨帧保留状态的边界 | `18141528bc8587ade4f7b1e330ee35dc8e31d6c6ab34d86a2dff59d362d0d6c9` |
| `CPP/7zip/Compress/LzxDecoder.cpp` | CAB 模式的树、raw 块、补齐、重复距离、E8、历史和结束行为；不以优化代码或内存布局作为实现设计 | `e78bf50f27f21e04966785fafe7f94fddc533a9e6103f79b81c4e7d7613d1aab` |
| `CPP/7zip/Compress/HuffmanDecoder.h` | 仅完整树/空树的准入部分，固定副本约第 109 至 164、430 至 497 行 | `22f93e29b2408ce7e1c1bca5f5f807c1383a8d1634fe3ae6cfcab15f084109c0` |
| `CPP/7zip/Archive/Cab/CabHandler.cpp` | 仅压缩方法、CFDATA 帧和历史接续部分，固定副本约第 1020 至 1252 行 | `77b82387eea40eda02929b90d3006877c08765089d6b96c5c12b9cfd3f56b8f7` |
| `C/7zVersion.h` | 固定版本身份 | `f7dc58b3a3c01569e061eda9cfe633ad8983667eb0fa10ab94a6c364ddd7e699` |
| `DOC/License.txt` | 文件许可默认规则和例外 | `9ac2b4a97ab5d523965534d8b2d5868e511b39096d51fff458ab72c38b80fccc` |
| `DOC/copying.txt` | LGPL 原文的身份摘要，未作为法律意见解释 | `1e7e6bae5a5bde32f1ae5a7c37a082d1ab03cf89354f7f936ac40be9e39a6531` |

行号仅用于限定该固定副本的阅读范围，文件身份以摘要为准。升级参考版本时须重新记录差异，不能沿用行号认定行为不变。

## 分析与独立实现的分工

前期准备只阅读格式文档及项目自有接口。随后参考分析者明确阅读了上表指定源码，将缺失或矛盾的位流、数学更新、重置、终止和错误条件整理为不含代码表达的行为规格，再交给未阅读这些参考源码的 LZX、Quantum 实现者。

两个算法的 Rust 实现由各自未读参考源码的实现者独立组织状态、类型、控制流程及测试。分析者不承担这两个算法的 Rust 实现，另行负责 CAB 容器。由此不能对整个 CAB 工作声称无人接触参考源码，不能将本流程称为严格洁净室。行为规格不得包含参考代码、逐行改写、结构搬运或注释翻译；规范名称、常量和数学关系须保留来源。

参考阅读、独立编码和黑盒互操作分别回答不同问题。自有编码器与自有解码器相互成功不能证明独立兼容；参考工具成功也不能证明其所有宽容行为都适合本项目完整性契约。参考源码、文档、工具和样本各自保留原许可，Rust 重写不构成法律风险已经清零的结论。许可与发行要求另见[许可契约](licensing.md)及[对应源码契约](source-delivery.md)。

## 必须保留的技术区分

以下是格式文档和限定参考分析共同约束的行为边界，不规定 Rust 内部数据结构或优化方式。

- CAB 的 folder 是压缩历史单元，文件是其输出区间。文件边界、CFDATA 边界及跨卷延续不自动重置历史；只有新逻辑 folder 重置。`cbUncomp=0` 的跨卷压缩片段按同一逻辑帧拼接，每个物理 CFDATA 的校验仍独立验证。该校验是字节/字的异或，含数据保留区，不能用 CRC 或拼接后的单次校验代替。元数据、长度、卷关系与负载范围须分别验证。
- MSZIP 的每帧由 CK 和完整裸 Deflate 流组成。树和位流随帧重置，至多 32 KiB 历史随 folder 保留；帧最多 32,780 压缩字节、32,768 输出字节。复用项目共享 Deflate 的预置历史接口，不能新建第二套 Deflate。零填充或强行补齐截断流不能代替严格终止检查。
- CAB LZX 使用小端 16 位字、字内最高位优先的位流。窗口 15 至 21 位的 position slot 数依次为 30、32、34、36、38、42、50；旧文概览中的大窗口计数须结合完整距离表校正。树长差值按模 17 做减法，aligned 树先于 main/length 树。main、pretree、aligned 树要求完整，length 树可全空，但实际请求该空树时必须失败。
- LZX raw 块仍有通用块长，进入 raw 数据前跳过 1 至 16 个零位，即使恰好按字对齐也须跳过一个字。奇数 raw 块的零补字节可以在下一帧开始消费；仅在 folder 最终结束且没有下一块时兼容省略该补字节。普通压缩帧只允许 0 至 15 个零位补齐，不能接受额外完整字或任意尾字节。LZX 块可以跨 CFDATA 帧，最终 `finish` 必须拒绝仍有声明输出尚未完成的块，不沿用旧 CAB 接入遗漏结束检查的宽容。
- LZX 的 LZ 历史保存 E8 还原前的字节；新 folder 没有可回溯的有效零历史。E8 在每帧末十字节前扫描，translation size 按完整无符号 32 位解释，位置按 folder 累计。旧文末六字节的描述和仅适用于正有符号大小的比较不能直接用于实现。
- Quantum 每帧重置 16 位闭区间算术码状态，九个概率模型和实际字典历史随 folder 保留。selector 的七种取值不等于只有七个模型。模型符号频率每次增加 8，总频率严格大于 3800 时才在下次使用前重标定；每个模型独立的累计频率减半、第四次及其后每五十次重排、同频顺序均属于位流行为，不能随意替换为语言默认排序。字典窗口为 10 至 21 位，level 为 1 至 7，level 不改变解码语义。
- Quantum 没有独立结束 token，精确输出长度由容器提供，同一压缩字节串可能满足不同合法声明长度。到达期望输出后仍须消费至少两个零位，取使输入达到字节边界的最少位数，即 2 至 9 位，然后恰好耗尽压缩输入。追加零字节、非零终止位、截断及跨帧 match 均不能成功。新 folder 也不能回溯尚未生成的窗口内容。

宿主继续负责冻结卷集合和源身份、名称解析、条目身份、路径预检、任务预算、取消、暂存及提交。格式层只读取宿主授权的流，不按 CAB 内部名字自行打开文件。原始名字与 UTF-8 声明分别保留，普通重名记录不合并，仅规范明确的 continued 文件合并。算法错误不得掩盖宿主的取消、配额或 I/O 错误。具体接口和资源上限以共享契约为准，不从参考缓冲区尺寸或 HRESULT 分类推导新的产品权限。

English: CAB framing and checksums use the archived Microsoft MS-CAB document; MSZIP uses MS-MCI and RFC 1951. The 1997 Microsoft CAB SDK documents are third-party-hosted copies whose hashes are pinned, without an authenticated SDK-archive comparison. MS-PATCH describes LZX Delta and does not replace CAB LZX. Available Quantum materials do not fully specify the CAB bitstream, and Russotto's original text was not obtained. A designated analyst read the listed, hash-pinned 7-Zip 26.05 files and supplied code-free behavioral specifications to separate LZX and Quantum implementers who had not read those sources. The analyst also implemented the CAB container. This is not a strict clean-room or a legal-clearance claim. Original licenses, host authority, bounded decoding, exact termination and independent interoperability requirements remain applicable.
