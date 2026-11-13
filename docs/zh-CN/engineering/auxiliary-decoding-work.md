# 算法辅助工作计量

解压和检查分别累计文件解码字节、实际输出与算法辅助工作。PPM 模型初始化、容量重启、控制及参数解码等辅助工作不是文件内容，不能增加 `decoded_bytes`、`written_bytes`、文件进度或压缩膨胀比，也不能产生预计费交付额度。任务快照不新增面向用户的辅助计数字段。

`DecodedSink::charge_auxiliary(bytes)` 只报告已经执行的辅助工作，默认实现不改变任何文件计数。算法先用自己的剩余预算准入未来工作，执行后再结算实际数量，包括失败和取消前已经完成的部分。未执行的预留不报告为实际工作，已执行工作不因错误、模型清除或密码重试退款。RAR 的私有总工作预算继续约束文件解码与辅助工作之和。

## 隔离边界

- worker 协议 `AUX_WORK=24` 携带唯一小端 `u64` 数量，不附带内容。每包必须为 1..65,536，帧必须精确结束，宿主成功计量后回复空 `ACK`。空数量、截断、尾随字段、超量或非法操作阶段均拒绝。
- 该消息仅允许 RAR 的负载解码，不能用于元数据扫描、创建、修改或其他格式。CXX 同步传递数量，不保留 Rust 指针；仍有的源和输出切片只在调用期间借用。C++ 异常在回调内映射为稳定错误，Rust unwind 不跨 CXX，abort 或进程退出使未完成输出失败。
- Native 负载实例单独累计最多 20 GiB 辅助工作，跨条目、worker 重启和密码更换保留。它在调用宿主 sink 前记录已执行数量；sink 拒绝不会使后续重试获得退款。`AUX_WORK` 不增加 `WorkerCredit`。
- runtime 解压和检查任务再独立累计最多 20 GiB 辅助工作，计量对象位于条目及密码重试循环外。此额度与实际文件字节和输出限额分别检查。超限或取消返回失败，不出版未完成的文件，也不依靠更改公开计数来规避限制。
- `WORK=17` 继续表示已经解码的文件字节。只有其成功准入能为同一个有效工作进程的 `PRECHARGED_DATA=23` 提供一次性交付额度，交付前扣减，重启即清零。RAR、7z 与 CAB 使用该路径，扫描与创建不能使用。辅助数量不能用于支付这类文件输出。

高压缩率准入使用冻结的声明尺寸与 packed/整包尺寸，不引用辅助计量。执行期文件解码和输出限额仍独立成立。一个 87 字节文件执行数 MiB 模型初始化后，文件计数仍只反映其实际 87 字节内容。

## 必须保持的验收条件

- 正常小文件在 64 KiB 以上辅助工作及模型初始化后可以完成解压和检查，文件计数精确，最终内容校验成立。
- Native 和 runtime 分别验证实际重复辅助包到达 20 GiB 后拒绝；密码重试和 worker 更换不重置累计成本，不出现未完成文件的提交。
- 伪 worker 的其他格式、元数据阶段、空包、超包、尾随字段和仅以辅助工作申请预计费输出均失败。
- 正常 `WORK` 可支付同一 worker 的 RAR 预计费输出，额度只能消费一次；错误或重启后不能复用。
- 协议夹具只验证状态机和计量。完整 Rust RAR worker、runtime 业务链、三平台和桌面验收分别保存实际证据，不由协议夹具成功推定。

协议夹具仅编译项目自有 `tools/native-codec/main.cpp` 与自有 stub，不含压缩算法：

```sh
cmake -S crates/xdzip-codec-native/tests -B build/auxiliary-protocol
cmake --build build/auxiliary-protocol --config Release
```

将 `XDZIP_NATIVE_CODEC_WORKER` 指向该夹具，并设置 `XDZIP_AUXILIARY_PROTOCOL_FIXTURE=1` 后，显式运行 `cargo test -p xdzip-codec-native auxiliary_protocol -- --ignored`。普通定向回归使用 `cargo test -p xdzip-codec-native -p xdzip-runtime auxiliary`，通用默认 sink 的兼容行为由 codec API 测试核验。生成证据只进入 build 或 CI，任务结果和剩余项由[迁移任务 #51](https://github.com/xiaodaozhi/xdzip/issues/51)登记。

English: Auxiliary algorithm work is separate from decoded file bytes, writes, expansion ratios, progress and prepaid delivery credit. Algorithms privately admit future work and report only completed work through `DecodedSink::charge_auxiliary`, including work performed before failure. `AUX_WORK=24` carries one little-endian u64 count from 1 to 65,536 and requires an ACK after successful host accounting. Only RAR payload decoding may use it. Native payload instances and runtime tasks independently enforce cumulative 20 GiB auxiliary limits across entries, worker restarts and password attempts, without refunds or public UI counters. Synchronous CXX callbacks retain no Rust pointers and convert exceptions before crossing the boundary. Existing file WORK can fund a single prepaid delivery in an admitted format; auxiliary work never can. Small-file correctness, malformed/scope-forged packets, repeated limits, retries, cancellation and credit isolation require separate checks. The authored protocol stub proves transport behavior only; complete RAR worker, runtime, platform and desktop acceptance require their own evidence.
