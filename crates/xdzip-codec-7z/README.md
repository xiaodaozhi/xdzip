# 7z 读写后端

此 crate 通过共享 CXX 边界在隔离工作进程执行，产品固定使用 Rust。公开元数据入口为 `SevenZipCodec`、`read_index` 和 `read_index_with_decoder`；公共 `Entry.method` 继续为 0，完整方法 ID、图、原始属性和物理文件到子流的映射保存在 `Index` 中。不提供文件系统或进程权限，编号分卷仍由宿主作为一个连续 `ReadSource` 提供。

AES-CBC 没有独立密码验证值。已知加密元数据在提供密码后发生的 Invalid，以及加密 folder 实际解码的 Integrity，分别返回明确的 `PasswordOrIntegrity`，表示密码不匹配或加密数据损坏。外层明文头 CRC、条目快照身份不符、来源、资源限额、取消、不支持特性和输出错误保留原分类；失败实例仍 poison，由宿主重开实例并保留任务累计计费后重试，不把不确定校验失败称为确定的错误密码。

`HeaderDecoder` 从受控 `HeaderSource` 读取，向 parser 所有的 `MetadataSink` 写 terminal folder 输出。输入读取、已分配元数据、解码头、外置属性及重试累计受限；collector 独立核对精确输出大小和 folder/substream CRC。默认 `MethodGraphDecoder` 执行已准入的 Copy、LZMA1、LZMA2、Deflate、Deflate64、BZip2、PPMd、AES、ISA/Delta/Swap 过滤器，以及四输入 BCJ2。节点使用完整全局端口与绑定映射，packed 输入保留序列化顺序。未知方法或其他端口结构明确 Unsupported。`CopyHeaderDecoder` 仍可显式注入。plain Header 的内容方法先保留为图，不把元数据解析成功当成算法可解码或内容验证成功，不回退外部后端。

实现依据为官方 7-Zip 26.05 DOC 中的 `7zFormat.txt`（文档版本18.06）与 `Methods.txt`（版本26.04）。文档未展开的 SubStreams 默认与 CRC 槽、外置属性流组织和接受边界，由独立参考分析者提供不含实现代码/类型布局的事实交接，并用自有 Copy 格式向量及固定官方工具交叉验证。未阅读或翻译官方/第三方容器实现。来源文档、固定工具身份、工件与实测记录只保留在当前 worktree 的 `build/`。

本阶段的明确准入策略：

- 顶层 Header/Streams 段必须按格式顺序完整结束。FilesInfo 的已知属性精确消费其长度，外置值从整个 folder 输出 blob 的起点独立读取并完整消费。DataIndex 不按文件子流展开。
- FilesInfo 重复的语义属性明确拒绝，避免覆盖冲突；允许多个全零 Dummy padding。未知 FilesInfo 属性明确 Unsupported。非零 Dummy、损坏 UTF-16、名称缺失、外置/内联残余字节明确 Invalid，不继承参考库的坏头警告后继续行为。
- 位图 MSB-first，未用低位不作为数据，非最短但完整的 UINT64 编码可读。EmptyFile/Anti 按当时的 EmptyStream 数量解释，目录、空文件和零长度的有流文件分开保留。Anti/链接/特殊对象不获得输出权限。
- EncodedHeader 解出唯一完整 folder blob 后必须为普通 Header，不递归接受嵌套 EncodedHeader，不拼接多个 folder。AdditionalStreams 和 EncodedHeader 的自外置表不能引用不存在的外部数据。
- 保留零子流 folder 及其非零输出大小、CRC 和 packed 输入，不能因为没有文件引用就丢掉该图。Check 维持逐个物理 FilesInfo 记录的语义，元数据结构和范围校验不等于验证未引用的 payload。

Payload 入口重新扫描并比对冻结 `Entry` snapshot，按物理 ID 选择记录。有独立子流 CRC 的文件，在该文件精确长度和 CRC 验证后可返回成功，并保留图状态供下一文件继续；后续失败不会追溯撤销已完成的文件。没有独立子流 CRC 时，必须读完该 folder 并验证压缩终止和实际提供的 CRC 才能成功。抵达 terminal EOF 时，每个上游节点和 packed span 都必须完成精确尺寸、终态及 CRC 校验。失败后会话失效，后续调用不能重试。输出只交给宿主 sink，不持有路径或文件写入权限。

元数据图受64MiB collector 限制，使用与 payload 相同的方法计划和流式节点。Payload 不缓存整个 folder，各节点每次交付至多8KiB，过滤器仅保留有界短尾。字典、模型、节点缓冲和保留元数据先合计准入，默认1GiB，再分配算法内存。单文件限额8GiB，packed 输入和 terminal 实际工作量分别累计限额20GiB；不会仅因未读取 folder 的声明总尺寸较大就拒绝当前小文件。宿主 WORK 只计最终 terminal 输出，包含未选文件区间；中间节点使用独立工作上限，AES KDF 另有跨头、外置属性和内容节点累计的 rounds/hash-input 预算，任务内最多16个 KDF 身份、16KiB预准入的密钥缓存仅在 miss 时计费，同一 salt/cycles 的不同 IV 可复用，密码改变会清空缓存。

同一 folder 的正向访问继续当前流。反向访问、切换 folder、变更密码或此前为完整验证而读过目标时，可有界重启，重放的真实输入、解码和 KDF 工作均再次累计，不能承诺所有选择顺序都不重放。输出预付只来自当前实际 terminal 字节，不给重放或重复交付退款。宿主固定使用此 Rust 路由，编号卷冻结成有界连续来源，读写、重试和提交沿用现有隔离协议。实现检查、跨平台验证与原生桌面验收分别记录。测试用自编码结构向量只证明组合状态；独立官方作者输入和官方工具读回另行记录，不能混称。


`SevenZipCodec` 另实现 `CreationCodec` 与受限 `RewriteCodec`。创建提供 Copy 级别0、真实 LZMA2 级别1/6/9，名义字典8MiB，可选择固实、AES 内容加密与头加密；头加密必须有密码。每个 AES folder 与加密头使用新 IV，任务内密钥缓存仅复用相同密码及 KDF 参数的派生结果。未加密头可直接使用普通 Header。名称使用 UTF-16LE，保留目录、空文件与可表示的 UTC mtime；非空文件子流、folder、packed stream、起始头及下一头写入对应 CRC。

创建从宿主的 `EncodeInput` 按冻结大小读取，每个普通文件（含空文件）在读满后最多探测1 byte。提前 EOF、超长、来源错误或取消均失败。由宿主提供新的空白私有暂存，codec 不具备截断旧目标的权限。仅有 `EncodeOutput::write_at` 暂存权限，每块最多8KiB；AES 小块先合并，压缩输出与大型固实数据不整包物化。签名头只在数据和最终元数据写完后回填，flush 或回填失败仍是失败，暂存看似有效不构成发布许可。实际来源读取量驱动 progress，宿主仍须完整复验、重查来源并控制最终提交。

写入预检最多100,000条记录、单个 UTF-8 名称4096 bytes、总名称16MiB、元数据64MiB、单文件8GiB、任务输入与实际暂存写入20GiB。字典、匹配索引、元数据预留、条目容量与任务密钥缓存先合计受1GiB预算约束；不会在生产依赖中引入第三方压缩器或临时文件权限。

重写仅接纳未加密、非分卷且全部记录可安全保留的普通包，keep/renames 按原物理记录顺序解释，重复名称不合并。保留与新增内容工作集合计最多512MiB，通过本 crate reader 校验保留内容后重建；此工作集属于 worker 总体1GiB预算，decoder 额外内存必须扣除工作集与元数据预留。旧记录从 `Index.files` 保留未定义与零值的区别、原始 attributes 及 ctime/atime/mtime 的完整 FILETIME 精度，不从公开 DOS 时间反推。链接、特殊对象、reparse 属性、Anti、加密、分卷或无法保留的非空 ArchiveProperties 明确 Unsupported。原归档与宿主目标都不授予 codec 写入权限，原目标保护及最终发布由宿主验证。重写与既有 Native 一样枚举 FilesInfo 实体，只保留所选普通记录的内容和已理解属性，再重新编码；不承诺保留原压缩图、未被文件引用的 folder payload/blob 或外置属性的物理布局。零子流 folder 不会凭空变成输出文件。

English: This independent 7z backend also implements streaming creation and bounded rewrite, without publication authority. Creation supports Copy level 0 and real LZMA2 levels 1/6/9 with an 8 MiB dictionary, solid data, fresh-IV AES and optional encrypted headers. The host supplies a new empty private staging stream. The codec has no truncation authority over an old target. It consumes exact frozen sizes with at most a one-byte EOF probe and stages bounded random writes. Ordinary unencrypted single-volume rewrites retain physical record identity, raw attributes and precise timestamps, with a 512 MiB retained-plus-added content working set inside the overall 1 GiB memory policy. Unsupported special or unknown semantic metadata is rejected. Rewrite preserves selected FilesInfo entities, not the original coder graph, unreferenced payload/blob or external-property layout. Independent tool checks, complete host verification and platform acceptance remain separate evidence.
