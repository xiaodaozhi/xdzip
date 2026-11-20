# Browse, select and preview files

English | [简体中文](../zh-CN/user-guide/browse.md) | [繁體中文](../zh-TW/user-guide/browse.md) | [日本語](../ja-JP/user-guide/browse.md) | [한국어](../ko-KR/user-guide/browse.md) | [Français](../fr-FR/user-guide/browse.md) | [Deutsch](../de-DE/user-guide/browse.md) | [Español](../es-ES/user-guide/browse.md) | [Português (Brasil)](../pt-BR/user-guide/browse.md) | [Русский](../ru-RU/user-guide/browse.md)

1. Open an archive. Enter a folder from the directory tree or by double-clicking its row. Use the breadcrumb to go back to a parent folder.
2. Type part of a filename and press Enter. Search covers the current folder and its descendants. Use list or grid view and choose a sort order. Folders stay ahead of files in both directions.
3. Select entries with their checkboxes or use your platform's selection keys. **Select All** applies to the current query, including rows outside the visible view. Check the selection count before extracting.
4. With file properties visible, select one file to see its preview. Hover over the preview and use the refresh icon at the upper right to reload it. If you select several items, the card shows a selection message. Supported text must be UTF-8 or BOM-marked UTF-16 and at most 256 KiB. Images have separate size limits. Static SVG is limited to the supported subset, and HEIC/HEIF is available on macOS. If a preview is unavailable, you can still extract the file when its archive entry supports extraction. [Preview formats and limits](../engineering/content-preview.md)

Double-clicking a regular file extracts a temporary copy and asks the operating system to open it. Editing that copy does not update the archive. Extract to a chosen location when you need a permanent copy. [Temporary opening](../engineering/archive-opening.md)

Archives open in tabs by default. Change the setting to open them in separate windows. An ordinary single-file ZIP/ZIP64 can index up to one million physical records. Large indexes load rows on demand. Disk-indexed ZIP archives can still support selected-file extraction, but whole-archive testing and modification are disabled. [Large archive limits](../engineering/large-index.md)
