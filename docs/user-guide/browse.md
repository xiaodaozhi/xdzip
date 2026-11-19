# Browse, select and preview files

English | [简体中文](../zh-CN/user-guide/browse.md)

1. Open an archive. Enter a folder from the directory tree or by double-clicking its row. Use the breadcrumb to go back to a parent folder.
2. Search by name in the current directory and its descendants. Submit the search to apply it. Choose list or grid view and sort by a column. Folders stay ahead of files in both sort directions.
3. Select entries with their checkboxes or use your platform's selection keys. **Select All** applies to the current query, including rows outside the visible view. Check the selection count before extracting.
4. Open file properties and request a preview. The properties card previews supported text and still images, including supported static SVG. HEIC and HEIF previews require macOS. A file may be readable yet exceed preview limits or use an unsupported encoding. You can still try extracting it if the archive supports extraction.

Double-clicking a regular file extracts a temporary copy and asks the operating system to open it. Editing that copy does not update the archive. Extract to a chosen location when you need a permanent copy. [Temporary opening](../engineering/archive-opening.md)

Each archive can open in a tab or a separate window, depending on the setting. Large regular ZIP archives use an index that loads rows on demand. Some whole-archive actions become unavailable when the index moves to disk. [Large archive limits](../engineering/large-index.md). [Preview formats and limits](../engineering/content-preview.md)
