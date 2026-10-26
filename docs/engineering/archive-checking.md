# 压缩包完整性检查

工具栏“测试”对当前归档执行一次真实后台检查，覆盖全部物理记录，包含重复名称、目录和空文件，不受当前目录、搜索或选择影响。检查只持有冻结的输入来源与元数据，不取得目标目录授权、不创建暂存或输出文件，不使用解压路径映射和链接发布流程。

## 格式与结果

ZIP／ZIP64、Store／Deflate、受支持的编号分卷和 PKZIP 分卷逐条解码到有界丢弃接收器。检查包括中央目录与本地头、描述符、大小、压缩流准确终止、CRC，以及受支持 AES-256 的认证。安全名称的 ZIP 符号链接只检查负载，不解析目标或创建链接，因此不受写出链接的平台限制。未知方法、加密方式、危险名称与其他未准入记录仍失败，不跳过后宣称整包通过。

7z 和 RAR 使用官方后端逐物理条目解码到同一个有界丢弃接收器，验证解码终止、声明大小和容器实际提供的校验。固实块内为到达当前选择而解码的其他数据只累计工作量，不增加当前文件的已验证计数。目录、空文件和加密条目均不能跳过后报告整包成功。RAR 旧版、RAR5 和 RAR 7 新算法分别使用独立已知内容样本验收，不能仅以相同魔数代表版本兼容。

CAB 使用隔离工作进程内的自研 Rust 容器与共享算法，验证卷集合、实际解码大小和完整数据帧提供的校验。ISO/UDF 检查描述符、目录、数据范围和声明大小，并实际读取所有受支持普通文件；它们没有普遍的逐文件内容校验和，不能发现任意保持结构的内容篡改。镜像链接、设备及其他不支持记录不能跳过后宣称整包通过。混合镜像检查冻结的 UDF 命名空间，不在失败后改查 ISO 命名空间。

TAR 与 TAR.GZ 在检查任务内重新顺序扫描完整流，对照冻结的物理记录元数据，检查头校验、长度、两个结束块及尾部。TAR.GZ 继续消费到 EOF，核对 GZIP CRC／ISIZE，不在完整扫描之后再次解压全部内容。TAR 没有文件内容校验和，只能确认格式结构与可读性，无法发现任意保持结构的内容篡改。TAR 链接、特殊对象和稀疏文件仍不支持。本功能不证明归档来源可信，也不检查恶意软件。

首个结构、负载、来源、资源或不支持条目错误结束任务。结果区分成功、失败与取消，显示已验证及未检查文件数，保留失败条目和稳定错误码。所有物理记录均参与校验，`files`／`verified_files` 仍只统计非目录文件，空文件参与文件数。目录和纯目录包也必须完成结构校验后才能成功。检查中或此前已完成的读取不等于整包通过。

## 调度、来源与资源

`Check { session }` 冻结来源 `Source::fork()`、格式、元数据和原始会话身份，进入与创建、解压相同的有界重任务队列及应用级串行准入。开始、读取后及成功终态前复核来源句柄、全部卷路径的身份、大小和时间信息。来源变化返回 `source_changed`，只失效对应的原会话。此检测是尽力检查，不是输入内容的原子快照，不承诺检测保留身份和时间信息的并发原地修改。

实际解码受单文件 8 GiB、任务累计 20 GiB 限制，错误密码重试不重置累计工作量。ZIP 保留独立 64 MiB 定位元数据预算和共享 20 GiB 实际输入读取预算；TAR／GZIP 保留既有 24 GiB 整流读取上限。7z／RAR／CAB／ISO／UDF 额外使用原生进程 1 GiB 内存上限，输入读取与固实跳过同样有界，超限为资源限制而非完整性通过。任务与事件数量、快照明细和保留内存沿用既有队列上限。成功终态不设置写入、提交、保存目录或输出路径计数。

任务种类由 `kind=check` 明确传至 Qt。`read_bytes` 表示实际来源读取量，包含头部与重试；`decoded_bytes` 表示实际解码工作，`verified_bytes`／`verified_files` 只在要求的校验完成后增加。TAR／GZIP 完整扫描期间不编造文件级百分比，实际读取量与速度仍可见，成功后才将整流文件计为已验证。

AES 密码复用[密码协议](password-extraction.md)：独立控制槽、一次性提示身份、任务内缓存、最多每条十次错误验证值响应、认证破坏直接失败及秘密归零。取消和退出唤醒密码等待，不依赖普通命令队列容量。关闭浏览页或进度窗口不取消已接收任务，结果从全局任务入口重开；明确退出仍等待取消与异步 join。

## 验证边界

English: The independent Rust CAB container and shared algorithms run inside the isolated worker, checking the authorized volume chain, decoded sizes and checksums available in complete data frames. ISO/UDF checks descriptors, directories, extent bounds and actual readability of supported files. Disc images do not generally provide per-file content checksums, so a successful check cannot detect every content change. Unsupported special records fail the whole-archive check. Hybrid checks retain the selected UDF namespace instead of falling back to ISO after an error. The existing file, task, read and native-memory budgets remain in force.

核心回归覆盖支持格式、空文件、目录、重复名称、ZIP 链接负载、密码重试／取消、损坏负载、来源变化、实际计数和无输出。桥接与 Qt 验证真实工具栏到后台任务和结果闭环，密码路由、历史任务、跨会话准入及退出独立检查。双语多字号、浅深色与原生桌面视觉验收分别登记在 issue，生成证据仅保存于 build/ 或 CI 工件。

English: 7z and RAR use independent Rust engines and a bounded discard sink, including encrypted, empty and solid records. Work decoded only to reach a solid record does not inflate verified-file counts. Test checks every physical record of the current archive in a background read-only task, independently of the current view or selection. ZIP payloads are fully decoded into a bounded discard sink, including AES authentication and ZIP symlink payloads without creating links. TAR and TAR.GZ are scanned completely again, including TAR end blocks and the GZIP trailer. TAR has no content checksums. The task stops at the first error, preserves actual read and verified-file counts, writes no output, reuses password/cancellation controls, and retains the existing resource and best-effort source-identity boundaries. Passing a check does not establish source trust or malware safety.

旧式 RAR 加密条目缺少可可靠区分错误密码与损坏内容的验证值，因此每个加密条目解码前分别请求密码，不能把前一条密码直接套用到可能使用不同秘密的下一条。RAR5 和受支持的其他格式继续任务内复用。未取得可信密码验证值的加密解码或加密头完整性失败报告 `password_or_integrity`，保留错误密码或内容损坏的歧义；明文头校验、来源变化、取消、限额及输出 I/O 不误归类为密码问题。

English: Older RAR encrypted records request a password separately for each entry because their checks cannot reliably distinguish a wrong cached secret from corruption. RAR5 retains task-local reuse. Without a trusted password verifier, encrypted integrity failures report `password_or_integrity`. Plaintext header checks, source changes, cancellation, limits and output I/O preserve their own error classes.

包含未知原始大小的归档仍可浏览；整包检查的总量标为未知，并在打开 payload 或请求密码前以 `unsupported_feature` 失败，已验证和解码产量保持实际值。未知大小不按空文件验证，也不将内部 sentinel 用作进度分母。

English: Archives with unknown uncompressed sizes remain browsable. Whole-archive checks report an unknown total and fail with `unsupported_feature` before opening payloads or requesting passwords. Actual decoded and verified counters are retained; unknown values are never verified as empty files or used as progress totals.
