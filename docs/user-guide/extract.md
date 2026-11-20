# Extract all or selected files

English | [简体中文](../zh-CN/user-guide/extract.md) | [繁體中文](../zh-TW/user-guide/extract.md) | [日本語](../ja-JP/user-guide/extract.md) | [한국어](../ko-KR/user-guide/extract.md) | [Français](../fr-FR/user-guide/extract.md) | [Deutsch](../de-DE/user-guide/extract.md) | [Español](../es-ES/user-guide/extract.md) | [Português (Brasil)](../pt-BR/user-guide/extract.md) | [Русский](../ru-RU/user-guide/extract.md)

## From the archive window

1. Open the archive. The main toolbar button extracts the current selection when you have one, or the whole archive when nothing is selected. Use the arrow next to the button to explicitly choose **Extract all** or **Extract selected**.
2. Confirm the scope and choose a writable destination. Select the option to create an archive-named folder if you want one and the option is available.
3. Choose a conflict policy. **Ask what to do** pauses for a decision, **Skip existing files** keeps the existing file, **Rename and keep both versions** saves a separate file, and **Overwrite existing files** replaces an eligible file after verification.
4. Start extraction and check the task result. A failure or cancellation can leave earlier files in place. Use the result details to identify what remains before retrying. [Checks, tasks and results](checks-and-tasks.md)

The extraction menu also offers **Extraction metadata preflight** for selections of up to 128 actual archive records. It reports issues such as unsafe names, unsupported methods and declared limits. Select explicit entries for this report. An implicit folder shown only for navigation can still be used for extraction when supported. Preflight does not read the payload or authorize writing. [Preflight and extraction limits](../engineering/extraction-preflight.md)

Large disk-indexed ZIP archives can extract selected regular files within task limits. When the whole archive exceeds the extraction limit, select a smaller group. If the source changes or is replaced, reopen it and make a new selection. [Large archive limits](../engineering/large-index.md)

## From the file manager

On Windows 11, look under **Show more options** for the classic menu. The Store edition can also show actions in the modern menu. Ubuntu uses the Nautilus extension. On macOS, enable the Finder extension in system settings or use the registered Finder Service. If the menu is unavailable, open the archive in XDZip and extract from its window.

The file-manager menu can extract into each archive's current directory, create a separate directory per archive, use a smart directory, or let you choose a destination. Smart extraction skips the wrapper only when the validated output already contains exactly one top-level folder. A batch shows a result per archive and continues after an individual failure. Cancel to stop later items and request a stop to current work. [File-manager actions and settings](../engineering/shell-actions.md)
