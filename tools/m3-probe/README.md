# M3 真实解压探针

生成 64 个 Deflate 文件、8 个目录、32 MiB 确定性内容，在自有临时目标中执行真实解码、文件同步、原子提交，然后通过独立读句柄逐字节复核。不会用零字节元数据样本替代解压工作负载。

```sh
cargo build --release --locked -p xdzip-m3-probe
python3 scripts/measure-m3-core.py --binary target/release/xdzip-m3-probe --output /tmp/xdzip-m3-performance.json
```

脚本创建并清理自己的临时目录。探针直接调用时要求提供自有 ZIP 样本及明确的空目录，输出实际文件；不会自行删除所给目标。压缩样本只写入 build／临时目录，不进入 git。

自有夹具位于探针报告的原生临时目录基址，与 Rust／Qt 文件系统回归一致；Windows 继承私有父目录的 ACL，不用 Python 版本相关的 0700 权限合成。输出仍通过实际 Grant 权限检查。CLI 保留所选路径，不用 canonicalize 消除链接；失败报告只包含稳定错误码和计数，原生权限诊断仅用于新建自有目标。

`--generic --fixture-password-stdin` 仅供 `scripts/check-advanced-creation.py` 使用固定自有 AES 夹具，通过有界 stdin 读取测试密码并响应一次运行时请求。此探针不随应用发行，不是产品密码 CLI；产品密码只通过独立控制槽提交，密码不进入命令参数、任务快照或验证报告。该脚本在 macOS／Linux 用独立 7-Zip 验证 Store／Deflate 双向互操作，外部工具也通过终端提示获取夹具密码。

报告中的解压时间包括安全目标检查、解码、fsync、冲突判断、提交和私有清理；独立内容验证时间另列。Linux 读取进程 VmHWM，macOS 通过 `/usr/bin/time -l` 读取峰值 RSS，Windows 此项为未测量。只记录单次原生／本地环境，缓存、共享 runner 和文件系统会影响结果，不作为性能承诺。
