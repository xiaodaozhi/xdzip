# Troubleshooting XDZip

English | [简体中文](../zh-CN/user-guide/troubleshooting.md)

| What happened | What to try |
| --- | --- |
| An archive does not open | Check the format and all volume files. Copy the archive to an accessible local folder if the original location is unavailable, then reopen it. A familiar extension alone cannot make damaged data readable. |
| Filenames look wrong or cannot be written | Check whether the archive can be recreated with UTF-8 names. XDZip detects some legacy encodings, with limits for short or mixed names. Extract only entries that the plan can map safely. [Filename handling](../engineering/filename-encoding.md) |
| The file shows properties but no preview | Check type, size, platform and encoding. Preview is limited and separate from extraction. [Preview limits](../engineering/content-preview.md) |
| The target is rejected, or results are incomplete | Choose a writable private directory. Check for name collisions or unsupported entries, and inspect the task result before retrying. [Extract files](extract.md) |
| The current file was replaced while XDZip was working | Reopen the new copy and build a new selection. XDZip checks the source again before writing. |
| Explorer, Finder or Files does not show the expected action | Check the installation channel and whether shell integration is enabled. On Windows 11, use **Show more options** to check the classic menu. Check the platform-specific fallback. [File-manager actions](../engineering/shell-actions.md) |
| Update or installation fails | Check the current version and channel. Verify checksums for manual downloads and follow the matching installation instructions. [Update recovery](settings-and-updates.md) |

When reporting a problem, include the version and build from **About xdzip**, your OS and architecture, format and operation, and steps with expected and actual results. Use your own small sample if possible. Remove passwords and private paths before sharing. [Report a problem](https://github.com/xiaodaozhi/xdzip/issues/new/choose)
