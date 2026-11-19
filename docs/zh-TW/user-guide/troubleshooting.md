<a name="xdzip-xdzip-常见问题与排错"></a>

# XDZip 常見問題與排錯

[English](../../user-guide/troubleshooting.md) | [简体中文](../../zh-CN/user-guide/troubleshooting.md) | 繁體中文（台灣）

| 遇到的問題 | 可以怎麼處理 |
| --- | --- |
| 壓縮檔無法開啟 | 核對[格式支援](../../../README.zh-TW.md#xdzip-%E6%A0%BC%E5%BC%8F%E6%94%AF%E6%8C%81)、壓縮或加密方式及完整卷組。若來源不可訪問，請保留原名，將歸檔及所有分卷複製到可訪問的本地目錄後重新開啟。 |
| 檔名亂碼或無法寫出 | 確認能否用 UTF-8 檔名重新打包。XDZip 可嘗試識別部分舊編碼，短檔名或混合編碼有一定限制。只提取可以安全對映的條目。[檔名處理](../engineering/filename-encoding.md) |
| 屬性可見，檔案無法預覽 | 核對檔案型別、尺寸、平臺和編碼。預覽與提取有各自的限制。[預覽限制](../engineering/content-preview.md) |
| 輸出目標被拒絕或任務只完成一部分 | 改選可寫的私有目錄，檢查重名與不支援的條目，重試前核對任務結果。[解壓檔案](extract.md) |
| 操作期間原始檔被替換 | 開啟新的副本並重新選擇條目。XDZip 在寫入前會重新檢查來源。 |
| Explorer、Finder 或 Files 沒有出現操作入口 | 核對安裝渠道及擴充套件是否已啟用。Windows 11 的經典選單位於“顯示更多選項”。macOS 可在系統設定中啟用 Finder 擴充套件，或使用已登記的服務。也可直接在 XDZip 中開啟歸檔。[從檔案管理器解壓](extract.md#xdzip-%E4%BB%8E%E6%96%87%E4%BB%B6%E7%AE%A1%E7%90%86%E5%99%A8%E8%A7%A3%E5%8E%8B) |
| 大型 ZIP 無法測試或修改 | 如果 XDZip 正在使用磁碟索引，可只選擇需要的普通檔案，在任務限額內解壓。[大歸檔限制](../engineering/large-index.md) |
| 更新或安裝失敗 | 核對目前版本和發行渠道。手動下載時先校驗檔案，再按平臺安裝步驟處理。[更新恢復](settings-and-updates.md) |

意見回饋問題時，請提供“關於 xdzip”中的版本和構建號、系統與架構、壓縮格式和操作、復現步驟，以及預期和實際結果。儘量使用自制小樣本，分享前移除密碼和私人路徑。[意見回饋問題](https://github.com/xiaodaozhi/xdzip/issues/new/choose)
