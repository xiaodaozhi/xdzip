# 在脚本中使用 XDZip

[English](../../integration/README.md) | 简体中文

独立 CLI 无需 Qt 或桌面会话。从 [GitHub 最新发布页](https://github.com/xiaodaozhi/xdzip/releases/latest)下载适合平台的 `xdzip-cli-` 包并解压，保留可执行文件旁的工作进程。在解压目录中，macOS 或 Linux 执行 `./bin/xdzip --help`，Windows PowerShell 执行 `.\bin\xdzip.exe --help`。下文以 PATH 中的 `xdzip` 为例。需要登记独立 CLI 时，从解压目录运行 CLI 的 `setup` 命令，再将命令所在目录加入 PATH。当前用户的安装位置和移除方法见[命令安装](../engineering/command-line.md#构建与命令安装)。

## 列出、选择和解压

可以用自制的小型普通 ZIP 练习，并在自己拥有的可写私有目录下选择新输出目录。

```sh
xdzip list sample.zip --offset 0 --limit 128 --json
xdzip extract sample.zip --entry-id 3 -C output --conflict skip --non-interactive --json
```

把示例中的 `3` 换成第一条命令返回的某个普通文件 `id`。ID 从 1 开始，可以区分同名的物理条目。来源变化后请重新列出。大归档可按 `data.next_offset` 请求下一页，直到值为 null。按名称选择时使用区分大小写的精确路径和正斜杠。转义后的 `display_name` 用于展示。

## 创建和检查

```sh
xdzip create backup.zip documents/ --level normal --non-interactive --json
xdzip test backup.zip --non-interactive --json
```

创建时使用新的归档名称及已存在的上级目录，并选择格式支持的选项。自动化处理 ZIP 密码时，由调用进程通过专用标准输入管道提供一行 UTF-8 密码，并使用 `--password-stdin`。避免把密码写入参数、终端记录或日志。如果操作需要密码或重名决定而调用端没有提供，结果会报告需要交互。[密码和格式选项](../engineering/command-line.md#归档行为)

## 处理结果

使用 `--json` 时，从标准输出解析单个对象，读取 `schema_version`、`status`、`error_code`、`data`、`summary` 和 `warnings`。进度和诊断文字走标准错误。应结合进程退出码和 JSON 结果判断后续处理。

| 退出码 | 处理方式 |
| --- | --- |
| 0 | 命令成功。检查主动跳过数量，并区分已交接与实际安装完成。 |
| 1 | 操作失败、部分完成或有清理警告。重试前查看汇总和警告。 |
| 2 | 修正命令或参数。 |
| 3 | 按报告补充所需的交互、授权或安装条件。 |
| 130 | 取消已确认。检查是否已有文件保存。 |

未知值可能是 null。根据结果状态和实际保存数量决定是否缩小重试范围。完整选项与输出约定见[命令行参考](../engineering/command-line.md)。
