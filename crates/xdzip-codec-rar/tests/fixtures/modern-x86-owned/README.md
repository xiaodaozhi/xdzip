# 现代 RAR x86 已知原文互操作配方

本配方使用官方 RAR 作者程序生成健康压缩流，以独立已知原文核对现代 E8 与 E8/E9 过滤器。作者程序、PE 数据副本、归档、TSV 和日志仅生成到 `build/`，不提交二进制，也不执行输入副本或解出的文件。不实现 RAR 编码器。

## 输入与作者身份

- 通过仓库的 [参考工具准备入口](../../../../../scripts/prepare-native-interop.py)取得官方 7zr 26.05。Windows `7zr.exe` 的 SHA-256 必须为 `9bf582b894dfa665f852d7e9112b02290a1c303c2d95a21fcdc41ffcb03143b7`，作为数据复制到实验目录。
- 用正常 Foundation 流程构建本项目 Windows x64 native worker，记录其完整 SHA-256，再复制为另一个 PE 数据输入。解码时只启动单独授权的原始 worker 路径；复制的数据路径不启动、加载或注册。
- 官方 RAR 7.23 作者程序的 SHA-256 为 `f561764bc3e9ed208744321a89a819b562edeaf06e203c02a06976121fda1991`。工具仅用作独立测试作者，不进入产品构建、部署或运行回退。

从实验目录分别对两个输入执行正常压缩配置：

```text
<official-Rar.exe> a -cfg- -idq -y -mt1 -tsm- -s- -ep -md32m -m3 <case>.rar <input>.exe
```

作者工作集上限为 1 GiB，时间上限为 45 秒；可用物理内存低于 2.5 GiB 或可用提交低于 1.5 GiB 时停止。要求作者返回 0、官方 UnRAR `t` 返回 0，归档中实际方法非 Store、现代生效算法为 0。请求的 32 MiB 字典允许作者正常缩小，但不能修改头部来制造算法或过滤器身份。

## 独立字节与实际过滤器核验

每次冻结输入和归档的完整 SHA-256。在实验目录写入 `cases.tsv`，每行七个字段用真实制表符分隔：

```text
<case>  50  <case>.rar  <archiveSHA256>  <input>.exe  <originalSHA256>  <requiredKind>
```

`requiredKind=1` 要求实际 E8 描述符，`requiredKind=2` 要求实际 E8/E9 描述符。设置 `XDZIP_RAR_FILTER_AUTHOR_FIXTURES` 为该实验目录，运行已有的显式测试：

```text
cargo test -p xdzip-codec-rar --locked modern_independent_author_known_bytes_and_filter_descriptors -- --ignored --nocapture --test-threads=1
```

测试同时核对输入及归档摘要、完整直接 LZ 输出、完整 Payload 输出与 CRC，并检查实际过滤器描述符；描述符计数须伴随完整解码成功，不能单独当作成功变换证据。另用 [native 协议检查器](../../../../../scripts/check-native-codecs.py)的 `run_worker` 以 `format_id=4`、`selected=0`、`digest_only=True` 完整解码，要求尺寸已知，完整字节数、SHA-256、CRC 终态及实际解码工作一致，不保存解出文件。

过滤器选择是作者的启发式行为，不能由 `.exe` 后缀、工具版本或 PE 类型推断。编译器或文件元数据变化可能改变本项目 PE 及归档身份；每次重新记录身份并检查实际种类，未选中所需种类就不能计为该过滤器的互操作成功。该配方不证明 RAR7 算法 1、加密、固实、跨卷过滤器或全部历史组合通过。

English: This opt-in recipe treats a pinned official 7zr and a separately built project worker as known PE data. Only the authorized decoder path is launched; copied inputs and extracted members are never executed. A pinned official RAR7.23 author creates ordinary modern algorithm0 archives under bounded memory and time. Full original hashes, direct LZ bytes, CRC-verified Payload output, actual filter descriptors and the real worker's complete output hash must agree. Recheck heuristic selection on every new build. All binaries and evidence stay in build; this does not establish algorithm1 or encrypted, solid or split coverage.
