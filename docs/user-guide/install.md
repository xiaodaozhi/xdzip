# Install XDZip and open an archive

English | [简体中文](../zh-CN/user-guide/install.md) | [繁體中文](../zh-TW/user-guide/install.md) | [日本語](../ja-JP/user-guide/install.md) | [한국어](../ko-KR/user-guide/install.md) | [Français](../fr-FR/user-guide/install.md) | [Deutsch](../de-DE/user-guide/install.md) | [Español](../es-ES/user-guide/install.md) | [Português (Brasil)](../pt-BR/user-guide/install.md) | [Русский](../ru-RU/user-guide/install.md)

Download the package for your system from the [latest stable GitHub release](https://github.com/xiaodaozhi/xdzip/releases/latest), or use Microsoft Store on Windows. The GitHub page lists the filename, version and `SHA256SUMS` for verifying direct downloads. See the [support policy](../engineering/support-policy.md) for the system requirements.

| Desktop system | Install or launch |
| --- | --- |
| Windows 11 x64 | Search for **XDZip** in Microsoft Store or run the GitHub `-setup.exe` for a per-user install. The GitHub installer is unsigned. To use the portable ZIP, extract the entire package and start `xdzip.exe` from the extracted directory. |
| macOS 14 or later, Apple Silicon | Open the DMG and drag `xdzip.app` to **Applications**. For an `.app.zip`, extract it first and move the app there. Start the Applications copy. GitHub builds use ad-hoc integrity signing and are unnotarized. If macOS displays a warning, verify the release source and checksum. |
| Ubuntu 24.04 LTS x64, X11, glibc 2.39 or later | Install the `.deb` through your package manager. For the portable `.tar.gz`, extract all files and run `./xdzip` from the extracted runtime directory. Keep the launcher with the runtime files. |

The desktop support scope excludes Windows ARM64, Intel Macs, and Linux ARM64 and Wayland. The standalone Linux CLI works without X11 and requires system OpenSSL 3 and certificates. To use it, choose an asset starting with `xdzip-cli-` and follow the [CLI quick start](../integration/README.md).

## Open an archive

Use **Open archive** in XDZip or drop the archive onto the welcome page. From a file manager, choose **Open with** where the installed package is registered. To make XDZip the default for a file type, choose it in the system settings. The Windows Store edition provides **Set default apps in Windows** from its file associations page. Default apps and compression context-menu actions have separate settings. See [file-manager extraction](extract.md#from-the-file-manager) and [installation and associations](../engineering/installers-associations.md).

Next, [browse and preview files](browse.md).
