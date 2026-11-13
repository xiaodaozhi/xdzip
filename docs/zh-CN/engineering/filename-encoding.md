# 文件名编码自适应契约

## 解析与身份

解码在不依赖 Qt 的 Rust 格式层完成，不使用宿主系统代码页、界面语言、来源系统猜测、文件内容或网络服务。ZIP 的 raw_name 保留名称字段原始字节；TAR 保留 PAX/GNU 选择并按既有规则去掉前置 ./ 的原始名称。物理 EntryId 与名称无关。decoded_name 保存需要转码或补充编码声明的 UTF-8 拼写，name_encoding 保留解析来源，name_bytes 提供冻结的有效名称；display_name 只用于转义展示，不能作为输出路径。

ZIP 按以下顺序解析：

1. UTF-8 标志置位时要求原字段为有效 UTF-8，损坏声明不改用旧编码修复。
2. 未置位时读取 Info-ZIP Unicode Path 0x7075，接受版本 1、原名称 CRC32 匹配、完整有效 UTF-8 的值。未知版本和 CRC 不匹配按规范忽略；已匹配但无效的 UTF-8、重复有效字段或损坏字段结构标记需检查。
3. 没有可用扩展，且整组未声明名称均为有效 UTF-8 时，接受未标记的 UTF-8。
4. 否则整组未声明非 ASCII 名称使用全包样本的统计检测，覆盖 GBK/GB18030、Big5、Shift-JIS、EUC-JP、韩文及其他检测器支持的字符集。ZIP 的西欧默认猜测按格式历史默认回退 CP437。检测到 GBK 时使用兼容的 GB18030 严格解码器。

TAR 普通头、GNU 长名称及显式 hdrcharset=BINARY 的 PAX path 使用相同的无声明名称策略。普通 PAX path 要求有效 UTF-8，不用检测掩盖损坏声明；UTF-8 和 BINARY 以外的显式 hdrcharset 不支持。TAR 不套用 ZIP 的 CP437 默认。TAR.GZ 与 TAR 使用同一名称解析。

规则依据为 [ZIP APPNOTE 6.3.10](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT)、[POSIX pax](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/pax.html)及 [WHATWG 编码标准](https://encoding.spec.whatwg.org/)。Unicode Path 的本地头可缺省，但若提供有效值，必须与冻结的有效名称一致；载荷仍校验原始名称、本地/中央头、CRC、大小、密码认证和流结束。

CAB 保留原始名称字节和 UTF-8 属性。显式 UTF-8 严格验证，其余名称复用整包有界检测，不使用系统代码页。CAB 的反斜杠是格式目录分隔符，但仅在完整解码后规范化为正斜杠，不能误拆 Shift-JIS 多字节尾字节 0x5C。原始字节不变，有效路径仍经过相同安全与碰撞检查。ISO 非 Joliet 名称保留原始字节并使用相同有界解析，避免 Rock Ridge 名称经过宿主 OEM 转码。Joliet/UDF 使用处理器提供的 Unicode 名称并严格验证，不将有损替换字符视为原始名称。

English: CAB retains raw name bytes and its explicit UTF-8 flag, otherwise using bounded archive-level legacy detection. CAB separators are normalized only after decoding, preserving Shift-JIS trail bytes. Non-Joliet ISO names retain raw bytes, avoiding host OEM conversion of Rock Ridge names. Joliet/UDF Unicode names are strictly validated. Every effective path remains subject to the shared output-safety and collision checks.

## 检测边界与资源

检测器最多读取 1 MiB 完整的未声明、非 UTF-8 名称，以换行隔开记录，避免截断多字节序列跨名称拼接。不将明确 Unicode 声明或无效扩展作为检测样本。只要样本含非 UTF-8 名称，就对未声明名称统一采用归档级结果，避免 GBK 中恰好合法的 UTF-8 片段把同一个目录拆成两个名字。单条严格解码失败时才尝试合法 UTF-8 或该名称的局部结果。禁用 ISO-2022-JP 猜测，不解释名称中的转义状态。严格解码不插入替代字符、不删除 BOM，也不转换斜杠、规范化 Unicode 或修补路径。

原始名称合计最多 16 MiB，单个原始/解码名称最多 4096 字节，解码名称与转义展示合计最多 16 MiB。ZIP Unicode 扩展的每条及累计分配在扫描时检查，不延迟到整包结束。解析、检测与逐条转换检查取消。ZIP 载荷额外保留最多 16 MiB 的有效名称校验表，后续 decode 不能更换冻结的名称。

统计检测不能唯一恢复任意短名称或混合旧编码，也不能判断已被其他软件保存为有效 Unicode 的乱码文本。一个包内明确声明的 Unicode 与旧编码可共存；多个未声明编码只能尽力检测。当前没有手动编码切换入口。无法严格解码、声明损坏或路径异常时继续转义显示并标记需检查，不授予解压、临时打开、导出或修改能力。不能承诺所有历史包均正确识别。归档存在未解析名称时，其真实目录归属可能未知，因此拒绝目录展开解压，避免成功结果静默漏项；明确选择的已解析普通文件仍可单独解压。

## 全链路与保存

目录树、直属索引、递归搜索、目录选择扩展、预检、解压、临时打开、外部拖出及包内修改使用同一个有效名称。完成解码后仍执行 PortablePath、Unicode NFD/大小写别名、重名与输出授权检查。Shift-JIS 中作为多字节字符尾字节的 0x5C 不当作目录分隔符，除 CAB 的格式分隔符外，解码后真正的反斜杠仍拒绝。

普通 ZIP、TAR、TAR.GZ 添加和移动在规划结束后将旧编码/Unicode 扩展/未标记 UTF-8 名称重新声明为 UTF-8，保留逻辑移动规划中的原位置与跳过含义。ZIP 保持压缩载荷，TAR 使用明确的 UTF-8 PAX path；写后重新扫描、逐项验证名称与内容，最后执行原有同卷替换。TAR 改写 BINARY PAX path 时移除该路径的 BINARY 声明；如果这会改变其他非 UTF-8 PAX 文本的解释，则明确失败并保留原包。加密和分卷的读取、解压沿用相同名称，添加/移动限制不变。

## 可重复验收

使用自有原始字节夹具及 Python 标准库独立编码的归档，覆盖未标记 UTF-8、GBK、四字节 GB18030、Big5、含 0x5C 的 Shift-JIS、韩文、俄文、ZIP CP437 和 Unicode Path。验证完整名称、原字节、目录/搜索/选择、预检、实际输出字节、临时打开、导出、普通包添加/移动、AES 和分卷；同时检查声明损坏、扩展优先级、路径拒绝、Unicode/大小写碰撞、取消和预算。

scripts/check-filename-encoding.py 通过实际服务探针解压 Python 生成的 ZIP/TAR/TAR.GZ，另以 Python 独立读取 Rust 改写的归档，检查 UTF-8 标志/PAX、名称和内容。测试数据只来自源码常量，不使用用户包。具体结果与剩余人工验收记录于 issue 和 build/CI 工件，不在本契约登记通过状态。

English: Filename resolution is host-independent and frozen in the Rust snapshot, separate from original name bytes and escaped display text. ZIP honors the UTF-8 flag first, then a version-1 Unicode Path extra bound to the raw name by CRC32, then unmarked UTF-8 when the whole unlabelled group is valid, otherwise bounded archive-level legacy detection with a CP437 fallback for the western ZIP default. TAR/GNU and BINARY PAX names use detection; ordinary PAX paths retain their UTF-8 requirement. All navigation, selection, extraction, temporary opening, export and ordinary updates consume the same resolved name and repeat path, Unicode/case, conflict and publication checks. Updates emit resolved names as UTF-8 without changing payloads or bypassing the ordinary-archive restriction. Sampling and allocations are bounded and cancellation remains effective. Detection is heuristic, cannot uniquely recover every short or mixed-encoding name, and does not repair already-mojibaked Unicode. No manual selector is provided. Damaged declarations and unsafe/unresolved names remain inspectable but cannot authorize output. If unresolved names exist, directory expansion is refused because their membership is uncertain; explicitly selected resolved files remain extractable. Native desktop acceptance remains separate from automated worker and format checks.
