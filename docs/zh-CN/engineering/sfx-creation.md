# 自有 Windows ZIP 自解压包

[English](../../engineering/sfx-creation.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/engineering/sfx-creation.md) | [日本語](../../ja-JP/engineering/sfx-creation.md) | [한국어](../../ko-KR/engineering/sfx-creation.md) | [Français](../../fr-FR/engineering/sfx-creation.md) | [Deutsch](../../de-DE/engineering/sfx-creation.md) | [Español](../../es-ES/engineering/sfx-creation.md) | [Português (Brasil)](../../pt-BR/engineering/sfx-creation.md) | [Русский](../../ru-RU/engineering/sfx-creation.md)

首版 SFX 配置仅支持 Windows x64，使用明确的 `.sfx.exe` 创建后缀。它把无加密、单卷 Store/Deflate ZIP 与项目编译的 `xdzip-sfx.exe` 解压程序封装。创建时绝不运行该程序。通用 `.exe` 不进入创建、拖放或系统关联登记。

宿主仅从应用目录，或私有 `cli` 目录的共享父目录读取解压程序。不跟随链接，限制普通文件大小，并验证 PE32+、x64、节表范围和自有版本标记。ZIP 结束记录的注释包含 64 字节版本化封装，记录解压程序长度、ZIP 长度及前面所有字节的 SHA-256。运行时先验证封装，再使用共享 Rust ZIP 与提取契约。这能发现损坏和错误封装，不证明发布者身份，也不认证任意可执行代码。

独立解压程序不依赖 Qt 或已安装的 xdzip。Windows 系统对话框选择已有目标目录和明确的冲突策略，随后显示进度与取消。归档操作仍由 Rust 在界面线程外执行，复用已有授权、冻结身份、资源预算、逐文件校验、暂存、冲突处理和清理。取消保留已提交文件，清理未完成暂存。成功或失败均不打开或运行解压文件，不调用脚本，不安装软件，不提权，不修改关联。

程序另提供 `--test`（只验证、不写入）和 `--extract-to <已有目录> --conflict skip|rename|overwrite` 自动化入口。错误封装和不支持或加密的 ZIP 配置在输出前失败。不支持载荷密码、分卷、自定义启动命令、静默安装或自动运行可执行文件。生成文件未签名，签名和发布者信誉属于独立发行事项，遵循 [PE/Authenticode 规则](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)。

桌面继续使用 Qt Quick。独立 SFX 的系统对话框属于 [ADR-013](../design/architecture-decisions.md#adr-013-独立-windows-sfx-宿主) 登记的有限架构例外。
