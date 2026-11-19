# Create and modify archives

English | [简体中文](../zh-CN/user-guide/create-and-modify.md)

## Create an archive

1. Open the creation window from the welcome page or the New command. Add files or folders and review the input list.
2. Set the output name and location. Choose a format and one of its available compression settings. ZIP is a common choice for sharing. Use the [format table](../../README.md#format-support) to check creation options. A single-file stream such as GZIP takes one regular file.
3. ZIP and 7z offer encryption and split volumes where the selected options allow them. For a data disc image, select ISO. For a self-extracting ZIP, use the Windows x64 SFX profile. Start the operation and check the task result for the files actually saved. [Passwords and volumes](passwords-and-volumes.md)
4. If you have to cancel or retry, check the result first. An archive being built is staged before the finished result is published.

## Add, replace or move files

Open an eligible ZIP, 7z, TAR or TAR.GZ archive and navigate to the destination folder inside it. Drop local files or folders onto the list or grid, then review and confirm the addition. The conflict options include replacing eligible regular files in the archive. To move an existing entry, drag it to another folder inside the same archive and confirm. Dragging outside exports a copy. [Drag-and-drop behavior by platform](../engineering/archive-transfers.md)

Creation and addition omit `.DS_Store` files and `__MACOSX` and `_MACOSX` directories. Archive modification depends on the format, contents and source. Encrypted, split and disk-indexed archives cannot be edited through the normal modification flow. When editing is unavailable, extract to a new folder and create a new archive. [Modification and drag-and-drop rules](../engineering/archive-transfers.md)


## Rename or delete entries

In a regular folder of an editable archive, right-click a file or folder in the list or grid. Right-clicking a selected item keeps the selection. Right-clicking an unselected item selects it alone.

- **Rename** is available for one item. Enter the new name in the same folder. Renaming a folder updates all the paths under it. Choose a different name if the destination name already exists.
- **Delete** accepts one or more selected items. Review the name or selection count before confirming. Deleting a folder also deletes its contents. After deleting every item at the archive root, the archive remains available as an empty archive.

To edit an item found through search, open its containing folder first. [Archive modification rules](../engineering/archive-transfers.md)
