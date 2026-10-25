# CAB 测试输入

`lzx-*.cab.gz` 与 `mszip-history.cab.gz` 是 Microsoft Windows makecab 生成的固定小型测试输入，合计约 78 KiB，仅包含本项目自有的确定字节。它们覆盖七种 LZX 窗口、多帧、同 folder 跨文件切点、空文件、aligned 块与 MSZIP 前帧字典依赖。来源、作者工具版本与 SHA256、操作系统、输入配方、文件内容哈希和压缩前后归档哈希见 [lzx-sources.json](lzx-sources.json) 与 [mszip-sources.json](mszip-sources.json)。不包含用户文件或 Microsoft 可执行代码，也不成为产品运行依赖。

[generate-makecab.py](generate-makecab.py) 在所有 CI 平台只使用 Python 标准库校验固定压缩输入、展开到 `build/`，并从公开配方重新生成已知原文，逐项核对文件大小和内容哈希。CI 不安装、下载或运行 makecab：

```sh
python3 tests/fixtures/cab/generate-makecab.py --output build/cab-makecab
```

Windows 开发环境可用 `--author` 调用系统 [makecab](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/makecab)，在不存在或为空的 `build/` 子目录重新生成语义相同的输入。生成器只把自己写入的三个已知文件列入 DDF，不扫描目录或接纳其他文件，不挂载和访问设备。MSZIP 作者验证使用独立 Python zlib 检查每帧终止、完整原文，并确认后帧没有前 32 KiB 字典时解码失败。生成时间、系统与工具可能改变 CAB 元数据和归档哈希，不保证跨环境生成相同 CAB 字节；固定文件使用固定哈希，重新生成的内容使用已知配方核验。

```powershell
python3 tests/fixtures/cab/generate-makecab.py --author --output build/cab-makecab-new
```

[generate-quantum.py](generate-quantum.py) 是项目自有的数学测试编码器，其来源与限制单独见 [quantum-sources.json](quantum-sources.json)。这些编码流经过固定官方参照解码器逐字节验证，但不是独立第三方 Quantum 压缩工具生成的样本，不能与 makecab 的作者来源混称。

旧八个 CAB 物理输入保留在 `tests/fixtures/cab-iso/`，其中五个物理卷组成同一逻辑 folder，原来源和许可证继续适用。实际直读、独立参照和宿主验证结果由迁移 issue 与本地/CI 报告分别记录，不能用自研编码器和自研解码器相互成功代替互操作，也不把测试路由工作进程当作已经切换的默认产品路由。
