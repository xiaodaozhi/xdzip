# 自有 RAR29 固实 Store 混排夹具

`archive.rar` 是未修改的官方 RAR 6.24 输出，仅包含项目自有数学原文。作者按 RAR4 格式和给定顺序生成三个文件，实际头为 `UnpVer29 / m3 / 非固实`、`UnpVer20 / Store / 固实`、`UnpVer29 / m3 / 固实`。归档为 197,445 字节，原文共 458,755 字节。工具来源、固定 SHA-256、完整参数和逐成员 CRC/SHA 见 `sources.json`。

原文按以下公式生成，不保存原文文件：

1. `first.bin[i] = (31*i + 7*(i>>5) + 13) mod 256`，`0 <= i < 131071`。
2. 从 `k=0` 开始连接 `SHA256(ASCII("xdzip-rar20-store-middle-v1") || LE32(k))`，取前 196,613 字节为 `middle.rnd`。
3. `third.bin = first.bin[173:] || first.bin[:173]`。

官方作者及固定独立 UnRAR 对归档进行 `t` 校验，并以 `p` 核对全部成员原文。冻结交接还记录新 Rust worker 为每个逆序选择启动新进程，完整 SHA/CRC 一致。该工具证据不冒充持久 NativeReader 或 GUI 验收；执行日志保留在本地构建工件。

普通组件测试固定原始归档身份和真实文件头，对同一 reader 的顺序、逆序和重复选择，以及新 reader 的逐项逆序选择，逐字节比较完整数学原文并核对 SHA/CRC。选择第三个文件时必须恢复第一个压缩文件的状态并实际处理 Store 前缀，Store 不能成为新的固实恢复点、清空压缩状态或写入压缩字典。损坏 Store 前缀、宿主输出拒绝、工作预算拒绝和取消均应失败且不能输出所选第三文件，失败 reader 不能再继续。

此夹具仅验证未加密的 RAR29 压缩状态跨真实 UnpVer20 Store 记录，不代表所有 13/15/20/26 族混排、加密或分卷组合。归档与自有数学原文遵循仓库许可；官方工具的许可和来源独立保留，不分发作者程序、安装器、参考源码或构建日志。归档字节已固定，作者头部时间等字段不保证重复生成时逐字节相同。

English: This unchanged official RAR 6.24 archive contains only project-owned mathematical input defined above. The physical sequence is actual UnpVer29/m3/nonsolid, UnpVer20/Store/solid, then UnpVer29/m3/solid. The 197,445-byte archive represents 458,755 original bytes. Ordinary tests pin its identity and headers, compare every original byte plus SHA/CRC in sequential, reversed and repeated selections, and separately reopen each reversed member. They require Store to preserve compressed solid state and not become a recovery point. Corrupt Store prefixes, rejected output/work and cancellation must fail without selected-target output and leave the reader poisoned. Frozen official `t`/`p` and fresh Rust-worker reverse-selection evidence are provenance, not a claim of persistent NativeReader or GUI acceptance. This one unencrypted RAR29/Store combination does not establish every older generation, encrypted or split combination. Only the small archive and source recipe are vendored under the repository license; author executables, installers, plaintext, reference source and execution logs are excluded. Archive bytes are pinned rather than claimed reproducible across writer timestamps.
