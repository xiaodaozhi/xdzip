<a name="xdzip-qtrust-桥接与应用服务"></a>

# Qt/Rust 橋接與應用服務

[English](../../design/qt-rust-bridge.md) | [简体中文](../../zh-CN/design/qt-rust-bridge.md) | 繁體中文 | [日本語](../../ja-JP/design/qt-rust-bridge.md) | [한국어](../../ko-KR/design/qt-rust-bridge.md) | [Français](../../fr-FR/design/qt-rust-bridge.md) | [Deutsch](../../de-DE/design/qt-rust-bridge.md) | [Español](../../es-ES/design/qt-rust-bridge.md) | [Português (Brasil)](../../pt-BR/design/qt-rust-bridge.md) | [Русский](../../ru-RU/design/qt-rust-bridge.md)

Qt 桌面透過連結到程式內的 `cxx` 橋接呼叫 Rust 執行時。每個 `BrowserController` 有自己的 `RuntimeConnection` 和對應的服務例項。橋接在桌面程序內傳遞型別化命令和自有資料的結果。CLI 直接呼叫同一個 Rust 服務實現。[系統架構](system-architecture.md)、[桌面會話](../engineering/application-sessions.md)

<a name="xdzip-句柄线程与所有权"></a>

## 控制代碼、執行緒與所有權

`new_runtime()` 啟動 Rust 命令執行緒和任務執行緒。臨時的 `Runtime` 各交出一個 `CommandClient` 和 `EventReceiver`。介面執行緒使用命令控制代碼，專用的 Qt 接收執行緒持有事件控制代碼並在 `next_event()` 中阻塞等候。Rust 命令執行緒持有目前歸檔和索引，獨立的任務執行緒在執行期間持有已準備的作業。

CXX 傳遞值型別和自有資料的 Rust 容器。Qt 接收方在持有這些值時儲存或轉換它們。橋接不向 Qt 傳遞歸檔或原生檔案控制代碼。控制器將需要保留完整 64 位精度的編號和位元組數以字串傳給 QML。生成的標頭檔案和靜態庫與宿主一起構建。該介面屬於產品構建，隨包提供的歸檔工作程序使用獨立的[程序協議](../engineering/native-codecs.md)。

[橋接宣告與轉換](../../../crates/xdzip-qt-bridge/src/lib.rs)、[Qt 連線](../../../apps/desktop/RuntimeConnection.cpp)、[Rust 服務](../../../crates/xdzip-runtime/src/service.rs)

<a name="xdzip-请求查询与任务身份"></a>

## 請求、查詢與任務身份

| 身份 | 用途 |
| --- | --- |
| 請求編號 | Rust 服務接收命令時分配，編號在該連線內有效。`Submission` 返回 `Accepted`、`Busy`、`Closed`、`Invalid` 或 `Exhausted`。拒絕時編號為零。 |
| 會話編號 | 標識目前開啟的歸檔快照。針對該歸檔的命令必須匹配會話編號。新開啟的歸檔獲得新編號。 |
| 查詢身份與分頁偏移 | 分頁繫結會話、查詢編號、介面代數、修訂號和偏移。控制器核對這些值後才接收列表行。 |
| 物理條目或導航編號 | 物理條目的身份在排序和翻頁後保持不變。虛擬目錄有獨立的導航身份。行下標、顯示名和查詢範圍均不會授予檔案系統訪問許可權。 |
| 任務與提示編號 | 任務編號在所屬連線內有效。提示編號指向任務中一個待答覆的重名或密碼提示。共享任務入口還記錄對應的控制器。 |
| 事件序號 | 接收器取出事件時，在連線內分配遞增的序號。Qt 丟棄序號不大於已處理序號的事件。任務更新也使用這一序號，並透過原請求編號建立關聯。 |

<a name="xdzip-当前桥接命令"></a>

## 目前橋接命令

| 命令或方法 | 用途與普通響應 |
| --- | --- |
| `Open`、`OpenWithPassword` | 開啟並掃描所選來源。成功時返回第一頁 `Page` 和受限的導航資料。加密目錄可能要在開啟會話前先提供密碼。 |
| `Browse` | 發起查詢，或用匹配的身份和偏移繼續查詢。每次最多返回一個 `Page`，上限為 128 行和 64 KiB 名稱資料。 |
| `FreezeSelection`、`Preflight` | 把目前選擇器解析為 `Selected` 中自有資料的物理或導航編號，或在 `Preflight` 中彙總後設資料檢查結果。實際操作還會重新核對快照和選中範圍。 |
| `Preview` | 定位一個物理條目。成功時返回自有資料且有容量限制的 `Preview` 位元組，以及條目與會話身份。 |
| `CheckTarget`、`InspectInputs` | 執行時開啟並清理探測工作區後，在 `TargetChecked` 中返回剩餘項目數量。獨立建立檢查使用會話編號零。輸入概況按傳入順序返回 `Inputs`，只檢查淺層後設資料，無需遞迴掃描目錄。兩種響應均不授予提交許可權。 |
| `Extract`、`Create`、`Check`、`OpenEntry` | 校驗操作意圖並將任務入隊。`TaskAccepted` 攜帶分配的任務編號。目標授權和提交由執行時與 I/O 層處理。 |
| `Update`、`Move`、`Export`、`PublishExport`、`ExportManifest` | 歸檔修改和資料釋出進入重任務佇列。接收 `Update` 或 `Move` 後，服務釋放目前歸檔快照，後續請求不能再使用該快照的舊條目編號。`ExportManifest` 作為普通響應返回相對路徑等傳輸後設資料。 |
| `Snapshot`、`CloseSession` | 返回目前歸檔會話編號或零。關閉指定會話時釋放瀏覽快照，任務可以繼續持有自己保留的輸入。 |
| `cancel`、`cancel_task`、`decide_task`、`respond_password`、`release_export`、`shutdown` | 獨立於普通命令佇列的控制方法。請求取消有自己的請求編號和 `Finished`，任務控制指向已有任務。 |

Rust 服務還提供 `Catalog` 和 `SelectMembers` 等 CLI 專用操作。橋接在接收命令前校驗輸入大小與列舉值。例如，建立輸入路徑每條限 4 KiB，路徑合計限 4 MiB。執行時校驗會話和操作語義，I/O 層負責開啟檔案和取得輸出能力。[執行時服務契約](../engineering/product-runtime.md)、[命令列契約](../engineering/command-line.md)、[檔案授權](file-security.md)

<a name="xdzip-事件交付与背压"></a>

## 事件交付與背壓

正常交付時，每個已接收的普通請求最多返回一份資料，隨後由一個 `Finished` 結束。出錯時 `Finished` 攜帶錯誤碼。控制器等待請求成功結束，再應用匹配的分頁或預覽。被後續操作替代的結果會丟棄，對應請求仍會結束。大索引路徑在 Qt 中最多快取 1,024 行，按需請求未載入的分頁，並在開啟解壓確認前凍結跨頁選區。[大型歸檔索引](../engineering/large-index.md)、[內容預覽](../engineering/content-preview.md)

`TaskAccepted` 和 `Finished` 屬於普通請求交付。`TaskUpdate` 使用每個任務獨立的待交付槽，攜帶原請求編號、任務編號和來源會話編號。任務更新可能早於 `TaskAccepted` 到達，也可能晚於 `Finished`。控制器透過原請求關聯提前到達的任務，再將後續更新送到對應任務。後到的任務更新不會再次結束普通請求。待交付槽保留任務的最新快照，進度可以合併，最終快照一直保留到接收器取走。等待答覆期間，提示包含在任務快照中。[任務狀態與取消](task-scheduling.md)

- **Rust。** 每個連線允許 128 個尚未結束的普通請求，待執行命令的位元組預算為 8 MiB，普通事件負載預算為 32 MiB。已接收的普通請求有結果和結束事件的保留容量。請求取消回執有獨立的有界佇列，關閉無需佔用該佇列的空位。
- **Qt。** 郵箱最多容納 256 個事件，其中載荷事件最多 128 個，計入預算的負載最多 32 MiB。容量用盡時，接收執行緒等待介面執行緒取走資料。一條排隊通知鏈每輪最多處理 32 個事件或使用 4 毫秒。通知發往介面執行緒，排隊事件持有自己的資料。

<a name="xdzip-提示失败与关闭"></a>

## 提示、失敗與關閉

- 重名答覆使用 `decide_task`，任務密碼使用 `respond_password` 和獨立的密碼提示編號。兩者都校驗目前仍未答覆的提示。密碼答覆進入釋放時會清零的 Rust 儲存空間，不進入結果或進度事件。`OpenWithPassword`、`Preview` 和加密建立攜帶各自操作的密碼輸入，橋接會把它們轉入相應的 Rust 密碼型別。[密碼處理](../engineering/password-extraction.md)
- 准入、請求和任務結果使用穩定錯誤碼，例如 `stale_session`、`stale_query`、`source_changed`、`unsafe_target`、`target_changed`、`disk_full` 和 `password_required`。控制器根據錯誤碼本地化顯示。來源變化會使相關瀏覽快照失效。`RuntimeFault` 或意外斷開時，介面將未完成操作按 `runtime_unavailable` 處理，連線丟失可能使原本的結束事件無法到達。
- `shutdown` 停止接收新命令，取消已接收的工作，包括等待輸入的任務。Rust 執行緒退出期間，Qt 接收執行緒繼續取出事件。Rust 接收器等待工作執行緒退出後再返回 `ShutdownReady`，隨後橋接返回 `End`。Qt 郵箱排空後報告正常斷開。連線若提前銷燬，會關閉郵箱並喚醒接收執行緒，介面執行緒無需等待它結束。[任務關閉](task-scheduling.md)、[執行時生命週期](../engineering/product-runtime.md)

修改這一邊界時，可用[橋接探針](../../../tools/bridge-probe/README.md)、Rust 服務測試和[桌面控制器測試](../../../apps/desktop/tests/browser_test.cpp)核對各自覆蓋的層級。相關構建目標見[開發構建指南](../engineering/developer-build.md)。
