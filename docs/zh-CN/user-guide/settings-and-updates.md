# 设置与更新

[English](../../user-guide/settings-and-updates.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/user-guide/settings-and-updates.md)

## 调整工作区

从应用菜单或工具栏打开“设置”。在对应设置中选择语言、主题、字号及界面密度。自动语言在支持时跟随系统。还可按平台和版本设置用标签页或窗口打开压缩包、是否记录最近文件，以及托盘行为。设置随选择即时生效，保存失败时请查看对应页面的提示。[设置行为说明](../ux/settings-localization.md)

文件关联由操作系统管理，右键菜单动作单独配置。macOS 的 Finder 扩展由系统管理启用状态，已登记时还可使用 Finder 服务。[文件管理器操作](../engineering/shell-actions.md)

## 更新应用

- **Microsoft Store 版。** 通过 Microsoft Store 更新。应用以 MSIX 包身份运行时，“关于”页没有 GitHub 更新入口。
- **GitHub 安装版。** 打开“关于 xdzip”，主动检查更新。应用内安装需要符合条件的 Windows 当前用户安装、macOS“应用程序”中的副本或 Ubuntu DEB 安装，并且发布包须通过校验。Windows 可能需要在安装向导中确认，Ubuntu 使用系统授权。若安装正在等待活动任务结束，可等它们完成，或取消本次更新等待。
- **便携版或开发构建。** 下载匹配的包。Windows ZIP 和 Linux TAR.GZ 请完整解压到新目录。macOS 请先退出旧版，再替换“应用程序”中的副本。详见[安装步骤](install.md)。

GitHub 更新由用户主动检查，并按平台支持范围使用代理设置。如果应用只提供手动下载，请通过对应的发布链接下载并核对文件。下载完成后仍须安装。更新后重新打开“关于 xdzip”，确认当前运行的版本。Microsoft Store 与 GitHub 独立发布。[更新行为与恢复](../engineering/software-update.md)
