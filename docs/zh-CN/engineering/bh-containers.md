# BH 容器与准入的 BlakHole 配置

[English](../../engineering/bh-containers.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/engineering/bh-containers.md)

`xdzip-codec-bh` 从主机提供的源读取顺序 BlakHole（`.bh`）成员。产品只准入保守配置：要求 `BH 05 07` 前缀、实测固定字段、普通文件元数据、准确的压缩和解码长度，以及成员声明的 IEEE CRC-32。成员按物理顺序保留，重复名称仍是不同记录，冻结行身份独立于显示名称解码。

当前唯一准入的载荷方法是方法 8，即 raw RFC 1951 Deflate。共享有界 Deflate 读取器要求码流完整结束并准确消费压缩区间；尾随字节、截断、解码长度不符和 CRC 不符都会使载荷任务失败。未知方法保留为带警告的可见记录，不能解压，也不能计入已验证内容。

当前准入的头部布局是五字节固定前缀，随后依次为小端 16 位头部长度、常量字节 `25 d8 0c 00`、方法、四字节 DOS 形状时间戳、压缩长度、解码长度、CRC-32、属性 `0x20`、五个零保留字节、四字节文件名长度和文件名。头部长度必须等于 `37 + 文件名长度`；成员连续排列，最后一个载荷必须准确结束于物理 EOF。不接受归档尾部、目录记录、自解压前缀、重新同步、填充或邻接文件扫描。

BH 没有公开且持续维护的正式规范。格式观察固定使用 `ashang/libxad` 的 `support/formats/BLAKHOLE.TXT`（revision `dbe67a865207e49cd917ad9170a11b9eaa83998a`，SHA-256 `a77d570bba1ceafc443fa2404dfaccc8d8872a6a`）和 Hawkynt/CompressionWorkbench 的 `docs/BLAKHOLE-ON-DISK.md`（commit `ab61c59f789d82c628d3c5b4bac264afbdc0c7a7`，SHA-256 `2b7572396683e96f8b5a47ff0b3f0dde2de4b839`）。后者只有一个带来源记录的五成员样本，不能证明 Store、加密、目录、固实或分卷行为。

IZArc Command Line Add-On v1.1 作为仅构建期参考保存。官方下载公布的 MD5 是 `9002c7634768e5336244f9accb2223b0`，本地 `IZArcCL.exe` 的 SHA-256 是 `959050AFC513C64FD61F9EE15837054D0FAEE5EFB98C9107B4DC5D9A528A62D4`。当前环境中，解包得到的 `IZARCC.exe` 和 `IZARCE.exe` 都在产出结果前以 Windows `0xC0000005` 退出，因此 Store、压缩级别、密码或提取能力均不以该工具作为证据。参考工具只保留在 `build/toolchain`，不进入产品依赖。

BH 仅支持读取，不支持创建、修改、加密、分卷，也不声明专用 Linux MIME。格式登记和[归档检查契约](archive-checking.md)描述这条精确边界，`.bh` 后缀只是入口候选，完整内容校验仍以实际字节为准。
