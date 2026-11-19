<a name="xdzip-在脚本中使用-xdzip"></a>

# 在指令碼中使用 XDZip

[English](../../integration/README.md) | [简体中文](../../zh-CN/integration/README.md) | 繁體中文（台灣） | [日本語](../../ja-JP/integration/README.md) | [한국어](../../ko-KR/integration/README.md) | [Français](../../fr-FR/integration/README.md) | [Deutsch](../../de-DE/integration/README.md) | [Español](../../es-ES/integration/README.md) | [Português (Brasil)](../../pt-BR/integration/README.md) | [Русский](../../ru-RU/integration/README.md)

獨立 CLI 無需 Qt 或桌面會話。從 [GitHub 釋出頁](https://github.com/xiaodaozhi/xdzip/releases)下載適合目前系統的 `xdzip-cli-` 包，完整解壓，保留可執行檔案旁的工作程序。從解壓目錄啟動時，macOS 或 Linux 執行 `./bin/xdzip --help`，Windows PowerShell 執行 `.\bin\xdzip.exe --help`。

下文以 PATH 中的 `xdzip` 為例。用解壓後的 CLI 執行 `setup` 可登記目前使用者的命令入口，按需將命令目錄加入 PATH。執行 `xdzip --version-json`，確認找到的程式報告 `"component":"cli"`。桌面啟動程式和 CLI 可能使用相同的命令名，衝突時直接指定解壓後的 CLI 路徑。[命令安裝與移除](../engineering/command-line.md#xdzip-%E6%9E%84%E5%BB%BA%E4%B8%8E%E5%91%BD%E4%BB%A4%E5%AE%89%E8%A3%85)

<a name="xdzip-创建和检查"></a>

## 建立和檢查

選一個新的私有可寫目錄作為工作目錄。在其中建立 `docs` 資料夾並放入 `hello.txt`，再把另一份檔案 `notes.txt` 放在資料夾旁。下面的命令依次建立 ZIP、把第二份檔案新增到已有的包內目錄 `docs/`，並檢查結果。

```sh
xdzip create sample.zip docs --level normal --json
xdzip add sample.zip notes.txt --into docs --conflict skip --json
xdzip test sample.zip --json
```

可從[格式支援表](../../../README.zh-TW.md#xdzip-%E6%A0%BC%E5%BC%8F%E6%94%AF%E6%8C%81)選擇建立字尾和選項。輸出須放在輸入樹外，且上級目錄已存在。`add` 僅用於符合條件的現有歸檔，`--into` 指定已有包內目錄，省略時新增到根目錄。`test` 不寫出解壓檔案，成功時由 `data.check_kind` 說明檢查強度。詳見[格式登記](../engineering/format-registry.md)與[完整性檢查](../engineering/archive-checking.md)。

<a name="xdzip-列出选择和解压"></a>

## 列出、選擇和解壓

繼續在同一工作目錄使用剛建立的歸檔。請選擇新的目標位置，或明確如何處理同名檔案。

```sh
xdzip list sample.zip --limit 2 --json
xdzip extract sample.zip docs/hello.txt -C output --conflict skip --json
```

包內路徑必須精確匹配、區分大小寫並使用正斜槓。有多條記錄使用同一路徑時，可用 `list` 返回的 `id` 選擇。ID 從 1 開始。例如，如果列表中目標檔案的 ID 為 2，就把第二條命令中的 `docs/hello.txt` 換成 `--entry-id 2`。選擇多個條目時重複 `--entry-id`。用 `path` 匹配條目，用 `display_name` 展示。歸檔變化後重新列出。

第一頁從偏移量 0 開始。讀取下一頁時，把 `data.next_offset` 傳給 `--offset`，並保持 `--limit` 不變，直到 `next_offset` 為 null。偏移量是從 0 開始的行位置，ID 標識物理記錄。每條命令都會重新開啟歸檔，分頁期間應保持來源不變。[CLI 選擇與列舉限制](../engineering/command-line.md#xdzip-%E5%BD%92%E6%A1%A3%E8%A1%8C%E4%B8%BA)

<a name="xdzip-无交互运行"></a>

## 無互動執行

`--json` 禁用互動提示。需要無人值守執行時，請明確選擇重名策略。對於支援密碼的命令，例如 `extract`、`test` 或加密的 `create`，可指定 `--password-stdin` 並透過專用標準輸入管道傳入一行 UTF-8 密碼。加密建立還需要 `--encrypt`。缺少必要的決定或密碼時，CLI 返回 `interaction_required`。[密碼與格式選項](../engineering/command-line.md#xdzip-%E5%BD%92%E6%A1%A3%E8%A1%8C%E4%B8%BA)

<a name="xdzip-解读结果"></a>

## 解讀結果

歸檔操作使用 `--json` 時，從 stdout 解析一個 UTF-8 物件，讀取 `schema_version`、`status`、`error_code`、`data`、`summary` 和 `warnings`。stderr 可用於讀取診斷資訊。`--version-json` 返回獨立的構建身份物件。

| 退出碼 | 處理方式 |
| --- | --- |
| 0 | 檢視結果及跳過數量。適用命令返回 `handed_off` 時，表示請求已交給桌面應用。 |
| 1 | 檢視錯誤、警告和彙總，確認是失敗、部分完成還是有清理警告。 |
| 2 | 修正命令或引數。 |
| 3 | 檢查報告的互動、授權、平臺或安裝要求。 |
| 130 | 已確認取消，請核對已儲存的檔案。 |

未知值可為 null。失敗或取消前已儲存的檔案可能保留。重試前，若結果包含 `summary.committed_files`，請先核對該值。桌面交接、更新回執與完整輸出契約見[命令列參考](../engineering/command-line.md)。
