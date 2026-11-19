<a name="xdzip-浏览选择与预览文件"></a>

# 瀏覽、選擇與預覽檔案

[English](../../user-guide/browse.md) | [简体中文](../../zh-CN/user-guide/browse.md) | 繁體中文（台灣） | [日本語](../../ja-JP/user-guide/browse.md) | [한국어](../../ko-KR/user-guide/browse.md) | [Français](../../fr-FR/user-guide/browse.md) | [Deutsch](../../de-DE/user-guide/browse.md) | [Español](../../es-ES/user-guide/browse.md) | [Português (Brasil)](../../pt-BR/user-guide/browse.md) | [Русский](../../ru-RU/user-guide/browse.md)

1. 開啟壓縮檔。在目錄樹中選擇資料夾，或雙擊列表中的資料夾進入。用麵包屑路徑返回上一級。
2. 輸入檔名的一部分並按回車。搜尋範圍是目前目錄及其後代。可以切換列表或網格檢視並選擇排序方式，升序和降序都將目錄排在檔案之前。
3. 用核取方塊或平臺支援的選擇快捷鍵勾選條目。“全選”作用於目前查詢，包括檢視中尚未顯示的行。解壓前先核對已選項數量。
4. 開啟檔案屬性，主動請求預覽普通檔案。文字須為 UTF-8 或帶 BOM 的 UTF-16，最大為 256 KiB。圖片有單獨的大小限制。靜態 SVG 須符合支援範圍，HEIC／HEIF 只在 macOS 可預覽。若無法預覽且該條目允許解壓，仍可將檔案解壓出來。[預覽格式和限制](../engineering/content-preview.md)

雙擊普通檔案時，XDZip 會解壓臨時副本，再請系統開啟。修改副本不會寫回壓縮檔。要長期保留檔案，請解壓到自己選擇的位置。[臨時開啟說明](../engineering/archive-opening.md)

預設在標籤頁開啟壓縮檔，也可改為獨立視窗。普通單檔案 ZIP／ZIP64 可索引最多一百萬條物理記錄。大索引按需載入列表行。使用磁碟索引時仍可解壓所選的普通檔案，整包測試和修改不可用。[大歸檔限制](../engineering/large-index.md)
