<a name="xdzip-文件名编码自适应契约"></a>

# 檔名編碼自適應契約

[English](../../engineering/filename-encoding.md) | [简体中文](../../zh-CN/engineering/filename-encoding.md) | 繁體中文 | [日本語](../../ja-JP/engineering/filename-encoding.md) | [한국어](../../ko-KR/engineering/filename-encoding.md) | [Français](../../fr-FR/engineering/filename-encoding.md) | [Deutsch](../../de-DE/engineering/filename-encoding.md) | [Español](../../es-ES/engineering/filename-encoding.md) | [Português (Brasil)](../../pt-BR/engineering/filename-encoding.md) | [Русский](../../ru-RU/engineering/filename-encoding.md)

<a name="xdzip-解析与身份"></a>

## 解析與身份

解碼在不依賴 Qt 的 Rust 格式層完成，不使用宿主系統內碼表、介面語言、來源系統猜測、檔案內容或網路服務。ZIP 的 raw_name 保留名稱欄位原始位元組；TAR 保留 PAX/GNU 選擇並按既有規則去掉前置 ./ 的原始名稱。物理 EntryId 與名稱無關。decoded_name 儲存需要轉碼或補充編碼宣告的 UTF-8 拼寫，name_encoding 保留解析來源，name_bytes 提供凍結的有效名稱；display_name 只用於轉義展示，不能作為輸出路徑。

ZIP 按以下順序解析：

1. UTF-8 標誌置位時要求原欄位為有效 UTF-8，損壞宣告不改用舊編碼修復。
2. 未置位時讀取 Info-ZIP Unicode Path 0x7075，接受版本 1、原名稱 CRC32 匹配、完整有效 UTF-8 的值。未知版本和 CRC 不匹配按規範忽略；已匹配但無效的 UTF-8、重複有效欄位或損壞欄位結構標記需檢查。
3. 沒有可用的擴充欄位，且整組未宣告名稱均為有效 UTF-8 時，接受未標記的 UTF-8。
4. 否則整組未宣告非 ASCII 名稱使用全包樣本的統計檢測，覆蓋 GBK/GB18030、Big5、Shift-JIS、EUC-JP、韓文及其他檢測器支援的字符集。ZIP 的西歐預設猜測按格式歷史預設回退 CP437。檢測到 GBK 時使用相容的 GB18030 嚴格解碼器。

TAR 普通頭、GNU 長名稱及顯式 hdrcharset=BINARY 的 PAX path 使用相同的無宣告名稱策略。普通 PAX path 要求有效 UTF-8，不用檢測掩蓋損壞宣告；UTF-8 和 BINARY 以外的顯式 hdrcharset 不支援。TAR 不套用 ZIP 的 CP437 預設。TAR.GZ 與 TAR 使用同一名稱解析。

規則依據為 [ZIP APPNOTE 6.3.10](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT)、[POSIX pax](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/pax.html)及 [WHATWG 編碼標準](https://encoding.spec.whatwg.org/)。本機檔頭可以沒有 Unicode Path 欄位，但若提供有效值，必須與凍結的有效名稱一致；載荷仍校驗原始名稱、本地/中央頭、CRC、大小、密碼認證和流結束。

CAB 保留原始名稱位元組和 UTF-8 屬性。顯式 UTF-8 嚴格驗證，其餘名稱複用整包有界檢測，不使用系統內碼表。CAB 的反斜槓是格式目錄分隔符，但僅在完整解碼後規範化為正斜槓，不能誤拆 Shift-JIS 多位元組尾位元組 0x5C。原始位元組不變，有效路徑仍經過相同安全與碰撞檢查。ISO 非 Joliet 名稱保留原始位元組並使用相同有界解析，避免 Rock Ridge 名稱經過宿主 OEM 轉碼。Joliet/UDF 使用處理器提供的 Unicode 名稱並嚴格驗證，不將有損替換字元視為原始名稱。

<a name="xdzip-检测边界与资源"></a>

## 檢測邊界與資源

檢測器最多讀取 1 MiB 完整的未宣告、非 UTF-8 名稱，以換行隔開記錄，避免截斷多位元組序列跨名稱拼接。不將明確 Unicode 宣告或無效擴充欄位作為檢測樣本。只要樣本含非 UTF-8 名稱，就對未宣告名稱統一採用歸檔級結果，避免 GBK 中恰好合法的 UTF-8 片段把同一個目錄拆成兩個名字。單條嚴格解碼失敗時才嘗試合法 UTF-8 或該名稱的個別偵測結果。禁用 ISO-2022-JP 猜測，不解釋名稱中的轉義狀態。嚴格解碼不插入替代字元、不刪除 BOM，也不轉換斜槓、規範化 Unicode 或修補路徑。

原始名稱合計最多 16 MiB，單個原始/解碼名稱最多 4096 位元組，解碼名稱與轉義展示合計最多 16 MiB。ZIP Unicode 擴充欄位的每條及累計分配在掃描時檢查，不延遲到整包結束。解析、檢測與逐條轉換檢查取消。ZIP 載荷額外保留最多 16 MiB 的有效名稱校驗表，後續 decode 不能更換凍結的名稱。

統計檢測不能唯一恢復任意短名稱或混合舊編碼，也不能判斷已被其他軟體儲存為有效 Unicode 的亂碼文字。一個包內明確宣告的 Unicode 與舊編碼可共存；多個未宣告編碼只能盡力檢測。目前沒有手動編碼切換入口。無法嚴格解碼、宣告損壞或路徑異常時繼續轉義顯示並標記需檢查，不授予解壓、臨時開啟、匯出或修改能力。不能承諾所有歷史包均正確識別。歸檔存在未解析名稱時，其真實目錄歸屬可能未知，因此拒絕目錄展開解壓，避免成功結果靜默漏項；明確選擇的已解析普通檔案仍可單獨解壓。

<a name="xdzip-全链路与保存"></a>

## 全鏈路與儲存

目錄樹、直屬索引、遞迴搜尋、目錄選擇展開、預檢、解壓、臨時開啟、對外匯出及包內修改使用同一個有效名稱。完成解碼後仍執行 PortablePath、Unicode NFD/大小寫別名、重名與輸出授權檢查。Shift-JIS 中作為多位元組字元尾位元組的 0x5C 不當作目錄分隔符，除 CAB 和 LHA 的格式分隔符外，解碼後真正的反斜槓仍拒絕。

普通 ZIP、TAR、TAR.GZ 新增和移動在規劃結束後將舊編碼/Unicode 擴充欄位/未標記 UTF-8 名稱重新宣告為 UTF-8，保留邏輯移動規劃中的原位置與跳過含義。ZIP 保持壓縮載荷，TAR 使用明確的 UTF-8 PAX path；寫後重新掃描、逐項驗證名稱與內容，最後執行原有同卷替換。TAR 改寫 BINARY PAX path 時移除該路徑的 BINARY 宣告；如果這會改變其他非 UTF-8 PAX 文字的解釋，則明確失敗並保留原包。加密和分卷的讀取、解壓沿用相同名稱，新增/移動限制不變。

<a name="xdzip-可重复验收"></a>

## 可重複驗收

使用自有原始位元組夾具及 Python 標準庫獨立編碼的歸檔，覆蓋未標記 UTF-8、GBK、四位元組 GB18030、Big5、含 0x5C 的 Shift-JIS、韓文、俄文、ZIP CP437 和 Unicode Path。驗證完整名稱、原位元組、目錄/搜尋/選擇、預檢、實際輸出位元組、臨時開啟、匯出、普通包新增/移動、AES 和分卷；同時檢查宣告損壞、擴充欄位優先順序、路徑拒絕、Unicode/大小寫碰撞、取消和預算。

scripts/check-filename-encoding.py 透過實際服務探針解壓 Python 生成的 ZIP/TAR/TAR.GZ，另以 Python 獨立讀取 Rust 改寫的歸檔，檢查 UTF-8 標誌/PAX、名稱和內容。測試資料只來自原始碼常量，不使用使用者包。具體結果與剩餘人工驗收記錄於 issue 和 build/CI 工件，不在本契約登記透過狀態。

LHA 保留目錄與檔名的原始欄位邊界和目錄內原始 `0xff` 分隔符，共享有界檢測分別取樣各完整欄位。逐分量嚴格解碼後，才解釋解碼所得的反斜槓並組合 `/`，原始位元組邊界禁止跨欄位借用多位元組字元。目錄展開與輸出使用同一有效名稱，詳見 [LHA 契約](lha-containers.md)。
