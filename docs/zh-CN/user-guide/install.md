# 安装 XDZip 与打开压缩包

[English](../../user-guide/install.md) | 简体中文 | [繁體中文](../../zh-TW/user-guide/install.md) | [日本語](../../ja-JP/user-guide/install.md) | [한국어](../../ko-KR/user-guide/install.md) | [Français](../../fr-FR/user-guide/install.md) | [Deutsch](../../de-DE/user-guide/install.md) | [Español](../../es-ES/user-guide/install.md) | [Português (Brasil)](../../pt-BR/user-guide/install.md) | [Русский](../../ru-RU/user-guide/install.md)

从 [GitHub 最新稳定版](https://github.com/xiaodaozhi/xdzip/releases/latest)选择对应系统的下载包。Windows 用户也可通过 Microsoft Store 安装。GitHub 发布页列出文件名、版本和 `SHA256SUMS`，可用来核对直接下载的文件。具体要求见[支持政策](../engineering/support-policy.md)。

| 桌面系统 | 安装或启动 |
| --- | --- |
| Windows 11 x64 | 在 Microsoft Store 搜索 **XDZip**，或运行 GitHub 的 `-setup.exe` 安装到当前用户。GitHub 安装器未签名。使用便携 ZIP 时，先解压整个包，再从解压目录运行 `xdzip.exe`。 |
| macOS 14 或更新版本，Apple Silicon | 打开 DMG，将 `xdzip.app` 拖到“应用程序”。若下载的是 `.app.zip`，先解压，再将应用移到该位置。从“应用程序”启动副本。GitHub 版采用 ad-hoc 完整性签名，未经过 Apple 公证。系统出现提示时，先核对发布来源和校验和。 |
| Ubuntu 24.04 LTS x64，X11，glibc 2.39 或更新版本 | 用包管理器安装 `.deb`。使用便携 `.tar.gz` 时，完整解压后在运行目录执行 `./xdzip`，保留启动器及配套运行文件。 |

桌面支持范围不涵盖 Windows ARM64、Intel Mac、Linux ARM64 和 Wayland。Linux 独立 CLI 无需 X11，运行需要系统 OpenSSL 3 和证书。可选用以 `xdzip-cli-` 开头的下载包，并按[命令行入门](../integration/README.md)启动。

## 打开压缩包

在 XDZip 中选择“打开压缩包”，或把压缩包拖到欢迎页。已注册文件类型也可从文件管理器的“打开方式”选择 XDZip。要设为某种类型的默认应用，请在系统设置中挑选。Windows 商店版可从文件关联页选择“在 Windows 中设置默认应用”。默认应用与右键压缩解压动作分别设置。详见[从文件管理器解压](extract.md#从文件管理器解压)与[安装和文件关联说明](../engineering/installers-associations.md)。

接下来可以[浏览和预览文件](browse.md)。
