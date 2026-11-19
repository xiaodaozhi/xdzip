<a name="xdzip-格式工作进程的宿主日历与编码服务"></a>

# 格式工作程序的宿主日曆與編碼服務

[English](../../engineering/host-platform-services.md) | [简体中文](../../zh-CN/engineering/host-platform-services.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/host-platform-services.md) | [한국어](../../ko-KR/engineering/host-platform-services.md) | [Français](../../fr-FR/engineering/host-platform-services.md) | [Deutsch](../../de-DE/engineering/host-platform-services.md) | [Español](../../es-ES/engineering/host-platform-services.md) | [Português (Brasil)](../../pt-BR/engineering/host-platform-services.md) | [Русский](../../ru-RU/engineering/host-platform-services.md)

格式欄位、密碼相容規則和校驗由 Rust 處理。作業系統的歷史時區、OEM/ANSI 內碼表和本地 C 編碼屬於宿主環境，透過 `ReadSource` 的可選服務提供，不能由格式層根據檔名、HostOS 或目前 UTC 偏移猜測。服務不依賴 Qt，不授予路徑訪問或檔案釋出許可權。

<a name="xdzip-日历投影"></a>

## 日曆投影

`Timestamp` 明確區分 1601 epoch 的 FILETIME 100ns 值與 1970 epoch 的無符號 Unix 整秒。`ReadSource::local_calendar` 返回可選 `Calendar`，普通檔案和記憶體源的預設策略是確定性的 UTC；產品 worker 透過同步 CXX 回撥使用作業系統的本地時區規則。Windows 使用系統時區 API，Unix 使用 `localtime_r`，均檢查轉換結果。轉換不可用時返回空日曆，不能讀取未初始化欄位。

Rust 驗證年月日與時分秒後打包 DOS 展示值。7z 延續 UTC 和 1980..2107 的既有展示範圍；RAR 按其[固定相容規則](rar-format-sources.md)先處理內部模運算、時間覆蓋及平臺差異，再使用宿主日曆和歷史年份迴繞。原始精度、未設定標記與完整物理頭保留在格式私有索引，展示精度不會反向覆蓋它們。RAR 的安全候選政策將無法投影的時間顯示為 0，不復制參考實現未檢查 OS 失敗後的未定義內容。

<a name="xdzip-受控编码映射"></a>

## 受控編碼對映

| 服務 | 輸入與輸出 | 宿主邊界 |
| --- | --- | --- |
| `oem_to_ansi_wide` | 512 個 UTF16 單元到 512 個 UTF16 單元，含 NUL | Windows OEM 到 ANSI 的完整往返，保留系統實際對映與替代行為 |
| `wide_to_ansi` | 含 NUL 的不超過 512 個 UTF16 單元到 512 位元組 | Windows 目前 ANSI 內碼表，不自造統一替代字元 |
| `wide_to_local_prefix` | 含 NUL 的不超過 128 個寬標量到 512 位元組 | Unix 目前 C 編碼，轉換失敗按固定相容策略清空重試並保留已轉換字首 |

這些方法返回不含 NUL 的長度。缺少宿主服務的預設實現返回 `None`，格式呼叫者只能使用已定義的可移植回退或明確拒絕，不能靜默猜測內碼表。產品回撥檢查維度、NUL 與返回範圍，無法執行的對映返回穩定的 Unsupported。

Windows 的 [CharToOemBuffW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-chartooembuffw) 長度引數計輸入寬字元，並繼續轉換 NUL 後的補齊單元，不是目標位元組容量。宿主先用 `GetCPInfo(CP_OEMCP)` 准入每單元至多四位元組的內碼表，再用 2,048 位元組臨時目標對映完整 512 單元，隨後才設定位元組 511 為 NUL 並投影 ANSI。無法確認該容量前提時拒絕轉換；不能用 512 位元組目標接收 512 個寬字元，或先裁剪輸入來回避容量問題。實際 OS 探針包含完整 511 單元 DBCS 邊界及獨立直接 WinAPI 對照。

512 單元准入、首 NUL、OEM 補償條件、127 單元及位元組截斷、舊標記協議和 macOS 的顯式 UTF8 都屬於 Rust 的格式相容策略，不放進 C++。尤其不能先截斷密碼再對其執行本來位於截斷之前的 OS 對映，也不能把 DBCS 中間截斷擅自改成字元邊界截斷。

<a name="xdzip-所有权和失败"></a>

## 所有權和失敗

所有切片只在同步呼叫期間借用，C++ 不儲存指標。Rust 呼叫方負責秘密輸入與輸出的歸零生命週期；C++ 的有界臨時寬、窄緩衝在所有返回路徑擦除。此保證不擴充套件為作業系統內部或全部歷史記憶體副本已擦除。原語不記錄密碼，不把它放進程序引數或環境變數。

Reader 在服務前檢查取消。服務結果不生成解壓輸出額度，不改變已讀、已解壓或已寫位元組。失敗保留已有的來源、取消與限額類別，不能誤作密碼錯誤；既定的密碼/完整性歧義僅由格式層依據實際 verifier 狀態決定。
