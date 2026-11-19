# Extract all or selected files

English | [简体中文](../zh-CN/user-guide/extract.md)

1. Open the archive. Select the files or folders to extract and choose **Extract selected**, or choose **Extract all** for the entire archive.
2. Check the scope shown in the confirmation window. Choose an output directory where you can write, and choose whether to create an archive-named folder when offered.
3. Choose how to handle matching names. Ask pauses for a decision, Skip keeps the existing file, Rename saves a separate file, and Overwrite replaces an eligible file after verification. For unfamiliar archives, use a new destination folder.
4. Start extraction and open the task result. A write failure, cancellation or unsupported entry can leave earlier committed files in place. Use the result details to see what finished before you retry.

The metadata preflight can show naming, capability or space-related concerns before you extract. Final checks still happen during extraction. Some links, special entries or unsafe destinations cannot be written. If the source changes while work is in progress, reopen its current copy and select the entries again. [Extraction and preflight rules](../engineering/extraction-preflight.md)

To start from the file manager, use the XDZip context menu where supported. Choose the destination and inspect the operation window. Smart extraction can avoid an extra wrapper when there is one top-level directory. A batch handles each selected archive in order and shows the result for each. [Available file-manager actions](../engineering/shell-actions.md)
