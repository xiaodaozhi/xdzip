# 格式工作进程的宿主日历与编码服务

格式字段、密码兼容规则和校验由 Rust 处理。操作系统的历史时区、OEM/ANSI 代码页和本地 C 编码属于宿主环境，通过 `ReadSource` 的可选服务提供，不能由格式层根据文件名、HostOS 或当前 UTC 偏移猜测。服务不依赖 Qt，不授予路径访问或文件发布权限。

## 日历投影

`Timestamp` 明确区分 1601 epoch 的 FILETIME 100ns 值与 1970 epoch 的无符号 Unix 整秒。`ReadSource::local_calendar` 返回可选 `Calendar`，普通文件和内存源的默认策略是确定性的 UTC；产品 worker 通过同步 CXX 回调使用操作系统的本地时区规则。Windows 使用系统时区 API，Unix 使用 `localtime_r`，均检查转换结果。转换不可用时返回空日历，不能读取未初始化字段。

Rust 验证年月日与时分秒后打包 DOS 展示值。7z 延续 UTC 和 1980..2107 的既有展示范围；RAR 按其[固定兼容规则](rar-format-sources.md)先处理内部模运算、时间覆盖及平台差异，再使用宿主日历和历史年份回绕。原始精度、未设置标记与完整物理头保留在格式私有索引，展示精度不会反向覆盖它们。RAR 的安全候选政策将无法投影的时间显示为 0，不复制参考实现未检查 OS 失败后的未定义内容。

## 受控编码映射

| 服务 | 输入与输出 | 宿主边界 |
| --- | --- | --- |
| `oem_to_ansi_wide` | 512 个 UTF16 单元到 512 个 UTF16 单元，含 NUL | Windows OEM 到 ANSI 的完整往返，保留系统实际映射与替代行为 |
| `wide_to_ansi` | 含 NUL 的不超过 512 个 UTF16 单元到 512 字节 | Windows 当前 ANSI 代码页，不自造统一替代字符 |
| `wide_to_local_prefix` | 含 NUL 的不超过 128 个宽标量到 512 字节 | Unix 当前 C 编码，转换失败按固定兼容策略清空重试并保留已转换前缀 |

这些方法返回不含 NUL 的长度。缺少宿主服务的默认实现返回 `None`，格式调用者只能使用已定义的可移植回退或明确拒绝，不能静默猜测代码页。产品回调检查维度、NUL 与返回范围，无法执行的映射返回稳定的 Unsupported。

Windows 的 [CharToOemBuffW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-chartooembuffw) 长度参数计输入宽字符，并继续转换 NUL 后的补齐单元，不是目标字节容量。宿主先用 `GetCPInfo(CP_OEMCP)` 准入每单元至多四字节的代码页，再用 2,048 字节临时目标映射完整 512 单元，随后才设置字节 511 为 NUL 并投影 ANSI。无法确认该容量前提时拒绝转换；不能用 512 字节目标接收 512 个宽字符，或先裁剪输入来回避容量问题。实际 OS 探针包含完整 511 单元 DBCS 边界及独立直接 WinAPI 对照。

512 单元准入、首 NUL、OEM 补偿条件、127 单元及字节截断、旧标记协议和 macOS 的显式 UTF8 都属于 Rust 的格式兼容策略，不放进 C++。尤其不能先截断密码再对其执行本来位于截断之前的 OS 映射，也不能把 DBCS 中间截断擅自改成字符边界截断。

## 所有权和失败

所有切片只在同步调用期间借用，C++ 不保存指针。Rust 调用方负责秘密输入与输出的归零生命周期；C++ 的有界临时宽、窄缓冲在所有返回路径擦除。此保证不扩展为操作系统内部或全部历史内存副本已擦除。原语不记录密码，不把它放进进程参数或环境变量。

Reader 在服务前检查取消。服务结果不生成解压输出额度，不改变已读、已解压或已写字节。失败保留已有的来源、取消与限额类别，不能误作密码错误；既定的密码/完整性歧义仅由格式层依据实际 verifier 状态决定。

English: Archive interpretation, truncation and integrity remain in Rust. Borrowed host services provide date-specific local calendars and bounded operating-system encoding maps without Qt, path access or publication authority. Bare sources use deterministic UTC calendars and expose no optional code-page service. Product callbacks validate dimensions, terminators and result fields; conversion failures have defined results. CharToOemBuffW consumes all512 input wide units, including NUL padding; its temporary destination has2048 bytes after checking a maximum four-byte code-page expansion. Byte511 termination follows the complete OS conversion. RAR retains raw timestamps and modular compatibility before DOS projection, while 7z preserves its UTC display policy. Secret slices are borrowed synchronously, never retained or logged, and bounded temporary buffers have erasure lifetimes. These services neither grant decoded-output credit nor replace cancellation, source checks or format-specific password classification.
