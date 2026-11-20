<a name="xdzip-asar-容器契约"></a>

# ASAR 容器契約

[English](../../engineering/asar-containers.md) | [简体中文](../../zh-CN/engineering/asar-containers.md) | 繁體中文 | [日本語](../../ja-JP/engineering/asar-containers.md) | [한국어](../../ko-KR/engineering/asar-containers.md) | [Français](../../fr-FR/engineering/asar-containers.md) | [Deutsch](../../de-DE/engineering/asar-containers.md) | [Español](../../es-ES/engineering/asar-containers.md) | [Português (Brasil)](../../pt-BR/engineering/asar-containers.md) | [Русский](../../ru-RU/engineering/asar-containers.md)

格式依據 Electron 的 [ASAR 說明](https://github.com/electron/asar#format)和 Chromium 的 [Pickle 契約](https://chromium.googlesource.com/chromium/src/+/main/base/pickle.h)。八位元組尺寸 Pickle 後跟包含 JSON 目錄樹的字串 Pickle，檔案偏移以十進位制字串表示，相對於載荷區起點。即使多個檔案共享同一段儲存位元組，每個樹條目仍保留獨立身份。可選完整性宣告包含整個檔案及各固定尺寸塊的 SHA-256，這些是損壞檢查，不代表軟體包或應用認證。

<a name="xdzip-准入与身份"></a>

## 准入與身份

解析目錄樹前驗證四個小端長度欄位、四位元組對齊、精確字串長度、零填充及源邊界。頭部上限為 8 MiB，路徑深度最多 64 個元件，節點數和名稱分配受共同掃描上限限制。通用 JSON 解析必須有界，拒絕重複鍵並保留物件順序。拒絕非法 UTF-8、欄位型別、非整數或不安全尺寸、十進位制偏移溢位及未知必需配置。名稱元件不能透過拼接字首變得安全，原本不安全的名稱保留可見警告和不支援標誌，由宿主拒絕輸出。

按 JSON 樹順序分配物理身份，包括顯式目錄和空檔案，根物件僅為結構。精確共享載荷區間的檔案保留各自身份，拒絕部分重疊、授權源之外的範圍及未引用尾部資料。無完整性宣告的檔案只提供結構和可讀性檢查；存在宣告時須完整校驗摘要長度、演算法、塊尺寸與數量及空檔案規則，不忽略不支援的宣告。

<a name="xdzip-只读导出与复核"></a>

## 只讀匯出與複核

後端只接收已授權可定位流和宿主輸出介面，不開啟其他路徑、不跟隨連結、不執行應用。`unpacked` 條目可見但不支援輸出，相鄰 `.asar.unpacked` 目錄不構成輸入授權。宣告的連結保留身份，但不支援輸出。可執行標誌不授予許可權恢復或執行能力。本配置不提供密碼、分卷、建立或重寫。

載荷使用前重新解析完整頭部，將投影索引與不可變宿主快照比較，並私下保留有界偏移與可選摘要。累計限制實際輸入與解碼位元組，分塊讀取儲存區間並檢查取消。報告載荷成功前驗證整檔案和塊 SHA-256，包括空檔案。校驗期間內容只能進入私有暫存，最終釋出由宿主控制。SHA-256 按實際 64 位元組壓縮塊計量演算法工作，包括填充。輸入、已交付位元組和雜湊工作分別受 24 GiB 私有上限限制，宿主任務上限仍可更嚴格；共享區間重複讀取不返還預算。讀取器發生錯誤後不可繼續。源身份檢查仍是盡力變化檢測，不構成原子或已認證快照。

登記前須以固定作者工具生成或獨立讀取自有樣本，覆蓋目錄、空檔案、UTF-8、共享載荷、無摘要和多塊摘要、外接記錄及可執行宣告。迴歸覆蓋重複 JSON 鍵、算術、對齊、截斷、重疊、錯誤摘要、不安全名稱、源變化、精確配額、取消及輸出失敗。實際 runtime、CLI、Qt 任務和受影響的候選、只讀邊界、檢查說明與佈局採用自動化驗證。參考工具和生成證據留在 `build/`，測試輸入及生成器原始碼納入版本管理，進度與實際驗收登記在交付 issue。
