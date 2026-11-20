# ACE、ALZ 与 EGG 的资料和许可门槛

[English](../../engineering/ace-alz-egg-format-sources.md) | 简体中文 | [繁體中文](../../zh-TW/engineering/ace-alz-egg-format-sources.md) | [日本語](../../ja-JP/engineering/ace-alz-egg-format-sources.md) | [한국어](../../ko-KR/engineering/ace-alz-egg-format-sources.md) | [Français](../../fr-FR/engineering/ace-alz-egg-format-sources.md) | [Deutsch](../../de-DE/engineering/ace-alz-egg-format-sources.md) | [Español](../../es-ES/engineering/ace-alz-egg-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/ace-alz-egg-format-sources.md) | [Русский](../../ru-RU/engineering/ace-alz-egg-format-sources.md)

本文说明 xdzip 对 ACE、ALZ 和 EGG 格式的支持范围，以及实现所参考的公开技术资料与许可限制。参考资料保留在 `build/toolchain/references`，不作为产品依赖。

## ACE

- 技术说明：Marcel Lemke，《Technical information of the archiver ACE v1.2》，来自 `wdlkmpx/unace`，SHA-256 为 `F645C91348D5DCD560A19CA21DEF849D25C304EFFAACF65B03EFBB0A893E51F9`。
- 独立实现：`droe/acefile`，提交 `11ed076ae19e8228148731cfbc05f019a395b3b9`，BSD-2-Clause 实现许可 SHA-256 为 `AD376BD9695F0E355F5C523443ABF822BB896999A3252AAA614D313CC4F1F5B5`，实现文件 SHA-256 为 `BC701569AD8521964C62256472AB44976AC57FB46E52B147A21018BC96348F1F`。
- 该实现说明它能读取 ACE 1.0 和 2.0，包括 LZ77、DELTA、EXE、PIC、SOUND、密码和分卷。许可只覆盖 Python 实现，并明确不对 ACE 技术本身主张权利。

技术资料足以定义块头、CRC、存储成员和旧 LZ77 家族，但不能直接等同于 ACE 技术的商业许可。实现还引用 WinAce 与 unace，不能把引用实现本身当作独立互操作样本。产品决定现已永久放弃 ACE，不实现或登记 ACE，也不增加 ACE 后缀、探测器、MIME 或关联入口。

## ALZ

- 格式规范：`alkegi/docs`，提交 `36230c34d3056604cc63519e9855a79e894496b7`，CC0 许可 SHA-256 为 `F4E7F373B9B996950337E8D41A4A2939C2D90B7725E9BAF3D5084A22717AD328`。
- `alz.md` SHA-256 为 `CBD35E77AB53CFD08C0FF192FA2793438150E4935522608936B1A0942560E237`。
- 资料定义 ALZ 和本地文件签名、可变宽度尺寸、CRC-32、UTF-8/CP949 名称、Store、原始 RFC 1951 Deflate 和 ALZ 改造的 BZip2 流，也定义 PKWARE 传统加密和分卷布局。

ALZ 已在 xdzip 中准入保守的只读产品切片：普通单卷、无加密的 Store 和原始 Deflate，要求准确 CRC、尺寸和物理结束。codec、runtime、CLI 和关联入口均有自动化验证。改造 BZip2、ZipCrypto 和分卷需要独立完整样本以及单独的资源和安全检查。CC0 文档本身不能证明所有实现细节都没有第三方专利或商标义务。

## EGG

- 公开格式资料：`alkegi/docs`，同为 `36230c34d3056604cc63519e9855a79e894496b7`，使用上面的 CC0 许可。
- `egg.md` SHA-256 为 `4CA2CB6E8FA9A4F40C30EDE05A5E977BE113A96E1527E28A7B1ED0836D8A43A3`。
- 参考模块：`dterracino/UnEgg`，提交 `f21e653b02f7ab467bbc16af879e68a48dabcb2b`，EGG 规范 PDF SHA-256 为 `0BB7E6F174CAFB9F6726D70B7DEE8DBF33C034717058B1B0EB2B372D9A8FBB37`。
- UnEGG 源码包许可 SHA-256 为 `9B012928756D9243CC549FA0A7D7B5742ED8F69B00A2BE1C964F42EF21F1D8D6`，模块包许可 SHA-256 为 `30F2E45148D43D7A77574403807C13AA359EE9ACE841A00B6A5348AD961423F0`。

UnEGG 许可写明商业使用需要 ESTsoft 批准，模块可在非商业条件下使用和分发，并禁止用模块开发 EGG 压缩器，同时保留 ESTsoft 的全部权利。因此只能把它作为测试参考，不能复制、链接或作为产品依赖。

线格式与受限模块在技术上可以分离。xdzip 现在已独立实现并准入无加密、非固实普通单卷 Store/raw Deflate 切片，codec、runtime、CLI 和关联测试覆盖准确块尺寸与 CRC-32。标准 BZip2 和文档定义的 LZMA 封装仍需独立完整样本和复核。AZO、AES/LEA、ZipCrypto、分卷和固实配置继续排除在当前产品边界外。

## 必要实现门槛

1. 资料继续放在产品树外，使用前核对固定哈希。
2. 每个准入方法都要有独立完整样本。只有头部或 Store 样本不能证明方法互操作。
3. 容器、runtime、CLI 和桌面入口必须一起接入，未知方法可见但不能获得载荷权限。
4. 密码、分卷、固实和特殊对象必须分别完成资料、安全与取消测试后才可启用。
5. 在改变公开能力登记前，把法律结论和样本身份记录到 issue。
