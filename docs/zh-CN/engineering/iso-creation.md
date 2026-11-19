# ISO9660 与 Joliet 创建

[English](../../engineering/iso-creation.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/engineering/iso-creation.md) | [日本語](../../ja-JP/engineering/iso-creation.md) | [한국어](../../ko-KR/engineering/iso-creation.md) | [Français](../../fr-FR/engineering/iso-creation.md) | [Deutsch](../../de-DE/engineering/iso-creation.md) | [Español](../../es-ES/engineering/iso-creation.md) | [Português (Brasil)](../../pt-BR/engineering/iso-creation.md) | [Русский](../../ru-RU/engineering/iso-creation.md)

首版写入器创建单卷、不可启动的数据镜像，使用 2048 字节扇区，同时写入 ISO9660 主命名空间和 Joliet 补充命名空间。依据为 [ECMA-119 第五版](https://ecma-international.org/wp-content/uploads/ECMA-119_5th_edition_december_2024.pdf)，包括附录 C 收录的 Joliet 规范。不链接或启动外部镜像制作引擎。

仅准入普通文件和显式目录。Joliet 使用大端 UCS-2，原样保留名称，每个组件最多 64 个字符。完整 Joliet 路径按标识符字节、文件版本和父目录计数合计，最多 240 字节。补充平面字符、分号、不安全的可移植路径、重复路径、缺少父目录和文件/目录冲突在写入前拒绝。主命名空间使用确定且唯一的 ASCII 别名，不替换 Joliet 视图中的原名。首版目录层级限制为包含根目录的八级，目录数量最多 65535 个且包含根目录，单文件小于 4 GiB，镜像总大小最多 20 GiB，规划的元数据最多 64 MiB。时间使用 UTC，限制在盘上可表示的范围内。

写入器先规划两套目录树和大小端路径表，再流式写入文件内容。目录记录不能跨扇区，扇区用零填充，数值的大小端表示必须一致。冻结的输入索引仅授予流访问，不授予文件系统路径。取消、输入缩短或增长、输出错误和预算耗尽均终止编码，只有宿主能够发布暂存文件。

创建后重新打开完整镜像，按解码后的 Joliet 名称匹配记录，不依赖枚举顺序，并在最终提交前把解码文件字节与冻结源逐字节比较。这不同于任意 ISO 的结构检查，ISO 不提供普通文件校验和。独立读取器必须验证主命名空间别名、准确的 Joliet 名称和文件字节。

不提供 UDF、Rock Ridge 写入、El Torito/启动盘、压缩、密码、多卷或现有镜像修改。CLI 与桌面复用相同的创建登记和任务流程。
