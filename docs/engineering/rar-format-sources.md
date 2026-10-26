# RAR 独立实现依据与兼容边界

RAR 自研迁移保留旧版、RAR5、RAR7、密码、头部加密、固实与授权分卷的完整只读范围，不开发 RAR 编码器。能力替代及官方依赖退出仍由[迁移任务 #51](https://github.com/xiaodaozhi/xdzip/issues/51)验收；容器代码或 Store 读取存在不代表完整迁移。现有产品路由与来源见[隔离后端契约](native-codecs.md)。

## 来源和角色

现代容器字段依据 RARLAB 的 [RAR 5.0 格式技术说明](https://www.rarlab.com/technote.htm)，RAR7 算法辨识参考 [UnRAR7 说明](https://www.rarlab.com/unrar7notes.htm)。技术说明并未完整公布压缩、历史密码、VM 或 checksum tweak 算法，不能从方法名称推导缺失状态机。在线技术说明没有独立版本号或签名，不把保存的 HTML 当作上游签名发布。

旧容器及官方技术说明未覆盖的接受行为，经独立分析者按固定 UnRAR 来源形成纯事实交接。固定参考的上游内部版本为 7.30 beta1，下载归档文件名为 `unrarsrc-7.3.1.tar.gz`，不据文件名另推上游发行版本。归档 SHA-256 为 `634900842a3737d9cc15bbcc71d4c74cc713437e0bca296a573424fe5f2660ab`，许可为 LicenseRef-UnRAR，精确原始许可与来源说明见[有限映射材料](../../third_party/rar-wire-mappings/README.md)。容器、现代 LZ/过滤器、固实与密码分别交接，不据某一稿存在宣称其他算法已经完成分析。

English: The pinned reference identifies itself as UnRAR 7.30 beta1. Its download filename is unrarsrc-7.3.1.tar.gz; that filename does not establish a separate upstream release version. The exact original notice and pinned archive provenance are preserved separately for the finite mappings, without inferring legal clearance.

容器初次实现时，实现者未阅读该参考核心，使用官方文档和不含参考代码表达的数学交接 `RAR-CONTAINER-2026-10-26-v1`。冻结容器事实稿 SHA-256 为 `116ed2926619ea3009f00a6b6991f5d939581bca30a2060df2c77c51724f9839`，Unicode 补充边界为 `b456b85b59e97c96c78c1936df6d46c893a9f9b26cc61511cfc411dec20a2750`，来源记录为 `b68b0def40ca7b7f3220fa6caf928f239113366bf9a1969372c52063ce240827`。后续旧服务兼容补充中，原容器作者在代码冻结后受限阅读对应头解析及字节读取范围，转为事实分析者并停止修改对应容器；未暴露的另一实现者依据数值事实独立完成最终服务准入。旧服务补充主文 SHA-256 为 `2554d6752cd5c8e0f59f97c5a90a9291ba2d8908960a65f50e1c5b54301bad08`，冻结清单为 `39cad28299020679ab1a8e98e723c3eb8310008861bcea9c6ba76aa80f8fc853`。生成的事实稿、来源范围与验证报告作为本地或 CI 工件保存，不复制参考代码、结构、注释或逐行语言转换。不宣称严格洁净室，也不将独立实现视为许可结论；参考工具及既有交付仍履行各自许可义务。

现代 LZ 使用纯事实交接 `RAR5-RAR7-LZ-FACTS-2026-10-26-v1`，总清单 SHA-256 为 `b86978fb63c90c38b4dc0fa0831a27598fff12407f561d25fef8d923d83f8ea5`。其中 LZ core 为 `02e31d56e5a1538d5b0d72222908e2dcc626259e68ec00e6d32e11b5cb92df13`，接受边界补充为 `7fdef948fdd8ab82376ed1aa8f1a3bba6016569fb1c036f5e0ec53f8f37d0e49`，过滤器为 `09933ca42c68676a63092a3de9fecb38dda4086cac13a1b7a63f9bc0f818fb30`，RAR7 差异为 `43ada71358a7df4f2c6ed84f0e7661dfeb16169e7f6d1bee3cebe46f6243139c`。Store/窗口补充 `RAR-SOLID-STORE-WINDOW-2026-10-26-v1` 为 `37de8e2d60a96ba92c0e4b400c77556d15a0adc448bc94e4328144979d0713f4`。实现者没有读取对应参考核心，不开发 RAR 压缩编码器；固定声明式数学位向量是测试数据，不能冒充独立作者压缩样本。

大逻辑字典的 Constant/Raw 存储由项目依据自身预算、顺序写入和固实不变量独立设计，实现者读取本项目设计及数值事实，没有阅读对应第三方解码器。真实 RAR7 测试数据的分析者只将固定归档身份、头部数值、已知全零配方和工具记录交给实现者；测试入口下载压缩数据及原始 notice，不下载上游实现或 README。测试数据来源及许可边界保留在脚本生成的材料记录中，不能据此推导法律结论。English: The large-window representation is an original project design based on its own memory and streaming invariants. Implementers receive numerical fixture facts rather than third-party decoder code; build-only test data and its notice retain separate provenance and license review.

现代 crypto version 0 采用纯事实交接 `RAR5-CRYPTO-FREEZE-2026-10-26-v1`，密码/封装主文为 `029f03e1cf7dff2ae804fb99f7dbacb8ac0306498476119a4d851cbe9a52e9fb`，BLAKE2sp profile 为 `1574159e24c956a2287d605501dff63f1ee67d0028cfd9fe3dadcdbbeee8f795`，来源记录为 `e13a21f12cc3df777d8d1d2332e38d9e525aceed02ddb0ce93a54c4cf8a2577a`。共享 `xdzip-crypto` 提供原创 SHA256 状态封装、RFC2104 HMAC 与 BLAKE2s/sp 组合，AES 与 SHA2 压缩原语遵循通用依赖准入，不复写参考的轮函数或表。

RAR29 PPM 使用纯事实交接 `RAR29-PPM-2026-10-26-v1`，主文 SHA-256 为 `9425e386ee75acac31f843d61480b02dbba7ed56ae6ef550e45e54fd8ee32114`，来源记录为 `4de014dc4cf1de581d32e10c6b166bc2e14542c04ca1cc6cf46cee1dced39482`。文件重入、packed 尾及 escape 补充 `RAR29-PPM-BOUNDARIES-2026-10-26-v1` 主文为 `540795a89e33a2bbf43b3317f30b8c0f0550884740f0230594f33cf949a9aa92`，独立 freeze 为 `e3b6e475bd2c7438de3c366c19d2f156bcaec6ca7426c250aff1e61eb04ae8e5`。实现者复用本项目原创 PPMd H 统计模型及其数学事实澄清，未阅读、翻译第三方对应模型、range 或分配器实现。原 PPMd 部件的 public-domain 声明不自动重新许可 RAR 封装或其他修改；参考工具与公开 fixture 保留原有来源和许可。

RAR29 LZ 与六种标准过滤器分别依据纯事实主文 `e375c541e106d43a3e58a2fe8aab79f60ffc71beff08b76c2a66f26d0fd355cb`、`eeecbd9a247d4a8c98e3d1cd11371106c16fbb5bd565406731908241e7432d8a`，旧密码主文为 `9650bce5affb20ea4d616e1e68c25d52e5fa703bd0b7c56b3915fad825abdcb4`。古老容器补充 `RAR14-CONTAINER-2026-10-26-v1` 主文为 `d3a86f3cad6e1893959d40cc691e7c7d4ddfec4b96abc9f4ba12ea98fd7f484c`，独立冻结清单为 `e6adcbf16ad688c82ad5bd39ae7cdd4379003ae82dd19b30b4dd53ce52223f9c`。这些资料只交接数值、边界与接受行为；各实现者在各自首次实现相应模块时未阅读对应 UnRAR 核心，固定公开归档与官方工具仍仅作为独立黑盒对照。

13/15 族依据旧算法事实主文 `d4cd5d61ee4b16fa7202cb6840d0e9592d67205c4fdbfd02f3e515320ebf96ef`，有限前缀映射和 Short 映射以独立数据表交接，SHA-256 分别为 `2e50aa5c3c9f97609cf7b7c91676b7266de9ca65ee977f9184c42ec104c9edeb`、`31d4186b3dbf06bdbf5aabd5f08cfa800bc810709cfd042fe6f32ef22c9cad1b`。数据表来源仍是固定 LicenseRef-UnRAR 参考，不把有限线映射、独立 CSV 或原创 Rust 组织方式当作版权或再许可结论。

旧容器例外与 Windows 宿主字符投影采用追加冻结 `RAR-COMPATIBILITY-GAPS-2026-10-26-v1`，事实主文为 `142e47f9a3c2920e0e0c674555df669ecde0bc29549a9a45f03133d11957a4ef`。Unix/macOS 的追加事实 `RAR-COMPATIBILITY-GAPS-UNIX-2026-10-26-v2` 为 `3853684c35ee672956ccaadb3efc1f1fffb58e0a7fa472d41c261fbacd69a479`。时间投影事实为 `2ea490c2304c8fe5188af371d77fe9013b23ae66e6ff4fe05cf489cb7c914a20`。纯系统服务只做本地字符或日历转换，格式选择、限长、截断和 DOS 数学留在 Rust；事实中的参考未检查系统失败行为不成为返回未初始化数据的许可。

来源隔离按实际暴露范围记录。现代 ARM 实现及数学测试完成后，一次测试数据搜索的结果摘要意外展示了另一第三方解码器的 ARM 表达片段。既有 ARM 实现随后冻结，该实现者不再修改对应算法；如需修正，由未暴露的实现者依据事实交接完成。对应解压核心的未阅读声明不扩展为从未见过任何第三方片段，也不覆盖上述后来受限阅读的服务容器；具体查询、结果范围与先后证据保留为本地工件。

## 容器候选契约

`xdzip-codec-rar` 依赖项目的 codec API、domain、共享 compression/crypto，以及已准入的通用 AES、SHA2 压缩原语与归零设施。CRC32 复用共享实现，不引入第三方 RAR、归档或压缩格式库。它实现 `MetadataCodec`/`PayloadCodec`，经共享 CXX 桥接在既有隔离 worker 中执行，没有官方失败回退。该接线本身不替代完整兼容验收，产品路由切换仍遵守[迁移契约](independent-rust-codecs.md#实现边界)。

- 宿主提供有序授权流及每个物理卷的长度，最多 1,024 卷。卷内范围先检查再换算绝对位置，包内名称不赋予路径访问权限。每卷重新识别头部，检查格式代、卷号、头加密状态及继续关系；只有显式 split-before/after 合并，普通同名记录保留独立身份。普通文件与 service 继续链分别保存，service 不占普通 EntryId。
- 1.5..4 普通头低 16 位校验来自通常 CRC32，现代头保存完整 CRC32；更早的 1.4 容器没有头 CRC，其独立内容 checksum 不能混用。先验证普通明文头校验，再解释有界字段与 extra。现代 vint 最多 10 字节，最后字节只允许有效位 0/1，允许数学值正确的非最短表示；head-size 自身最多 3 字节，明文总头最多 2 MiB。未知可跳 extra 按其边界跳过，未知必要解码特性不能假作普通内容成功。
- 私有索引保留每片原始头、卷/头/packed 范围、known/unknown 尺寸、原始名称、解码版本、压缩强度、声明/生效算法、字典、solid、时间、类型、CRC/hash 与密码描述。公开 method 保留 UnpVer，例如 50 或 70；公开 packed_size 为首次可见片的 packed 大小，不替换成所有片总和。EntryId 按首次可见普通记录顺序从 1 连续分配。
- 旧名称先按声明 Unicode 的数学命令解析，否则调用共享归档级旧编码检测。保留原始名称区，字符解码完成后才转换旧路径分隔符，避免误拆多字节字符中的 `0x5c`。Unicode surrogate 必须完整配对，拒绝截短命令、嵌入 NUL 及越界复制。RAR5 严格验证声明 UTF-8，映射 Unix 名字和 redirection 保留警告，链接及特殊对象不能普通输出。
- 元数据读取最多 64 MiB，物理头记录最多 100,000，单名称最多 4,096 字节，原始名称合计及解码/展示各最多 16 MiB，调用方更低限制优先。负载重扫继续累计原有输入余额，实际输入和解码工作各最多 20 GiB，单文件最多 8 GiB；算法工作区在执行前按默认 1 GiB 内存上限准入。
- 算法注入接口只接收有界明文 packed 流、借用控制与宿主 sink。容器负责片级 packed 校验、解密、完整输入消费、精确文件尺寸及最终完整内容校验，算法负责其状态、压缩流终止和自身内存预检。候选执行 Store、13/15 族、20/26 族、RAR29 LZ/PPM 与现代 LZ，按不超过 8 KiB 的块输出，不累积整个文件。目录仅在零尺寸、零 packed、原头复验及所需空内容校验成功后完成；名称 warning、link 与特殊记录先拒绝，不能绕过公共条目准入直接输出。
- 识别到的领域 I/O 错误先保留其类别，只有没有包装错误的裸 Interrupted 可以重试。取消、配额、来源改变、I/O、输出失败或校验失败后实例 poison；不会把失败当 EOF、entry_done 或发布资格。宿主持有来源身份、路径检查、私有暂存及最终提交。公共 DecodeError 暂无独立 SourceChanged 变体，映射为 SourceIo，来源宿主保留具体原因。

## 现代 LZ 候选规格

- 实际算法 0 使用 64 距离符号，算法 1 使用 80 距离符号。声明版本 1 的兼容位仍选择实际算法 0，不能按工具版本或文件扩展名选树。独立 MSB 位读取与规范 Huffman 树读取声明有效位，不复用 Deflate 的 LSB 码序。新表的目标码长分别为 430/446 项，未使用的树可以为空，消费不存在代码、过度订阅或越界游程明确失败。
- 块头校验、末字节有效位与最后块标志共同决定位流终态。已知输出尺寸、最终内容 CRC、完整 packed 消费各自检查，不能在产生声明大小时提前成功。空块不做无符号下溢；没有有效表不能解码内容。候选对越界游程、未完成过滤范围、尾随 packed 字节等采取严格策略，健康样本通过不证明这些策略与所有历史恢复行为一致。
- 非 solid 压缩文件重置字典有效历史、最近距离、上次长度和表；solid 压缩文件保留它们，每文件仍重置输入块、过滤器和文件偏移。采用首次压缩文件确定的有效窗口，最小工作窗口为 256 KiB；非二次幂窗口按真实模数环绕。后续较小或非幂声明不改变既有模数，中途增大明确 Unsupported，不复制依赖旧分配缓存的坏链接受例外。非 solid 更换窗口时先释放旧窗口，避免两个大窗口同时存活。
- Store 自身无论 solid 位是否置位都只复制并校验文件，不向 LZ 字典加入内容，也不清空表、最近距离或上次长度。目录和 service 不成为新的压缩初始化点。随机选择 solid 文件从前一个非 solid 压缩文件恢复，按物理顺序实际解码前缀；Store 不是恢复点，跳过内容和重复重放都计入实际工作。
- Delta、x86 E8、x86 E8/E9 与 ARM 过滤器处理独立的完整逻辑区间。过滤前字节始终作为 LZ 历史，变换后的副本才交给文件 CRC 与 sink。区间可跨块/物理卷，不能跨文件；起点及文件偏移不与环形下标混淆。单区间最多 4 MiB、挂起项最多 8,192，重叠、倒序、未知变换及不完整区间拒绝。资源上限是候选治理策略，不声称为格式极值。
- 指令批次最多 8 KiB 计划输出或 256 条指令。执行前仅在本地预留剩余工作配额，不向公共 sink 报告尚未执行的进度。执行后，包括部分失败和取消前已完成的字节，只结算真实 WORK 并转换为单次预计费额度；未执行的预留不能成为输出额度，失败后不复用。完整过滤区间暂存期间也持续累计实际工作。预计费交付前扣减容器私有额度，最终尺寸、CRC、取消和写入失败仍检查；默认 sink 可能保守再次计费。Store 保持普通输出计量。worker 通过 PRECHARGED_DATA 交付已有额度的 RAR 内容，Native 接收端与运行时分别核验宿主额度、精确尺寸及配额；重启、结束或失败后不能复用旧 worker 额度。
- LZ 字典之外预留 288 MiB，用于有界元数据、过滤副本、队列及其他状态，合计先按 1 GiB worker 上限准入。它是保守内存政策，不是线格式最大字典，不能把格式可声明尺寸当可分配许可。取消和错误使整个负载状态 poison，不以保留的半字典继续另一条目。

### 大逻辑字典的精确存储

现代实际算法 1 的压缩文件在完整字典超出连续存储准入时，采用精确 Constant/Raw 分页历史，逻辑窗口最多 1 TiB。页尺寸为 64 KiB，每页独立保存一个常量字节或独占原始缓冲，读写和距离仍按原逻辑窗口的真实模数执行。尚未产生的历史不可读，重叠匹配逐字节执行；历史保存过滤前内容，不与最终输出缓冲别名。其他算法、Store、未知方法和服务记录不获得大字典准入例外。

分页目录、槽表、空闲池、所有保留容量、临时峰值、私有状态及 16 MiB allocator allowance 合计最多 512 MiB，外围 288 MiB 与密码状态另计，worker 的 1 GiB 上限保持不变。每次分配之前检查预算，不能按逻辑窗口预留巨大地址空间或转为磁盘历史。常量页发生不同值写入前完整初始化一个 Raw 页，只有完整顺序覆盖该页的有效长度且所有新字节相同时才折叠。半页的固实续写保留原尾部与候选状态，非固实替换先释放旧存储。归还池中缓冲仍按真实容量计费。

目录、槽表和 Raw 页的实际初始化使用独立辅助工作通道，按最多 8 KiB 分块检查取消并结算；它不增加文件进度或交付信用。写入先完成可失败的存储准备，再接受输出字节，最后无分配地提交历史。输出拒绝时不改变历史字节或游标，已执行的初始化不退款，错误仍使负载 poison。该表示可高效处理常量页，不保证任意低熵流或任意大字典都能在预算内完成；Raw 工作集过大仍明确 Limit。

真实算法 1 的互操作检查仅在 build/CI 获取固定来源的压缩测试数据与 notice，检查原始头身份以及全部原文 SHA-256、CRC、尺寸、WORK、AUX 和保留容量；不保存大明文或第三方实现。数学页测试与独立健康压缩流分别记录，单个成功流不证明所有扩展距离或加密、固实、跨卷组合已验收。

English: Modern effective algorithm 1 compressed files whose complete dictionary exceeds dense admission use exact 64 KiB Constant/owned-Raw pages, with at most a 1 TiB logical window. The true modulus, available-history checks, bytewise overlap and prefilter history remain unchanged. Directory, pools, retained capacities, temporary peaks, private state and a 16 MiB allocator allowance share a 512 MiB cap; the existing 288 MiB outer allowance, crypto state and 1 GiB worker limit remain separate. Every allocation and actual initialization is admitted and auxiliary work never funds file delivery. Output acceptance precedes the infallible history commit; errors poison the payload and completed work is not refunded. Only complete constant page rewrites fold; partial solid rewrites retain their tails, and retained free buffers remain charged. Raw working sets may still exceed the budget. Other algorithms, Store and services receive no large-dictionary exception. Build-only pinned data checks and mathematical page tests remain distinct from complete stream and platform acceptance.

现代 x86 的独立已知原文可按[受控 PE 数据配方](../../crates/xdzip-codec-rar/tests/fixtures/modern-x86-owned/README.md)验证。官方作者正常生成的流须实际选择 E8 或 E8/E9，并同时通过直接 LZ、Payload 和真实 worker 的完整内容核对；PE 名称、工具版本或描述符计数本身不足以证明成功。数据输入不执行，不在仓库保存 PE 或生成归档，其他固实、加密、跨卷组合仍分别验收。

## RAR29 PPM 候选规格

- `xdzip-compression` 共享有序统计上下文、二元概率、SEE、排除、后继实体化及名义容量账本。RAR29 和 7z 通过不同的 range 适配器使用同一模型，不复制第二套统计实现。RAR 采用四字节 carryless 初值、独立归一化及双分支丢弃除法余数的二元区间；7z 保持原有五字节初值、余数和终态规则。
- 模式头从已对齐的完整 F 字节开始。reset 位决定整个模型是否重建，内存字段表示 1..256 MiB 名义容量，order 按头字段映射至可编码的 2..64 范围。可选 escape 仅在声明时更新，reset 不把它自行恢复为默认值。保留请求必须已有连续性明确的有效模型；每次模式初始化仍重建四字节 range，不能用无头的跨文件续读替代。独立随机起点缺失此前模型时明确失败。
- 名义 12 字节单位及确定性等级、碎片、合并和借用规则控制格式规定的模型重启；Rust 对象、历史、索引、遍历 scratch、固定状态及其他解码状态另外按真实存储准入。reset 先释放旧 arena，避免双份大分配。容量重启保留当前已经解出的 symbol 和 range；实际 OOM、配额、取消、源 I/O 或模型不变量错误均为终止失败，不能冒充容量重启。
- 新 arena 初始化和容量重启先准入辅助工作预算，实际填充或清空按最多 64 KiB 的存储块及有界概率表行执行，检查取消并累计已完成的初始化字节。后续 range 读取失败、取消和 `clear` 都不退回已经执行的工作。外层在成功与失败路径结算实际差额，与控制及参数 symbol 的辅助工作共同计入累计 20 GiB 上限；独立 AUX 通道不产生可交付文件字节的预计费额度，未来预留也不冒充实际进度。
- PPM 每次交付一个完成统计更新并归一化的模型字节，escape、过滤器参数和匹配参数均正常更新模型。RLE 或 LZ 复制产生的文件字节不再次喂回 PPM。文件终止、模式切换、输出尺寸、内容 CRC 和声明 packed 边界由外层核验；RAR 不接受逃逸出根作为 7z END，也不施加 7z 的末尾 code-zero 条件。失败模型保持 poison，只有外层明确恢复时才清除状态。
- 实际解出 PPM EOF 后，外层排空受完整声明 packed 范围约束的剩余输入，继续检查输入配额、取消和分片校验。PPM 算术尾不套用现代 CBC 的 0..15 字节对齐限制；没有真实 EOF、尺寸或 CRC 不符时不能借尾部政策成功。PPM EOF 后下一个 solid 文件重新读取模式头和新 range，reset 位单独决定是否保留统计模型。固定参考对 escape 后控制 1 及 6..255 均输出一个字面 escape；若候选严格拒绝 6..255，应单独登记接受范围差异，不能冒称参考也拒绝。

English: RAR29 and 7z share the project's original PPMd H statistics and deterministic nominal-capacity ledger through distinct range adapters. RAR retains four-byte carryless initialization and its own interval arithmetic, header/reset and termination rules. Nominal capacity remains separate from admitted Rust storage. Arena initialization and model restarts admit work first, record only completed bounded chunks and retain that cost after cancellation, later failures or explicit clearing. Initialization, control and parameter work does not fund delivery credit. Host allocation, quota, cancellation and I/O errors remain terminal instead of triggering a format restart. After a real PPM EOF, the wrapper drains only the admitted packed remainder with normal input, cancellation and fragment checks. A following solid file rereads its mode header and range; statistics follow the reset bit. Unknown escape controls 6..255 produce a literal escape in the reference, so any stricter rejection remains an explicit compatibility difference. The implementation uses code-free facts and original project code; component provenance does not imply a new license for RAR changes or completed format/platform acceptance.

## RAR29 LZ 与标准过滤器候选规格

- RAR29 独立读取 MSB 位流，404 个差分码长分成 299/60/17/28 符号树。旧码长基线、低四位距离重复、最近四距离和上次匹配长度在压缩状态内保存，不把现代 64/80 距离表套入旧算法。文件终止符同时控制下一固实文件是否重读模式；未初始化的复用、空树消费、越界游程、输出越界及历史之前的距离均失败。
- PPM 外层 escape 控制文件终止、模式切换、字面 escape、过滤器与两种复制。参数字节仍经模型解码，匹配输出只进入 LZ 历史与最终文件流，不反向修改统计模型。PPM EOF 的剩余 packed 排空政策与纯 LZ 的位流终态分别处理。
- 标准过滤器程序先检查有界 packet、程序长度、XOR 与固定 CRC/长度身份，仅选择 E8、E8/E9、Itanium、Delta、RGB、Audio 六种已有产品基线变换。候选不执行任意包内 VM。定义、缓存长度及最后选择保留固实状态，每文件重新计算偏移，未完成区间不能被 reset 丢弃。
- 未过滤字节保持为 LZ 历史，相同起点和长度的标准变换可串联，其他重叠、倒序及未完成区间拒绝。单区间长度 1..262,143 字节，最多 8,192 个定义和挂起项，是候选资源政策。Delta 通道最多 1,024，Audio 最多 128；各自线参数和初始化缓冲均先准入。
- 旧 E8 的 PC 使用完整 32 位文件位置，不按现代规则取 16 MiB 模。Itanium 按原始 bundle/template 和指令字段变换，RGB 与 Audio 各有独立预测状态。Audio 包含首个采样的适配及负系数到 -17 的边界，不能由现代 Delta 或 E8 数学向量代替。
- 每个实际变换字节、PPM 参数与实际模型初始化均扣辅助工作预算。辅助工作通过独立 `charge_auxiliary` 通道、至多 64 KiB 的非零消息结算，不能增加 decoded_bytes、膨胀率分子或交付 credit，跳过固实前缀仍转发该通道。worker、Native 接收端与运行时分别执行[辅助工作契约](auxiliary-decoding-work.md)；将辅助量交给 WORK 的早期候选不满足该计量契约。处理循环分块检查取消，失败前已完成的辅助工作不退款。

English: RAR29 owns its MSB differential tables, 60 distance symbols, solid state and LZ/PPM transitions. Only six recognized standard filter identities are admitted; no arbitrary archive VM executes. Prefilter bytes remain history, exact matching ranges may chain, and final transformed bytes feed content integrity. Bounded parameters, queued intervals, allocation and actual transformation work are checked separately. The worker, Native receiver and runtime independently enforce auxiliary accounting without increasing decoded byte counts or delivery credit; routing auxiliary work through WORK violates this contract.

## 古老容器与历史密码候选规格

- 1.4 主头最小 7 字节包含 `RE~^` 标记，文件头固定 21 字节后跟有界名称和可跳额外材料，payload 从声明 headSize 末尾开始。main 的 0x01/0x08 分别表示卷和全局 solid；原档的 0x80 不是后代头加密位。文件只用 0x01/0x02/0x04 表示 split-before/after/加密，目录来自 DOS 属性，不复用后代文件窗口或 solid 位。字典固定 64 KiB。
- 古老方法值直接使用 0..5，selector 2 映为公开 UnpVer13，其余映为 10。压缩准入仅开放有既有基线的 13/15，10 只允许 Store。全局 solid 从第二个逻辑文件开始延续状态，物理继续片继承同一逻辑文件状态；新文件重置位读取和内容校验，Store 不向压缩字典注入内容。
- 每个古老逻辑文件从 16 位 c=0 开始，对实际最终输出 byte x 计算 `c=ROTL16((c+x) mod65536,1)`，最后直接比较最终头的 u16，不末 XOR。中间片 u16 原值保留，但不作为 packed CRC；完整内容 checksum 连续跨片，solid 前文件的 checksum 不传给下一文件。后代容器的同代算法仍使用其普通 CRC32。
- 古老加密始终选择 old13 字节密码族，无 salt/IV，selector10 不改变它。后代 13/15 使用各自字节族，20/26 使用旧分组族，RAR3 使用原创 SHA1 派生组合和 AES128-CBC。每逻辑文件 fresh cipher，物理 split 片连续，密码状态不随压缩固实状态跨文件。old13/15 不接受块对齐尾，旧分组密码的 Store 根据真实尺寸截取，RAR29 PPM 的声明尾政策另外适用。
- RAR3 加密头每头重新读取 8 字节 salt，先解首块取得真实头长，再核验本卷完整对齐跨度及普通头 CRC。保留原始 salt/密文供负载复验。外层先检查 4,096 UTF8 字节，旧路径另外保持原 native callback 的完整输入最多 511 个本地宽单元，包括首个 NUL 后的输入；之后才取首个 NUL 前内容并应用 crypt 的 127 单元边界。它是历史宿主入口限制，不是格式密码长度规范，现代 KDF 的现有入口政策另列。
- Windows 仅对非古老容器且 HostDOS 的旧文件，先将零填充完整 512 UTF16 单元送宿主 OEM 到 ANSI 宽投影，再在宽索引 127 截止。旧 13/15/20 接着经 ANSI 编码到 512 字节缓冲，再在原始 byte127 截止，允许落在 DBCS 中间；RAR3 直接使用投影后的宽单元低 16 位。古老内容不执行后代 DOS 补偿。转换使用同步借用宿主服务，系统失败明确 Unsupported，typed 来源错误及取消保持原类别；无服务时仅使用可证明相同的 ASCII 路径。
- macOS 的旧窄密码先取 127 Unicode scalar，再明确编码 UTF8，最后原始 byte127 截止，允许形成不完整尾序列。普通 Linux 使用当前线程 locale 的有状态转换，EILSEQ 重试后保留实际前缀；本项目独立 worker 未为此切换进程 locale。带 U+FFFE 的 Linux 恢复路径在 Rust 移除标记、恢复 U+E080..E0FF 原 byte，普通单元仍交给宿主转换，不能把该协议套到 Apple 或无标记字符串。RAR3 非 Windows 宽投影保留原始 scalar 前缀，再取单元低 16 位，低 16 位为零不被误作原始 NUL。中间宽、窄、单字符及密钥缓冲均归零，固定投影 scratch 单独预算 4 KiB。
- 没有可信独立 verifier 时，头 Invalid 或内容 Integrity 返回 PasswordOrIntegrity，供宿主安全重试；SourceChanged/SourceIo、Limit、Cancelled、Output、Unsupported 不被包装成密码歧义。已验证的现代 check mismatch 仍是 WrongPassword，已验证密钥下的损坏仍是完整性错误。物理原头复验变化按来源错误处理，不据密文变化误报密码。

English: Ancient headers keep their own method, flags, fixed window and 16-bit final content checksum. Only explicit continuations merge, and final integrity spans all fragments. Compressed UnpVer10 remains unsupported while Store10 retains old13 encryption. Legacy ciphers restart per logical file; RAR3 encrypted headers preserve original salt and ciphertext. Legacy input first keeps the historical 511 native-wide-unit callback limit, then applies NUL and crypt-prefix rules. Windows conditionally projects complete DOS passwords before wide and raw-byte truncation. Apple explicitly uses UTF8; Linux retains locale-prefix and marked-byte-recovery semantics through borrowed host services. Intermediate buffers are erased. Unverified integrity failures remain PasswordOrIntegrity, while source, quota, cancellation, output and capability errors retain their classes. These candidates do not establish complete historical compatibility.

## 历史辅助头与前缀候选规格

- 嵌入旧 comment 时，main parent CRC 只覆盖固定 13 字节，file parent CRC 覆盖实际消费的固定字段、LARGE、名称、salt 和已解释的前三种扩展时间；archive-time 槽不消费。type75 child 独立验证固定 13 字节 header CRC，并限制在父 HeadSize 中。注释内容可有界跳过，不声称验证其内容 CRC，也不把参考后续 full-read/BrokenHeader 恢复流程翻译成格式规则。
- AV76、SIGN79 只按 common7、LONG 时额外 dataSize4 和声明数据范围跳过；UO 仅 type77+subtype0x0101 精确免 generic CRC，按公共 14 字节和声明范围定位。其他 subtype 和普通文件仍执行正常头 CRC，不扫描 NUL 猜测历史字符串尾。
- END 在可选 archiveCRC/卷号后跳过已声明 opaque 尾。只有 END+REVSPACE 的正常 CRC 失败，且本次读取结束前七个物理原字节全零，才能使用精确历史豁免。它相对当前物理头末尾，不相对卷 EOF；头加密时检查 salt/cipher 物理区间的末七字节，不能检查解密后的逻辑尾或消费 END 后 trailer。加密 REV 仍缺独立实样。
- 古老 SFX 在偏移 1 保留特殊接受路径；偏移至少 2 且实际扫描长度超过 31 字节时要求绝对 byte28..31 为 RSFX。完整授权搜索的古老标记最大起点为 4 MiB−19。候选有界补齐授权流的短读，不复制旧 native 单次 64 KiB 返回量造成的更窄搜索范围。前缀永不执行。
- Windows 旧 Unicode 无修正高位 run 按已验证的 signed-char 到 16 位单元扩展，带修正的 run 仍按低 byte 数学处理，二者不能混用。非 Windows 对该高位无修正分支暂保留 Unsupported，等待本地 ABI 与宽转换实样；公共字符串仍须为有效标量且经过共享名称安全政策。

English: Historical comments use separate parent and child header CRC ranges; ignored comment contents do not receive an integrity guarantee. Only AV, SIGN and the exact UO subtype bypass generic header CRC. The REV exception examines the current physical header tail, including ciphertext for encrypted headers, rather than archive EOF. Ancient SFX applies the absolute RSFX marker rule without executing prefixes. Windows high-bit uncorrected Unicode runs retain the measured signed-char extension; other platforms remain explicitly unverified.

## 13/15 族候选规格

- 13 与 15 保留各自公开 UnpVer，进入同一独立自适应解码器。窗口固定 64 KiB，有限规范码表、字面排名、距离排名、标志模型、Short 变体及特殊模式分别表达，不复用后代差分码长表或复制另一套 Deflate/PPM。
- 首个非 solid 压缩文件重置历史和统计；后续 solid 压缩保留字典、排名及匹配状态。每文件仍重新初始化位游标、flags/repeats/special 状态，不能把前文件剩余控制位或特殊模式带到新文件。Store 不调用压缩初始化，也不填充或清空压缩历史。
- 每次实际输出前先检查声明剩余尺寸和有效历史距离，匹配不得越过文件末尾。达到真实 unpacked size 后完整消费宿主授权的剩余 packed 输入，继续计入输入配额和取消，不虚构本代不存在的现代末块标志。没有足够真实位时失败，不用零字节补齐。
- 有界匹配和至多 8 KiB 输出批次先预留本地预算，已执行的字节在成功或失败路径结算，只有原始产生字节可建立一次性交付额度。错误后压缩实例 poison，只有明确 reset 才能使用新的独立状态；solid 起点缺前驱状态时不能推断恢复。

English: UnpVer13 and15 retain their public identities while sharing the original adaptive decoder and fixed64KiB history. Non-solid initialization resets statistics and history; solid files preserve them while each file resets its bit cursor and flags/repeats/special state. Real unpacked size terminates output, and the admitted packed remainder is consumed with normal bounds and cancellation. Missing bits are never synthesized. Bounded actual output is accounted before delivery, and errors poison the state. The finite wire mappings retain their original source and license record without asserting copyright clearance.

## 现代密码与完整性候选规格

- 明文 archive crypt 只允许在本卷 main 前出现一次。后续每头以自身 IV 重启 AES256-CBC，先解首块取得真实头长，再检查本卷完整的 IV 与对齐密文跨度。头 CRC 仅覆盖真实字段；对齐尾不解析成 extra。私有物理片保留原始 IV 和密文头，负载重扫逐字核验，不能用解密后的等价元数据代替来源身份。
- 同一 PBKDF2-HMAC-SHA256 链的 `2^L`、再加 16、再加 16 次分别形成 AES key、hash key 与折叠 check，`L<=24`。任务私有 16 槽 cache 及累计预算沿元数据扫描继续进入负载对象，不因每个文件或卷重新获得配额。默认最多 `2^26` 次实际 HMAC 和 4 GiB 派生哈希输入；命中不重复派生，取消前执行的工作不退款。16 KiB 保守 cache/scratch 预算单独计入算法内存预检。
- 外层密码最多 4,096 UTF8 字节，先检查该限制，再在首个 U+0000 截止。native 兼容 profile 在 Windows 截前 127 个 UTF16 单元，在 Unix 截前 127 个 Unicode scalar，再生成派生字节。Windows 截断边界的孤立 surrogate 仅存在于私有兼容字节，不成为公共 str 或名称。该平台差异是参考兼容行为，不冒充格式统一规则。密码、派生中间值和密钥使用归零生命周期，不能承诺擦除系统的全部历史副本。
- 只有 check 自身 SHA256 checksum 有效，且不是旧 service 的全零不可信值，才能据 mismatch 返回 WrongPassword。checksum 坏意味着禁用快速检查，正确密钥仍可凭结构与内容校验成功。无可信 verifier 的 Invalid/Integrity 通过公共 PasswordOrIntegrity 保留歧义，不能把这种失败宣称为可靠 wrong-password。
- 内容 CBC 每文件重启，跨授权 split 片连续，块可由前片末尾与后片开头合成。每片 packed CRC/BLAKE2sp 始终先覆盖物理密文字节，再进入解密；最终内容校验覆盖过滤后的完整明文。片链当前要求完整 crypt 参数一致，这是严格候选政策，健康样本不能证明全部历史分卷写法均等价。
- Store 根据 unpacked size 截取真实内容，LZ 根据完整最后块和有效位终止；其后只允许最小 0..15 字节对齐尾，尾值不限，不解释 PKCS7。LZ 的预读缓冲也参与尾部计数，不能只看外层 CBC 已消费量。完整声明密文仍须消费和计入输入。允许更多整块垃圾的参考恢复行为不在候选成功范围。
- 复用共享 BLAKE2sp32 流式状态。存在已知 BLAKE2sp extra 时选其完整宽度校验，保留基本 CRC 字段但不同时要求二者，这是固定参考的选择语义。flag2 声明 keyed tweak 时，普通 CRC 的 LE4 经 HMAC 后异或八个 LE32，普通 BLAKE2sp32 经完整 HMAC32；无 flag2 时即使加密也不 tweak。未知 hash 仍 Unsupported，CRC/check 均不提供来源真实性。

## RAR20/26 LZ 与 Audio 候选

20/26 族依据纯事实交接 `RAR20-26-2026-10-26-v1`，主文 SHA-256 为 `d240cac882e8120eb5f20c7894395732b9798bd133479604ae79d5bde24b04d5`。实现者未阅读对应 UnRAR、Junrar 或其他第三方 LZ/Audio 算法实现，不提供 RAR 压缩编码器。原始测试归档、已知 WAV、测试断言与许可仅用于独立数据对照。来源搜索曾意外返回第三方通用归档识别代码摘要，未包含或用于本代 LZ/Audio 算法；不据此作严格洁净室声明。

20 与 26 共享原创熵与 Audio 状态机，通过共享有界 MSB 输入、最多 1 MiB 环形历史和实际输出助手解码。新码长表不字节对齐，完整构建后才提交，保留未覆盖的 1,028 项旧基线；坏游程越界严格拒绝。最近四次匹配按时间保留重复距离。Audio 是写入真实 PCM 历史的预测模式，支持 1..4 通道、跨通道差值、每通道第 32 个样本后适配及 -17..16 系数，模式或表切换不清空预测状态，非固实才重置。

容器的 known unpacked size 决定内容边界，匹配不得跨过剩余尺寸。本代没有额外强制 EOF 符号。到达尺寸后，候选按当前物理字节起真实剩余至少五字节的稳定规则观察一个符号，识别下一批表控制时必须完整解析成功，供后续固实文件保留；其他符号不产生额外内容或更新 PCM。然后排空完整已授权 packed 范围，保留输入错误、取消和容器校验。这是明确的旧尾部接受政策，不声称等同于参考工具受缓冲状态影响的所有恢复行为，也不把任意尾部解释为有效内容。失败后实例保持 poison，格式 reset 不解除失败状态。

历史初始化与表控制先预留辅助工作，再结算真正完成的操作，不提供输出 credit；真实生成内容才提供一次交付 credit。候选内存预检包括保守的外围 288 MiB 及保留的历史容量，最多使用 worker 的 1 GiB 实现预算。该数值是实现资源政策，不是格式限制。

固定 Junrar `v8.1.0`、提交 `57091f9ccd43661cf8f12c389917cc24950df707` 的 RAR2.06 加密压缩样本提供完整 87 字节独立内容。另两个原始 RAR20 Boat 归档随小型来源清单保留在 [legacy20 fixtures](../../crates/xdzip-codec-rar/tests/fixtures/legacy20/sources.json)，对应公开 WAV 的完整尺寸、SHA-256 和 CRC，包含实际双通道 Audio 与普通 LZ 对照；原档字节未修改，内容的 CC BY-SA 4.0 归属及上游原始许可分别保留。声明式位向量与独立 Python PCM 数学对照另验模式切换、1..4 通道、适配边界、固实状态、距离、错误、配额和输出失败，不能替代作者实样。固定官方 RAR 2.90 作者程序另以本项目数学原文生成普通、固实和强制 Audio 固实三个原始归档，见 [自有 RAR20 fixtures](../../crates/xdzip-codec-rar/tests/fixtures/legacy20-owned/README.md)。每档三个成员均为真实 UnpVer20，固实后续成员带真实 solid 标志；普通回归对九份完整输出共 591,561 字节逐字节比较，包含全新 reader 的前驱重放，不依赖作者程序或网络。另有官方 2.90 原生生成的 [RAR26 大文件实样](../../crates/xdzip-codec-rar/tests/fixtures/legacy26-owned/README.md)，真实 UnpVer26、method1、LARGE，已知原文为 2 GiB 加 1 字节，归档约 1 MiB。显式 release 回归只使用有界 sink 与增量 SHA，要求完整 CRC、字节数和一次交付额度检查成功，不保存大明文；普通 debug 测试默认忽略这项重负载。所有旧尾表控制转换、其他实际 Audio 通道数及其他分卷、固实、加密组合仍须分别覆盖；修改版本字段的数学向量不计作者版本 26 互操作。

另有原始官方作者生成的[自有 RAR20 七卷](../../crates/xdzip-codec-rar/tests/fixtures/legacy20-split-owned/README.md)与[自有 RAR26 十七卷](../../crates/xdzip-codec-rar/tests/fixtures/legacy26-split-owned/README.md)。逐卷摘要和真实 split/LARGE 字段独立固定；20 的普通回归逐字节比较 589,826 字节数学原文，26 的显式 release 回归复用单卷的有界 sink，对同一 2 GiB 加 1 字节原文再核完整 SHA、CRC 与一次交付额度。公开 packed_size 始终保留首片尺寸。测试不依赖作者程序、不生成大明文；这两个非固实、无加密正例不外推其他分卷组合，也不替代 Store 混排实样。

English: The owned seven-volume version 20 fixture compares all 589,826 original mathematical bytes in ordinary tests. The owned seventeen-volume version 26 fixture reuses the single-volume bounded sink for a second explicit SHA, CRC and prepaid-credit check of the same complete 2 GiB + 1 byte original. Every physical volume identity and actual split/LARGE flag is pinned; public packed_size remains the first fragment's size. Component tests use no author executable and create no large plaintext. These unencrypted nonsolid positives establish neither other split profiles nor Store mixing.

English: The independent RAR20/26 candidate shares bounded MSB input, a maximum 1 MiB history and actual-work delivery helpers. Tables commit atomically without byte alignment and preserve untouched prior lengths; chronological distances retain duplicates. Predictive Audio supports one to four interleaved channels and preserves state across table/mode changes. Known output size ends content; an observed tail-table marker must parse completely before success, and the remaining authorized packed range is drained without inventing EOF. Auxiliary initialization/control work funds no delivery credit. The original Junrar RAR2.06 encrypted sample and two RAR20 Boat archives provide independently known content, including observed two-channel Audio. Mathematical vectors are separate. Three additional project-owned mathematical inputs were archived by the pinned official RAR2.90 writer in ordinary, solid and forced-Audio solid profiles. Normal tests compare every byte of all nine members,591,561bytes, including fresh-reader prefix replay; all actual unpack versions are20. A separate genuine official2.90 version26/method1/LARGE archive expands to2GiB+1bytes. Its explicit release regression uses bounded chunks, incremental SHA, complete CRC and prepaid-credit checks without materializing plaintext; ordinary debug tests ignore this workload. All legacy tail-table transitions, other actual channel counts and other split, solid or encrypted combinations retain separate coverage requirements. Original source and content notices are preserved; no RAR encoder or strict-clean-room claim is made.

## 尚未闭合的兼容条件

13/15 族、20/26 族、RAR29 LZ/PPM/标准过滤器、历史密码与早期 checksum 已有候选实现，各历史组合仍需逐项实样与整链验收。组件或单个作者档成功不替代完整迁移。RAR7 生效算法 1 的独立压缩成功样本由 [#43](https://github.com/xiaodaozhi/xdzip/issues/43)跟踪，必须分别核对固定来源、完整原文和 worker/runtime 业务链，Store、数学位向量和兼容算法 0 样本不替代它；每种过滤器的独立真实样本覆盖也分别记录，不能由过滤代码存在推定全部验收。固定现有 UnRAR 基线仅执行六种旧标准过滤器，不据历史 VM 名称扩展为任意包内程序解释器。

公开 `Entry.size_known` 区分未知尺寸与已知空文件。完整片链取得一致 known 尺寸时恢复尺寸；全部 unknown 可浏览并发布 `size_known=false,size=0`，不会发布历史内部 sentinel `0x7fffffff7fffffff`。Native ROW 的 `0x40` 仅供 RAR 表示未知尺寸，必须搭配 size=0；保留 CAB 的 `0x20` 语义，其他格式或保留 `0x80` 拒绝。运行时与 Qt 保留该标记，原始大小及含未知尺寸的选中汇总显示 `—`，状态明确提示未知，大小升降序都将未知项放在已知项之后，以物理身份稳定打破平局。预检报告 `unknown_size` 且总量不可知；整包检查、解压、临时打开及导出在取得内容前明确拒绝，不把未知项当空文件，也不使用它充当解码或输出配额。

现代 Unix/FILETIME 时间保留私有完整值，公开 DOS 时间通过[宿主日历服务](host-platform-services.md)投影：直接 API 默认 UTC，产品 worker 使用日期对应本地规则，Rust 保留历史模运算、128 年 wrap、偶秒截断及异常 fraction 忽略规则。OS 无法投影时明确定义为 0，有别于参考未检查系统失败产生的未定义结果；三平台完整产品证据仍分别验收。

旧服务的 LARGE/extended-time 可在完整头 CRC、必需 32/40 字节基础、名称、64 位 packed 范围与预算通过后有界跳过。原始尾部保留为不解释的数据，不猜测 salt/time 分区，不构造补零密码参数，不改变普通文件时间，不提供服务输出或服务内容认证。数学元数据正负对照与健康独立作者档分别记录，缺少健康大服务内容样本仍是证据边界。已知原文作者覆盖包含古老压缩分卷、RAR20 普通与 Audio 固实、RAR20 与 RAR26 压缩分卷，以及真实 UnpVer26 的 2 GiB + 1 字节单文件；这些有限正例不扩展为所有分卷、通道或尾表转换已经验收。非 Windows 的旧 Unicode 高位无修正 run、历史非 ASCII 密码密文、旧压缩和 Store 混排等兼容条件继续分别跟踪，不以数学容器、纯 OS 转换或声明版本变异冒充健康作者互操作。

现代无 end 的档案目前 fail-closed，旧格式允许恰好记录边界 EOF。SFX 搜索限定为前 4 MiB，古老边界另按上述精确范围。重复安全 extra 的严格拒绝等坏输入策略不冒称官方格式禁止。跳过服务内容不提供其完整内容校验保证。来源身份检查仍是宿主尽力检测，不宣称元数据重扫形成输入内容的原子快照。

English: The product uses independent Rust for RAR containers, Store, legacy 13/15 and 20/26, RAR29 LZ/PPM/standard filters and modern LZ/crypto, with no compiled or deployed UnRAR and no runtime fallback. It preserves physical identities, authorized volumes, raw names, unpack versions and first-fragment public packed sizes. Decoding separates prefilter history from final output and keeps compression state across Store records. Actual produced bytes fund delivery credit; auxiliary work has a separate bounded channel. Encrypted headers retain salt/IV/ciphertext; file ciphers cross authorized fragments and applicable hashes cover ciphertext before decryption. Bounded task-private KDF caches span scan and payload; platform password profiles and ambiguous password/integrity failures remain explicit. Unknown sizes retain an explicit flag through the worker, runtime and UI, sort after known sizes, and cannot be decoded or output. Local calendar projection is supplied by the host while Rust owns DOS packing. Historical combinations, independent real-filter coverage and platform compatibility require separate evidence; actual RAR7 algorithm 1 acceptance requires pinned provenance, complete original-content checks and separate worker/runtime evidence. An accidental third-party ARM search snippet was recorded after the corresponding implementation was complete and that implementation was frozen. A later scoped service-parser analysis followed the original container freeze, and an unexposed implementer made the subsequent container change from the numeric handoff. Opaque service skipping never authenticates service contents. These source boundaries are neither strict-clean-room nor legal clearance.
