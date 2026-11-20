<a name="xdzip-构建部署与发行架构"></a>

# 構建、部署與發行架構

[English](../../design/build-release.md) | [简体中文](../../zh-CN/design/build-release.md) | 繁體中文（台灣） | [日本語](../../ja-JP/design/build-release.md) | [한국어](../../ko-KR/design/build-release.md) | [Français](../../fr-FR/design/build-release.md) | [Deutsch](../../de-DE/design/build-release.md) | [Español](../../es-ES/design/build-release.md) | [Português (Brasil)](../../pt-BR/design/build-release.md) | [Русский](../../ru-RU/design/build-release.md)

<a name="xdzip-构建与打包输入"></a>

## 構建與打包輸入

CMake 構建 Qt 桌面、C++ 適配層及原生助手。Cargo 構建 Rust 執行時並生成要連結的 CXX 橋接。獨立 CLI 使用自己的構建入口，無需 Qt。`scripts/desktop.py` 與 `scripts/cli.py` 組織構建和測試。目前原始碼鎖定 Rust 1.99.0 和 Qt 6.8.2 SDK。比較結果時要同時核對構建身份、目標架構、鎖檔案和工具鏈來源。[工具鏈](../engineering/toolchain.md)、[開發構建指南](../engineering/developer-build.md)

桌面部署各平臺包需要的 Qt 庫、QML 模組、系統資源和隨包助手。專用圖片程序使用 Qt 進行圖片解碼及靜態 SVG 渲染。CLI 包攜帶所需的歸檔工作程序。構建配置、打包清單和部署檢查由指令碼和平臺契約維護。單個可執行檔案的大小無法說明完整依賴或安裝體積。[部署與依賴審計](../engineering/deployment-audit.md)、[CLI 分發](../engineering/command-line.md)

<a name="xdzip-当前分发渠道"></a>

## 目前分發渠道

| 目標 | 安裝包與信任邊界 |
| --- | --- |
| macOS Apple Silicon | GitHub 提供 DMG 和應用包 ZIP。目標基線為 macOS 14，應用包使用 ad-hoc 完整性簽名。安裝和更新行為見平臺說明。 |
| Windows 11 x86_64 | GitHub 提供未簽名的每使用者安裝程式及便攜 ZIP。Microsoft Store 管理 MSIX 的包身份、簽名和更新渠道。 |
| Ubuntu 24.04 LTS x86_64 | GitHub 提供執行時目錄壓縮檔和 DEB。支援基線包含 X11 及 glibc 2.39 或更高版本。 |
| 獨立 CLI | 單獨分發，擁有自己的包與更新身份。桌面安裝中附帶的 CLI 遵循其所在安裝的整合和交接規則。 |

[支援政策](../engineering/support-policy.md)界定系統及架構範圍。[專案首頁](../../../README.zh-TW.md)與對應發行說明列出實際附件，[安裝程式指南](../engineering/installers-associations.md)說明檔案關聯與整合限制。

<a name="xdzip-常规检查原生构建与发行"></a>

## 常規檢查、原生構建與發行

常規 [CI 工作流](../../../.github/workflows/docs.yml)執行輕量檢查。原生 Foundation 和 Qt 檢查由獨立工作流手動執行。普通 Apple Silicon CI 執行編譯、測試和啟動驗證，無需部署可供下載的應用包。專用發行工作流按授權執行部署、簽名、打包和釋出。[發行協議](../engineering/release-protocol.md)和 [CI 策略](../../../developers/engineering/ci-policy.md)定義門檻與證據。

發行清單、校驗和、元件清單、SBOM 和許可材料描述實際交付的工件。發行工作流不生成或上傳專案原始碼壓縮檔。GitHub 自帶的標籤快照透過獨立機制提供。依賴義務依據實際隨包元件與專案的[許可記錄](../engineering/licensing.md)和[原始碼交付政策](../engineering/source-delivery.md)。

<a name="xdzip-更新与后续分发"></a>

## 更新與後續分發

更新選擇和驗證獨立於歸檔任務。符合條件的安裝版按更新契約核對可信後設資料、已驗證工件和安裝身份，並按活動任務狀態完成交接。Windows 和 Linux 使用對應的助手流程，macOS 使用 Sparkle，商店版由 Microsoft Store 提供更新。便攜與開發配置各有限制。[軟體更新](../engineering/software-update.md)

擴大架構、系統基線、包格式或商店範圍時，需要獨立的構建和驗收證據。歷史發行決策與複核觸發條件見[架構決策記錄](architecture-decisions.md)。
