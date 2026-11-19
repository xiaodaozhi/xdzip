# Check archives and manage tasks

English | [简体中文](../zh-CN/user-guide/checks-and-tasks.md)

## Test an archive

1. Open the archive and choose **Test** in the toolbar.
2. Follow any password prompt and wait for the result. The task checks eligible contents without writing extracted files.
3. Review the failure reason and the affected entry, if shown. Some formats provide checksums or authentication, while others check structure and readability. A passed check only covers what the format verifies. [Integrity check behavior](../engineering/archive-checking.md)

Some archives cannot be tested. For example, disk-indexed ZIP supports selected-file extraction within task limits, and NSIS exposes installer metadata only. See [large archive limits](../engineering/large-index.md) and [format support](../../README.md#format-support).

## Progress, cancellation and results

Open the task menu to return to work in progress or view results. Closing an archive tab does not cancel its background task. Password and conflict dialogs may need a response before work can continue.

Choose **Cancel** in the task window to request a stop. Wait until the task reports its final state. Files already committed can remain. Expand the result details to distinguish completed, skipped and unfinished items, then use the displayed counts and reasons when deciding what to retry. Clearing finished history removes its task entries and keeps files on disk. [Task window behavior](../ux/task-windows.md)
