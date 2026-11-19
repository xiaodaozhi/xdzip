# 创建与修改压缩包

[English](../../user-guide/create-and-modify.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/user-guide/create-and-modify.md) | [日本語](../../ja-JP/user-guide/create-and-modify.md) | [한국어](../../ko-KR/user-guide/create-and-modify.md) | [Français](../../fr-FR/user-guide/create-and-modify.md) | [Deutsch](../../de-DE/user-guide/create-and-modify.md) | [Español](../../es-ES/user-guide/create-and-modify.md) | [Português (Brasil)](../../pt-BR/user-guide/create-and-modify.md) | [Русский](../../ru-RU/user-guide/create-and-modify.md)

## 创建压缩包

1. 从起始页或“新建”打开创建窗口，添加文件或文件夹，并核对输入列表。
2. 设置输出名称和位置，选择格式及可用的压缩选项。分享文件时可以优先考虑 ZIP。创建能力见[格式支持表](../../../README.zh-CN.md#格式支持)。GZIP 等单文件压缩流一次接收一个普通文件。
3. 所选配置允许时，ZIP 和 7z 可加密或分卷。数据光盘映像请选择 ISO，自解压 ZIP 请选择 Windows x64 的自解压配置。开始创建后，在任务结果中确认实际保存的文件。[密码与分卷](passwords-and-volumes.md)
4. 取消或重试前先看结果。创建过程使用暂存区，完成校验后才保存成品。

## 添加、替换或移动文件

打开符合条件的 ZIP、7z、TAR 或 TAR.GZ 压缩包，进入包内目标目录。把本机文件或文件夹拖到列表或网格中，核对内容后确认添加。重名时可选择替换符合条件的包内普通文件。包内移动可将条目拖到同一归档的其他目录后确认。拖到应用外则导出副本。[各平台的拖放行为](../engineering/archive-transfers.md)

创建和添加会排除 `.DS_Store` 文件以及 `__MACOSX`、`_MACOSX` 目录。修改能力取决于格式、内容和输入方式。加密归档、分卷归档以及使用磁盘索引的归档无法通过普通流程修改。无法编辑时，可解压到新目录，再创建新压缩包。[归档修改与拖放规则](../engineering/archive-transfers.md)


## 重命名或删除条目

在可编辑压缩包的普通目录中，可以右键单击列表或网格中的文件或文件夹。右键单击已选项会保留整个选区，右键单击未选项会只选中该项。

- **重命名**适用于单个条目。输入同一目录下的新名称。文件夹改名时，其下所有条目的路径也会更新。如果新名称已存在，请另选一个名称。
- **删除**适用于单个或多个选中条目。确认前请核对名称或选中项数。删除文件夹也会删除其中的内容。删空根目录后，仍可打开这个空压缩包。

要编辑搜索命中的条目，请先进入其所在目录。[归档修改规则](../engineering/archive-transfers.md)
