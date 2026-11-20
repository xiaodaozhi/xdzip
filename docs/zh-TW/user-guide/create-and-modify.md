<a name="xdzip-创建与修改压缩包"></a>

# 建立與修改壓縮檔

[English](../../user-guide/create-and-modify.md) | [简体中文](../../zh-CN/user-guide/create-and-modify.md) | 繁體中文 | [日本語](../../ja-JP/user-guide/create-and-modify.md) | [한국어](../../ko-KR/user-guide/create-and-modify.md) | [Français](../../fr-FR/user-guide/create-and-modify.md) | [Deutsch](../../de-DE/user-guide/create-and-modify.md) | [Español](../../es-ES/user-guide/create-and-modify.md) | [Português (Brasil)](../../pt-BR/user-guide/create-and-modify.md) | [Русский](../../ru-RU/user-guide/create-and-modify.md)

<a name="xdzip-创建压缩包"></a>

## 建立壓縮檔

1. 從起始頁或“新建”開啟建立視窗，新增檔案或資料夾，並核對輸入列表。
2. 設定輸出名稱和位置，選擇格式及可用的壓縮選項。分享檔案時可以優先考慮 ZIP。建立能力見[格式支援表](../../../README.zh-TW.md#xdzip-%E6%A0%BC%E5%BC%8F%E6%94%AF%E6%8C%81)。GZIP 等單檔案壓縮流一次接收一個普通檔案。
3. 所選配置允許時，ZIP 和 7z 可加密或分卷。資料光碟映像請選擇 ISO，自解壓 ZIP 請選擇 Windows x64 的自解壓配置。開始建立後，在任務結果中確認實際儲存的檔案。[密碼與分卷](passwords-and-volumes.md)
4. 取消或重試前先看結果。建立過程使用暫存區，完成校驗後才儲存成品。

<a name="xdzip-添加替换或移动文件"></a>

## 新增、替換或移動檔案

開啟符合條件的 ZIP/ZIPX、7z、LHA/LZH、TAR 或含外層壓縮的 TAR 壓縮檔，進入包內目標目錄。把本機檔案或資料夾拖到列表或網格中，核對內容後確認新增。重名時可選擇替換符合條件的包內普通檔案。包內移動可將條目拖到同一歸檔的其他目錄後確認。拖到應用外則匯出副本。[各平臺的拖放行為](../engineering/archive-transfers.md)

建立和新增會排除 `.DS_Store` 檔案以及 `__MACOSX`、`_MACOSX` 目錄。修改能力取決於格式、內容和輸入方式。加密歸檔、分卷歸檔以及使用磁碟索引的歸檔無法透過普通流程修改。無法編輯時，可解壓到新目錄，再建立新壓縮檔。[歸檔修改與拖放規則](../engineering/archive-transfers.md)

<a name="xdzip-rename-or-delete-entries"></a>

## 重新命名或刪除項目

在可編輯壓縮檔的資料夾中，以滑鼠右鍵點選清單或格狀檢視中的檔案或資料夾。點選已選取的項目會保留原有選取範圍，點選未選取的項目時則只選取該項目。

- **重新命名**只能用於單一項目。請在同一資料夾內輸入新名稱。重新命名資料夾時，其中所有項目的路徑都會更新。若新名稱已存在，請改用其他名稱。
- **刪除**可用於一個或多個選取項目。確認前請核對項目名稱或選取數量。刪除資料夾時也會刪除其中所有內容。刪除壓縮檔根目錄中的所有項目後，仍會保留一個空的壓縮檔。如果使用一般壓縮副檔名會讓空壓縮檔被辨識成單檔串流，請改用 `.tar.xz` 等複合 TAR 副檔名。

若要編輯搜尋結果中的項目，請先開啟它所在的資料夾。[壓縮檔修改規則](../engineering/archive-transfers.md)
