# 自研 Rust 格式与算法契约

本契约定义第三方格式后端的逐项替代规则。实际进度、测试结果与未完成项由[迁移任务 #45](https://github.com/xiaodaozhi/xdzip/issues/45)跟踪，不以架构决策或代码存在推定格式已完成验收。

## 实现边界

容器解析、编码、解码与压缩算法采用独立 Rust 实现，允许经准入的通用密码学、字符编码、系统及桥接基础库。第三方格式库、算法库和命令行工具可以作为独立测试参照，不成为迁移完成格式的产品运行依赖。仅删除直接依赖不足以证明替代完成，须检查普通依赖及构建依赖的传递闭包和交付文件。

格式完成全部既有能力验收后才替换，未完成格式继续使用原后端，不在运行失败时静默回退。ZIP、TAR、TAR.GZ、7z 保留既有读写及修改限制，RAR、CAB、ISO、UDF 只读。格式名称不意味着支持该格式的所有历史和未来扩展。RAR7 LZ 必须通过独立压缩流成功样本，Store 成功及超预算字典拒绝不能替代。

领域层、Qt 桥接和公共 codec trait 保持现有所有权。后端不获得目标路径或自行发布文件。名称原始字节、物理 EntryId、重复条目、编码解析、来源身份、卷授权、路径预检、暂存、校验与提交继续遵守现有工程契约。共享算法只实现一次，容器独立处理封装、块边界、字典重置及校验语义。原生后端迁移为 Rust 后仍保留工作进程的取消、终止和资源治理。

## 来源与独立实现

先依据官方技术资料定义字段、状态转换、数值范围、错误行为及测试向量，再编写代码。参考实现只用于补充行为理解，不进行逐行语言转换、改名复制、结构搬运或注释复制。规范规定的标识和常量记录章节；计算表优先用明确数学规则生成。测试参照的来源及许可证与产品依赖分别管理。

已经接触参考源码的开发者不宣称严格洁净室开发。需要来源隔离的模块，由分析者提交不含代码表达的行为规格，再交给未接触参考实现的实现者。参考源码、文档、测试数据本身的许可证不会因 Rust 重写消失；旧版交付及过渡依赖继续履行各自义务。

RAR 官方技术说明不完整，解码算法缺口须记录并进行专项许可审查；不开发 RAR 编码器，不将 UnRAR 的限制解释为允许重建 RAR 压缩算法。公开规范中的专有扩展、专利提示和再分发条件需要分别核查，独立编写不等于法律风险归零。

## TAR 记录规格

独立记录层位于 `xdzip-codec-tar/src/record.rs`。布局依据 [POSIX pax 的 ustar 与 pax 交换格式](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/pax.html)，GNU 扩展参见 [GNU tar 格式说明](https://www.gnu.org/software/tar/manual/html_node/Standard.html)。不引用参考库的内部实现或私有 API。

- 记录为 512 字节，头校验使用无符号字节和，并把校验字段的八个字节视为空格。名称、大小、时间、类型和 ustar 前缀按规定偏移读取。GNU 头的前缀位置不是 ustar 路径前缀。
- 普通数值接受有界八进制和非负 GNU base-256。负数及 u64 溢出拒绝；无效时间保留未知展示，不改变有效载荷定位。数据按 512 字节补齐，结束至少两个全零记录，之后只允许预算内的全零填充。
- PAX 长度包含十进制长度本身、空格、键值和换行。长度须有进展且不越界，键非空，记录以换行结束，不含 NUL。局部 path、size、mtime 和 hdrcharset 沿用现有解释，GNU 长名称和全局 comment 的限制见 [TAR/GZIP 契约](m4-tar-gzip.md)。
- 创建写入 ustar；非 ASCII 或超长名称使用局部 PAX UTF-8 path。超出八进制范围的大小与时间用 PAX 字段，普通头写可表示的占位值。扩展数据最多 64 KiB。输入严格读取冻结大小，随后最多探测一个字节，来源变长或变短均不成功。
- 重写保留普通头的所有者、mode、时间和已准入的其他元数据；重命名更新路径和字符集声明并重新计算校验。PAX size 控制数据长度，不能按普通头占位大小截断数据。坏头、坏扩展、结束记录缺失及资源超限都在宿主发布之前失败。

`tar` 仅作为开发测试的独立读写参照。[依赖政策](../../scripts/rust-codec-policy.json)由 Foundation 检查生产依赖闭包，禁止 tar、filetime 和已退出的压缩库回流至 TAR 实现。锁文件仍记录开发测试参照，不能仅据锁文件判断产品可达性。

## 共享 Deflate 与 GZIP 规格

`xdzip-compression` 无第三方生产依赖。格式定义依据 [RFC 1951 第 3 节](https://www.rfc-editor.org/rfc/rfc1951)及 [RFC 1952 第 2.3 节](https://www.rfc-editor.org/rfc/rfc1952)，不搬运附录示例代码。长度、距离和额外位数为规范中的线格式表，CRC 查表由 IEEE 多项式在编译期生成。

共享公开接口为 `DeflateReader`、`DeflateWriter`、`GzipReader`、`GzipWriter` 与增量 `Crc32`。ZIP 与 GZIP 使用同一套 Deflate 编解码器。裸流 Reader 只读取已准入的压缩范围，区分实际消费与缓冲预读；只有 BFINAL 结束且实际消费恰等于压缩大小时返回成功 EOF。不能用输出大小或 CRC 相同代替压缩流终止，也不接受范围内的第二条流或尾随垃圾。

`DeflateReader::with_dictionary` 接收至多 32 KiB、按时间排列的已准入历史，只初始化匹配窗口，不将历史作为新流输出。ZIP/GZIP 的普通构造仍从空窗口开始。CAB 的 `MszipDecoder` 依据 [MS-MCI](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-mci/27f0a9bf-9567-4e40-ad66-6ae9ab9d2786)复用此接口，每个 CK 帧重置 Deflate 位流、保留逻辑 folder 的末 32 KiB；压缩帧最多 32,780 字节、输出最多 32,768 字节，要求精确尺寸与完整裸流 EOF。只有完整帧通过后更新历史，失败后不能继续使用该 folder，`finish` 后不能追加帧。共享算法不验证 CAB 物理片段校验及卷链，由独立容器负责；CAB 的产品切换与其他方法验收见 [#48](https://github.com/xiaodaozhi/xdzip/issues/48)。

English: MSZIP reuses the independent raw Deflate decoder with an explicitly admitted history of at most 32 KiB. Each CK-prefixed frame resets the compressed stream while retaining folder history, verifies exact input termination and output size, and updates history only after success. Failed or finished folder states cannot accept more frames. CAB fragment checksums, volume authorization and the production migration gate remain separate.

- 解码处理 Store、固定和动态 Huffman，跨块保留 32 KiB 环形历史，允许重叠匹配。拒绝保留块类型、LEN/NLEN 不匹配、过度订阅码表、不允许的不完整码表、缺失结束符、越界重复长度、保留符号和历史之前的距离。
- 编码使用有界哈希链 LZ77、频率构造的规范 Huffman 码和码长游程编码。树深度超出 15 时使用固定树，不截断码长。级别 0 以至多 65,535 字节存储块输出，1/6/9 以至多 2 MiB 块和 4/32/128 个候选搜索上限编码；收益不足时拆为存储块，避免无压缩收益的数据膨胀。编码历史每块重建，压缩率及输出字节不承诺与参考库一致。
- 解码中间状态为窗口及有界码表，编码中间状态随单块上限有界，不收集整个归档。宿主继续累计输入、解码工作及输出配额。编码块有界完成后返回控制权，不在共享算法中新增线程或文件权限。
- GZIP 校验魔数、方法和保留标志，处理 extra、名称、注释及可选头 CRC。单成员头最多 64 KiB，超限拒绝。逐成员验证 CRC-32 和模 2^32 的 ISIZE，所有成员串接为同一 TAR 流。必须读至 EOF；截断、坏尾部和非成员垃圾不能作为成功结束。
- Writer 只有成功 finish 才产生最终块和尾部，Drop 不写出数据。输出失败后不能继续作为成功流使用。内存缓冲不赋予发布权限，宿主仍在全量复验后提交。

有限固定种子变异测试只证明其实际覆盖，不声称完成长期模糊测试。编码性能、压缩率及三平台结果独立记录，后续优化不能放宽完整性和配额检查。

## ZIP 记录与密码封装规格

ZIP 的 EOCD、ZIP64、中央/本地头、额外字段及描述符依据 [PKWARE APPNOTE](https://www.pkware.com/documents/casestudies/APPNOTE.TXT)，AES 封装依据 [WinZip AE-1/AE-2 规范](https://www.winzip.com/en/support/aes-encryption/)。独立实现容器、名称扩展、记录定位、编解码组合和认证边界；通用密码学原语按[依赖准入](dependencies.md)使用，不实现 RAR 编码或额外 ZIP 专有加密扩展。

- 物理中央记录逐项保留，同名条目不合并。尾部必须完整、count/size/offset 一致；存在多个恰好覆盖 EOF 的尾记录候选时拒绝歧义，分卷归一前后采用相同规则，不能用注释内嵌尾记录隐藏真实目录。ZIP64 哨兵按实际存在的字段顺序解释，经典尾记录的两个磁盘号和两个计数可独立使用 0xFFFF 占位，读取 ZIP64 实值后再判断单卷；所有非占位值必须与对应实值一致。拒绝损坏、重复的关键字段和未经宿主归一的多磁盘元数据。扫描只展示有界元数据，声明的大数值保持精确 u64，内容范围在负载阶段复验。
- 单任务元数据读取预算 64 MiB，之后的本地头、描述符、压缩数据、密文及密码重试累计消费同一个 20 GiB 输入预算。名称和快照身份不能在负载阶段重新猜测。实际输出先暂存，完整读取并验证成功后才由宿主提交。
- 逐条核对本地/中央名称、关键标志、方法、CRC、大小、AES profile 和 ZIP64 宽度。数据和可选描述符必须位于下一条本地记录及中央目录之前，不能跨入其他记录。Store 核对精确尺寸与 CRC，Deflate 额外要求严格裸流 EOF；32/64 位及有/无签名描述符均按字段匹配验证，不能把恰等于签名的 CRC 误认成签名。
- AES-256 使用 16 字节随机盐、PBKDF2-HMAC-SHA1 的格式规定参数、两个 32 字节密钥和两字节密码验证值；CTR 计数从 1 开始，按小端块编码。只对密文计算 HMAC-SHA1，并验证前 10 字节认证码。AE-1 同时验证 CRC，AE-2 要求声明 CRC 为零。认证失败是完整性失败，不冒充确定的错误密码。
- 密码仍由宿主借用，格式层不持久化、不打印。派生密钥和流状态使用归零生命周期，AES/HMAC/SHA1 开启基础库提供的清理特性；不保证 Qt、输入法或 CPU 的所有历史副本都可擦除。认证完成之前的输出不获得发布资格。
- 创建使用流式输出、显式描述符和 ZIP64 边界，目录明文，普通文件可选择 Store/Deflate 与 AES。编号分卷仍由宿主在整包验证后分割和提交。普通包重写保留有效压缩载荷、注释及已准入元数据，重新计算记录偏移；加密、分卷、自解压前缀及无法安全保留的元数据继续拒绝修改。空包也检查中央目录前的材料，不能因没有本地记录而丢弃前缀；真正空包仍可加入内容并保留归档注释。
- 修改时间写入 DOS 字段并携带精确秒扩展；对非负且适合有符号 32 位的 Unix 秒写入 UT extra，其他可表示时间写入 NTFS FILETIME mtime。NTFS 的 atime/ctime 为零，依据 [Common ZIP 约定](https://commonzip.org/spec/#lastmodifiedtimestamp)不为其赋予有效时间含义；APPNOTE 本身不定义零值为缺失。负数 Unix 秒不使用 UT，避免独立工具将其解释为未来日期。普通重写保留原时间扩展，不补造访问或创建时间。
- 改名写入 UTF-8 标志并移除陈旧 Unicode Path extra；该标志也声明条目注释为 UTF-8，因此原注释不满足 UTF-8 时明确拒绝改名，保留原包，不静默转码或丢弃注释。未改名记录可保留原始标志和注释。

`rawzip`、`zip`、`flate2` 仅作测试参照，已迁移 ZIP 的普通及构建依赖闭包禁止这些格式和算法库回流。测试参照的许可继续保留，容器可读取不等于所有声明方法都可解码。

## ISO与UDF只读规格

`xdzip-codec-optical` 只接收借用的 `ReadSource`，不访问文件系统或挂载镜像。ISO 依据 [ECMA-119 第五版](https://ecma-international.org/wp-content/uploads/ECMA-119_5th_edition_december_2024.pdf)的卷描述符、目录、扩展属性和附录 C Joliet，以及 [Rock Ridge 1.12](https://bitsavers.org/projects/cdrom/Rock_Ridge_Interchange_Protocol.pdf)、SUSP 1.12 与 [El Torito 1.0](https://bitsavers.org/projects/cdrom/El_Torito_CD-ROM_Spec.pdf)编写。UDF 依据 [ECMA-167 第三版](https://ecma-international.org/publications-and-standards/standards/ecma-167/)第 1、3、4 部分及 [OSTA UDF 2.60](https://www.osta.org/specs/pdf/udf260.pdf)的结构限制、分区与字符集规定编写。实现不转换第三方解析器源码，规范原文和第三方测试工具不作为本项目原创材料再许可。

- ISO 核对双端整数、卷范围、目录记录及文件区段，支持多区段普通文件、Joliet UCS-2、Rock Ridge 名称续接、SUSP CE、CL/PL/RE 目录重定位与链接/特殊类型识别。非 Joliet 原始名称交给共享有界编码解析，不调用系统 OEM 转码。启动目录及镜像仅表示为可读取记录。跨卷 ISO、交错文件、面向记录的扩展属性、Rock Ridge SF 稀疏及 ZF 压缩扩展明确拒绝；不能把未知必须特性当普通文件解码。
- UDF 区分逻辑块号与物理偏移，验证 descriptor tag 的位置、校验和、CRC 和尺寸，处理主/备锚点、VDS 续链、FSD、FE/EFE、FID 与 ICB。物理、sparable、VAT 1.50/2.x、metadata/mirror 分区映射分别准入。Sparable 分区的标准 `*UDF Non-Allocatable Space` 系统记录按 OSTA 3.3.7.2 验证名称、类型、Metadata/System/Stream 标志、零信息长度和未记录尾分配，仅作系统元数据，不成为输出条目。短、长、扩展 AD 及续接按实际 recorded/information 长度解释，支持内联、未记录零区段和非连续内容，不将数据范围相加后假设连续。目录图、分配续链及映射循环均有界，CS0 严格转 Unicode。
- UDF 普通文件的命名 streams、未知系统 streams、transformed 数据、未知映射/ICB 策略和不能解释的 extended AD 长度组合明确拒绝。实时文件及其他特殊类型标为不可输出，符号链接和多链接普通文件同样拒绝输出。已识别 UDF 的失败不会改用 ISO；VDS/metadata 的规范内冗余恢复不改变所选命名空间。
- EntryId 按选定命名空间的原始目录记录遍历顺序分配，不按解码名称排序或合并重名记录，不将文件数据的扇区偏移充作条目身份。解析结果保存数据、零区段和内联内容的独立 extent 列表；负载打开时重扫并核对冻结元数据，解码时核对条目身份、范围与精确输出长度。没有逐文件 CRC 的格式不补造内容校验保证，同尺寸内容变化可能仍然可读；宿主来源身份检查继续独立执行。
- 元数据读取最多 64 MiB，普通条目最多 100,000，单名称最多 4,096 字节，总名称和展示文本使用共享扫描预算；总输出 extent 最多 1,000,000，内联总量最多 64 MiB。ISO 目录深度最多 256，单记录 CE 最多 128 个区域、1 MiB；宿主可输出路径仍受更严格的共享限制。UDF 分区映射最多 256、单分配表最多 262,144 个 extent、续链最多 4,096。所有加法、乘法和源范围均检查溢出。
- 实际解码保留单文件 8 GiB、累计工作及输入各 20 GiB 限制，以有界缓冲读取和输出。`xdzip-optical-bridge` 在既有工作进程内通过同步借用的 CXX 回调传递授权流；异常与 unwind 不跨边界，进程终止使未完成输出失败。宿主保留内存治理、取消、强制终止、来源复验、私有暂存及最终提交。官方 7-Zip Iso/Udf 处理器从构建和注册表移除，不保留运行时回退。

独立样本必须具有可核验的生成工具或来源及已知内容；自编码规范向量只验证实现的内部状态组合。metadata、VAT、sparable 等扩展的独立实样与三平台验收单独登记于[迁移任务 #47](https://github.com/xiaodaozhi/xdzip/issues/47)，不能由代码存在或物理分区样本通过推定其他映射已验收。测试工具、参考 worker 与产品 worker 分别记录身份，不把参考工具不支持的样本记为验证通过。

English: ISO/UDF parsing is independently implemented in Rust from ECMA-119, SUSP/Rock Ridge, El Torito, ECMA-167 and OSTA UDF specifications. The existing isolated worker calls it through a thin CXX boundary; official Iso/Udf handlers are excluded, with no runtime fallback. Physical records and raw names are preserved, metadata is rechecked before bounded extent streaming, and the host retains source validation, cancellation, resource limits, staging and publication. Images are read-only and checks establish structure and readability, not universal per-file content authenticity. The standard non-allocatable-space system record on sparable partitions is structurally validated as metadata. Unknown mandatory features, transformed data, ordinary named streams and unknown system streams are rejected; links and special objects cannot be output. Independent fixture coverage for each partition map and platform is tracked separately in issue #47.

## 能力与验证基线

| 后端 | 必须保持的能力 | 独立实现要素 |
| --- | --- | --- |
| ZIP | Store/Deflate、ZIP64、AE-1/AE-2 AES-256、现有分卷、创建与普通包修改 | 记录、额外字段、数据描述符、加密封装、共享 Deflate |
| TAR、TAR.GZ | ustar、既有 PAX/GNU、普通文件与目录、创建与修改、多成员 GZIP | 记录层、扩展状态、流式写入，GZIP 封装与共享 Deflate |
| 7z | Store、LZMA/LZMA2、PPMd、BZip2/Deflate 解码、BCJ/BCJ2、分支与 Delta 过滤器、固实、密码与头加密、编号分卷、创建与受限重建 | 解码图、实际准入的方法组合，资源与编码策略；不因底层有编码器而扩大产品创建选项 |
| RAR | 旧版、RAR5、RAR7 的既有只读边界、密码、固实和分卷 | 分版本解码器、过滤器、加密封装与状态重置 |
| CAB | Store、MSZIP、Quantum、LZX、同目录卷链 | 记录、文件与文件夹映射、跨块字典和跨卷文件 |
| ISO、UDF | ISO9660、Joliet、Rock Ridge、UDF、混合映像选择 | 卷及分区映射、命名空间、区段、目录图和特殊项限制 |

每项切换需要双向互操作（只读格式为独立样本读取）、损坏与截断、数值溢出、资源限制、取消和真实宿主业务链回归。自研编码器与解码器互相成功不足以证明兼容。模糊测试和基准使用固定来源，记录实际覆盖和时间，有限随机变异测试不冒充长期 fuzz。Windows、Linux、Apple Silicon 的自动检查与原生人工验收分别登记，正式发行仍需独立许可复核。
