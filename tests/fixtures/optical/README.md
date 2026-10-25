# 光盘格式测试输入

`generate.py` 生成格式边界与坏输入向量，来源及限制见文件内说明。它们与独立工具生成的成功样本分别验收。

`imapi/` 保存固定 Windows IMAPI2FS 版本生成的五个小型 UDF 映像，覆盖 1.02、1.50、2.00、2.01、2.50。2.50 映像包含物理分区映射与 Metadata Partition，其他四个采用物理分区映射。这些映像仅用作开发与 CI 的独立测试参照，不是产品运行依赖。

输入仅包含项目自有测试内容：一段 UTF-8 文本，一个空文件，以及字节 0 至 255 重复 32 次形成的二进制文件。完整名称、字节数、内容哈希、映像压缩前后哈希、生成操作系统与 IMAPI2FS 版本均记录在 [sources.json](imapi/sources.json)，不包含用户文件、私人路径或 Microsoft 可执行代码。

在 Windows 上使用 [generate-optical-imapi.ps1](../../../scripts/generate-optical-imapi.ps1) 重新生成语义相同的输入：

```powershell
pwsh -File scripts/generate-optical-imapi.ps1 -Output build/optical-imapi
```

输出目录必须不存在或为空。生成器调用 [Microsoft IFileSystemImage](https://learn.microsoft.com/en-us/windows/win32/api/imapi2fs/nn-imapi2fs-ifilesystemimage) 与 [UDFRevision API](https://learn.microsoft.com/en-us/windows/win32/api/imapi2fs/nf-imapi2fs-ifilesystemimage-put_udfrevision)，不挂载映像或访问刻录机。生成时间、系统版本与实现细节可能改变描述符和最终映像字节，不保证跨系统生成相同映像哈希。

固定样本在使用前校验 `sources.json` 中的大小与哈希。结构、命名空间与文件内容使用独立已知字节以及官方参照后端逐项验收，不能用两个自研解析器互相成功替代互操作证据。实际结果与未完成的 VAT、sparable 路径由迁移 issue 跟踪。
