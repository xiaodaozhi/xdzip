# Use XDZip in scripts

English | [简体中文](../zh-CN/integration/README.md)

The standalone CLI runs without Qt or a desktop session. Download the CLI package for your platform and keep its bundled worker next to the executable. The examples use `xdzip` on PATH. You can also call `bin/xdzip` or `bin/xdzip.exe` from the unpacked package. [Download and setup](../engineering/command-line.md#build-and-command-installation)

## List, select and extract

Practice with a small regular ZIP that you own, and use a fresh output directory under a private, writable parent you own.

```sh
xdzip list sample.zip --offset 0 --limit 128 --json
xdzip extract sample.zip --entry-id 0 -C output --conflict skip --non-interactive --json
```

Use the actual `id` of a regular entry returned by the first command in place of `0`. IDs distinguish physical entries even when names repeat. Re-list when the source changes. For a large listing, request the next page using `data.next_offset` until it is null. Exact member paths are case-sensitive and use forward slashes. The escaped `display_name` is meant for display.

## Create and check

```sh
xdzip create backup.zip documents/ --level normal --non-interactive --json
xdzip test backup.zip --non-interactive --json
```

Use a new archive name and an existing output parent for creation. Select a supported format and options. When handling an encrypted ZIP in automation, provide one UTF-8 password line through the calling process's private stdin pipe and add `--password-stdin`. Keep passwords out of arguments, transcripts and logs. An operation that needs a password or conflict approval without an available response reports that it needs interaction. [Password and format options](../engineering/command-line.md#archive-behavior)

## Interpret results

With `--json`, parse the single stdout object using its `schema_version`, `status`, `error_code`, `data`, `summary` and `warnings`. Keep stderr separate for progress and diagnostic text. Use the process exit code and the result together.

| Exit code | Handling |
| --- | --- |
| 0 | The command succeeded. Check counts for explicit skips and distinguish a handoff from completed installation. |
| 1 | The operation failed, partially completed or reported a cleanup warning. Inspect the summary and warnings before retrying. |
| 2 | Correct the command or arguments. |
| 3 | Supply the interaction, authorization or installation condition reported by the command. |
| 130 | Cancellation was confirmed. Check for files already committed. |

Unknown values in JSON may be null. Use the saved-file counts and the result status to decide whether to retry a smaller selection. For complete options and the output contract, see the [CLI reference](../engineering/command-line.md).
