# xdzip：06 格式适配与扩展 SDK

## 1. 两类扩展

**内置源码级扩展**实现 Rust SDK，随产品构建，使用 Cargo feature 挑选后端。Rust 编译器 ABI 不稳定，不将它们做成任意版本可加载的 `.so/.dylib/.dll`。

**进程级扩展**实现版本化线协议，由 `xdzip-plugin-host` 适配为同一格式接口。外部扩展可以用 Rust、C++、Go 等语言编写，独立发布。线协议与 Qt 的 cxx 接口没有关系；外部进程永远不能创建 Qt 对象或弹出自己的密码窗口。

官方原生后端也可使用工作进程协议，以隔离内存错误；“官方”不免除输入检查。第三方发布在沙箱验证阶段前默认不可执行。

## 2. 格式能力

| 能力组 | 字段/内容 |
|---|---|
| 识别 | 格式 ID、魔数探测、所需最大探测字节、容器/流层 |
| 读取 | 元数据扫描、按项读取、仅顺序读取、CRC/摘要、目录加密 |
| 写入 | 新建、允许算法、压缩级别范围、参数 schema、流式输出 |
| 加密 | 具体算法、读/写、文件名加密；不能只写 `encryption=true` |
| 分卷 | 读/写、卷命名和顺序、缺卷行为 |
| 执行 | seek 需求、每会话并发、内部线程、内存估算、取消安全点 |
| 特殊结构 | 固实块、链接、时间精度、编码、权限与额外属性 |

每次打开生成 `EffectiveCapabilities`，取后端声明、实际归档特性、宿主政策和平台能力的交集。QML 根据这个集合显示选项，核心启动时再次检查。

探测只允许读取有限字节，不执行任意扫描。多个候选按确定性优先级和可信来源选择；低信任扩展不能用“探测置信度更高”抢占内置后端。用户可在详情中选择已安装兼容后端，重新打开会话。探测错误不能触发无限递归尝试。

## 3. 分层和首批格式

TAR 是容器，GZIP 是压缩流；`tar.gz` 组合为有限层数的 Source → GzipDecoder → TarReader。ZIP/7z 由各自容器处理。层组合由已登记能力和魔数确定，最大 4 层，不根据文件名无限拆后缀。嵌套归档是用户打开一个条目的新会话，不是解压时自动递归展开。

| 格式 | v0.1 | 后续 | 依赖选择门槛 |
|---|---|---|---|
| ZIP Store/Deflate | 浏览、读、创建 | 具体加密算法、其他压缩方法、分卷 | 候选 `zip` 等 Rust 实现，审计 features、ZIP64、重复名、CRC、取消和许可证 |
| TAR | 浏览、读、创建 | 稀疏项等独立能力 | 候选 `tar`，不得调用未受宿主约束的直接 unpack |
| GZIP/TAR.GZ | 读/写及组合 | 多 member 等逐项验证 | 候选 `flate2`，明确后端实现及许可证 |
| BZIP2/XZ/Zstd 层 | 未支持 | v0.2 后逐层接入 | 限制窗口/字典、原生依赖、内存预估 |
| 7z | 未支持 | 先真实读，再决定写/加密/固实 | 7-Zip SDK、库或工作进程候选逐项审查，不默认等价全功能 |
| RAR | 未支持 | 条件满足时只读 | 实现/许可、分卷和加密验证；不承诺 RAR 创建 |

表中 crate 是候选，不表示当前代码已集成或其默认 API 满足安全要求。源码、版本、feature、构建配置、互操作结果和许可证通过清单后才能锁入 Cargo.lock/SBOM。

## 4. Rust SDK 契约

概念接口：

```text
CodecFactory.probe(ProbeSource, ProbeBudget) -> Detection
CodecFactory.open(ReadSource, OpenOptions, OperationContext) -> CodecSession
CodecSession.scan(MetadataSink, OperationContext) -> ScanReport
CodecSession.read_entry(EntryToken, OutputSink, OperationContext) -> ReadReport
CodecFactory.create(InputCatalog, ArchiveSink, WriteOptions, OperationContext) -> WriteReport
```

这是职责示意，不是已编译签名。实现阶段 trait 返回类型保留 `Result`，并显式标明 Send/并发要求。`OperationContext` 含取消、预算、进度和短期秘密访问，不能把全局应用服务/文件系统放进去。

ReadSource 只暴露读取、有限 seek 和已授权卷；OutputSink 只能写当前批准条目，不暴露目标路径。ArchiveSink 只能写新归档临时对象。适配器主动每个安全点检查取消；宿主计量实际字节并返回限额错误。适配器不能捕获错误后继续偷偷写。

普通解压 OutputSink 为顺序写；ArchiveSink 如需修订头部，可协商受限 seek/write，所有偏移及加法检查溢出，高水位文件长度和累计写入均受额度限制，不能以 seek 制造超额稀疏文件。无需 seek 的后端优先流式写。

扫描向 MetadataSink 输出原始路径、record token、大小、类型和属性；Sink 有批次和字段限额。后端声明的大小不作为配额可信输入。结束时返回完整性和扫描计数，不能只返回一个 bool。

如格式需再次 seek，同一个授权源仍受 SourceRevision 检测；如 TAR/固实块只能顺序解码，计划按物理顺序处理已选项并跳过无关内容，同时计量解码工作，不能承诺 O(1) 任意条目读取。

## 5. 插件包与 manifest

独立包包含 manifest、目标平台 executable、许可证、校验和、可选签名及能力 schema。manifest 至少包含 `plugin_id`、版本、作者、支持平台/架构、协议主版本范围、格式能力、入口、资源请求和来源。

入口是包内相对路径；禁止 `..`、绝对路径、链接逃逸及 shell 命令字符串。宿主使用绝对可验证可执行文件及参数数组启动，不使用 `sh -c`。清理环境变量，不传应用凭据、代理密码、用户 PATH 扩展等。

manifest 草案示例（哈希/签名由包清单提供；仅演示字段，不是可安装扩展）：

```json
{
  "manifest_version": 1,
  "plugin_id": "org.example.xdzip.reader",
  "version": "0.1.0",
  "author": "Example Maintainer",
  "protocol_majors": [1],
  "targets": ["macos-aarch64"],
  "entrypoint": "bin/xdzip-reader",
  "formats": [{"id": "example", "scan": true, "read": true, "create": false}],
  "permissions": {"broker_source_read": true, "broker_output_write": true, "network": false},
  "resources": {"memory_bytes": 268435456, "threads": 1},
  "license_files": ["LICENSE"]
}
```

格式能力 schema 比此示例更完整，必须声明算法和读取模式；未知 permission 不自动授予。源码 SDK 和 manifest/协议的类型定义随 conformance 工具统一发布。

安装包作为不可信归档由同一安全提取服务解包到暂存目录，验证清单/签名/平台后原子发布。签名说明发布者身份，不能证明代码安全。用户第一次启用能看到来源、格式和权限；协议兼容和权限变更需要重新评估，不能静默继承旧授权。

## 6. 线协议 v1 草案

传输为私有继承 IPC 管道；不监听 TCP，不创建公开端口。双向帧是 `u32 little-endian 长度 + CBOR 对象`。在分配前检查长度；每帧最大 1MiB，数据消息中 bytes 最大 256KiB。CBOR 深度最大 16，不允许无界长度、重复关键字段或未知必需消息种类。

包络为 `{kind, request_id, operation_id, sequence, payload}`。CBOR 使用固定字段键和类型；字段 schema 随协议包发布。首帧只允许 Hello，后端在 2 秒握手预算内声明版本，宿主选择共同主版本和协商能力。相同主版本新增可选字段允许忽略；未知枚举/必需字段不忽略；不兼容变化升级主版本。

| 消息组 | 消息 | 契约 |
|---|---|---|
| 握手 | Hello、Welcome、Reject | 版本、后端身份、资源限额、协商能力 |
| 调用 | Probe、Open、Scan、ReadEntry、Create、Close | request/operation ID 唯一；先握手再调用 |
| 受限 I/O | ReadAt/ReadNext、ReadData、BeginOutput、WriteData、FinishOutput | 句柄令牌、偏移、数量、信用额度；没有真实输出路径 |
| 元数据 | EntryBatch、ScanFinished | 主机校验字段、记录令牌唯一及数量 |
| 交互 | NeedSecret、NeedVolume、SupplySecret、SupplyVolume | UI 由宿主呈现；秘密仅限对应操作 |
| 控制 | Cancel、CancelAck、Ping/Pong、Shutdown | 控制优先；收到 Ack 不等于任务已清理 |
| 结果 | Progress、OperationFinished、Error | sequence 单调；一次终态，不信任声称的字节/CRC |

每个数据方向默认最多 4 个 256KiB 未确认块，信用耗尽时停止发送；控制消息不被数据信用阻塞。宿主验证 operation、句柄、范围和当前状态。超额/乱序/重复终态/非法状态转移关闭工作进程并令任务失败。

关键 payload 约束如下；整数使用无符号有界类型，字节字段使用 CBOR bytes，而非把原始路径强行转 UTF-8：

| 消息 | 必需字段及约束 |
|---|---|
| Open | source_token、options；token 绑定当前 operation 的授权源 |
| EntryBatch | session_token、entries；记录含 entry_token/raw_path/type/optional sizes，单批最多 128 且帧不超 1MiB |
| ReadEntry | session_token、entry_token、approved_sink_token；不在当前计划内的条目不能获得 Sink |
| ReadAt | source_token、offset:u64、length:u32；length ≤256KiB，范围不越授权源，EOF可短读 |
| WriteData | sink_token、block_sequence、bytes；只能按批准顺序/seek能力写，重复块拒绝 |
| FinishOutput | sink_token、reported_size、checksum_info；宿主使用实测字节和自身计量决定是否提交 |
| OperationFinished | outcome、counts、integrity_info；操作只允许一次终态，存在未完成 Sink 时不能声明成功 |

令牌不可复用到另一会话、任务或工作进程；取消撤销后不能继续 BeginOutput。Create 的输入目录只包含用户已授权输入的相对归档名和读取令牌，不允许扩展按本机路径自行增添文件。

宿主需要独立 stdout/stdin IPC 和有界 stderr 诊断；插件日志不能混入协议通道。诊断默认不记录秘密及完整路径，并对总量限流。宿主不能假定未知插件会自觉遵守日志隐私，因此插件启用说明必须涵盖其对用户内容和密码的接触。

宿主重新计量实际写入并计算可验证的校验值，但如果预期 CRC/摘要及解析结果也完全来自不可信插件，这不能独立证明内容忠实于原归档。沙箱保障的是权限边界，内容正确性仍依赖后端信任和互操作；需要更强保证时使用独立受信解析/校验，不宣称恶意插件可以被通用 broker 自动验证正确。

## 7. 文件权力与沙箱

所有源、卷和输出由 broker 以操作令牌提供。默认不给第三方原始源路径或广泛目录句柄；只授权受限读范围和某个当前输出 Sink。密码算法需要明文秘密时，插件不可避免会接触该秘密；不能宣称宿主能让恶意解密插件永远看不到密码。

Windows 评估受限令牌/AppContainer、Job 对象和句柄白名单；Linux 评估 namespaces、seccomp、Landlock/挂载限制、cgroups；macOS 评估可支持发行方式下的沙箱 helper、entitlements 和 broker。每个平台都需实证禁止任意写、联网、派生逃逸和访问未授权文件。具体 API 可用性不由本设计假定。

无法实现/验证沙箱的构建禁用第三方执行；官方受信工作进程可以按发行政策单独允许，但界面不能标为“沙箱安全”。沙箱不保护同一授权归档的机密免受用户主动选择的恶意格式扩展读取，因此来源、权限与信任仍是必要条件。

## 8. 生命周期与失败

每后端默认最多一个工作进程；每进程默认一个活动重任务。宿主监督内存、协议大小、响应和 CPU/无进展预算。无进展预算不能简单把“60 秒未完成”当崩溃；读取复杂块时结合心跳、字节/进度和用户取消判断。

取消后给后端 2 秒合作期限；未退出可终止工作进程。终止只释放未提交输出，不撤回已提交条目。后端崩溃不自动无限重试，同一输入连续崩溃隔离后端并提示。更新等待所有关联任务结束，旧包保留至引用归零。

## 9. 开源扩展开发流程

开发者实现参考骨架 → 声明真实能力 → 运行 SDK conformance → 提交最小及对抗性样本 → 独立参考工具互操作 → 检查取消/配额/损坏 → 许可证/SBOM → 平台包验证。提供源码级 ZIP/TAR 样例和进程级只读示例，文档按接口定义生成。

Rust SDK 使用 SemVer，破坏 trait 变更升级 major；线协议另有 major/minor。协议冻结前须由两个独立实现完成互通，不能只测宿主与自己生成的同一份代码。支持窗口至少覆盖当前及上一受支持协议主版本，具体撤销周期在 v1.0 发布政策确定。
