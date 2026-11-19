# Troubleshooting XDZip

English | [简体中文](../zh-CN/user-guide/troubleshooting.md) | [繁體中文（台灣）](../zh-TW/user-guide/troubleshooting.md) | [日本語](../ja-JP/user-guide/troubleshooting.md) | [한국어](../ko-KR/user-guide/troubleshooting.md) | [Français](../fr-FR/user-guide/troubleshooting.md) | [Deutsch](../de-DE/user-guide/troubleshooting.md) | [Español](../es-ES/user-guide/troubleshooting.md) | [Português (Brasil)](../pt-BR/user-guide/troubleshooting.md) | [Русский](../ru-RU/user-guide/troubleshooting.md)

| What happened | What to try |
| --- | --- |
| An archive does not open | Check [format support](../../README.md#format-support), the compression or encryption method, and the complete volume set. If the source is inaccessible, copy the archive and all its parts to an accessible local directory, keeping their names, then reopen. |
| Filenames look wrong or cannot be written | Check whether the archive can be recreated with UTF-8 names. XDZip detects some legacy encodings, with limits for short or mixed names. Extract only entries that the plan can map safely. [Filename handling](../engineering/filename-encoding.md) |
| The file shows properties but no preview | Check type, size, platform and encoding. Preview is limited and separate from extraction. [Preview limits](../engineering/content-preview.md) |
| The target is rejected, or results are incomplete | Choose a writable private directory. Check for name collisions or unsupported entries, and inspect the task result before retrying. [Extract files](extract.md) |
| The current file was replaced while XDZip was working | Reopen the new copy and build a new selection. XDZip checks the source again before writing. |
| Explorer, Finder or Files does not show the expected action | Check the installation channel and whether the extension is enabled. Windows 11's classic menu is under **Show more options**. On macOS enable the Finder extension in system settings or try the registered Service. You can always open the archive in XDZip first. [File-manager extraction](extract.md#from-the-file-manager) |
| Test or modification is disabled for a large ZIP | If XDZip uses a disk index, select only the regular files you need and extract them within the task limits. [Large archive limits](../engineering/large-index.md) |
| Update or installation fails | Check the current version and channel. Verify checksums for manual downloads and follow the matching installation instructions. [Update recovery](settings-and-updates.md) |

When reporting a problem, include the version and build from **About xdzip**, your OS and architecture, format and operation, and steps with expected and actual results. Use your own small sample if possible. Remove passwords and private paths before sharing. [Report a problem](https://github.com/xiaodaozhi/xdzip/issues/new/choose)
