# Create and modify archives

English | [简体中文](../zh-CN/user-guide/create-and-modify.md)

## Create an archive

1. Open the creation window from the welcome page or the New command. Add files or folders and review the input list.
2. Set the output name and location. Choose a format and one of its available compression settings. ZIP is a common choice for sharing. Use the [format table](../../README.md#format-support) to check creation options. A single-file stream such as GZIP takes one regular file.
3. Set a password or volume size only if that format offers the option. Choose a different output location if the current one is refused. Start the operation and use the task result to find the files actually saved.
4. If you have to cancel or retry, check the result first. An archive being built is staged before the finished result is published.

## Add, replace or move files

Open an eligible archive and use the add command or drop files onto the archive page. Review the target and confirm before XDZip writes changes. For allowed replacements, check the destination name and the conflict decision. Dragging an entry to a folder inside the archive can move it after confirmation. Dragging it outside exports a copy. Back up archives you need to preserve before editing.

Creation and addition omit `.DS_Store` files and `__MACOSX` and `_MACOSX` directories. Archive modification depends on the format, contents and source. Encrypted, split and disk-indexed archives cannot be edited through the normal modification flow. When editing is unavailable, extract to a new folder and create a new archive. [Modification and drag-and-drop rules](../engineering/archive-transfers.md)
