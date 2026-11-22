<a name="xdzip-zip-解压与输出依赖准入"></a>
<a name="zip-解壓與輸出依賴准入"></a>

# ZIP 解壓縮與輸出相依套件的採用條件

[English](../../engineering/zip-extraction-admission.md) | [简体中文](../../zh-CN/engineering/zip-extraction-admission.md) | 繁體中文 | [日本語](../../ja-JP/engineering/zip-extraction-admission.md) | [한국어](../../ko-KR/engineering/zip-extraction-admission.md) | [Français](../../fr-FR/engineering/zip-extraction-admission.md) | [Deutsch](../../de-DE/engineering/zip-extraction-admission.md) | [Español](../../es-ES/engineering/zip-extraction-admission.md) | [Português (Brasil)](../../pt-BR/engineering/zip-extraction-admission.md) | [Русский](../../ru-RU/engineering/zip-extraction-admission.md)

本頁定義安全解壓縮與密碼解壓縮共用的相依套件邊界。最初採用時的套件來源、授權及功能摘錄見[固定版本的基礎資料](../../../specs/engineering/zip-extraction-dependencies.json)。目前的確切版本，包括 AES，以及 crates.io 檢查碼以根目錄 Cargo.lock 為準；使用 `cargo tree --locked -e features` 檢查完整啟用的相依關係。基礎摘錄不能視為目前完整的相依套件清單。

| 相依套件 | 用途和決定 |
| --- | --- |
| xdzip-compression | 專案自有的共用 Deflate 和 CRC-32 實作，使用固定緩衝區，嚴格檢查最終區塊及壓縮位元組的精確消耗量。獨立格式層處理 ZIP 容器、描述項和 AES 封裝，借用主程式的輸入與輸出接收端，維持輸出授權和完整驗證邊界。flate2 僅供開發測試參考。 |
| cap-std / cap-fs-ext 4.0.3 | 持有目錄能力，以 nofollow 開啟子目錄、獨占暫存並以相對路徑發布。不同於串接字串並在 canonicalize 後建立檔案，主程式會驗證各路徑元件的授權並重新確認識別；不呼叫通用 unpack。cap-primitives 等間接相依套件也固定版本。 |
| getrandom 0.4.2 | 使用作業系統亂數產生私人工作區 nonce，絕不使用時間戳記、偽隨機值或壓縮檔名稱。停用預設功能。 |
| rustix 1.1.6 | 提供 POSIX 有效使用者身分與檔案系統基礎操作，沿用固定版本。停用預設功能，只啟用 std/process 和間接需要的 fs。 |
| windows-sys 0.61.2 | 查詢 Windows 物件擁有者及 DACL，並檢查 SID。手寫 unsafe 程式碼經隔離，借用有效控制代碼、DWORD 對齊的緩衝區，並以 RAII 呼叫 LocalFree。不同於只檢查唯讀屬性，DACL 檢查可確認私人輸出的前提。新增 FFI 必須驗證控制代碼、緩衝區、釋放前提及查詢失敗時的行為。 |
| unicode-normalization 0.1.25 | 為工作名稱別名產生 NFD 加小寫的採用判斷鍵，保留原本的輸出名稱；此鍵不視為完整的檔案系統大小寫折疊。相較於僅用 ASCII 小寫，它涵蓋 macOS 常見的 Unicode 等價名稱。 |
| AES、HMAC、SHA-1、PBKDF2、constant_time_eq | 已採用的通用密碼學基礎元件；確切版本和功能見[相依套件契約](dependencies.md)。ZIP CTR 計數、鹽值和驗證值的配置、密文驗證及錯誤分類由專案獨立實作。zip 9.0.5 僅供開發測試參考。 |
| zeroize 1.9.1 | 管理密碼、衍生金鑰及測試探針受限 stdin 緩衝區的清零生命週期；AES/HMAC/SHA-1 也啟用各自的清理功能。不保證擦除 Qt、輸入法或所有歷史記憶體副本。 |

通用相依套件須核對 Cargo 設定、授權與公開 API；獨立格式實作的來源規則見[獨立實作契約](independent-rust-codecs.md)。cap 系列由 Bytecode Alliance 維護，getrandom 和 rustix 提供系統介面，Unicode normalization 是通用 Unicode 函式庫。升級仍須固定版本並重跑原生邊界迴歸測試，維護者身分不能取代迴歸測試或安全公告檢查。新增套件在執行時不發出網路請求；含建置指令碼的套件僅探測編譯器與目標設定，或選擇 Rust 系統呼叫後端，不下載外部程式碼，也不編譯 C 解碼器。授權通知收集完整的上游原文，實際授權表示式以固定資料為準。

依[驗證方法](../../../developers/qa/README.md)保留 OSV 確切版本查詢的實際回應；查詢失敗不能解讀為沒有安全公告。這不等同完整的 cargo-audit。既有 cxx 公告與版本選擇仍記錄於 [M2 相依套件紀錄](dependencies.md)。

中繼資料通過檢查不代表自動允許解碼。獨立配接器會驗證本地標頭、串流結束、CRC 和 32/64 位元資料描述項，並依中央與本地 ZIP64 哨兵值選擇寬度。獨立 Python force_zip64 測試資料涵蓋有效的混合寬度及遭竄改的大小。rawzip、zip、flate2 均不在 ZIP 和主程式的一般或建置相依套件閉包中；歷史固定版本摘錄只保留當時的來源紀錄。

固定資料列出跨目標解析的所有 Cargo 相依套件。getrandom 的 WASI/WIT/WebAssembly 工具鏈只適用於不支援的目標，不會進入 macOS、Windows 或 Linux 桌面版的編譯或執行環境。Cargo 全目標清單不能解讀為桌面版執行時的相依範圍。

開發套件的授權通知依 `cargo metadata --filter-platform <rustc host>` 的實際解析節點收集目前平台工作區與建置工具的授權，並記錄 target.json。未編譯的 WASI/UEFI 套件不視為桌面版發行相依套件。缺少授權全文仍會導致打包失敗，不能以 Cargo 授權名稱取代全文。r-efi 6.0.0 的完整 MIT 授權及版權聲明位於 AUTHORS；收集器僅對該版本及固定 SHA-256 接受此文字例外。
