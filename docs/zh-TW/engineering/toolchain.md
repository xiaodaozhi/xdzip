<a name="xdzip-工具链与平台冻结表"></a>

# 工具鏈與平臺凍結表

[English](../../engineering/toolchain.md) | [简体中文](../../zh-CN/engineering/toolchain.md) | 繁體中文（台灣）

開發工具版本由 rust-toolchain.toml、Cargo.lock、CMake 和 CI 固定，支援範圍遵循[支援政策](support-policy.md)。本表定義升級及發行前的核驗要求，具體觀測記錄留在 issue／Actions，不把單機工具列表當作發行依賴鎖。

| 項目 | 目前方向 | 凍結所需證據 |
| --- | --- | --- |
| Rust | rust-toolchain.toml 固定 1.99.0，edition 2024，rust-version 1.99 | 升級時核對依賴 MSRV、三目標測試和 Clippy，保持工具鏈檔案與 Cargo 配置一致 |
| Qt | 固定 6.8.2 SDK，QtNetwork 採用 `6.8.2-xdzip-network.1` 官方安全補丁派生庫，本地統一放在 `build/toolchain/Qt/6.8.2/<platform-subdirectory>`，動態部署 | 固定來源／雜湊、模組許可、三平臺 smoke；每次發行檢查安全維護狀態，升級須複驗原始碼與替換流程 |
| C++ | C++17、Qt 匹配的編譯器 | macOS clang、Windows MSVC、Linux 編譯器版本 |
| CMake | 根配置要求 3.24 及以上，使用 Ninja | 核對實際構建版本、生成頭依賴與乾淨配置；CI 工具版本由工作流固定 |
| Python | 共用桌面入口要求 3.10+，打包校驗工具使用隔離環境 | 同一直譯器執行後續 Python 指令碼，按階段檢查固定工具依賴；不將瀏覽器設計工具列為產品構建依賴 |
| cxx | 1.0.205，構建期生成橋接，不提供穩定二進位 ABI | crate／生成器同版本，關閉競態和異常邊界迴歸 |
| 系統 | macOS 14+ arm64、受維護 Windows 11 x64、Ubuntu 24.04 X11 x64 | 按已批准支援政策核對真實環境，不把 CI 宿主擴大為全部系統版本保證 |

Windows 使用 MSVC 與 Ninja 時，桌面構建、Qt 原始碼構建和原始碼交付重建在構建期間臨時使用 UTF-8 控制檯內碼表，並在結束後恢復原值；沒有控制檯時使用臨時隱藏控制檯，供本程序和子程序使用，外部程序不得主動附加。同一現有控制檯中的受保護構建按附加程序列表互斥，獨立控制檯可以並行；各程序在保護期間必須保持附加，無法取得有效列表時停止，等待超過 30 秒則提示重試。無視窗控制代碼的舊版指令碼與新版鎖不相容；該協調也不控制其他程式對控制檯的修改。構建前後檢查已編譯物件的 Ninja 依賴記錄，發現直接使用引號包含或同目錄尖括號包含的原始檔沒有依賴記錄時停止，保留原目錄並要求使用新的構建目錄，不自動刪除或重用舊物件。Ninja 可能有意省略系統標頭檔案，因此僅使用系統頭的物件允許零依賴；檢查不覆蓋其他搜尋路徑的尖括號包含或所有編譯器隱式輸入，也不解析預處理條件，可能保守地拒絕未啟用的包含項。直接執行 CMake／Ninja 不經過這些入口保護，應自行保證編譯器輸出與 Ninja 字首一致，並在更換內碼表或工具鏈後驗證標頭檔案改動確實觸發重編譯。

GitHub 託管的 Apple Silicon 作業使用 macOS 15 執行器，產品部署目標仍為 macOS 14.0。執行器升級不等於在 macOS 14 實機上完成驗證；平臺相容與人工驗收分別記錄。

日常準備、檢查與執行統一見[開發構建指南](developer-build.md)。`scripts/desktop.py doctor` 預設檢查 build；`doctor --for-stage package` 提前檢查完整打包鏈，prepare 的 `--install-rust`、`--install-qt`、`--packaging` 顯式準備相應工具與鎖定材料。系統編譯器／SDK 仍由開發者安裝，不把預檢當作隱式安裝器。

本地工具鏈目錄固定為 `build/toolchain/`。Qt SDK、`qmllint` 和 `qmake` 使用 `build/toolchain/Qt/6.8.2/<platform-subdirectory>`，`aqtinstall` 及共享 Python 工具環境使用 `build/toolchain/desktop-tools`，Qt 下載暫存使用 `build/toolchain/qt-install-tool`。這些目錄由統一入口共享，避免 build、test、package 或單獨的 Qt 檢查各自建立快取。CI 若使用 runner 臨時目錄，必須透過顯式引數傳入，不改變本地預設佈局。

預檢執行實際 PATH 中的 Rust／Cargo，核對鎖定版本與原生 host，並檢查 CMake、Ninja、Qt 字首及平臺工具。編譯器已安裝但目前 shell 不可呼叫也視為缺失。打包前還檢查 Rust 鎖定依賴後設資料、Qt SPDX／許可材料及 schema 快取；所有後繼程序使用同一檢查過的環境，避免構建成功卻在許可收集時找不到 rustc／cargo。缺少 Docker 或 Inno Setup 只影響需要它的階段，不阻止普通開發構建。

每條記錄包含版本、下載來源、SHA-256、許可證、目標、負責人、驗證提交和日期。Rust/Qt 安全更新觸發重新驗證，不無限保留已失去維護的補丁。最低系統候選不等於正式支援承諾。

系統維護狀態、Qt 安全更新渠道及升級複驗責任以[支援政策](support-policy.md)為準，來源與許可結論按[發行審查流程](../../../developers/qa/release-review.md)登記。

Windows 主視窗的客戶區延伸由 WindowsFrame 使用公開 Qt 視窗介面和 Win32 實現，不連結 GuiPrivate。首次繫結普通視窗時按目前 QWindow 客戶區邏輯尺寸、DPR 和原生邊框度量確定外框，後續主題或 DPI 重應用只重新整理邊框。在沒有局部縮放覆寫、Qt PassThrough 舍入的 PMv2 普通視窗中，WM_GETDPISCALEDSIZE 保留待處理 SIZE 的縮放前調整量，縮放客戶區後加目標 DPI 的實際邊框；無效度量、溢位及其他狀態不寫入 SIZE，交回 Qt。借用的原生訊息及其可寫結構僅在同步回撥中使用。升級 Qt 或替換 SDK 時複驗相關客戶區、邊框和視窗狀態；物理雙屏拖動與視覺按需檢查，程式化結果不冒充未執行的物理驗證。

日常 CI 在 runner 臨時目錄中獨立安裝 `rust-toolchain.toml` 固定版本及 rustfmt、clippy，並檢查實際編譯器位於該目錄後執行 Rust 格式檢查，避免複用宿主不完整的 rustup 工具鏈。完整 Rust 與簽名工具測試按變更在本地或相關按需工作流執行。
