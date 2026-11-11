# 外部只读 SDK 试验

`xdzip-plugin-protocol` 和 `xdzip-plugin-host` 实现[格式扩展设计](../design/06-codec-extension-sdk.md)的独立只读试验。它不进入桌面／CLI 的格式发现、安装、打包或自动启动路径。用户显式执行试验命令并选择包与来源；产品继续禁用第三方扩展。当前仅协商 `xdp1`，不提供创建、密码、分卷、链接、GUI 或任意宿主路径权限。这不是完整插件生态或正式 SDK 发布。

English: This explicit read-only trial is separate from desktop/CLI discovery and distribution. Product third-party execution remains disabled. Only `xdp1` probe/open/scan/read is negotiated; creation, secrets, volumes, links and UI are not advertised. It is not a full plugin ecosystem or a released SDK.

## 可重现入口

```sh
python3 scripts/sdk-trial.py test
python3 scripts/sdk-trial.py example --output-dir build/sdk-example
target/debug/xdzip-plugin-host scan build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp
mkdir build/sdk-example/output
target/debug/xdzip-plugin-host read build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp 0 build/sdk-example/output
```

Windows 使用 `python` 和 `.exe`。`test` 构建原生参考程序，执行 Rust framing 回归、独立 Python 互通、恶意协议、取消、源变化、冲突与内核权限回归。Python 只由编译为测试的受信 conformance 入口启动；发布宿主不接受解释器、参数、shell 或“不使用沙箱”开关。例子只在 `build/` 生成自有包、manifest 与 XDP1，不触发发行或第三方下载。日志去除工作区／用户主目录，JSON 绑定提交、dirty 状态和二进制哈希，均为本地／CI 工件。Foundation 的同一原生矩阵调用此入口；独立 [SDK trial 工作流](../../.github/workflows/sdk-trial.yml)用于先收敛沙箱平台兼容，不重复运行无关格式与 Qt 管线。新工作流尚未进入默认分支时，可通过既有 Foundation 入口选择 `sdk_only=true` 调用同一可复用工作流；该模式跳过完整 Foundation gate，不产生完整 Foundation 结论。Windows 证明的是拒绝执行。

English: `test` runs both independent peers and adversarial/native checks. Python is launched only by the test-compiled trusted harness; the shipped host has no interpreter, argument, shell or unsandboxed execution option. The owned example and sanitized, source/binary-bound evidence stay under `build/`. Foundation invokes the same entry point; Windows verifies refusal.

## 包准入

严格 JSON manifest 使用 `deny_unknown_fields`。字段为 `manifest_version=1`、`plugin_id`、`version`、`author`、`protocol_majors=[1]`、`targets`、包内 `entrypoint`、小写十六进制 `sha256`、`formats=["xdp1"]`、`permissions`、`resources`、`license_files`、HTTPS `source_url`。参考 manifest 由上面的 `example` 生成，是当前 schema 的可运行入口。

`permissions` 恰为 `broker_source_read=true`、`broker_output_write=true`、`network=false`；`resources` 恰为 `memory_bytes=536870912`、`threads=1`。manifest ≤32KiB，原生程序 ≤64MiB，许可文件最多四个、各 ≤128KiB。入口与许可文件不能绝对寻址、包含父目录或链接／reparse 逃逸；每个包成员均通过持有的 cap-std 目录逐层 no-follow 打开，核对后不再按原字符串重开。宿主通过持有的普通文件复制、校验实际字节数、SHA-256 与前后文件版本，复制到私有临时目录后才执行，保持副本至进程与 IPC 结束。哈希声明证明选择的包内容一致，不认证发布者，也不证明代码安全。

English: Admission uses a strict manifest, fixed read-only permissions/resources, bounded files, relative non-link members, and a verified private executable copy retained through process teardown. Digest consistency is not publisher authentication or proof of safety.

## 线协议与状态

依据 [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html)实现项目自有的有界 CBOR 子集，不引入第三方 CBOR 实现。参考 Rust peer 复用 framing crate；[Python peer](../../examples/sdk/reference.py)独立手写编码、解码与 XDP1 解析，不导入 Rust 或生成的协议代码。互通覆盖多块实际内容与宿主摘要。

帧为 `u32 LE length + CBOR`，先检查长度再分配，帧 ≤1MiB，单个 bytes/text ≤256KiB，深度 ≤16，数组 ≤128，map ≤32，解码节点 ≤8192。只接受 UInt、bytes、严格 UTF-8 text、array、UInt 键 map、bool；拒绝非最短整数、重复键、不定长、负数、标签、浮点及尾随数据。包络 map 的键 `1..5` 依次为 kind、request、operation、sequence、payload。每方向 sequence 从 1 连续递增；request 在每次调用递增，operation 为宿主随机非零身份。每次响应必须精确匹配当前身份。payload 未知键 `<128` 拒绝，`>=128` 为可忽略可选字段。

| kind | 消息 | payload 数字键 |
| --- | --- | --- |
| 1 | Hello | 1 最小 major，2 最大 major，3 包 ID，4 `xdp1` |
| 2 | Welcome | 1 major，2 block 上限，3 单块信用，4 总读取预算，5 输出预算 |
| 10 | Probe | 1 source token，2 来源长度 |
| 11 | Open | 1 source token |
| 12 | Scan | 1 session token |
| 13 | ReadEntry | 1 session，2 entry，3 sink，4 信用数 1，5 输出预算，6 首块信用 token |
| 14 | Close | 1 session |
| 20、21 | ReadAt、ReadData | 1 source token，2 offset，3 length 或 bytes |
| 30 | EntryBatch | 1 session，2 entries；每记录 1 token、2 raw path、3 类型 0 文件／1 目录、4 已知大小 |
| 31 | ScanFinished | 1 实际记录数 |
| 40 | WriteData | 1 sink，2 从 0 递增块序号，3 bytes，4 当前信用 token |
| 41 | FinishOutput | 1 sink，2 声明输出数，3 SHA-256 bytes |
| 42 | WriteAck | 1 sink，2 下一块序号，3 新随机信用 token |
| 50、51、52 | Cancel、CancelAck、Shutdown | 空 map |
| 61 | OperationFinished | 1 outcome，0 成功／1 取消；Probe 另有 2 格式，Open 另有 2 session |

其他 kind 即使 framing 认识也不在本试验状态机中接受。Hello 必须先到达，双方选择 major 1；握手身份为零，后续调用均非零。ScanFinished 后仍必须唯一成功终态。每次只处理一个操作，无后台扩展重试。当前单块信用比设计最大四块更严格，每次实际接受输出后刷新不可预测的信用 token，不能预支或重放。

English: The numeric schema above is normative for this trial. Strict bounded CBOR and per-direction sequence/current-request identities are mandatory. Optional keys start at 128; unadvertised kinds/states fail. Output uses one fresh, unpredictable credit per acknowledged block. Scan and output sealing require their unique successful terminal.

## Broker、预算与提交

Source token、sink 和每块信用均为随机 128 位值，不能复用到其他操作。peer 不接收归档与目标路径。宿主每次只返回授权来源范围内最多 256KiB，偏移加法溢出、越 EOF、零长和累计读取 >64MiB 均失败；每次读取前后及最终提交前检查持有对象和路径对应的 `FileVersion`。这沿用现有身份／时间／长度指纹，不宣称能识别同账户攻击者恢复全部元数据的任意并发改写。

元数据最多 1000 物理记录，路径与 token 合计 ≤256KiB，记录 token 唯一；raw path 在宿主按 PortablePath 严格验证，重复名称保留不同身份。一次显式选择一个普通条目，声明大小 ≤4MiB；拒绝目录作为文件输出。宿主持有目标 Grant、私有 Workspace 和 Stage，按实际字节计量并计算 SHA-256，不向 peer 交付文件句柄或提交权。

所有输出块、FinishOutput 的声明数／摘要、唯一终态、Close、Shutdown 后正常退出及纯 EOF、源身份与最终取消检查都通过，才允许既有安全 I/O 无覆盖提交。任何异常、重复终态、崩溃或超时都先撤销输出；析构先移除 stage 再清理 workspace。既有目标保持原样。输入解析和预期摘要仍来自扩展，宿主计算不能证明恶意扩展输出忠实于归档。

握手 2 秒，每次试验调用总时限 5 秒，不以 progress 刷新。单进程 CPU 上限 3 秒，文件描述符 32，不允许直接文件输出，core dump 关闭。IPC 接收队列最多四帧，发送队列一帧，写管道有 2 秒上限；stderr 直接丢弃，预算为零，不转发不可信内容。取消即时撤销 sink，给 2 秒合作期限后杀死并回收进程和管道，不将 Ack 当成提交许可。资源预算是小型试验约束，不是普通大包格式的性能承诺。

English: The host owns range validation, 64MiB actual reads, 1000 records/256KiB metadata, a selected 4MiB output, byte metering, SHA-256 and no-overwrite publication. Private staging survives until successful close/clean EOF and final source/cancel checks. Invalid states, crashes, limits and cancellation discard it. Handshake/call/write/cancellation deadlines and bounded queues prevent a silent peer from hanging the host. Host hashing cannot establish fidelity when an untrusted peer supplies both parsing and expected content.

## 平台门禁

| 平台 | 试验策略与失败行为 |
| --- | --- |
| macOS | 固定 `/usr/bin/sandbox-exec` deny-default，精确私有可执行副本、系统库／dyld cache、根目录加载器读取及 `/var` 元数据。禁止用户文件、写入、网络和 fork；清空环境，继承仅 stdio。原生 rlimit 加 20ms RSS 监督，超过 512MiB 或计数读取失败即终止。RSS 为定期监督，采样间短暂峰值可超过阈值；该接口不承诺可用于未来正式分发。 |
| Linux x86_64／aarch64 | Landlock ABI ≥3，处理全部 v3 文件权利，无写规则，仅程序与系统加载库读取；`no_new_privs` 加架构核对后的 seccomp syscall 允许表，含 Rust 标准管道所需的 readv／writev，拒绝 socket、fork/clone、ptrace、io_uring 等。CPU／AS 512MiB 等 rlimit。规则、ABI、seccomp 或所需系统调用不可用，exec 前失败。原生 CI 才能建立该平台的实际结论。 |
| Windows 及其他平台 | 固定拒绝执行。未实现并验证 AppContainer、Job 和句柄白名单，不用普通子进程代替沙箱。 |

Linux API 依据[内核 Landlock 文档](https://docs.kernel.org/userspace-api/landlock.html)与 UAPI，Windows 后续依据 [AppContainer 隔离](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation)。宿主 native unsafe 限于沙箱模块，注明布局、调用前提和失败行为；pre-exec 不做分配或日志。参考 peer 仅在显式审计模式增加受控 fork／_exit／waitpid 探针，子进程不运行 Rust 清理或格式逻辑。平台回归以自有可读／可写标记、自有 localhost 监听器和 self-exec 先作正向对照，再验证内核拒绝读取、写入、联网、直接 fork 和 self-exec，核对标记、文件和连接真实状态。没有测试的平台不记通过，也不启用桌面第三方执行。

English: macOS uses an explicit deny-default Seatbelt trial with native limits and RSS supervision. Linux requires Landlock ABI ≥3 plus architecture-checked seccomp allowlisting and rlimits, failing before exec if installation is unavailable. Windows/other platforms refuse execution until their kernel isolation is implemented and verified. Owned positive controls precede real read/write/network/direct-fork/self-exec denial tests. Passing one platform never enables desktop plugins or claims another platform passed.

## 自有 XDP1 示例格式

XDP1 只用于互通，不替代任何真实归档格式。`XDP1` 四字节后为 `u32 LE` 记录数，各记录为 `u16 LE` UTF-8 名称字节数、`u32 LE` 内容数、名称与未压缩内容。最多 1000 条，不能有尾随数据，名称和内容按宿主预算读取。条目 token 为原物理序号的四字节 LE，不以路径去重。Python 的故障模式只在 conformance 测试显式指定，用于验证越权、序号、重放、重复身份、路径、信用、摘要、崩溃、取消和重复终态。

English: XDP1 is an owned interoperability fixture: magic, LE record count, then LE name/content lengths, UTF-8 name and stored bytes. It has no trailing data and preserves physical ordinal tokens. Python fault modes are explicit conformance inputs, not advertised codec capabilities.
