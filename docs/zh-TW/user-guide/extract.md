<a name="xdzip-解压全部或所选文件"></a>

# 解壓全部或所選檔案

[English](../../user-guide/extract.md) | [简体中文](../../zh-CN/user-guide/extract.md) | 繁體中文（台灣） | [日本語](../../ja-JP/user-guide/extract.md) | [한국어](../../ko-KR/user-guide/extract.md) | [Français](../../fr-FR/user-guide/extract.md) | [Deutsch](../../de-DE/user-guide/extract.md) | [Español](../../es-ES/user-guide/extract.md) | [Português (Brasil)](../../pt-BR/user-guide/extract.md) | [Русский](../../ru-RU/user-guide/extract.md)

<a name="xdzip-从归档窗口解压"></a>

## 從歸檔視窗解壓

1. 開啟壓縮檔。已有選擇時，工具欄主按鈕解壓所選條目。沒有選擇時，主按鈕解壓整個壓縮檔。也可用按鈕旁的箭頭明確選擇“全部解壓”或“解壓所選”。
2. 核對解壓範圍並選擇可寫的目標目錄。需要以歸檔名稱新建一層目錄時，可在提供此選項的介面中勾選。
3. 選擇重名策略。“詢問”會暫停等待決定，“跳過”保留已有檔案，“重新命名”另存一份，“覆蓋”在校驗後替換符合條件的檔案。
4. 開始解壓並檢視任務結果。失敗或取消之前已經儲存的檔案可能保留。重試前根據結果詳情確認剩餘項。[檢查、任務與結果](checks-and-tasks.md)

選中最多 128 個實際歸檔條目時，解壓選單還提供“解壓後設資料預檢”，可提示危險名稱、不支援的方法和宣告資料超限等問題。預檢請選擇歸檔中有實際記錄的條目。僅為導航顯示的隱式目錄仍可按格式能力參與解壓。預檢不讀取負載，也不授予寫入許可權。[預檢與解壓限制](../engineering/extraction-preflight.md)

對使用磁碟索引的大型 ZIP，可以在任務限額內解壓所選普通檔案。整包超出限額時，請縮小選擇範圍。如果來源發生變化或被替換，請重新開啟並選擇所需條目。[大歸檔限制](../engineering/large-index.md)

<a name="xdzip-从文件管理器解压"></a>

## 從檔案管理器解壓

Windows 11 的經典選單位於“顯示更多選項”，商店版也可在現代選單顯示。Ubuntu 使用 Nautilus 擴充套件。macOS 需要在系統設定中啟用 Finder 擴充套件，也可透過已登記的 Finder 服務操作。找不到這些入口時，可在 XDZip 中開啟壓縮檔再解壓。

檔案管理器選單可解壓到各壓縮檔所在目錄、為每個壓縮檔新建目錄、使用智慧目錄，或讓你選擇目標位置。校驗後的輸出恰好有一個頂層目錄時，智慧解壓會省略外層目錄。批次操作會逐包顯示結果，單包失敗後繼續處理其餘壓縮檔。取消會停止後續子項，並向正在準備或執行的任務發出取消請求。[檔案管理器操作與設定](../engineering/shell-actions.md)
