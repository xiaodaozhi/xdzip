# 光盘格式测试输入

`generate.py` 生成格式边界与坏输入向量，来源及限制见文件内说明。它们与独立工具生成的成功样本分别验收。

`imapi/` 保存固定 Windows IMAPI2FS 版本生成的五个小型 UDF 映像，覆盖 1.02、1.50、2.00、2.01、2.50。2.50 映像包含物理分区映射与 Metadata Partition，其他四个采用物理分区映射。这些映像仅用作开发与 CI 的独立测试参照，不是产品运行依赖。

输入仅包含项目自有测试内容：一段 UTF-8 文本，一个空文件，以及字节 0 至 255 重复 32 次形成的二进制文件。完整名称、字节数、内容哈希、映像压缩前后哈希、生成操作系统与 IMAPI2FS 版本均记录在 [sources.json](imapi/sources.json)，不包含用户文件、私人路径或 Microsoft 可执行代码。

在 Windows 上使用 [generate-optical-imapi.ps1](../../../scripts/generate-optical-imapi.ps1) 重新生成语义相同的输入：

```powershell
pwsh -File scripts/generate-optical-imapi.ps1 -Output build/optical-imapi
```

输出目录必须不存在或为空。生成器调用 [Microsoft IFileSystemImage](https://learn.microsoft.com/en-us/windows/win32/api/imapi2fs/nn-imapi2fs-ifilesystemimage) 与 [UDFRevision API](https://learn.microsoft.com/en-us/windows/win32/api/imapi2fs/nf-imapi2fs-ifilesystemimage-put_udfrevision)，不挂载映像或访问刻录机。生成时间、系统版本与实现细节可能改变描述符和最终映像字节，不保证跨系统生成相同映像哈希。

固定样本在使用前校验 `sources.json` 中的大小与哈希。结构、命名空间与文件内容使用独立已知字节以及官方参照后端逐项验收，不能用两个自研解析器互相成功替代互操作证据。实际结果、分区映射覆盖与剩余路径由迁移 issue 跟踪。

`udftools/` 保存独立 mkudffs 2.3 与 udfclient 0.8.11 生成的 UDF 2.01 sparable 映像，采用同一组自有内容，二进制文件名为 `nested/data.bin`。它包含标准 `*UDF Non-Allocatable Space` 系统元数据流。压缩后约 130 KiB，逻辑长度为 128 MiB，测试准备器按固定哈希与尺寸流式展开，不将整像收集到内存。

[udftools/sources.json](udftools/sources.json) 记录作者工具的上游来源、Ubuntu 包版本、工具许可证及输入哈希。工具仅作独立测试参照，其源码和二进制不进入产品运行依赖。生成器 [generate-optical-udftools.py](../../../scripts/generate-optical-udftools.py) 在一次性 Ubuntu 24.04 Docker 容器内创建映像，写入三个已知文件后，以 udfclient 只读取回并逐字节比较：

```sh
python3 scripts/generate-optical-udftools.py --output build/optical-udftools
```

输出目录必须不存在或为空，容器只绑定该目录，不挂载映像或访问任何设备。128 MiB 容量用于满足作者工具的小卷写入保留空间要求。创建时间、卷标识及作者工具环境可能改变映像哈希，重新生成的结构与内容按相同的独立字节规则验收。旧官方参照后端拒绝此 sparable 映像，不能记录为官方参照成功；其独立来源由 mkudffs 作者工具、udfclient 写入与只读回读支撑。
