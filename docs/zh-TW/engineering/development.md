<a name="xdzip-开发者准备指南"></a>

# 開發者準備指南

[English](../../engineering/development.md) | [简体中文](../../zh-CN/engineering/development.md) | 繁體中文（台灣）

從[開發與貢獻](../contributing/README.md)選擇適合這次改動的構建和驗證入口。[系統架構](../design/system-architecture.md)與[倉庫結構](repository-layout.md)說明 Qt、Rust 執行時和各格式模組的邊界。

<a name="xdzip-获取与修改"></a>

## 獲取與修改

從目前 `main` 開始，保留工作區中的其他改動。按[貢獻約定](../../../CONTRIBUTING.zh-CN.md)處理範圍與審查，按[倉庫交付指南](../../../developers/engineering/repository-hygiene.md)完成已授權的合入。改動契約時同步兩種語言及受影響的驗證用例。顏色和尺寸以 `specs/visual` 為準。

<a name="xdzip-文档构建"></a>

## 文件構建

為每種受支援的介面語言維護對應的根 README 和產品文件。預設英文頁面位於 `README.md` 和 `docs/`，譯文使用 `README.<locale>.md` 和 `docs/<locale>/`。維護者手冊使用美式英語和簡體中文。標題下放置語言切換，目前語言顯示為純文字，其餘語言連結到同一主題。簡體中文讀者頁面連結到簡體中文維護者頁面，其他讀者頁面連結到預設英文維護者頁面。新增或遷移文件時同步更新目錄、已有連結及章節錨點。

文件工具依賴 Python 3.10+、lxml、Pandoc 和 Graphviz，字型推薦系統 CJK。執行 `python3 scripts/build-design-html.py` 生成 build/docs-html 的 16 篇詳細軟體設計及單檔案版，輸出下載包在 dist/。其餘工程和協作文件從 docs/README.md 閱讀。HTML 不作為產品介面技術選擇。

執行 `python3 scripts/check-docs.py`，檢查譯文覆蓋、語言切換互鏈、讀者頁面的連結語言、Markdown 章節錨點、JSON 和設計資產。此檢查在離線狀態下執行。HTML 構建器驗證生成頁面的連結與結構。修改檢查器時執行 `python3 scripts/test-check-docs.py`。

<a name="xdzip-产品工具链准备"></a>

## 產品工具鏈準備

macOS 準備 Xcode Command Line Tools 與 Qt；Windows 準備 MSVC Build Tools、Windows SDK 與相匹配的 Qt；Linux 準備編譯器、目標 glibc 環境和 Qt Wayland/X11 依賴。目前開發版本按根目錄工具鏈檔案與 CI 鎖定，來源與許可按[工具鏈凍結表](toolchain.md)記錄，不依賴開發者機器的“最新版本”。正式系統支援與維護方式見[支援政策](support-policy.md)，具體來源許可複核使用發行許可複核 issue。

從乾淨 checkout 構建與執行的命令見[Qt 開發應用](../../../apps/desktop/README.md)，包含 CMake 開關、macOS 架構和部署目標、離屏測試邊界。構建或安裝邏輯變更應複驗空格／中文路徑。目前個人專案不要求付費簽名身份；本地 DMG 的 ad-hoc 完整性與 CI 不部署 Mac 應用的邊界見[構建指南](developer-build.md)。

<a name="xdzip-独立工程探针"></a>

## 獨立工程探針

[工具鏈探針](../../../tools/toolchain-probe/README.md) 提供 Qt Quick 獨立視窗的最小構建和 CTest 命令，不構成桌面產品。版本與測試邊界見探針 README，實際結論在 [WP-01 issue](https://github.com/xiaodaozhi/xdzip/issues/1)保留。

[統一 CMake/Cargo 橋接探針](../../../tools/bridge-probe/README.md) 已提供根構建入口、鎖定依賴與生命週期檢查。依賴記錄見 [cxx 診斷准入](cxx-admission.md)。
