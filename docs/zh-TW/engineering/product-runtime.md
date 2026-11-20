<a name="xdzip-产品运行时与-qt-生命周期"></a>

# 產品執行時與 Qt 生命週期

[English](../../engineering/product-runtime.md) | [简体中文](../../zh-CN/engineering/product-runtime.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/product-runtime.md) | [한국어](../../ko-KR/engineering/product-runtime.md) | [Français](../../fr-FR/engineering/product-runtime.md) | [Deutsch](../../de-DE/engineering/product-runtime.md) | [Español](../../es-ES/engineering/product-runtime.md) | [Português (Brasil)](../../pt-BR/engineering/product-runtime.md) | [Русский](../../ru-RU/engineering/product-runtime.md)

本文介紹桌面程式如何透過命令與事件連線 Rust 執行時，包括佇列背壓、關閉流程與平臺構建。歸檔瀏覽及安全解壓的具體能力分別見[只讀瀏覽](readonly-browser.md)與[安全解壓](zip-safe-extraction.md)。

<a name="xdzip-已落地的服务"></a>

## 已落地的服務

`xdzip-runtime::service` 實現產品服務。GUI 取一次 `CommandClient`，專用 Qt 接收執行緒取一次 `EventReceiver`。Rust 命令執行緒持有連線中開啟的歸檔和瀏覽快照。獨立的 Rust 任務執行緒接收已準備的任務及其所需的來源和設定，並發布任務更新。歸檔控制代碼和同步掃描入口保留在 Rust 內部。ABI 見 [Qt/Rust 橋接](../design/qt-rust-bridge.md)，所有權和准入見[任務排程](../design/task-scheduling.md)。

基礎命令包括 Open、Browse、Preflight、CloseSession、Snapshot；現有真實瀏覽和預檢全部接入。每個已接納的普通請求分配不復用 ID，返回至多一個有界資料事件，再返回恰好一次 Finished；錯誤直接 Finished。頁面攜帶會話、請求和偏移身份；每個 RuntimeConnection 單會話、序列處理的 GUI 待處理請求，查詢還包含完整查詢 ID、GUI 代際和快照修訂，見 [ZIP 瀏覽契約](zip-readonly-browsing.md)。跨會話請求返回 stale_session。

GUI 普通准入使用 try_lock，不等待佇列空間，校驗路徑 4 KiB、搜尋 1 KiB、選擇 128 項、目錄 ID 21 位元組、排序碼 0–12。拒絕不消耗 ID，Busy 可重試。取消有獨立最多 128 項控制通道和自己的 Finished 確認；確認不保證目標尚未完成。關閉保留獨立、冪等 RequestId，預留最後一個可用 ID，即使正常 ID 耗盡或佇列滿也可關閉。

Qt 控制器僅保留一個待准入命令。短暫 Busy 在事件迴圈中以 1 ms 單次定時器重試，最多嘗試 32 次，持續飽和才提示重試；這不是完成時限保證。取消、關閉或斷連立即撤銷未接收命令，空閒時沒有重試定時器。非法引數與 ID 耗盡分別報告，不偽裝成忙碌；斷連後禁用開啟操作。

<a name="xdzip-有界传输"></a>

## 有界傳輸

- 未完成的普通請求最多 128 項，直到接收端取走 Finished 才釋放名額。每項預留兩個事件位置，終態不被資料擠掉。
- Rust 資料事件累計內容預算 32 MiB，包含最多 16 MiB 圖片和其容器，超過預算暫停釋出；取消/關閉可喚醒等待並改發 cancelled 終態。單事件超限返回 resource_limit。
- Rust 命令累計內容仍限 8 MiB，超限拒絕准入，不受圖片事件預算擴大影響。
- Qt 信箱 最多 256 個事件、32 MiB 內容位元組，其中普通資料最多 128 個位置，其餘為終態/控制保留；不會為每條 Rust 事件生成一個 Qt 回呼。
- 一個通知鏈按每輪最多 32 個事件、約 4 ms 的時間預算取出事件，未結束則投遞下一輪；單事件處理不可搶佔。佇列空時不再排程，接收執行緒由條件變數阻塞。
- Open 的 Page 額外攜帶完整有界目錄索引（至多 32,768 節點、名稱和字首共 4 MiB），由 `replace_directories` 標識替換；空索引也明確替換。Browse 不重複傳送索引，`navigation_limited` 明確索引降級。Rust 計入名稱和字首位元組，Qt 信箱 還計入橋接生成的節點/父 ID 字串；節點容器和深度另設上限。只有對應請求的成功 Finished 後才一起應用新頁面與目錄模型。
- Page 的 `total` 是目前列表行數，`archive_total` 獨立保留物理歸檔記錄數。普通瀏覽只列目前目錄的直屬檔案和目錄，搜尋遞迴返回物理匹配。缺少顯式記錄的目錄使用 `navigation_id` 和 `physical=false`；Qt 行鍵為 `dN`，原始物理 ID 保持獨立，虛擬節點不可提交 Preflight。完整路徑保留在行後設資料中，顯示名稱按普通瀏覽／搜尋分別使用 basename／全路徑。
- 字串預算不等於 RSS：容器、快照、轉換中的單個事件另有空間，但條目和頁大小始終有界。橋接批次最多 128 項；早期 100,000 條掃描基線及普通單檔案 ZIP 的後續擴充套件分別見[ZIP 瀏覽契約](zip-readonly-browsing.md)和[大歸檔索引](large-index.md)。

控制與普通事件交替取出，避免持續取消確認使普通終態飢餓。遞增序號在交付時分配；Qt 丟棄舊序號，控制確認和過期 request_id 不修改目前檢視。

獨立任務通道的 `Task`／`TaskUpdate` 包絡保留建立任務的原始 request_id，關聯記錄與有界任務控制槽一起保留，交付任務終態時釋放。它可早於接納資料或晚於請求 Finished，不佔用第二個普通資料名額，也不釋放或重新完成普通請求；用途是將準備佔位與真實 TaskId、源會話準確關聯。該 ID 不是取消任務的請求 ID，任務控制仍使用 TaskId。

<a name="xdzip-关闭销毁与故障"></a>

## 關閉、銷燬與故障

關閉單個歸檔或視窗後，應用和任務仍可能在托盤中執行。應用退出或重啟時，協調器停止接收新請求、請求取消，並在 Rust 釋放會話期間繼續接收剩餘事件。接收端在兩個 Rust 執行緒全部結束後才返回 `ShutdownReady`。桌面應用還要等 Qt 接收執行緒結束且信箱排空後才退出。GUI 執行緒不會等待仍在處理檔案或投遞事件的執行緒結束。

取消/關閉控制使用只含有界狀態操作的短臨界區，不受資料佇列容量約束。協作取消不能打斷作業系統阻塞讀，這不是硬實時退出保證。本節的只讀請求不產生輸出事務；建立與解壓的提交、暫存清理和退出結果分別按[建立](zip-creation.md)與[安全解壓](zip-safe-extraction.md)契約處理。

QObject 提前銷燬時，關閉信箱、喚醒接收執行緒、停止服務；背景僅持有共享佇列和 Rust 接收控制代碼，不捕獲控制器指標。Qt 自動撤銷針對已銷燬物件的通知。事件消費回撥若同步銷燬連線也有 QPointer 生命週期防護。放棄接收時不在解構函式中等待執行緒，工作會協作結束；只有正常的 ShutdownReady 才能證明等待已完成。

只讀工作處理函式發生可進行堆疊展開的 panic 時，邊界捕獲後廢棄整個服務，傳送 RuntimeFault，並將未完成請求終結為 runtime_unavailable，不繼續使用該會話。此處只定義只讀請求的故障行為，不承諾恢復半完成的建立或解壓事務；寫入任務的已提交項、結果不確定性與清理按對應契約處理。分配失敗、程序 abort、poisoned 內部鎖等不可恢復錯誤不承諾傳送終態。Qt 對意外 EOF/連線失敗統一結束待處理狀態，防止永久忙碌。

<a name="xdzip-验证与构建"></a>

## 驗證與構建

驗證方法見[通用入口](../../../developers/qa/README.md)。Rust 核心測試不需要 Qt；Qt 視窗、模型、壓力和關閉用例在三平臺原生構建執行。macOS CI 驗證 arm64 與 SDK 環境 Cocoa 啟動，不部署或上傳應用；本地可按明確需求生成開發 DMG。Windows/Linux 部署隨包 Qt 並驗證無 SDK 執行，範圍見[構建指南](developer-build.md)。這些檢查不等同正式發行獲批。

初始查詢分配不可變結果身份，後續批次必須匹配全部 QueryKey 欄位與引數；Qt 取消舊請求並合併最新意圖，序列與請求 ID 的既有檢查仍保留。單批另外限制名稱 64 KiB，不重複過濾／排序。歷史驗收結果仍以當時實際具備的能力為準。

應用層可建立多個獨立連線，領域執行時仍以連線隔離會話；全域性任務身份與序列重任務准入見[應用會話契約](application-sessions.md)。
