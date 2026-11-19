# 在脚本中使用 XDZip

[English](../../integration/README.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/integration/README.md) | [日本語](../../ja-JP/integration/README.md) | [한국어](../../ko-KR/integration/README.md) | [Français](../../fr-FR/integration/README.md) | [Deutsch](../../de-DE/integration/README.md) | [Español](../../es-ES/integration/README.md) | [Português (Brasil)](../../pt-BR/integration/README.md) | [Русский](../../ru-RU/integration/README.md)

独立 CLI 无需 Qt 或桌面会话。从 [GitHub 发布页](https://github.com/xiaodaozhi/xdzip/releases)下载适合当前系统的 `xdzip-cli-` 包，完整解压，保留可执行文件旁的工作进程。从解压目录启动时，macOS 或 Linux 执行 `./bin/xdzip --help`，Windows PowerShell 执行 `.\bin\xdzip.exe --help`。

下文以 PATH 中的 `xdzip` 为例。用解压后的 CLI 执行 `setup` 可登记当前用户的命令入口，按需将命令目录加入 PATH。运行 `xdzip --version-json`，确认找到的程序报告 `"component":"cli"`。桌面启动程序和 CLI 可能使用相同的命令名，冲突时直接指定解压后的 CLI 路径。[命令安装与移除](../engineering/command-line.md#构建与命令安装)

## 创建和检查

选一个新的私有可写目录作为工作目录。在其中创建 `docs` 文件夹并放入 `hello.txt`，再把另一份文件 `notes.txt` 放在文件夹旁。下面的命令依次创建 ZIP、把第二份文件添加到已有的包内目录 `docs/`，并检查结果。

```sh
xdzip create sample.zip docs --level normal --json
xdzip add sample.zip notes.txt --into docs --conflict skip --json
xdzip test sample.zip --json
```

可从[格式支持表](../../../README.zh-CN.md#格式支持)选择创建后缀和选项。输出须放在输入树外，且上级目录已存在。`add` 仅用于符合条件的现有归档，`--into` 指定已有包内目录，省略时添加到根目录。`test` 不写出解压文件，成功时由 `data.check_kind` 说明检查强度。详见[格式登记](../engineering/format-registry.md)与[完整性检查](../engineering/archive-checking.md)。

## 列出、选择和解压

继续在同一工作目录使用刚创建的归档。请选择新的目标位置，或明确如何处理同名文件。

```sh
xdzip list sample.zip --limit 2 --json
xdzip extract sample.zip docs/hello.txt -C output --conflict skip --json
```

包内路径必须精确匹配、区分大小写并使用正斜杠。有多条记录使用同一路径时，可用 `list` 返回的 `id` 选择。ID 从 1 开始。例如，如果列表中目标文件的 ID 为 2，就把第二条命令中的 `docs/hello.txt` 换成 `--entry-id 2`。选择多个条目时重复 `--entry-id`。用 `path` 匹配条目，用 `display_name` 展示。归档变化后重新列出。

第一页从偏移量 0 开始。读取下一页时，把 `data.next_offset` 传给 `--offset`，并保持 `--limit` 不变，直到 `next_offset` 为 null。偏移量是从 0 开始的行位置，ID 标识物理记录。每条命令都会重新打开归档，分页期间应保持来源不变。[CLI 选择与列举限制](../engineering/command-line.md#归档行为)

## 无交互运行

`--json` 禁用交互提示。需要无人值守运行时，请明确选择重名策略。对于支持密码的命令，例如 `extract`、`test` 或加密的 `create`，可指定 `--password-stdin` 并通过专用标准输入管道传入一行 UTF-8 密码。加密创建还需要 `--encrypt`。缺少必要的决定或密码时，CLI 返回 `interaction_required`。[密码与格式选项](../engineering/command-line.md#归档行为)

## 解读结果

归档操作使用 `--json` 时，从 stdout 解析一个 UTF-8 对象，读取 `schema_version`、`status`、`error_code`、`data`、`summary` 和 `warnings`。stderr 可用于读取诊断信息。`--version-json` 返回独立的构建身份对象。

| 退出码 | 处理方式 |
| --- | --- |
| 0 | 查看结果及跳过数量。适用命令返回 `handed_off` 时，表示请求已交给桌面应用。 |
| 1 | 查看错误、警告和汇总，确认是失败、部分完成还是有清理警告。 |
| 2 | 修正命令或参数。 |
| 3 | 检查报告的交互、授权、平台或安装要求。 |
| 130 | 已确认取消，请核对已保存的文件。 |

未知值可为 null。失败或取消前已保存的文件可能保留。重试前，若结果包含 `summary.committed_files`，请先核对该值。桌面交接、更新回执与完整输出契约见[命令行参考](../engineering/command-line.md)。
