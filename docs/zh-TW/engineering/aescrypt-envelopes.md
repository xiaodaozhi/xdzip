<a name="xdzip-aes-crypt-只读封装"></a>

# AES Crypt 只讀封裝

[English](../../engineering/aescrypt-envelopes.md) | [简体中文](../../zh-CN/engineering/aescrypt-envelopes.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/aescrypt-envelopes.md) | [한국어](../../ko-KR/engineering/aescrypt-envelopes.md) | [Français](../../fr-FR/engineering/aescrypt-envelopes.md) | [Deutsch](../../de-DE/engineering/aescrypt-envelopes.md) | [Español](../../es-ES/engineering/aescrypt-envelopes.md) | [Português (Brasil)](../../pt-BR/engineering/aescrypt-envelopes.md) | [Русский](../../ru-RU/engineering/aescrypt-envelopes.md)

獨立 Rust 後端 `xdzip-codec-aescrypt` 可讀取 AES Crypt 串流版本 0、1、2 和 3；應用程式版本號與串流版本分開。每個封裝呈現一個一般虛擬檔案，不提供建立、修改、分割檔案或擷取外部擴充資料的能力。此格式只加密資料而不壓縮，因此項目的方法顯示 Store，並標示為已加密。壓縮大小為完整實體封裝的長度。後端依內容偵測選擇，不依賴 `.aes` 副檔名。

<a name="xdzip-准入与认证"></a>

## 准入與認證

四個版本都必須在開啟時提供密碼。即使是空檔案，也須在顯示精確大小的中繼資料前驗證整段密文。不可變的壓縮檔工作階段透過既有主程式秘密儲存欄位保留受限的開啟憑證，供檢查、預覽與解壓縮使用，不新增逐項密碼機制。工作讀取器再次完成整體驗證後才獲准使用。匯出時會驗證實際解密的同一段密文，只有最終 MAC 與大小檢查成功才算成功。主程式只有在編解碼器成功及來源檢查通過後才發布暫存輸出；未驗證的資料區塊不能授權最終發布。

0–2 版使用指定的 8192 輪 SHA-256 ACKDF，密碼採 UTF-16LE 位元組。0 版直接衍生資料金鑰，無法區分密碼錯誤與密文損壞，回報 `password_or_integrity`。1–2 版先以 HMAC-SHA-256 驗證封裝的工作階段 IV／金鑰，再進行 CBC 解密。驗證失敗回報 `wrong_password`，封裝資料損壞也可能產生此結果。3 版使用 UTF-8 密碼的 PBKDF2-HMAC-SHA-512，驗證封裝的工作階段位元組及版本位元組 `03`，並以 AES-256-CBC 和 PKCS#7 填補處理資料。中繼資料准入與匯出時都先驗證實際密文，再驗證填補；不能以填補作為密碼驗證器。MAC 比較採固定長度，使用已准入的恆定時間基礎元件。

所有版本都驗證密文的 HMAC-SHA-256。但 0–2 版將最終明文長度對 16 的餘數存放在驗證範圍之外；更改此欄位可能改變回報和匯出的長度，卻不會使 MAC 失效。2–3 版的擴充資料也未加密、未驗證。因此 `format_declared_checks` 僅表示格式宣告的所有檢查均已通過，並不驗證每個封裝欄位、傳送者身分或檔案來源。3 版的加密填補可驗證明文長度；舊版檢查成功不能描述為明文長度已獲完整驗證。

<a name="xdzip-名称限制与失败"></a>

## 名稱、限制與失敗

虛擬基底名稱僅從主程式授權的來源基底名稱移除 `.aes` 得到，移除時不區分大小寫。不安全、空、單點或雙點名稱回退為 `content`。未經驗證的 `FILENAME`、`URI`、`CREATED_BY` 和容器擴充資料值不授予路徑、網路、程序或執行權限；不還原時間戳記或執行權限。第一個成員恰好名為 `AES` 的一般 TAR 經受限結構判別後仍為 TAR；擴充資料中含 TAR 標記的有效加密封裝仍為 AES Crypt。一旦接受為 AES Crypt，驗證失敗不會回退至 TAR、ZIP 或其他格式。

1–3 版要求保留位元組為零；擴充記錄須有以 NUL 分隔的識別碼。准入限制為 256 筆擴充記錄、1 MiB 標頭及第 3 版 1 至 2,000,000 次迭代。零次迭代、格式錯誤、截斷或未按區塊對齊的資料會在 KDF 工作前拒絕。未知版本不支援。舊版長度餘數須為 0–15，零長度密文的餘數須為零；3 版至少需要一個填補區塊。不提供損壞資料修復。

讀取器各自限制累計實際輸入、實際雜湊壓縮區塊工作、實際 AES 區塊工作與交付的明文，每項硬上限均為 24 GiB。計量包含中繼資料驗證、KDF 和後續讀取，失敗後不退還預算。呼叫者的中繼資料輸入與名稱限制只能使准入更嚴格。雜湊工作包含 KDF、HMAC 填補與結束處理；CBC 計入所有實際區塊，包括封裝的工作階段資料。額外准入與 KDF 工作在交付資料前計入主程式接收端。一般解碼仍受主程式的來源、輸出與工作預算約束，包括明文每檔 8 GiB、每項工作 20 GiB 上限。通用編解碼器介面最多接受 4096 個 UTF-8 密碼位元組，產品共用憑證上限仍為 1024 位元組。迭代上限是應用程式的資源政策，不是格式最大值。

KDF、每次讀取及每個受限 CBC 批次都會檢查取消。密碼衍生雜湊、中間 KDF 區塊、金鑰、IV 鏈和明文緩衝區使用可擦除狀態；已准入的 AES 基礎元件啟用清零功能。編解碼器保留衍生金鑰，不額外保留明文密碼。協作式取消無法中斷受阻的作業系統讀取。來源、取消、配額和輸出錯誤保留型別；只有單純的中斷讀取會重試。讀取器錯誤具有終止性。主程式的來源識別、大小和時間檢查僅盡力偵測變更，不是原子性來源快照。

<a name="xdzip-资料与回归输入"></a>

## 資料與迴歸輸入

封裝和密碼學規則主要依據作者的 [AES Crypt 串流規範](https://www.aescrypt.com/aes_stream_format.html)及[引擎串流說明](https://github.com/terrapane/aescrypt_engine/blob/master/stream_format.md)。獨立後端使用已准入的通用 AES 與 SHA-2 基礎元件，不引入 AES Crypt 執行階段或官方應用程式，也不執行加密檔案或繞過應用程式授權。共用秘密 SHA-512/HMAC 狀態遵循 RFC 2104 與 [RFC 4231](https://www.rfc-editor.org/rfc/rfc4231)。

[專案自有測試資料集](../../../tests/fixtures/aescrypt/README.md)包含獨立編碼的 0–3 版輸入，並由另一個 Python 密碼學解碼器交叉核對。固定的公開測試金鑰與 IV 只供確定性測試資料使用。參考實作保留於 `build/toolchain/`；產生的證據保留於 `build/` 或 CI。驗收項目為[驗收要求](../../../developers/qa/acceptance.md)中的 `FORMAT-AESCRYPT`。另見[密碼處理](password-extraction.md)、[壓縮檔檢查](archive-checking.md)及[安全解壓縮](zip-safe-extraction.md)。
