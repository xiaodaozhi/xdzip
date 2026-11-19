<a name="xdzip-接口契约与检查"></a>

# 介面契約與檢查

[English](../../engineering/contracts.md) | [简体中文](../../zh-CN/engineering/contracts.md) | 繁體中文（台灣）

權威介面見[領域與資料](../design/domain-data.md)、[Qt/Rust 橋接](../design/qt-rust-bridge.md)、[格式擴充套件設計](../design/codec-extension-sdk.md)和[任務排程](../design/task-scheduling.md)；本頁定義凍結檢查，不復制另一套簽名。內部橋接、雙控制代碼和生命週期已實現，具體邊界見[執行時契約](product-runtime.md)。查詢代際、來源版本檢查、建立與解壓任務已分別落地，見[瀏覽契約](readonly-browser.md)、[安全解壓](zip-safe-extraction.md)及[建立契約](zip-creation.md)。第三方程序協議已有獨立[只讀 SDK 試驗](sdk-trial.md)，尚未作為穩定 SDK 釋出或接入產品；下表規定的是接入門檻。

| 邊界 | 開工前確認 | 必測失敗行為 |
| --- | --- | --- |
| UI → bridge | 控制代碼所有權、RequestId、非阻塞提交、關閉時拒絕 | 佇列滿、重複關閉、物件已銷燬 |
| bridge → UI | session/query/generation/revision、批次頁面、事件序號 | 舊響應丟棄、整數窄化失敗、終態不丟失 |
| runtime → codec | 能力而非字尾、輸入 reader、輸出 sink、取消令牌 | 不支援特性、CRC 錯誤、短讀/短寫 |
| runtime → 安全 I/O | 授權根、對映、暫存、提交許可、提交回執 | 連結競態、磁碟滿、覆蓋目標已變化 |
| plugin → host | 握手版本、幀上限、能力、會話生命週期 | 畸形/超長幀、超時、亂序、拒絕取消 |

每個介面說明輸入有效性、所有權、執行緒、資源預算、錯誤種類、冪等性和取消邊界。未知值與零分開；TaskId 與 EntryId 不可混用；選擇身份繫結條目而非目前行號。

CLI 與 GUI 共享 Rust 服務，Catalog 提供最多 128 條物理後設資料，SelectMembers 在同一不可變會話內解析完整有效路徑與 EntryId，再提交現有任務。列表 ID 只屬於該次掃描，歸檔變化後不能作為持久身份。submit_wait 只等待短准入鎖，佇列容量及關閉拒絕規則不變。GUI 不開放這些內部命令的 QML 編碼，CLI 不使用轉義展示文字寫出檔案。

固定宿主後設資料使用 open_regular_nofollow，拒絕葉符號連結和重解析點，並與目錄授權、大小及身份檢查組合。既有使用者所選歸檔讀取仍保留自己的路徑別名語義。桌面 CLI 請求透過同使用者單例項的可選 cli_protocol=1 擴充套件傳送有界請求目錄，舊開啟協議仍為 1；沒有該擴充套件的例項不能確認 CLI 交接。具體結果、退出碼與安裝邊界見[命令列契約](command-line.md)。

內部契約可隨實現更新，並須在同一提交中保持一致。第三方線協議須兩個獨立實現互通並完成相容性套件後才能稱 v1 stable；版本號未知必須明確拒絕或按協商規則降級。
