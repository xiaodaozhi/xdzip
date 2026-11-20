<a name="xdzip-仓库目录与依赖"></a>

# 倉庫目錄與依賴

[English](../../engineering/repository-layout.md) | [简体中文](../../zh-CN/engineering/repository-layout.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/repository-layout.md) | [한국어](../../ko-KR/engineering/repository-layout.md) | [Français](../../fr-FR/engineering/repository-layout.md) | [Deutsch](../../de-DE/engineering/repository-layout.md) | [Español](../../es-ES/engineering/repository-layout.md) | [Português (Brasil)](../../pt-BR/engineering/repository-layout.md) | [Русский](../../ru-RU/engineering/repository-layout.md)

目前產品模組以根目錄 Cargo.toml 和 CMakeLists.txt 為準。下表描述實際目錄；架構設計中的未來拆分不代表已有獨立 crate。

| 目錄 | 所有權與依賴規則 |
| --- | --- |
| apps/desktop | Qt/QML、薄 C++ 控制器、模型、設定與系統互動；透過 bridge 訪問核心 |
| crates/xdzip-cli | 無 Qt 的引數、終端、JSON、管理式安裝與更新宿主，直接依賴 runtime、安全 I/O 和更新核心，不解釋格式 |
| crates/xdzip-domain | 純型別與不變數；不得依賴 Qt 或具體 codec |
| crates/xdzip-codec-api | 能力、借用輸入、編碼輸出和取消契約；依賴 domain |
| crates/xdzip-compression | 無第三方生產依賴的共享 Deflate/Deflate64/GZIP、LZMA/LZMA2、BZip2、PPMd7、MSZIP、LZX、Quantum 演算法與封裝，不擁有路徑或釋出能力；獨立實現規則見[契約](independent-rust-codecs.md) |
| crates/xdzip-crypto | 格式無關、固定記憶體的可清除敏感狀態 SHA/HMAC 與 BLAKE 雜湊，複用已准入基礎原語；不解釋歸檔密碼或決定釋出 |
| crates/xdzip-codec-zip、xdzip-codec-tar | ZIP、TAR/GZIP 格式實現；不獲取任意輸出路徑 |
| crates/xdzip-codec-lha、xdzip-codec-arj、xdzip-codec-arc、xdzip-codec-bh | 自研 [LHA](lha-containers.md)、[PMA](pma-streams.md)、[ARJ](arj-streams.md)、[ARC](arc-streams.md) 和 [BH](bh-containers.md) 結構與成員配置，不擁有檔案系統許可權；產品入口以格式登記為準，crate 存在不等於已准入 |
| crates/xdzip-codec-7z | 自研 7z 容器、方法圖、過濾器、AES 組合、流式建立及有界重寫，不擁有檔案路徑 |
| crates/xdzip-codec-cab | 自研 CAB 只讀容器，使用宿主授權卷和共享演算法；保留原始名稱、UTF-8 宣告及普通重名記錄，不自行發現卷或釋出輸出；見[自研契約](independent-rust-codecs.md#xdzip-cab%E5%8F%AA%E8%AF%BB%E8%A7%84%E6%A0%BC)與[來源](cab-format-sources.md) |
| crates/xdzip-codec-optical | ISO9660/Joliet/Rock Ridge/UDF 讀取器，以及用於建立新資料映像的 ISO9660/Joliet 寫入器，透過流讀寫，不持有路徑 |
| crates/xdzip-codec-bridge | 隔離工作程序到 Rust 7z、RAR、CAB 和光碟格式實現的薄 CXX 邊界，傳遞授權流、資源限額和取消狀態 |
| crates/xdzip-codec-native、tools/native-codec | Rust 7z／RAR／光碟／CAB 後端的隔離工作程序，薄 C++ 僅提供協議及系統日曆、編碼服務；Native 獨立核驗預計費輸出與輔助工作；I/O 邊界見[契約](native-codecs.md) |
| crates/xdzip-extract-plan | 後設資料預檢與有界報告；不擁有目標授權或寫入能力 |
| crates/xdzip-io | 輸入樹、原生授權與身份、安全暫存和檔案提交 |
| crates/xdzip-runtime | 會話、有界記憶體／磁碟索引、查詢、選擇、受限預覽、建立／解壓任務和排程 |
| crates/xdzip-qt-bridge | cxx 型別橋接、雙控制代碼及生命週期；依賴 runtime |
| crates/xdzip-plugin-protocol | 僅 std 的有界 CBOR/framing；不擁有路徑、程序或 Qt 權力 |
| crates/xdzip-plugin-host、examples/sdk | 獨立只讀 SDK 試驗、受控 I/O、原生沙箱門禁及獨立 Python peer；不進入產品格式發現與執行閉包 |
| tools | 獨立橋接、格式、I/O 與程序計數探針，不作為 GUI 產品入口 |
| scripts、.github | 生成、檢查、部署工具與 CI、協作模板 |
| release-notes | 可審閱的版本專屬發行說明及 version/build 後設資料，不存執行報告或生成包 |
| design/hifi、design/html、design/assets | 高保真原稿、可操作設計及執行素材輸入 |
| docs | Markdown 格式的持久設計、工程契約與可複用驗證方法，以及必要圖片；不累積執行報告 |
| specs | 機器可讀的視覺、術語與依賴清單；供產品資源、生成器和檢查工具使用 |
| third_party | 隨程式碼保留的第三方來源與許可材料 |

獨立 xdzip-index、xdzip-platform 和完整外部格式一致性工具屬於後續架構規劃；目前索引在 runtime、平臺檔案操作在 io 與桌面適配層實現。已存在的 plugin-host 僅為[只讀 SDK 試驗](sdk-trial.md)，不等同於完整外掛生態。測試主要隨 Rust 模組、apps/desktop/tests 和 scripts 的檢查工具維護，不建立無實現空殼。

CAB 產品固定使用 Rust，官方 Cab 處理器和 CAB 專用 LZX/Quantum 不進入編譯清單。呼叫鏈繼續使用同一個隔離工作程序，不新增產品可執行檔案，沒有測試開關或失敗回退。完整幀工作預付由 codec-api 定義，容器持有每幀額度，Native 接收端和 runtime 分別核驗自己的額度與輸出限制，見[自研契約](independent-rust-codecs.md#xdzip-cab%E5%8F%AA%E8%AF%BB%E8%A7%84%E6%A0%BC)。

依賴設計見[系統架構](../design/system-architecture.md)。避免 domain 反向引用 runtime、codec 直接通知 QML，以及平臺層決定 UI 文案。生成繫結、樣本、報告和構建輸出進入 build/、target/ 或 dist/，不提交到原始碼目錄。歷史相容測試的持久輸入另見[安裝與關聯](installers-associations.md#xdzip-%E5%85%BC%E5%AE%B9%E5%9F%BA%E7%BA%BF%E7%9A%84%E9%95%BF%E6%9C%9F%E8%BE%93%E5%85%A5)。

`tools/image-preview/` 是 QtGui／QtSvg 的私有靜態圖片工作程序，輸入為已驗證的有界內容，輸出為有界 RGBA，不持有歸檔和輸出路徑。平臺支援及失敗行為見[預覽契約](content-preview.md)。
