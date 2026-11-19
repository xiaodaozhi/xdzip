<a name="xdzip-zip-元数据依赖准入"></a>

# ZIP 後設資料依賴准入

[English](../../engineering/zip-metadata-admission.md) | [简体中文](../../zh-CN/engineering/zip-metadata-admission.md) | 繁體中文（台灣）

本頁保留 2026-10-11 首個只讀目錄掃描的歷史准入，不批准解壓、建立或第三方外掛執行。目前 ZIP 後設資料已遷移至[獨立 Rust 記錄層](independent-rust-codecs.md)，rawzip 僅作為開發測試參照；以下歷史選擇不代表目前生產依賴。

採用 [rawzip 0.5.2](https://github.com/nickbabcock/rawzip)，精確版本和校驗和見[鎖定清單](../../../specs/engineering/zip-metadata-dependencies.json)及根目錄 Cargo.lock。關閉預設特性，顯式啟用 std；無執行時依賴、無 build.rs。上游 MIT 許可全文儲存在 [third_party/rawzip/LICENSE](../../../third_party/rawzip/LICENSE)，隨使用該依賴的發行產物保留。

歷史准入審閱過 README、Cargo 配置和目錄讀取相關原始碼。選擇理由：可對借用的 Read + Seek 輸入逐條讀取中央目錄，保留物理順序和同名條目，宿主可限制讀取位元組及累計名稱大小。只讀掃描選型時評估過 zip 9.0.1，其按名稱建立索引的讀取結構不適合保留物理重複記錄。後續曾另行引入 zip-rs 9.0.3 用於 AES 寫入，未以該寫入依賴替換隻讀物理索引。目前獨立實現繼續保留這些物理索引契約。

RustSec 資料庫快照 `7eebec69c352c7191b1f13eb95dd510eeca5d1de` 中未找到 rawzip 公告。cxx 的 RUSTSEC-2026-0202 影響低於 1.0.195 的 let_cxx_string!；現有鎖定 1.0.205 已包含修復。這裡記錄的是定向公告核對，不是 cargo-audit 全依賴審計。

本後設資料介面卡不啟用解碼器，不驗證檔案內容、CRC 或本地頭與中央目錄的一致性。未知壓縮方法只顯示編號；“能夠列出”不代表能夠解壓。單卷 ZIP64 後設資料已按 M2 增補准入，驗證十萬條記錄與超過 2^53 的精確宣告大小；[分卷主機](archive-opening.md)將受支援卷組轉換為單卷邏輯檢視，介面卡仍拒絕未經轉換的多磁碟記錄；檔名的 Unicode 擴充套件及舊編碼解析按[獨立契約](filename-encoding.md)處理；實際內容解碼按[解壓依賴准入](zip-extraction-admission.md)及[安全解壓契約](zip-safe-extraction.md)獨立處理。限額和失敗行為見[實現契約](readonly-browser.md)。

後設資料迴歸須包含真實 ZIP64、非法 locator／尾記錄、ZIP64 擴充套件起始磁碟、名稱／顯示／讀取預算、取消及源變化用例。獨立實現額外覆蓋自解壓偏移、尾記錄歧義及有限固定種子變異；格式庫退出由生產依賴閉包檢查保證，不以鎖檔案中仍有測試依賴判斷遷移失敗。

M3 負載解碼與宿主輸出另行登記於 [解壓依賴准入](zip-extraction-admission.md)；本頁保留 M2 只讀階段的歷史範圍。
