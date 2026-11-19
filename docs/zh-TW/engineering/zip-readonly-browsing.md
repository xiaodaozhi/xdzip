<a name="xdzip-zip-只读浏览契约"></a>

# ZIP 只讀瀏覽契約

[English](../../engineering/zip-readonly-browsing.md) | [简体中文](../../zh-CN/engineering/zip-readonly-browsing.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/zip-readonly-browsing.md) | [한국어](../../ko-KR/engineering/zip-readonly-browsing.md) | [Français](../../fr-FR/engineering/zip-readonly-browsing.md) | [Deutsch](../../de-DE/engineering/zip-readonly-browsing.md) | [Español](../../es-ES/engineering/zip-readonly-browsing.md) | [Português (Brasil)](../../pt-BR/engineering/zip-readonly-browsing.md) | [Русский](../../ru-RU/engineering/zip-readonly-browsing.md)

本文定義根目錄直屬項、後代搜尋、穩定物理 ID 和目錄優先排序。ZIP 瀏覽讀取後設資料，不解碼條目負載或校驗其 CRC，也不寫出解壓後的條目。大型普通 ZIP 可使用私有的[磁碟後設資料索引](large-index.md)。

目前普通 ZIP 的大索引與 Qt 按需模型見[大歸檔索引](large-index.md)。本頁保留早期十萬條索引基線與查詢身份要求，後續擴大量級不取消其錯誤、取消和身份迴歸。

<a name="xdzip-浏览范围与限制"></a>

## 瀏覽範圍與限制

- ZIP／ZIP64 後設資料，最多 100,000 條物理記錄；分卷由[分卷讀取主機](archive-opening.md)轉換後讀取，編解碼器繼續拒絕原始多磁碟後設資料。
- 單名 4 KiB，原始名稱合計 16 MiB，解碼及轉義顯示名合計 16 MiB，讀取合計 64 MiB，掃描緩衝 256 KiB。宣告的 64 位大小保持精確整數。
- 目錄索引最多 32,768 節點、64 層、名稱與字首合計 4 MiB；超限丟棄整個導航索引並解釋降級，不釋出半棵樹。
- 目錄直屬索引與查詢結果複用；同一查詢的傳輸批次不重新過濾或排序；單批名稱另限 64 KiB。介面仍完整顯示目前目錄，沒有分頁按鈕。
- 頁面驗證會話、查詢 ID、GUI 代際、快照修訂和偏移。改變查詢取消舊工作併合並最新意圖，舊響應不能替換新結果。
- 來源的長度、修改時間、Unix 裝置／inode、Windows 卷號／檔案索引在掃描前後、查詢及預檢邊界檢查；異常返回 source_changed，Qt 查詢和預檢都會清除失效列表、選擇、目錄樹與詳情，重新開啟取得新會話。此檢查是盡力檢測，不是檔案系統原子快照或安全寫入授權。

<a name="xdzip-验收退出条件"></a>

## 驗收退出條件

真實生成的十萬條 ZIP64 歸檔完成掃描、全結果排序、檢索、目錄導航、完整 Qt 列表與選擇；所有批次共享查詢身份。驗證損壞 ZIP64、分卷、名稱／顯示／讀取預算、取消、過期響應、原始檔修改與替換。儲存實際耗時、資源、事件佇列和委託數量；按需進行三平臺構建或 Apple Silicon 開發包驗證時分別登記實際結果。

CI 查詢使用倉庫工作流執行列表，按 head_sha 和實際事件篩選；日常檢查隨 push／PR 觸發，Foundation 和 Qt 完整檢查按需手動觸發。具體結果與未執行範圍登記在對應 issue，不以其他事件的空結果推斷檢查狀態。
