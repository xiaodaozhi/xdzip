<a name="xdzip-密码与分卷"></a>

# 密碼與分卷

[English](../../user-guide/passwords-and-volumes.md) | [简体中文](../../zh-CN/user-guide/passwords-and-volumes.md) | 繁體中文 | [日本語](../../ja-JP/user-guide/passwords-and-volumes.md) | [한국어](../../ko-KR/user-guide/passwords-and-volumes.md) | [Français](../../fr-FR/user-guide/passwords-and-volumes.md) | [Deutsch](../../de-DE/user-guide/passwords-and-volumes.md) | [Español](../../es-ES/user-guide/passwords-and-volumes.md) | [Português (Brasil)](../../pt-BR/user-guide/passwords-and-volumes.md) | [Русский](../../ru-RU/user-guide/passwords-and-volumes.md)

<a name="xdzip-密码"></a>

## 密碼

AES Crypt 以及啟用了檔名加密的 7z 等歸檔，在顯示檔案列表前需要輸入密碼。其他歸檔可能在解壓、檢查或預覽時才提示。輸入時保留原有空格和大小寫。ZIP 支援符合條件的 AES-256，不支援 AES-128／192 和傳統 ZipCrypto。密碼無誤仍失敗時，請檢查加密方式及檔案是否損壞。[支援的加密方式](../engineering/password-extraction.md)

建立加密包時，選用提供加密選項的格式。ZIP 的 AES-256 會保護檔案內容，檔名仍可見。如果需要一併保護檔名，請選擇 7z 並開啟檔名加密。密碼請與壓縮檔分開儲存。

<a name="xdzip-分卷"></a>

## 分卷

1. 把同一壓縮檔的所有分卷放進一個目錄，保留原始檔名。
2. 開啟支援的分卷，XDZip 會在同目錄查詢其餘部分。編號 ZIP 和 7z、PKZIP，以及受支援的 RAR 和 CAB 卷組使用不同的命名規則。
3. 如果卷組不完整，請補齊缺失檔案再重新開啟。手工拼接或重新命名可能導致壓縮檔無法讀取。

建立分捲包時，選擇 ZIP 或 7z，並在建立視窗設定卷大小。分享時應傳送整組卷檔案。分捲包不能在 XDZip 中直接修改。[卷名規則與支援範圍](../engineering/archive-opening.md)
