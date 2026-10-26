# 隔离格式工作进程契约

7z、RAR、CAB 与 ISO/UDF 使用[独立 Rust 后端](independent-rust-codecs.md)，通过共享的薄 CXX 边界复用同一隔离工作进程。ZIP、TAR、TAR.GZ 使用进程内 Rust 后端。产品不从用户 PATH 查找替代工具，也不运行外部命令行解包。

CAB 产品固定执行独立 Rust 路由，没有测试开关、官方处理器或失败回退。完整帧、共享算法和资源边界见[自研契约](independent-rust-codecs.md#cab只读规格)，来源及参考分析角色见[CAB 格式来源](cab-format-sources.md)。

## 构建与来源

统一准备入口运行 `scripts/prepare-native-codecs.py`。[产品源码清单](../../scripts/native-codec-sources.json)当前为空，准备步骤不下载或解包官方格式源码；离线构建同样不需要 UnRAR 或 7-Zip 归档。通用来源工具保留有界解包及摘要校验，用于明确声明的来源材料，不能据缓存内容自动加入产品。

产品不编译或链接官方 UnRAR、7-Zip 容器、算法、通用设施、文件适配器或格式注册表，7z、RAR、CAB、ISO、UDF 均固定使用 Rust，没有运行失败回退。CMake 内部的 `XDZIP_7Z_BACKEND=rust`、`XDZIP_RAR_BACKEND=rust` 与 `XDZIP_CAB_BACKEND=rust` 记录固定身份。生成的 `native-codec-source-list.txt` 完整记录七个项目自有胶水文件，门禁核对路径与集合，拒绝官方来源和已退出的 `host_file.cpp` 重新进入。`platform_time.cpp` 与 `platform_encoding.cpp` 仅将 OS 本地日历和字符集映射交回 Rust，不解释归档时间格式或密码字节规则。共享 Deflate/Deflate64、LZMA、BZip2、PPMd、LZX、Quantum 按实际格式复用，不重复引入格式库。

来源对应材料随实际部署版本交付，不能只交付下载链接。历史构建与独立官方参考工具继续按各自原始条款处理，当前产品源码清单为空不改变既有交付材料的义务。

## 权力与生命周期

Rust 核心持有来源授权、分卷身份、目标授权、暂存和提交能力。Rust 7z/RAR/CAB/光盘后端在独立工作进程运行，通过有界协议读取授权流及输出当前批准条目的数据，不获得用户目标路径或任意文件访问接口。CXX 同步借用的字符串与切片只在调用期间有效，不保存其指针。C++ 异常和 Rust unwind 转成稳定错误；OOM/abort 使工作进程失败，宿主丢弃不完整暂存。

宿主对初始来源 seek、工作进程随机读取、创建输入及 EOF 探测统一保留 typed 限额、取消、不支持和来源变更错误；即使外层 I/O kind 是 Interrupted，也不抹掉其 typed 原因。只有没有 typed 原因的 Interrupted 可在检查取消后重试，预先计入的来源工作不退款。Scan 保留 SourceChanged，内容读取按现有来源错误语义返回，创建按来源错误拒绝提交。

工作进程不接收命令行密码，不输出归档提供的任意诊断文本，也不继承应用凭据和代理环境。错误映射为稳定错误码，打开加密目录的密码只保留在当前会话及其关联任务的归零容器中，不跨启动持久化；关闭会话和任务终态分别释放各自容器。取消停止读取与输出，超时后终止工作进程并等待回收；进程失败不得产生成功终态。进程隔离不等同于操作系统沙箱，不宣称能阻止已被攻陷的进程访问所有同用户资源。

7z 创建通过 EncodeOutput::write_at 将有界偏移与数据块写入宿主私有暂存，原生进程只保存流位置，不在内存中累积整个输出归档。宿主分别累计实际写入量和归档长度高水位，头部回写不增加分卷内容长度；偏移、块大小、累计输出量和取消均在主机端重新检查。未提供随机写能力的输出明确拒绝。

任务延续宿主条目、路径、元数据和实际字节预算，默认 native 预算为 1 GiB。Windows 使用 Job Object 提交内存限制，Linux 使用绝对虚拟地址空间限制，macOS 使用启动时虚拟地址空间基线加预算，排除 dyld 预留；Unix 地址空间限制不等同于 RSS 计量。字典及固实工作区在分配前准入，无法在预算内完成的操作明确失败，不静默降级为无界磁盘暂存。重试和固实跳过均累计实际工作量。

算法初始化和控制处理的实际辅助工作使用独立 `AUX_WORK=24`，当前仅允许 RAR 负载使用；宿主与 runtime 分别累计辅助预算，不将其计入文件解码、膨胀比、公开进度或预计费信用。完整协议、重试生命周期与必须保持的验收条件见[算法辅助工作计量](auxiliary-decoding-work.md)。

English: Actual auxiliary algorithm work uses a separate RAR-payload-only channel, `AUX_WORK=24`. Native and runtime independently retain cumulative auxiliary budgets across retries. This work changes neither decoded/output bytes, expansion ratios, public progress nor prepaid delivery credit. The [auxiliary work contract](auxiliary-decoding-work.md) defines its framing, lifecycle and required acceptance checks.

## 能力边界

7z 的浏览、解压、完整性测试和创建分别声明，创建使用 LZMA2，默认正常级别、非固实、不加密、不分卷。固实、内容加密、文件名加密和编号分卷作为独立能力验证。文件名加密必须设置密码。文件名加密包在元数据打开阶段请求密码，未完成扫描不能发布部分索引。

未加密、非分卷且所有条目可安全保留的 7z 包可复用添加、替换和包内移动。固实内容通过受限重建处理。加密、分卷、链接和不支持特殊记录继续拒绝修改，来源重新检查及完整验证通过后才允许同卷原子替换。独立删除和重命名入口不属于本次扩展。

RAR 固定使用 `xdzip-codec-rar`，只读且没有官方后端回退。旧格式、RAR5 及 RAR 7 扩展，密码、头部加密、固实和分卷分别验证，算法及兼容性限制以[自研规格](independent-rust-codecs.md#rar只读规格)为准。不提供创建、修改、修复或密码恢复。RAR 的初始化和控制工作走辅助通道；真实解码输出使用 WORK 与预付交付，不将模型初始化计入文件大小或膨胀比。

CAB 支持 Store、MSZip、Quantum 和 LZX，单卷与同目录分卷均只读。分卷先由 Rust 根据头部关系验证和冻结授权集合，再向工作进程传递原始卷名及各卷边界。缺卷和不匹配卷不能发布部分索引，详见[分卷契约](archive-opening.md)。CAB 原始名字和 UTF-8 声明保留，旧编码采用[统一文件名解析](filename-encoding.md)，不调用宿主代码页猜测输出路径。

ISO9660、Joliet、Rock Ridge 和 UDF 的解析由 `xdzip-codec-optical` 执行，不链接官方 Iso/Udf 处理器。混合镜像优先 UDF 命名空间，已识别的 UDF 损坏不静默回退 ISO。按内容识别，不依赖扩展名授权。ISO/UDF 仅浏览、解压、临时打开、导出和测试，不挂载磁盘、不执行启动镜像、不提供创建或更新。启动镜像记录仅作为可读取文件，链接及特殊对象保持不可输出。镜像测试检查结构、范围及实际内容可读取性，不声称普通文件具有内容校验和保证。分区映射和不支持扩展的明确边界见[自研规格](independent-rust-codecs.md#iso与udf只读规格)。

English: Independent Rust implements 7z, RAR, CAB and optical images through one borrowed CXX boundary inside the existing isolated worker. Official UnRAR and 7-Zip sources, handlers, algorithms, filesystem adapters and registries are absent from the product; pinned official tools remain separate test references. The product source manifest is empty, and offline preparation needs no official source archive. The generated native source list must match the seven project-owned glue files, including the OS-only calendar and character mapping callbacks. No failed operation falls back to an official backend. Rust codecs receive authorized streams and bounded output, while the host owns source identity, cancellation, quotas, staging and publication. RAR/CAB/disc images remain read-only; RAR initialization uses auxiliary work rather than decoded bytes or delivery credit. Hybrid images select UDF, and recognized UDF corruption does not trigger an ISO fallback. Image checks establish structure and readability without inventing file checksums. Implementation, cross-platform, manual desktop and release gates remain separate.

## 验证与交付

实际支持声明须由官方工具互操作、已知输出字节、损坏和资源限制样本支撑。仅识别魔数、调用返回成功或使用同一后端自读自写不足以独立验证互操作。三平台核心和 Qt 检查、部署验证、原生视觉及桌面拖放分别记录，阶段证据保留在 issue 和本地或 CI 工件。

RAR 路由验收必须覆盖实际 worker 的扫描、测试与安全解压，包含小文件、PPM 初始化、密码和授权分卷。小文件及 PPM 初始化不得单独触发高膨胀比，辅助工作不得增加公开字节计数；失败不得发布文件。源码门禁同时核对空产品清单、固定 Rust 身份、完整编译来源及薄 RAR 入口，不能只据符号或输出成功判断官方代码已退出。

English: RAR route acceptance covers scan, check and safe extraction through the built worker, including small files, PPM initialization, passwords and authorized volumes. Initialization must not cause high-ratio rejection or alter public byte counters, and failure must publish no file. Source gates check the empty manifest, fixed Rust identities, complete compiled source list and thin RAR entry together.

Foundation 使用 `scripts/check-rar-rust-interop.py --worker <built-worker> --output build/<evidence>` 运行原有完整 `native_backend_tests` 显式批次。入口先按 `scripts/rar-rust-fixtures.json` 固定 URL、大小和 SHA-256 准备公开小型 PPM、大 PPM 与旧密码样本，再生成仅含原有压缩流的单成员子集、自有数学未知尺寸和已知空文件头，以及自有 20/26 夹具所需环境。网络单响应最多 512 KiB，准备文件与证据仅进入 build，不生成 2 GiB 明文，不执行作者工具。缺失或身份不符必须失败，不能以跳过测试替代环境准备。包装还必须实际运行辅助工作恶意 worker、未知尺寸协议、Native 小文件计量、固实重读、自有 E8E9 全字节摘要和真实 OS 字符映射探针。探针只使用当前 worker 旁固定 Rust 输出目录的 CXX 头文件与严格唯一的 Release 静态库，记录头文件、静态库和探针摘要；缺失或歧义必须失败。大 26 的组件完整 SHA 验证单独运行，runtime 批次通过实际 worker 检查一次完整大文件的计数、CRC 和零暂存行为；可显式追加 `--large-digest` 验证 worker 完整 SHA。

English: Foundation runs the original complete explicit `native_backend_tests` batch through `scripts/check-rar-rust-interop.py --worker <built-worker> --output build/<evidence>`. The wrapper prepares fixed URL/size/SHA-256 public PPM and legacy-password data, an unchanged compressed single-member subset, original mathematical unknown-size/known-empty headers, and the owned version20/26 environment. Each network response is limited to 512 KiB. Data and evidence stay under build; no 2 GiB plaintext or author executable is created or run. Missing or mismatched data fails rather than silently skipping tests. Acceptance also runs hostile auxiliary/unknown-size protocol workers, Native small-file accounting and solid rereads, the complete owned E8E9 digest, and the real OS character mapping probe. The probe uses only CXX headers and the unique Release archive in the built worker's fixed Rust output layout, records header/archive/probe digests, and fails on missing or ambiguous inputs. The separate component wrapper verifies the large version26 SHA; the runtime batch checks its complete count, CRC and absence of staging through the actual worker once. Explicit `--large-digest` adds a complete worker SHA check.

互操作检查使用固定 26.05 官方参考工具，独立于产品工作进程。Windows 从同一官方发布下载并核对 7zr.exe 摘要，再从已核验的 7z2605-extra.7z 仅提取 x64/7za.exe，并复核该成员的独立 SHA-256。此流程不依赖系统 tar 的 LZMA 支持，不运行安装器，也不从 PATH 借用未固定工具。Linux 与 macOS 从已核验 tar.xz 仅读取指定 7zz 成员。下载、缓存和提取结果任一摘要不符均停止；这些参考文件只保留于 build/ 测试目录，不进入产品部署。

开发包维持实验性。组件许可、对应源码和重链接要求按实际构建单独复核，本功能完成不自动批准正式发行。
