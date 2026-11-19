# Settings and updates

English | [简体中文](../zh-CN/user-guide/settings-and-updates.md) | [繁體中文（台灣）](../zh-TW/user-guide/settings-and-updates.md) | [日本語](../ja-JP/user-guide/settings-and-updates.md) | [한국어](../ko-KR/user-guide/settings-and-updates.md) | [Français](../fr-FR/user-guide/settings-and-updates.md) | [Deutsch](../de-DE/user-guide/settings-and-updates.md) | [Español](../es-ES/user-guide/settings-and-updates.md) | [Português (Brasil)](../pt-BR/user-guide/settings-and-updates.md) | [Русский](../ru-RU/user-guide/settings-and-updates.md)

## Set up your workspace

Open **Settings** from the application menu or toolbar. Choose the language, theme, font size and interface density in their respective settings. The automatic language option follows the system where supported. Choose whether archives open in tabs or windows, and configure recent files and tray behavior if the setting is available for your platform. Changes apply as you make them. If a setting cannot be saved, use the error shown on that page. [Settings behavior](../ux/settings-localization.md)

Change file associations through the operating system. Configure context-menu actions separately. On macOS, Finder extension enablement is managed by the system, with Finder Services available where registered. [File-manager actions](../engineering/shell-actions.md)

## Update the app

- **Microsoft Store edition.** Use Microsoft Store for updates. An app running with MSIX package identity has no GitHub update control in About.
- **GitHub installations.** Open **About xdzip** and choose to check for updates. In-app installation requires an eligible Windows per-user installation, an eligible macOS copy in Applications, or an Ubuntu DEB installation, plus a verified release package. Windows may ask for installer confirmation. Ubuntu uses system authorization. If active tasks are delaying installation, let them finish or cancel the update wait.
- **Portable or development builds.** Download the matching package. For Windows ZIP and Linux TAR.GZ, extract the whole new package into a separate directory. On macOS, quit the old app before replacing it in Applications. See [installation steps](install.md).

GitHub update checks are user initiated. They use the supported proxy setting. If the app offers only a manual download, use its release link and verify the package. A completed download still needs installation. After updating, reopen **About xdzip** to confirm the running version. Microsoft Store and GitHub publish independently. See [update behavior and recovery](../engineering/software-update.md).
