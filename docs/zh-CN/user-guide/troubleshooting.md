# XDZip 常见问题与排错

[English](../../user-guide/troubleshooting.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/user-guide/troubleshooting.md)

| 遇到的问题 | 可以怎么处理 |
| --- | --- |
| 压缩包无法打开 | 核对[格式支持](../../../README.zh-CN.md#格式支持)、压缩或加密方式及完整卷组。若来源不可访问，请保留原名，将归档及所有分卷复制到可访问的本地目录后重新打开。 |
| 文件名乱码或无法写出 | 确认能否用 UTF-8 文件名重新打包。XDZip 可尝试识别部分旧编码，短文件名或混合编码有一定限制。只提取可以安全映射的条目。[文件名处理](../engineering/filename-encoding.md) |
| 属性可见，文件无法预览 | 核对文件类型、尺寸、平台和编码。预览与提取有各自的限制。[预览限制](../engineering/content-preview.md) |
| 输出目标被拒绝或任务只完成一部分 | 改选可写的私有目录，检查重名与不支持的条目，重试前核对任务结果。[解压文件](extract.md) |
| 操作期间源文件被替换 | 打开新的副本并重新选择条目。XDZip 在写入前会重新检查来源。 |
| Explorer、Finder 或 Files 没有出现操作入口 | 核对安装渠道及扩展是否已启用。Windows 11 的经典菜单位于“显示更多选项”。macOS 可在系统设置中启用 Finder 扩展，或使用已登记的服务。也可直接在 XDZip 中打开归档。[从文件管理器解压](extract.md#从文件管理器解压) |
| 大型 ZIP 无法测试或修改 | 如果 XDZip 正在使用磁盘索引，可只选择需要的普通文件，在任务限额内解压。[大归档限制](../engineering/large-index.md) |
| 更新或安装失败 | 核对当前版本和发行渠道。手动下载时先校验文件，再按平台安装步骤处理。[更新恢复](settings-and-updates.md) |

反馈问题时，请提供“关于 xdzip”中的版本和构建号、系统与架构、压缩格式和操作、复现步骤，以及预期和实际结果。尽量使用自制小样本，分享前移除密码和私人路径。[反馈问题](https://github.com/xiaodaozhi/xdzip/issues/new/choose)
