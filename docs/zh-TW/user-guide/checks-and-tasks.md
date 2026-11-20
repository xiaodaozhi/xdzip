<a name="xdzip-检查压缩包与管理任务"></a>

# 檢查壓縮檔與管理任務

[English](../../user-guide/checks-and-tasks.md) | [简体中文](../../zh-CN/user-guide/checks-and-tasks.md) | 繁體中文 | [日本語](../../ja-JP/user-guide/checks-and-tasks.md) | [한국어](../../ko-KR/user-guide/checks-and-tasks.md) | [Français](../../fr-FR/user-guide/checks-and-tasks.md) | [Deutsch](../../de-DE/user-guide/checks-and-tasks.md) | [Español](../../es-ES/user-guide/checks-and-tasks.md) | [Português (Brasil)](../../pt-BR/user-guide/checks-and-tasks.md) | [Русский](../../ru-RU/user-guide/checks-and-tasks.md)

<a name="xdzip-测试压缩包"></a>

## 測試壓縮檔

1. 開啟壓縮檔，在工具欄選擇“測試”。
2. 如有密碼提示，按需輸入後等待結果。檢查讀取符合條件的內容，不寫出解壓檔案。
3. 若檢查失敗，檢視顯示的原因和相關條目。部分格式支援校驗和或認證，另一些只檢查結構和可讀性。透過結果僅覆蓋該格式能驗證的內容。[完整性檢查規則](../engineering/archive-checking.md)

有些壓縮檔無法測試。例如，使用磁碟索引的 ZIP 仍可在任務限額內解壓所選普通檔案，NSIS 只能檢視安裝器後設資料。詳見[大歸檔限制](../engineering/large-index.md)與[格式支援](../../../README.zh-TW.md#xdzip-%E6%A0%BC%E5%BC%8F%E6%94%AF%E6%8C%81)。

<a name="xdzip-进度取消与结果"></a>

## 進度、取消與結果

透過任務選單可以回到正在執行的操作或檢視結果。關閉壓縮檔標籤頁不會終止背景任務。出現密碼或重名視窗時，操作可能需要你的決定才能繼續。

在任務視窗選擇“取消”以請求停止，並等待最終狀態。已儲存檔案可能保留。展開結果詳情，區分已完成、已跳過和未完成的條目，再結合數量和原因決定重試範圍。清除已完成的歷史記錄只移除對應任務入口，磁碟中的檔案會保留。[任務視窗說明](../ux/task-windows.md)
