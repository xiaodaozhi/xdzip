# 同步最新宣传素材

[English](../../engineering/marketing-archive.md) | 简体中文

将各平台最新的完整导出直接保存到 Google Drive 主账号的 `xdzip/assets/windows/`、`xdzip/assets/macos/` 或 `xdzip/assets/linux/`。保留导出的语言、素材分类目录和文件名，以及截图来源、`manifest.json` 和 `SHA256SUMS.txt`。不要上传 ZIP，也不建立版本目录。仅含底图的导出没有真实截图；包含截图的导出也可能只覆盖部分语言。生成和采集方法见 [Windows 截图指南](../market/windows/capture.md)及 [macOS、Linux 指南](platform-marketing.md)。

本地导出不会连接 Drive。同步时使用已连接的 Google Drive 应用和指定账号，逐个平台操作：

1. 完整列出目标文件夹树及所有分页，在 `build/` 保存 JSON 清单，其 `files` 数组的每项包括相对于平台目录的 `path` 和实际读到的 Drive 文件 `id`。同一文件夹内有重名时停止处理。如已有上次完成的 `manifest.json`，将其下载到 `build/`。不要从文件名猜测 ID。
2. 用实际导出目录、云端清单和上次清单生成计划：

   ```sh
   python3 scripts/plan-marketing-sync.py --platform macos --input build/macos-marketing --remote-inventory build/drive-inventory.json --previous-manifest build/previous-manifest.json > build/marketing-sync-plan.json
   ```

   平台文件夹为空时省略 `--previous-manifest`。命令按 SHA-256 清单校验导出，并拒绝覆盖上次清单未登记的同路径文件；输出所需目录及 `create`、`update`、`trash` 操作，本身不会上传或删除。
3. 创建缺失目录，先创建素材文件或使用已核对的文件 ID 原位更新，避免同名副本。全部素材上传成功后，把上次清单登记但本次已不存在的文件移入回收站，保留其他文件。最后依次更新 `manifest.json`、`SHA256SUMS.txt`。中途失败时停止，重新列出受影响目录，根据实际状态继续，不盲目重复创建。
4. 列出受影响的每个目录，对照导出核对文件名、文件 ID 和大小，并读回清单与校验文本。连接器提供云端摘要时再核对摘要。Drive 的多文件更新不是原子操作，运行过程中可能看到部分更新；素材操作结束后才更新最终清单。文件大小检查不能证明云端摘要一致。

后续同步包含截图的素材时，先确认输入是预期的完整最新导出：上次清单中存在而本次缺少的文件会被移除。不要将生成素材或 Drive 清单提交到 Git。运行 `python3 scripts/test-plan-marketing-sync.py` 可定向检查同步计划。
