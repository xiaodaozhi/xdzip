# M3 真实解压探针

生成 64 个 Deflate 文件、8 个目录、32 MiB 确定性内容，在自有临时目标中执行真实解码、文件同步、原子提交，然后通过独立读句柄逐字节复核。不会用零字节元数据样本替代解压工作负载。

```sh
cargo build --release --locked -p xdzip-m3-probe
python3 scripts/measure-m3-core.py --binary target/release/xdzip-m3-probe --output /tmp/xdzip-m3-performance.json
```

脚本创建并清理自己的临时目录。探针直接调用时要求提供自有 ZIP 样本及明确的空目录，输出实际文件；不会自行删除所给目标。压缩样本只写入 build／临时目录，不进入 git。

报告中的解压时间包括安全目标检查、解码、fsync、冲突判断、提交和私有清理；独立内容验证时间另列。Linux 读取进程 VmHWM，macOS 通过 `/usr/bin/time -l` 读取峰值 RSS，Windows 此项为未测量。只记录单次原生／本地环境，缓存、共享 runner 和文件系统会影响结果，不作为性能承诺。
