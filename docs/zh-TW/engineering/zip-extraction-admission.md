<a name="xdzip-zip-解压与输出依赖准入"></a>

# ZIP 解壓與輸出依賴准入

[English](../../engineering/zip-extraction-admission.md) | [简体中文](../../zh-CN/engineering/zip-extraction-admission.md) | 繁體中文（台灣）

本頁定義安全解壓及密碼解壓複用的依賴邊界。最初准入的依賴來源、許可與特性摘錄見[基礎鎖定材料](../../../specs/engineering/zip-extraction-dependencies.json)；含 AES 的目前精確版本與 crates.io 校驗和以根目錄 Cargo.lock 為準，完整啟用圖透過 `cargo tree --locked -e features` 核對，不能把基礎摘錄當作目前完整依賴清單。

| 依賴 | 用途和決定 |
| --- | --- |
| xdzip-compression | 專案獨立實現的共享 Deflate 與 CRC-32，固定緩衝，嚴格檢查最終塊及精確壓縮位元組消費量。ZIP 容器、描述符與 AES 封裝由獨立格式層處理，借用宿主輸入與輸出 sink，保留輸出授權和完整校驗邊界。flate2 僅為開發測試參照。 |
| cap-std / cap-fs-ext 4.0.3 | 持有目錄能力、nofollow 子目錄開啟、排他暫存和相對提交操作。相比字串拼接與 canonicalize 後建立，元件許可權和身份複核在宿主執行；不呼叫通用 unpack。cap-primitives 等傳遞包一併鎖定。 |
| getrandom 0.4.2 | 使用作業系統隨機源生成私有工作目錄 nonce；不使用時間戳、偽隨機或歸檔名字。關閉預設特性。 |
| rustix 1.1.5 | POSIX 有效使用者身份、傳遞檔案系統原語，複用已鎖版本；關閉預設特性，僅啟用 std/process，加上傳遞所需 fs。 |
| windows-sys 0.61.2 | 查詢 Windows 物件所有者和 DACL、檢查 SID；隔離手寫 unsafe，借用有效控制代碼、DWORD 對齊緩衝區和 RAII LocalFree。相比只檢測只讀屬性，DACL 能檢查私有輸出前提；新增 FFI 須驗證控制代碼、緩衝區與釋放前提及查詢失敗行為。 |
| unicode-normalization 0.1.25 | 為任務名稱別名生成 NFD＋小寫 admission key；原始輸出拼寫保留，不把此鍵當作檔案系統完整 casefold。相比只用 ASCII 小寫能覆蓋常見 macOS Unicode 等價名稱。 |
| AES、HMAC、SHA-1、PBKDF2、constant_time_eq | 經准入的通用密碼學基礎庫，精確版本與特性見[依賴契約](dependencies.md)。ZIP 的 CTR 計數、鹽和驗證值佈局、密文認證及錯誤分類由專案獨立實現。zip 9.0.3 僅為開發測試參照。 |
| zeroize 1.9.1 | 為密碼、派生金鑰和測試探針的有界 stdin 緩衝提供歸零生命週期，AES/HMAC/SHA-1 同時開啟其清理特性；不保證 Qt、輸入法及所有歷史記憶體副本均被擦除。 |

通用依賴核對 Cargo 配置、許可與公共 API；獨立格式實現的來源規則見[自研契約](independent-rust-codecs.md)。cap 系列由 Bytecode Alliance 維護，getrandom 和 rustix 為系統介面專案，Unicode normalization 為通用 Unicode 庫；升級仍須鎖版本並重跑原生邊界迴歸，不以維護主體代替迴歸測試與漏洞公告核對。新增包無執行時網路請求；含 build script 的包只探測編譯器／目標配置或選擇 Rust 系統呼叫後端，沒有下載外部程式碼或編譯 C 解碼器。許可通知收集完整上游原文，實際許可表示式以鎖定材料為準。

OSV 精確版本查詢須按[驗證方法](../../../developers/qa/README.md)保留實際響應；查詢失敗不能解釋成未發現公告。該檢查不是完整 cargo-audit。既有 cxx 公告與版本選擇仍見 [M2 依賴記錄](dependencies.md)。

後設資料准入不自動批准解碼。獨立介面卡驗證本地頭、流結束、CRC 與 32／64-bit 資料描述符，按中央及本地 ZIP64 哨兵判定寬度；獨立 Python force_zip64 夾具同時覆蓋合法混合寬度和篡改大小。rawzip、zip、flate2 不進入 ZIP 及宿主的普通或構建依賴閉包，歷史鎖定摘錄僅保留當時的來源記錄。

鎖定材料列出全部跨目標 Cargo 解析依賴，其中 getrandom 的 WASI／WIT／WebAssembly 工具包只適用於未支援的目標，不進入 macOS、Windows、Linux 桌面編譯或執行。Cargo 全目標清單不能解讀為桌面執行依賴範圍。

開發包 notices 按 `cargo metadata --filter-platform <rustc host>` 的實際解析節點收集目前平臺的工作區和構建工具許可，並登記 target.json；不會把未編譯的 WASI／UEFI 包當作桌面發行依賴。許可證缺失仍使打包失敗，不能用 Cargo 許可名稱代替正文。r-efi 6.0.0 的完整 MIT 授權和版權位於 AUTHORS，收集器僅接受該版本與固定 SHA-256 的原文例外。
