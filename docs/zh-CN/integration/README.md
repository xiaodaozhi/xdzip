# 在脚本中使用 XDZip

[English](../../integration/README.md) | 简体中文

独立 CLI 无需 Qt 或桌面会话。下载对应系统的 CLI 包，并将随包提供的格式工作程序与主程序放在一起。下文以 PATH 中的 `xdzip` 为例，也可直接调用解压目录中的 `bin/xdzip` 或 `bin/xdzip.exe`。[下载与设置命令](../engineering/command-line.md#构建与命令安装)

## 列出、选择和解压

可以用自制的小型普通 ZIP 练习，并在自己拥有的可写私有目录下选择新输出目录。

```sh
xdzip list sample.zip --offset 0 --limit 128 --json
xdzip extract sample.zip --entry-id 0 -C output --conflict skip --non-interactive --json
```

把 `0` 换成第一条命令实际返回的普通条目 `id`。即使存在同名文件，ID 也能区分不同的物理条目。来源有变化时重新列出。大型列表通过 `data.next_offset` 请求下一页，直到其值为 null。按名称选择时使用精确路径，区分大小写并使用正斜杠。转义后的 `display_name` 用于展示。

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
