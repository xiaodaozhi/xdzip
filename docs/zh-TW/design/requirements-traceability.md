<a name="xdzip-需求与验证追踪"></a>

# 需求與驗證追蹤

[English](../../design/requirements-traceability.md) | [简体中文](../../zh-CN/design/requirements-traceability.md) | 繁體中文（台灣）

[產品要求](product-requirements.md)定義需求編號及目前範圍。下表說明對應行為改動時應驗證什麼。執行結果、平臺範圍和限制記錄在對應變更中。[驗收用例](../../../developers/qa/acceptance.md)提供檢查入口。

<a name="xdzip-需求矩阵"></a>

## 需求矩陣

| 需求 | 設計參考 | 驗證重點 |
| --- | --- | --- |
| P01 瀏覽時不解壓 | [領域](domain-data.md)、[桌面](qt-quick-ui.md) | T01 開啟真實歸檔，核對瀏覽過程沒有解壓內容。 |
| P02 瀏覽、查詢與選擇 | [領域](domain-data.md)、[效能](performance.md) | T02 同名身份、穩定查詢、過期分頁、大索引儲存與跨頁選區。 |
| P03 解壓預期範圍 | [檔案授權](file-security.md)、[任務](task-scheduling.md) | T03 目錄展開、同名記錄、重新命名、跳過、覆蓋及實際磁碟結果。 |
| P04 建立歸檔 | [格式](codec-extension-sdk.md)、[檔案授權](file-security.md) | T04 受影響配置的獨立往返、暫存驗證和既有檔案保護。 |
| P05 遵守格式能力 | [橋接](qt-rust-bridge.md)、[格式](codec-extension-sdk.md) | T05 登錄檔、會話操作及執行時拒絕不支援的配置。 |
| P06 限制任務並支援取消 | [橋接](qt-rust-bridge.md)、[任務](task-scheduling.md) | T06 佇列飽和、提示路由、取消與提交競態及非同步關閉。 |
| P07 保護檔案與資源邊界 | [檔案授權](file-security.md)、[格式](codec-extension-sdk.md) | T07 適用的原生連結與身份核對、路徑校驗、配額和受保護物件。 |
| P08 提供可訪問的桌面 | [桌面](qt-quick-ui.md) | T08 模型和控制元件，以及受影響平臺的原生鍵盤、輸入法、輔助功能、縮放及焦點。 |
| P09 限制並測量資源 | [效能](performance.md) | T09 可比的構建、程序樹指標，按需核對啟動、排序、寫出吞吐或包大小。 |
| P10 將 Qt 與格式邏輯分開 | [架構](system-architecture.md)、[格式](codec-extension-sdk.md) | T10 無需 Qt 的核心、顯式路由及格式專用迴歸。 |
| P11 保護密碼及資料 | [檔案授權](file-security.md)、[隱私](../engineering/privacy.md) | T11 有作用範圍的密碼、脫敏診斷及明確的網路訪問。 |
| P12 審計交付 | [質量](quality-engineering.md)、[發行](build-release.md) | T12 鎖定輸入和工件身份、元件清單、許可材料、包信任及適用安裝檢查。 |
| P13 控制外部擴充套件准入 | [格式](codec-extension-sdk.md)、[SDK 試驗](../engineering/sdk-trial.md) | T13 獨立試驗的互操作和原生許可權結果。產品內安裝擴充套件仍是設計選項。 |
| P14 限制預覽與開啟 | [桌面](qt-quick-ui.md)、[檔案授權](file-security.md) | T14 顯式預覽、受支援內容、工作程序限額和私有臨時輸出。 |
| P15 報告實際結果 | [橋接](qt-rust-bridge.md)、[檔案授權](file-security.md)、[任務](task-scheduling.md) | T15 來源或目標變化、部分提交後的失敗與取消、清理及不確定結果。 |

<a name="xdzip-横向验证"></a>

## 橫向驗證

| 用例 | 適用改動 | 驗證重點 |
| --- | --- | --- |
| T16 橋接所有權與生命週期 | 橋接、連線或控制器生命週期 | 目前請求身份、接收執行緒喚醒、保留任務和安全銷燬。 |
| T17 格式互操作與失敗路徑 | 格式或編解碼修改 | 適用時使用獨立的有效樣本，覆蓋截斷或不支援的配置及輸出限額。 |
| T18 安裝、更新與移除 | 平臺或發行渠道改動 | 對應安裝包、乾淨環境的執行依賴、安裝身份、更新交接及支援的移除或恢復。 |

<a name="xdzip-代表性工作流程"></a>

## 代表性工作流程

1. 開啟 ZIP 並搜尋，凍結選中的結果，解壓後比較輸出位元組。“解壓全部”仍指整個歸檔。
2. 按指定重名策略解壓同名或衝突記錄，核對最終名稱與結果。
3. 在釋出前及釋出期間請求取消，核對已完成檔案仍有記錄，並確認建立在提交前失敗時保留原歸檔。
4. 在自有測試中改動來源或目標並驗證實際結果。異常終止後檢查遺留物件，恢復過程中無自動重放。
5. 依據各自契約驗證隨包格式工作程序或獨立 SDK 試驗異常時的處理，確認暫存內容不會以成功結果釋出。

靜態寫作、格式和連結檢查驗證文件。夾具、離屏、原生自動化與人工驗收分別保留各自的證據範圍。[質量工程](quality-engineering.md)、[驗證指南](../../../developers/qa/README.md)
