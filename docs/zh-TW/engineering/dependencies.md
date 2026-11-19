<a name="xdzip-依赖与格式后端准入"></a>

# 依賴與格式後端准入

[English](../../engineering/dependencies.md) | [简体中文](../../zh-CN/engineering/dependencies.md) | 繁體中文（台灣）

<a name="xdzip-外部-sdk-试验"></a>

## 外部 SDK 試驗

獨立試驗宿主複用鎖定的 serde 1.0.229、serde_json 1.0.154、sha2 0.10.9、getrandom 0.4.2、libc 0.2.190 與 tempfile 3.27.0，不新增第三方版本或格式／CBOR 庫。協議 crate 僅使用 std，自有有界 CBOR 與獨立 Python stdlib peer 依據 RFC 8949 互通。宿主不連結 Qt、不進入桌面和 CLI 執行閉包，也不改變正式發行材料。

tempfile 從既有鎖檔案提升為試驗的直接依賴，用於保留經校驗的私有可執行副本，替代自行維護通用臨時目錄實現；安全輸出仍復用 xdzip-io，不把 tempfile 當作提交授權。來源為 [Stebalien/tempfile](https://github.com/Stebalien/tempfile)，許可 MIT 或 Apache-2.0，預設特性與 fastrand、getrandom、once_cell、rustix／windows-sys 等實際目標閉包以 Cargo.lock／metadata 為準，沿用已有鎖定版本，沒有下載構建指令碼或新增 C 庫。已有基礎庫的用途、來源和許可繼續適用；本輪不把歷史公告查詢當成新安全結論。測試與門禁見 [SDK 試驗](sdk-trial.md)，Windows 未有 AppContainer 實現時固定拒絕。

包成員訪問複用已准入 cap-std／cap-fs-ext 4.0.3 的精確版本與既有特性，以持有的目錄逐層 no-follow 開啟，避免字串檢查後重新按環境路徑開啟的競爭。兩者沿用既有安全 I/O 的來源、MIT 或 Apache-2.0 許可、傳遞版本與原生行為，不新增第三方版本。

<a name="xdzip-cli-前端与-https"></a>

## CLI 前端與 HTTPS

CLI 將鎖檔案中的 clap 4.6.7 提升為實際產品前端依賴，僅啟用 std、help、usage、error-context、suggestions、string，不啟用 derive 或終端顏色。新增 ureq 3.5.1，關閉預設特性，僅啟用 native-tls-no-default、socks-proxy，顯式選擇 NativeTls 與系統信任根；不引入預設 GZIP 解碼、Cookie、JSON 輔助、外部 curl 或 Qt 網路執行依賴。serde、serde_json、zeroize、sha2、getrandom 及平臺繫結複用已有鎖定版本。

ureq 與 ureq-proto 來源為 [algesten/ureq](https://github.com/algesten/ureq)，許可選擇 MIT。關鍵傳遞輸入為 native-tls 0.2.18、ureq-proto 0.7.1、http 1.5.0、httparse 1.10.1；完整版本、特性和目標閉包以 Cargo.lock 和 host-filtered Cargo metadata 為準。Windows 使用 schannel 0.1.30，macOS 使用 security-framework 3.7.0 與系統框架，Linux 使用 openssl 0.10.82／openssl-sys 0.9.118 與系統 OpenSSL 3。RustCrypto der／PKI 型別只用於通用 TLS 封裝，不解釋歸檔格式。

Native TLS 的 Linux 構建透過 pkg-config 查詢系統 OpenSSL，不啟用 vendored，不下載或編譯一份私有 OpenSSL；Mac/Windows 使用系統 TLS。必要的 Cargo 構建指令碼、原生連結、目標條件和已知公告按實際鎖定版本檢查，查詢記錄留在本地／CI，摘要登記 CLI issue，不將單次無匹配公告寫成永久安全保證。CLI 自有終端、訊號與 OS 查詢 FFI 限定在平臺適配模組，失敗拒絕操作，密碼容器沿用歸零約束。

包內通知分別從 CLI 與 worker 的普通依賴閉包收集，排除 build/dev/proc-macro 作為執行歸屬證據。桌面執行歸屬增加 CLI 根。Windows 公共啟動器單獨靜態連結編譯器 CRT，使入口在版本化 app-local DLL 尚未載入時可啟動；實際載荷和 worker 繼續攜帶固定 VC++ 執行庫及原始許可，不攜帶 SDK 圖形庫。依賴來源、靜態／動態機器碼保留和許可義務仍以實際發行材料複核。

首次新增任何依賴，記錄用途、替代方案、版本、來源、許可證、傳遞依賴、特性、原生構建行為、已知漏洞和維護情況。僅有 crate 名稱不算準入完成。目前不列虛構的已審計版本。

| 候選方向 | 必須驗證 |
| --- | --- |
| ZIP Rust 實現 | Store/Deflate、ZIP64、CRC、重複項、名稱原始位元組、取消和宿主輸出 sink |
| TAR + GZIP Rust 實現 | 流式索引、PAX/GNU 後設資料、連結/裝置項拒絕、未知大小、損壞流 |
| cxx | 所有權、生成器一致性、跨平臺靜態核心連結、異常不越界 |
| Qt 模組 | 實際執行與傳遞依賴、可替換部署、許可證和安全更新渠道 |
| 圖示集合 | 明確再分發許可、統一線寬、可訪問性與資產清單 |

驗證順序：最小互操作 → 惡意/截斷輸入 → 取消/資源上限 → 許可/SBOM → 乾淨構建 → 記錄決定。不能透過呼叫庫的通用 unpack 繞過宿主路徑保護。

應用型 Rust workspace 提交 Cargo.lock。新增原生庫和網路下載 build script 需專門審查；禁用未使用 features。棄用或漏洞無修復時按實際可達性決定升級、替換或關閉能力，記錄原因。

<a name="xdzip-更新元数据与发行者验签"></a>

## 更新後設資料與發行者驗籤

`xdzip-update` 是不依賴 Qt 的清單、渠道和簽名驗證核心，產品透過獨立 cxx 邊界呼叫；釋出工具和安裝助手使用相同規則。選擇 `ed25519-dalek = 2.2.0` 的安全 `VerifyingKey::verify_strict`、弱公鑰拒絕和精確原始位元組驗籤，不自行實現簽名運算，不把 TLS 或 SHA-256 當作發行者簽名。預設啟用上游 fast、std、zeroize，未啟用 batch、hazmat、legacy_compatibility、rand_core 或 PKCS#8；釋出工具僅匯入由所有者配置的 32 位元組 seed，不生成生產私鑰。

| 直接或關鍵傳遞依賴 | 精確版本 | 來源、許可與用途 |
| --- | --- | --- |
| ed25519-dalek、curve25519-dalek | 2.2.0、4.1.3 | [dalek-cryptography](https://github.com/dalek-cryptography/curve25519-dalek)，BSD-3-Clause，Ed25519 與曲線運算 |
| base64ct | 1.8.3，alloc | [RustCrypto/formats](https://github.com/RustCrypto/formats)，Apache-2.0 或 MIT，Sparkle 相容簽名編碼 |
| sha2 | 0.10.9 | [RustCrypto/hashes](https://github.com/RustCrypto/hashes)，MIT 或 Apache-2.0，安裝包摘要 |
| semver | 1.0.28 | [dtolnay/semver](https://github.com/dtolnay/semver)，MIT 或 Apache-2.0，有界產品版本解析 |
| subtle | 2.6.1 | [dalek-cryptography/subtle](https://github.com/dalek-cryptography/subtle)，BSD-3-Clause，上游常量時間運算支援 |
| zeroize | 1.9.1 | [RustCrypto/utils](https://github.com/RustCrypto/utils)，Apache-2.0 或 MIT，釋出程序私鑰臨時容器歸零 |

serde 1.0.229 和 serde_json 1.0.154 複用鎖定版本。其餘傳遞依賴以 Cargo.lock 和實際特性樹為準，包括 digest、block-buffer、crypto-common、generic-array、typenum、cpufeatures 與 derive 宏。curve25519-dalek 的構建指令碼僅檢測本地編譯器能力，執行時可選擇上游的 CPU 特定實現；不下載外部原始碼、不新增 C/C++ 密碼學庫或 OpenSSL 執行依賴。已有通知和 vendor 收集入口根據 Cargo metadata 納入這些包，實際包的歸屬與許可仍由發行材料核驗。

保留系統密碼學 API 的替代方案會分裂三平臺驗證規則，外部 OpenSSL CLI 會擴大執行依賴及引數邊界，因此採用同一 Rust 核心。已知歷史 [RUSTSEC-2022-0093](https://rustsec.org/advisories/RUSTSEC-2022-0093.html) 的舊版 Keypair API 不用於目前實現；[RUSTSEC-2024-0344](https://rustsec.org/advisories/RUSTSEC-2024-0344.html) 要求的 curve25519-dalek 修復邊界為 4.1.3，鎖定版本滿足該邊界。這兩項核對不是全部依賴無漏洞的證明，後續升級繼續核對維護與公告。

迴歸同時驗證精確位元組變更、錯誤簽名、未知／重複／弱公鑰、空生產信任集、渠道回退、版本與 build 繫結、六包完整性和簽名 feed。工具臨時測試金鑰只存在於測試程式碼和隔離工件，禁止寫入產品信任資源。原生安裝和跨平臺結果分別記錄，不以單平臺核心測試替代安裝驗收。

<a name="xdzip-windows-来源身份"></a>

## Windows 來源身份

M2 在 Windows 執行時直接使用精確版本 `winapi-util = 0.1.11` 的 `file::information(&File)` 安全封裝，比較卷序列號與檔案索引。長度、修改／建立時間無法可靠檢測 NTFS 同名替換；標準庫對應檔案索引介面尚非穩定 API，自行寫 Win32 unsafe 會擴大審查範圍。因此複用鎖檔案已有的庫，不新增包版本。

來源為 crates.io／[BurntSushi/winapi-util](https://github.com/BurntSushi/winapi-util)，許可選擇 MIT（上游 MIT 或 Unlicense 雙許可）。Windows 傳遞依賴為已鎖定的 windows-sys 0.61.2、windows-link 0.2.1，均為 MIT 或 Apache-2.0；僅啟用上游宣告的 Win32 模組，禁用預設特性，無 build script、外部原生構建或網路下載。非 Windows 目標不編譯這條執行依賴。上游安全封裝借用有效 File 控制代碼，系統呼叫失敗使舊來源失效；不跨 FFI 傳遞這個控制代碼，不將身份檢查當作安全寫入授權。

版本來源、特性、原生編譯行為已檢查；Windows GNU 目標型別檢查已透過，原生 Windows 迴歸必須覆蓋保留修改時間的同名／同大小替換。上游為小型 Win32 安全封裝專案，升級由鎖檔案和原生迴歸約束。2026-10-13 的 OSV 精確版本查詢未返回這三個包的已知公告，歷史響應保留在原執行工件，新查詢按[驗證方法](../../../developers/qa/README.md)另記實際響應；這不是無漏洞證明。開發包繼續收集所有上游許可證。

<a name="xdzip-rust-格式后端与共享工作进程桥接"></a>

## Rust 格式後端與共享工作程序橋接

`xdzip-codec-7z` 使用專案 API/domain、無第三方生產依賴的共享壓縮演算法及通用密碼學原語。AES 0.9.3 啟用 zeroize，getrandom 0.4.2 與 zeroize 1.9.1 複用 ZIP 版本。可清除敏感狀態的 SHA-256 提取到格式無關的 `xdzip-crypto`，在已准入 sha2 0.10.9 的 compress 原語上管理自有增量狀態，不在各格式複製包裝。共享 crate 還依據公開數學規範實現 HMAC-SHA256、BLAKE2s/BLAKE2sp，來源及邊界見其 [README](../../../crates/xdzip-crypto/README.md)，不提供歸檔或釋出許可權，不升級第三方密碼學依賴版本。格式屬性、CBC 組合、KDF、建立及重寫均為獨立實現，不由這些基礎庫提供歸檔處理。來源、分析角色及許可區分見[7z 依據](sevenzip-format-sources.md)。

`xdzip-codec-optical` 的格式邏輯只使用標準庫及本專案 codec API/domain，不引入 ISO/UDF 或通用歸檔庫。規範、範圍與錯誤邊界見[獨立實現契約](independent-rust-codecs.md#xdzip-iso%E4%B8%8Eudf%E5%8F%AA%E8%AF%BB%E8%A7%84%E6%A0%BC)。`xdzip-codec-bridge` 複用已鎖定的 cxx/cxx-build 版本和生成流程，不新增外部包版本、下載構建指令碼或格式演算法庫。替代的程序內路由會失去既有光碟任務的獨立程序治理，因此保留既有 worker 和受控流協議。

`xdzip-codec-cab` 只依賴本專案 API、domain 與無第三方生產依賴的 `xdzip-compression`。MSZIP 複用 ZIP/GZIP 的 Deflate，LZX 和 Quantum 為獨立演算法模組，不引入額外 Deflate 或通用歸檔庫。7z、CAB 與光碟後端共享 `xdzip-codec-bridge` 及宿主授權流，建立僅增加受控輸入與隨機暫存回撥，不復制通用邊界。產品固定使用 Rust，官方 Cab 與其專用 LZX/Quantum 不進入構建；資源及格式邊界見[自研契約](independent-rust-codecs.md#xdzip-cab%E5%8F%AA%E8%AF%BB%E8%A7%84%E6%A0%BC)，參考邏輯的分析範圍與許可見[CAB 來源](cab-format-sources.md)。

`xdzip-codec-rar` 複用同一 codec API、共享壓縮演算法與密碼原語，不引入 RAR 或其他歸檔格式庫。PPMd H 統計模型由 7z 與 RAR 共用，格式各自定義 range、重置與終止；舊密碼的特殊相容組合保留在 RAR 模組。系統日曆及 OEM/ANSI/本地編碼由薄工作程序回撥提供，格式欄位、密碼截斷與校驗仍由 Rust 處理，Qt 不進入格式邏輯。

生產依賴政策同時檢查格式 crate 與橋接 crate 的普通/構建依賴閉包。官方 7-Zip 與 UnRAR 的容器、演算法和通用設施全部退出產品 CMake 清單，產品原始碼準備清單為空，沒有失敗回退。歷史交付的許可義務不因目前退出而消失。獨立 pycdlib、Microsoft IMAPI、mkudffs/udfclient 夾具、官方參考 worker 和規範向量僅用於測試，來源和工具許可與產品分別管理，不進入應用執行依賴。專案自有內容的小型固定輸入及生成指令碼納入測試目錄，展開的像和執行證據只留在 build/。

<a name="xdzip-html-复核字体与工具"></a>

## HTML 複核字型與工具

採用官方 Noto Sans SC 2.004-H2 的離線可變子集，替代審閱裝置的系統回退字型。固定 Google Fonts 提交、原始雜湊和生成配置、版權和 OFL-1.1 許可見 [材料清單](../../../specs/dependencies/ui-font.json)。構建時在構建目錄生成含 400–750 字重的 WOFF2 及同源 TTF，原始碼倉庫不儲存生成字型二進位制；無外部服務、傳遞執行依賴或原生編譯。保留系統字型方案會導致跨平臺文字度量不同，因此審閱稿採用隨包字型。每次構建檢查目前文字覆蓋，未知生成原稿字型仍不能證明匹配。字型上游持續維護，固定版本需隨維護情況複核。

桌面資源靜態檢查使用 Pillow（HPND）、FontTools（MIT）、Brotli（MIT）和工作流檢查所需 PyYAML（MIT），版本固定於[桌面資源工具要求](../../../scripts/desktop-asset-requirements.txt)，透過獨立 venv 安裝。Playwright（Apache-2.0）及 Chromium 只用於按需執行的本地 HTML 設計複核，其版本見[本地設計工具要求](../../../scripts/html-design-requirements.txt)；它們不進入預設 CI、應用或離線設計交付。上述工具均不進入產品執行程式碼。

<a name="xdzip-自研-zip-与通用密码学基础库"></a>

## 自研 ZIP 與通用密碼學基礎庫

ZIP 容器、流式建立、普通重寫與 AES 封裝採用[獨立 Rust 實現](independent-rust-codecs.md)，Deflate/CRC-32 複用 `xdzip-compression`。`rawzip = 0.5.2`、`zip = 9.0.3`、`flate2 = 1.1.10` 僅為開發測試參照，不進入 ZIP 生產或構建依賴閉包，不呼叫通用 unpack 或授權任意檔案路徑。歷史上使用它們的交付繼續保留原有許可義務。

密碼學基礎原語複用原有傳遞依賴的精確版本，提升為直接依賴：RustCrypto `aes = 0.9.3`、`hmac = 0.13.0`、`sha1 = 0.11.0`、`pbkdf2 = 0.13.0`，以及 `getrandom = 0.4.2`、`constant_time_eq = 0.6.1`。PBKDF2 只啟用 hmac，SHA1 關閉預設 alloc/oid，只啟用 zeroize；AES/HMAC 同時啟用 zeroize。沒有新增格式、原生壓縮庫或執行下載。CTR、隨機鹽封裝、派生結果分割、密碼驗證值和認證 EOF 由專案實現，不能借通用基礎庫重新引回 ZIP 解析器。

這些包及 cipher/digest 等傳遞依賴鎖定在 Cargo.lock，來源為 crates.io 與各自上游。AES/HMAC/SHA1/PBKDF2/zeroize/getrandom 採用 MIT 或 Apache-2.0，constant_time_eq 採用 CC0-1.0 或 MIT-0 或 Apache-2.0，以實際鎖定包原文為準。SHA-1/PBKDF2 引數來自 WinZip AES 格式，不用於另選的使用者密碼儲存方案。構建指令碼只做平臺配置，沒有網路下載或 C 解碼器編譯。

`zeroize` 統一精確升級為 1.9.1，以滿足 AES 0.9.3 的金鑰清理特性所需版本；所有宿主、原生適配、更新和探針密碼容器保持同一版本，避免兩套同主版本依賴。秘密 Debug 繼續脫敏，派生緩衝與 expanded key 採用基礎庫支援的歸零生命週期，不聲稱所有歷史記憶體副本均可擦除。准入須包含鎖檔案、許可、實際特性樹、故障測試及獨立 7-Zip 互操作；漏洞查詢原文只保留為構建工件，成功查詢未返回公告不等於無漏洞保證。

<a name="xdzip-wp-11-targzip"></a>

## WP-11 TAR/GZIP

精確固定 [tar 0.4.46](https://docs.rs/tar/0.4.46/tar/)（crates.io、[上游原始碼](https://github.com/alexcrichton/tar-rs)，MIT 或 Apache-2.0），關閉預設 xattr 特性，僅用作開發測試的獨立讀寫參照。產品 TAR 頭、PAX、建立與重寫使用[獨立 Rust 記錄層](independent-rust-codecs.md)，不呼叫第三方 TAR 解析或 Builder，不呼叫通用 unpack 或所有者／許可權恢復。生產依賴閉包禁止 tar 和 filetime 迴流，測試及歷史交付的許可證仍保留。編碼顯式寫 PAX UTF-8 path 及超範圍數字，保證跨平臺名稱互操作。

GZIP 由無第三方生產依賴的 `xdzip-compression` 獨立實現，包括 RFC 1951 Deflate、RFC 1952 封裝和 CRC-32。壓縮輸入與解碼輸出都遵守宿主取消和位元組預算，完整解碼時驗證每個成員的 CRC 與 ISIZE。`flate2` 1.1.10 僅供開發測試對照。ZIP 和 GZIP 複用同一套自研 Deflate 演算法。執行時區分[獨立 GZIP](single-file-streams.md)與 [TAR.GZ](tar-gzip.md)，前者提供一個邏輯檔案，後者提供 TAR 目錄樹。

tar 的測試傳遞依賴含 filetime 0.2.29（MIT 或 Apache-2.0），複用既有 cfg-if、libc；filetime 不進入該 codec 的產品依賴閉包。沒有新增下載構建指令碼、執行網路請求或外部程序格式後端。許可證由既有 collect-notices.py 根據鎖檔案收集，鎖檔案存在測試包不表示產品連結這些包。系統 tar 與 Python tarfile 僅用於測試，不進入應用執行依賴。

2026-10-15 對 tar 0.4.46、filetime 0.2.29、flate2 1.1.10 執行 OSV 精確版本查詢，未返回公告；查詢原文只儲存在 build/review/wp11。此結果不等於無漏洞保證。雙向互操作、壞輸入、配額、取消和三平臺驗證必須實際執行，證據入口見 [WP-11 契約](tar-gzip.md)。

<a name="xdzip-qtnetwork-安全派生库"></a>

## QtNetwork 安全派生庫

桌面 Qt 6.8.2 的 Network 動態庫使用 `6.8.2-xdzip-network.1`，從固定 QtBase 歸檔應用五份原樣官方補丁，修復 CVE-2026-116337 與 CVE-2026-105946。範圍是 HTTP/CONNECT 狀態行及頭部接收限制和認證解析邊界，不更換 QtCore、其他模組或 TLS 外掛。保留原有功能開關、匯出符號及對原 SDK QtCore 的匯入相容性。原始碼、順序、摘要、許可、替換及真實輸入驗證見[維護說明](../../../patches/qt/README.md)。生產繼續採用 LGPL-3.0 動態連結路徑，上游 GPL-only 測試不發行；OpenSSL 3.5.9 僅生成編譯宣告，不構建或分發新密碼庫。實際包的來源、通知與重連結條件仍單獨複核。

Linux 另加入受摘要約束的原始 ABI 保留單元，顯式例項化 SDK 已有的內建型別常量，保留原匯出而不增加網路行為；原始碼和 Apache-2.0 通知隨材料提供。五份官方補丁不改寫，其他平臺的產品庫不注入該單元。

<a name="xdzip-macos-更新提供者"></a>

## macOS 更新提供者

Mac 更新整合鎖定 Sparkle 2.10.0 和 `xdzip-proxy.2` 代理擴充套件，SDK、原始碼歸檔、補丁及完整許可正文的雜湊統一見
[依賴配置](../../../scripts/sparkle-dependency.json)。它只負責應用更新，不參與歸檔解碼。
官方 SDK 原始許可包含 MIT、BSD-2-Clause 和 Zlib 授權正文，隨 Mac notices 保留；
本地對應原始碼收集器保留鎖定原始碼與 SDK 原始歸檔，支援離線準備和動態框架重連結；按發行協議，此壓縮檔不作為 Release 或 Actions 下載附件提供。
框架主庫的 67 個上游 Objective-C 單元與 10 個 Ed25519 C 單元使用 Apple Silicon 原生 Clang 從鎖定原始碼和[受限補丁](../../../patches/sparkle/2.10.0-download-proxy.patch)編譯，保留上游構建特性和安裝 IPC。資源、安裝助手和簽名工具繼續來自官方 SDK，不宣稱已從原始碼重建這些二進位制。派生框架記錄獨立 variant、原始碼／補丁／構建器摘要和逐檔案構建回執；部署歸屬及 SBOM 區分此來源與官方 SDK 位元組。構建方法見[補丁維護說明](../../../patches/sparkle/README.md)。

擴充套件使用系統自帶 Network.framework 實現無直連回退的自定義代理，使用 CFNetwork 的會話代理開關實現明確直連，不增加可再分發執行庫或修改系統代理設定。

按需 Qt 工作流的 Mac 作業外部連結 SDK，不部署框架。專用打包在臨時副本嵌入框架，刪除非沙盒應用
未啟用的 XPC 服務，保留必要安裝助手。裁剪、重定位後按助手、framework、應用順序
做 ad-hoc 完整性簽名，助手保留所需 Hardened Runtime 和專屬 entitlements，禁止套用
通用原生庫的重籤命令。部署歸屬、SBOM、許可及原始碼材料隨同版本記錄。

<a name="xdzip-实验安装器工具"></a>

## 實驗安裝器工具

Windows 構建引入 Inno Setup 6.7.3，僅用於編譯自有執行目錄的目前使用者安裝器，來源為 [jrsoftware/issrc 官方發行](https://github.com/jrsoftware/issrc/releases/tag/is-6_7_3)，不可變原始碼提交、官方二進位制／中文語言資源／許可正文的 SHA-256 固定在 [installer-toolchain.json](../../../scripts/installer-toolchain.json)。選擇其目前使用者模式和累計解除安裝日誌；NSIS 與 WiX/MSI 是替代方案，本輪不另維護自制清理資料庫或企業部署規則。

許可採用該固定原始碼中的 Inno Setup License，保留版權／原始來源及引擎介面，不修改編譯器；正文隨 Windows notices 交付。傳遞程式碼使用官方編譯器／安裝存根提供的 Pascal Script、壓縮與平臺元件，本輪不重新編譯引擎，內嵌元件的來源與許可按實際交付範圍核對。編譯器安裝在 CI build/，不進入 Qt 執行目錄；安裝存根進入最終安裝器，其版本與安裝器雜湊由回執記錄，執行目錄 SBOM 不冒充安裝器存根的完整 SBOM。

僅啟用標準檔案複製、目前使用者登錄檔、自有快捷方式和互斥量保護，關閉自動關閉／重啟應用，無安裝時網路下載、自定義原生外掛或新 Rust 構建指令碼。Linux .deb 使用 Ubuntu 的 dpkg-deb 與系統維護工具，不引入額外應用編解碼器。Qt 下載工具工作目錄固定在 build/，避免 aqtinstall.log 汙染原始碼身份。

2026-10-16 查詢官方倉庫公開 security-advisories API 返回空列表，原文儲存在 build/review/installer-tool-advisories.json。這不代表無漏洞或 NVD 全庫檢索透過；[官方版本記錄](https://jrsoftware.org/files/is6-whatsnew.htm)用於後續安全維護複核，固定 6.7.3 不代表最新版本或長期支援承諾。雜湊校驗不代替簽名身份與完整引擎許可／安全准入。真實編譯、安裝、系統開啟、更新／回退／解除安裝和未透過項按[安裝契約](installers-associations.md)及實際 CI 記錄登記。

<a name="xdzip-文件名旧编码检测与解码"></a>

## 檔名舊編碼檢測與解碼

精確固定 [chardetng 1.0.0](https://docs.rs/chardetng/1.0.0/chardetng/) 和 [encoding_rs 0.8.42](https://docs.rs/encoding_rs/0.8.42/encoding_rs/)，來源均為 crates.io 和 hsivonen 上游。前者採用 MIT 或 Apache-2.0，後者的程式碼採用 MIT 或 Apache-2.0，WHATWG 對映資料額外採用 BSD-3-Clause，保留 LICENSE-WHATWG 與 COPYRIGHT。通知收集器包含 COPYRIGHT，不只收集許可全文。

關閉兩者預設特性，encoding_rs 只直接啟用 alloc，禁用檢測器的 multithreading/rayon。鎖定的新傳遞包為 core_detect 1.0.0、multiversion_no_op 1.0.0、scopeguard 1.2.0、simdutf8 0.1.5，均為 MIT 或 Apache-2.0；cfg-if 1.0.5、memchr 2.8.3 和構建宏 rustversion 1.0.23 複用鎖定版本。core_detect 提供本地 CPU 能力檢測，simdutf8 為上游 SIMD UTF-8 驗證，multiversion_no_op 是編譯期空操作宏；沒有新 C/C++ 原生庫、網路構建指令碼或執行下載。encoding_rs 構建指令碼只按編譯器版本選擇本地配置。CPU 特定 unsafe 實現留在受維護上游，宿主不新增 unsafe/FFI。

選擇同一 Rust 檢測器避免 Qt、系統 iconv 和 Windows 內碼表分裂規則；自行維護 CJK 統計模型或呼叫外部解壓工具都會擴大實現與部署邊界。chardetng 上游標記為被動維護，版本與模型變化需重跑獨立檔名樣本。ZIP CP437 是固定單位元組對映，依據[Unicode CP437 表](https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/PC/CP437.TXT)，不引入平臺 API；測試原始位元組另由 Python 標準庫生成。檢測是啟發式結果，不作為路徑授權或內容校驗證據。

2026-10-24 對六個新增鎖定包執行 OSV 精確版本查詢，響應未返回公告，原始響應只儲存在本地工件。這是定向核對，不是無漏洞保證。准入同時檢查實際 Cargo features、來源及許可全文，迴歸範圍與資源和失敗規則見[檔名編碼契約](filename-encoding.md)；三平臺構建、內容互操作和實際結果在對應 issue/CI 登記。

<a name="xdzip-qt-图片预览模块"></a>

## Qt 圖片預覽模組

沿用固定 Qt 6.8.2，增加官方 qtimageformats 模組，原始碼提交、歸檔摘要見[元件配置](../../../scripts/corresponding-sources.json)。使用 qwebp、qtiff、qtga、qicns；macOS 另使用 qmacheif 呼叫系統 ImageIO，不引入 libheif／libavif 或額外圖片後端。GIF、BMP、ICO／CUR、JPEG 複用 QtGui，SVG 複用既有 QtSvg，但不開放不受限資源載入。

該固定模組自帶 libwebp 1.5.0 和 libtiff 4.7.0，實際 SDK SPDX 標記其歸屬；COPYING／COPYRIGHT 原文透過固定提交和 SHA-256 收集，不把通用 NOASSERTION 標記等同於已確認許可缺陷。動態 Qt 模組沿用現有許可路線，第三方通知按具體交付元件保留。精確版本依賴查詢、三平臺解碼／壞輸入／部署結果留在 issue 和工件，不據一次未返回公告的查詢宣稱無漏洞。

圖片執行庫經固定 helper 使用，原圖、畫素、工作程序、輸入輸出及失敗限制見[預覽契約](content-preview.md)。預設準備補齊已有工具鏈，構建前檢查外掛，部署驗證實際 helper；不新增另一套 Qt SDK。

<a name="xdzip-文件管理器菜单适配"></a>

## 檔案管理器選單適配

Windows 擴充套件僅依賴 Windows SDK 系統 API 與靜態 MSVC 執行庫，macOS 服務 helper 僅依賴系統 AppKit 和 C++ 標準庫，均不把 Qt 或格式庫載入到檔案管理器宿主。Ubuntu 24.04 使用發行版的 `libnautilus-extension4` 與 GLib/GObject/GIO，構建依賴 `libnautilus-extension-dev`，不將 Files 主程式作為產品元件。

[Nautilus 46.0 的 MenuProvider 標頭檔案](https://github.com/GNOME/nautilus/blob/46.0/libnautilus-extension/nautilus-menu-provider.h)宣告 GNU Library General Public License 2 或後續版本；新擴充套件只使用該公開介面並動態連線系統庫，選擇後續 LGPL-3.0 條款。實際系統包版本與來源由 Linux 部署審計記錄，不隨產品複製一套 Nautilus/GLib。產品自有適配程式碼、介面使用與完整 Nautilus 應用的許可分別判斷；正式候選仍按其真實執行材料複核，不由介面編譯成功推定發行許可透過。
