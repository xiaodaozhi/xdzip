# 浏览、选择与预览文件

[English](../../user-guide/browse.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/user-guide/browse.md) | [日本語](../../ja-JP/user-guide/browse.md) | [한국어](../../ko-KR/user-guide/browse.md) | [Français](../../fr-FR/user-guide/browse.md) | [Deutsch](../../de-DE/user-guide/browse.md) | [Español](../../es-ES/user-guide/browse.md) | [Português (Brasil)](../../pt-BR/user-guide/browse.md) | [Русский](../../ru-RU/user-guide/browse.md)

1. 打开压缩包。在目录树中选择文件夹，或双击列表中的文件夹进入。用面包屑路径返回上一级。
2. 输入文件名的一部分并按回车。搜索范围是当前目录及其后代。可以切换列表或网格视图并选择排序方式，升序和降序都将目录排在文件之前。
3. 用复选框或平台支持的选择快捷键勾选条目。“全选”作用于当前查询，包括视图中尚未显示的行。解压前先核对已选项数量。
4. 打开文件属性，主动请求预览普通文件。文本须为 UTF-8 或带 BOM 的 UTF-16，最大为 256 KiB。图片有单独的大小限制。静态 SVG 须符合支持范围，HEIC／HEIF 只在 macOS 可预览。若无法预览且该条目允许解压，仍可将文件解压出来。[预览格式和限制](../engineering/content-preview.md)

双击普通文件时，XDZip 会解压临时副本，再请系统打开。修改副本不会写回压缩包。要长期保留文件，请解压到自己选择的位置。[临时打开说明](../engineering/archive-opening.md)

默认在标签页打开压缩包，也可改为独立窗口。普通单文件 ZIP／ZIP64 可索引最多一百万条物理记录。大索引按需加载列表行。使用磁盘索引时仍可解压所选的普通文件，整包测试和修改不可用。[大归档限制](../engineering/large-index.md)
