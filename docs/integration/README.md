# Use XDZip in scripts

English | [简体中文](../zh-CN/integration/README.md) | [繁體中文（台灣）](../zh-TW/integration/README.md) | [日本語](../ja-JP/integration/README.md) | [한국어](../ko-KR/integration/README.md) | [Français](../fr-FR/integration/README.md) | [Deutsch](../de-DE/integration/README.md) | [Español](../es-ES/integration/README.md) | [Português (Brasil)](../pt-BR/integration/README.md) | [Русский](../ru-RU/integration/README.md)

The standalone CLI runs without Qt or a desktop session. Choose an `xdzip-cli-` package for your system from the [GitHub releases](https://github.com/xiaodaozhi/xdzip/releases). Unpack the whole package and keep its worker beside the executable. From the extracted directory, run `./bin/xdzip --help` on macOS or Linux, or `.\bin\xdzip.exe --help` in PowerShell.

The examples use `xdzip` on PATH. Run `setup` with the extracted CLI to register a user-level launcher, then add its command directory to PATH if needed. Use `xdzip --version-json` to confirm that the resolved program reports `"component":"cli"`. The desktop launcher and the CLI can have the same command name. Use the extracted CLI's explicit path if they conflict. [Command installation and removal](../engineering/command-line.md#build-and-command-installation)

## Create and check

In a fresh, private, writable working directory, create a folder named `docs` containing `hello.txt`. Put a second file, `notes.txt`, beside the folder. Use the following sequence to create a ZIP, add the second file under the existing `docs/` member directory, and check the result.

```sh
xdzip create sample.zip docs --level normal --json
xdzip add sample.zip notes.txt --into docs --conflict skip --json
xdzip test sample.zip --json
```

Choose a creation suffix and options from the [format table](../../README.md#format-support). Keep the output outside the input tree and use an existing output parent. `add` requires an eligible existing archive. Use `--into` for an existing archive directory or omit it to add at the root. `test` produces no extracted files. Its `data.check_kind` indicates the strength of a successful check. [Format registry](../engineering/format-registry.md). See also [Integrity checks](../engineering/archive-checking.md)

## List, select and extract

Continue with the archive above in the same working directory. Use a fresh destination or explicitly choose how to handle existing files.

```sh
xdzip list sample.zip --limit 2 --json
xdzip extract sample.zip docs/hello.txt -C output --conflict skip --json
```

Member paths are exact, case-sensitive and use forward slashes. If several records have the same path, select by the `id` returned by `list`. IDs start at 1. For example, replace `docs/hello.txt` in the second command with `--entry-id 2` if the list shows ID 2 for that file. Repeat `--entry-id` for more members. Use `path` for member matching and `display_name` for display. Re-list after the archive changes.

The first page starts at offset 0. Pass `data.next_offset` as `--offset` with the same `--limit` to read the next page. Stop when `next_offset` is null. An offset is a zero-based row position. An ID identifies a physical record. Each command opens the archive again, so hold the source unchanged across paged calls. [CLI selection and listing limits](../engineering/command-line.md#archive-behavior)

## Run without prompts

`--json` disables interactive prompts. Choose a conflict policy explicitly when a job must finish unattended. For a command that supports passwords, such as `extract`, `test` or encrypted `create`, use `--password-stdin` and send one UTF-8 password line through a private stdin pipe. Encrypted creation also needs `--encrypt`. When a required decision or password is unavailable, the CLI returns `interaction_required`. [Password and format options](../engineering/command-line.md#archive-behavior)

## Interpret results

For archive operations with `--json`, parse the UTF-8 object on stdout. Read `schema_version`, `status`, `error_code`, `data`, `summary` and `warnings`. Keep stderr available for diagnostics. `--version-json` returns a separate build-identity object.

| Exit code | Handling |
| --- | --- |
| 0 | Read the result and skip counts. A `handed_off` status confirms delivery to the desktop for an applicable command. |
| 1 | Inspect the error, warnings and summary for failure, partial completion or cleanup warnings. |
| 2 | Correct the command or arguments. |
| 3 | Check the reported interaction, authorization, platform or installation requirement. |
| 130 | Cancellation was confirmed. Check for files already committed. |

Unknown values may be null. A failure or cancellation can leave earlier files committed. Check `summary.committed_files` when present before retrying. For desktop handoff, update receipts and the full output contract, see the [CLI reference](../engineering/command-line.md).
