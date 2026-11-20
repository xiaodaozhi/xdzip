# 密码与分卷

[English](../../user-guide/passwords-and-volumes.md) | 简体中文 | [繁體中文](../../zh-TW/user-guide/passwords-and-volumes.md) | [日本語](../../ja-JP/user-guide/passwords-and-volumes.md) | [한국어](../../ko-KR/user-guide/passwords-and-volumes.md) | [Français](../../fr-FR/user-guide/passwords-and-volumes.md) | [Deutsch](../../de-DE/user-guide/passwords-and-volumes.md) | [Español](../../es-ES/user-guide/passwords-and-volumes.md) | [Português (Brasil)](../../pt-BR/user-guide/passwords-and-volumes.md) | [Русский](../../ru-RU/user-guide/passwords-and-volumes.md)

## 密码

AES Crypt 以及启用了文件名加密的 7z 等归档，在显示文件列表前需要输入密码。其他归档可能在解压、检查或预览时才提示。输入时保留原有空格和大小写。ZIP 支持符合条件的 AES-256，不支持 AES-128／192 和传统 ZipCrypto。密码无误仍失败时，请检查加密方式及文件是否损坏。[支持的加密方式](../engineering/password-extraction.md)

创建加密包时，选用提供加密选项的格式。ZIP 的 AES-256 会保护文件内容，文件名仍可见。如果需要一并保护文件名，请选择 7z 并开启文件名加密。密码请与压缩包分开保存。

## 分卷

1. 把同一压缩包的所有分卷放进一个目录，保留原始文件名。
2. 打开支持的分卷，XDZip 会在同目录查找其余部分。编号 ZIP 和 7z、PKZIP，以及受支持的 RAR 和 CAB 卷组使用不同的命名规则。
3. 如果卷组不完整，请补齐缺失文件再重新打开。手工拼接或重命名可能导致压缩包无法读取。

创建分卷包时，选择 ZIP 或 7z，并在创建窗口设置卷大小。分享时应发送整组卷文件。分卷包不能在 XDZip 中直接修改。[卷名规则与支持范围](../engineering/archive-opening.md)
