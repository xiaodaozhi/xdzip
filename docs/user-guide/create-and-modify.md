# Create and modify archives

English | [简体中文](../zh-CN/user-guide/create-and-modify.md) | [繁體中文（台灣）](../zh-TW/user-guide/create-and-modify.md) | [日本語](../ja-JP/user-guide/create-and-modify.md) | [한국어](../ko-KR/user-guide/create-and-modify.md) | [Français](../fr-FR/user-guide/create-and-modify.md) | [Deutsch](../de-DE/user-guide/create-and-modify.md) | [Español](../es-ES/user-guide/create-and-modify.md) | [Português (Brasil)](../pt-BR/user-guide/create-and-modify.md) | [Русский](../ru-RU/user-guide/create-and-modify.md)

## Create an archive

1. Open the creation window from the welcome page or the New command. Add files or folders and review the input list.
2. Set the output name and location. Choose a format and one of its available compression settings. ZIP is a common choice for sharing. Use the [format table](../../README.md#format-support) to check creation options. A single-file stream such as GZIP takes one regular file.
3. ZIP and 7z offer encryption and split volumes where the selected options allow them. For a data disc image, select ISO. For a self-extracting ZIP, use the Windows x64 SFX profile. Start the operation and check the task result for the files actually saved. [Passwords and volumes](passwords-and-volumes.md)
4. If you have to cancel or retry, check the result first. An archive being built is staged before the finished result is published.

## Add, replace or move files

Open an eligible ZIP, 7z, TAR or TAR.GZ archive and navigate to the destination folder inside it. Drop local files or folders onto the list or grid, then review and confirm the addition. The conflict options include replacing eligible regular files in the archive. To move an existing entry, drag it to another folder inside the same archive and confirm. Dragging outside exports a copy. [Drag-and-drop behavior by platform](../engineering/archive-transfers.md)

Creation and addition omit `.DS_Store` files and `__MACOSX` and `_MACOSX` directories. Archive modification depends on the format, contents and source. Encrypted, split and disk-indexed archives cannot be edited through the normal modification flow. When editing is unavailable, extract to a new folder and create a new archive. [Modification and drag-and-drop rules](../engineering/archive-transfers.md)
