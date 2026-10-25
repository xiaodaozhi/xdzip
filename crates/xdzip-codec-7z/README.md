# 7z 只读候选

此 crate 尚未接产品路由。公开元数据入口为 `SevenZipCodec`、`read_index` 和 `read_index_with_decoder`；公共 `Entry.method` 继续为 0，完整方法 ID、图、原始属性和物理文件到子流的映射保存在 `Index` 中。不提供文件系统或进程权限，编号分卷仍由宿主作为一个连续 `ReadSource` 提供。

`HeaderDecoder` 从受控 `HeaderSource` 读取，向 parser 所有的 `MetadataSink` 写 terminal folder 输出。输入读取、已分配元数据、解码头、外置属性及重试累计受限；collector 独立核对精确输出大小和 folder/substream CRC。默认 `MethodGraphDecoder` 执行已准入的 Copy、LZMA1、LZMA2、Deflate、BZip2、AES、ISA/Delta/Swap 过滤器，以及四输入 BCJ2。节点使用完整全局端口与绑定映射，packed 输入保留序列化顺序，每路上游先独立完成尺寸、终态与校验。未知方法或其他端口结构明确 Unsupported。`CopyHeaderDecoder` 仍可显式注入。plain Header 的内容方法先保留为图，不把元数据解析成功当成算法可解码或内容验证成功，不回退外部后端。

实现依据为官方 7-Zip 26.05 DOC 中的 `7zFormat.txt`（文档版本18.06）与 `Methods.txt`（版本26.04）。文档未展开的 SubStreams 默认与 CRC 槽、外置属性流组织和接受边界，由独立参考分析者提供不含实现代码/类型布局的事实交接，并用自有 Copy 格式向量及固定官方工具交叉验证。未阅读或翻译官方/第三方容器实现。来源文档、固定工具身份、工件与实测记录只保留在当前 worktree 的 `build/`。

本阶段的明确准入策略：

- 顶层 Header/Streams 段必须按格式顺序完整结束。FilesInfo 的已知属性精确消费其长度，外置值从整个 folder 输出 blob 的起点独立读取并完整消费。DataIndex 不按文件子流展开。
- FilesInfo 重复的语义属性明确拒绝，避免覆盖冲突；允许多个全零 Dummy padding。未知 FilesInfo 属性明确 Unsupported。非零 Dummy、损坏 UTF-16、名称缺失、外置/内联残余字节明确 Invalid，不继承参考库的坏头警告后继续行为。
- 位图 MSB-first，未用低位不作为数据，非最短但完整的 UINT64 编码可读。EmptyFile/Anti 按当时的 EmptyStream 数量解释，目录、空文件和零长度的有流文件分开保留。Anti/链接/特殊对象不获得输出权限。
- EncodedHeader 解出唯一完整 folder blob 后必须为普通 Header，不递归接受嵌套 EncodedHeader，不拼接多个 folder。AdditionalStreams 和 EncodedHeader 的自外置表不能引用不存在的外部数据。
- 保留零子流 folder 及其非零输出大小、CRC和 packed 输入，不能因为没有文件引用就丢掉该图。后续整包检查/选择解码执行器仍须独立验证必要的完整 folder 与算法终止，计入跳过/尾部工作。

Payload 入口重新扫描并比对冻结 `Entry` snapshot，按物理 ID 选择记录。首次访问完整执行并校验整个 folder，每个节点的尺寸、终态、packed CRC，以及 folder/substream CRC 均须成功，才向宿主发布该文件。即使没有逐文件 CRC，也不会略过其 folder 的压缩终止或整体校验。失败后会话失效，后续调用不能重试。输出只交给宿主 sink，不持有路径或文件写入权限。

当前缓存策略是有明确资源限制的候选方案：单个 folder 及会话累计 terminal 输出最多256MiB，已验证 folder 在同一会话中不驱逐、不逐文件重放。字典、概率模型、packed 缓冲、各节点输出上界、已有缓存和64MiB元数据保留先合计准入，默认1GiB，再分配算法内存。packed 输入与 terminal 工作量分别累计受限，宿主 WORK 只计最终 terminal 输出，包含未选文件区间；中间节点使用独立工作上限，AES KDF 另有跨头、外置属性和内容节点累计的 rounds/hash-input 预算。缓存输出只消费本身私有预付余额，重复交付超出余额的部分按普通输出路径计量。

该方案尚不能覆盖既有8GiB文件、20GiB任务的大固实包边界，后续须改为流式执行，不能据此切换产品路由或声称对等迁移。尚未实现的方法、复杂端口图、创建/重建及分卷宿主接线仍须独立完成。测试用自编码结构向量只证明组合状态；独立官方作者输入和官方工具读回另行记录，不能混称。
