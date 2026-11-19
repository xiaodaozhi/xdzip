<a name="xdzip-xdzip-术语与实现限额"></a>

# XDZip 術語與實現限額

[English](../../design/glossary.md) | [简体中文](../../zh-CN/design/glossary.md) | 繁體中文（台灣）

<a name="xdzip-1-术语"></a>

## 1. 術語

| 術語 | 定義 |
|---|---|
| 歸檔/Archive | 一個容器或受支援壓縮流及其授權源 |
| 編解碼器/Codec | 識別、掃描、讀取或建立格式的實現 |
| 會話/Session | 開啟源、後端、索引和瀏覽上下文；不是任務 |
| 條目/Entry | 一個實際歸檔記錄；目錄可另有合成展示節點 |
| 來源版本/SourceVersion | 對已接收檔案或分卷用於盡力檢測變化的指紋 |
| 查詢快照/QuerySnapshot | 執行時的目前不可變結果順序，由會話、查詢、代數及修訂號標識 |
| 代次/Generation | 前端請求替代關係，避免過期結果覆蓋 |
| 已準備的作業 | 執行時為執行保留的有界輸入，包括本次操作所需的來源、選擇和選項 |
| 任務/Task | 作業進入任務佇列後的生命週期與執行結果 |
| 授權/Grant | 宿主持有的檔案/目錄訪問權，不是路徑字串 |
| Sink | 受計量、限額和事務控制的寫介面 |
| 工作執行緒與工作程序 | 命令和任務在宿主執行緒中執行，歸檔和圖片工作程序獨立執行。 |
| 提交/Commit | 透過已授權的檔案或歸檔操作釋出經過驗證的暫存內容 |
| 任務快照 | 最新的任務狀態、進度、提示或結果。中間快照可以合併。 |
| 能力/Capabilities | 特定實現、格式特性、平臺和政策的具體交集 |
| 固實/Solid | 多檔案共享壓縮塊，選擇性讀取可能需要解碼前置內容 |
| 首幀/First presented frame | 實際渲染呈現，不能以 ui_ready 日誌冒充 |

<a name="xdzip-2-单位和数值"></a>

## 2. 單位和數值

記憶體/檔案量使用二進位制 KiB/MiB/GiB，1MiB=1024² bytes；傳輸和計量保留整數 bytes。時間使用毫秒或秒。執行耗時採用單調時鐘。檔案時間在可用時保留日曆表達方式與來源精度。吞吐統一 MiB/s 並說明壓縮輸入、解碼或輸出哪一種位元組。

單核計 CPU 100% 表示佔滿一個邏輯核心，程序樹可超過 100%。PSS/RSS/USS/physical footprint 不互相等同。下載壓縮體積、安裝體積、executable 大小分別記錄。

<a name="xdzip-3-查询当前限额"></a>

## 3. 查詢目前限額

| 主題 | 實現參考 |
| --- | --- |
| 請求、分頁和郵箱限額 | [Qt/Rust 橋接](qt-rust-bridge.md) |
| 任務准入、進度與狀態碼 | [任務排程](task-scheduling.md) |
| 解壓位元組限額、檔案授權和提交 | [檔案安全](file-security.md) |
| 後設資料與 ZIP 磁碟索引限額 | [大型歸檔索引](../engineering/large-index.md) |
| 來源和分卷限額 | [開啟歸檔](../engineering/archive-opening.md) |
| 文字與圖片預覽限額 | [內容預覽](../engineering/content-preview.md) |
| 工作程序的資料幀、監督與資源限額 | [歸檔工作程序](../engineering/native-codecs.md) |
| 試驗性擴充套件協議 | [SDK 試驗](../engineering/sdk-trial.md) |
| 基準測試目標 | [效能設計](performance.md) |

<a name="xdzip-4-版本边界"></a>

## 4. 版本邊界

應用版本、內部 CXX 構建介面、外部通訊協議和持久化資料格式各自有版本邊界。產品版本無法直接推導 SDK 協議版本。見[編解碼擴充套件 SDK 設計](codec-extension-sdk.md)。
