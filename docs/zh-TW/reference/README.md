<a name="xdzip-产品设计与技术参考"></a>

# 產品設計與技術參考

[English](../../reference/README.md) | [简体中文](../../zh-CN/reference/README.md) | 繁體中文（台灣）

從[系統架構](../design/system-architecture.md)瞭解桌面與 CLI 的實際執行鏈路，再按下方主題查詢說明相應行為的契約。各格式能做什麼可速查[格式支援表](../../../README.zh-TW.md#xdzip-%E6%A0%BC%E5%BC%8F%E6%94%AF%E6%8C%81)。使用步驟和開發環境見[使用者手冊](../user-guide/README.md)、[CLI 示例](../integration/README.md)與[貢獻指南](../contributing/README.md)。

<a name="xdzip-架构与接口"></a>

## 架構與介面

- [設計總覽](../design/index.md)
- [產品需求與操作邊界](../design/product-requirements.md)
- [領域模型、條目身份與來源版本](../design/domain-data.md)
- [Qt/Rust 命令與事件橋接](../design/qt-rust-bridge.md)
- [任務排程、進度、取消與退出](../design/task-scheduling.md)
- [效能預算與驗證](../design/performance.md)
- [術語與實現限制](../design/glossary.md)
- [需求與驗證追蹤](../design/requirements-traceability.md)
- [架構決策](../design/architecture-decisions.md)
- [從原型到桌面產品與 Rust 執行時的演進](../design/architecture-evolution.md)
- [倉庫目錄與依賴](../engineering/repository-layout.md)
- [介面契約與檢查](../engineering/contracts.md)
- [產品執行時與 Qt 生命週期](../engineering/product-runtime.md)
- [應用會話與退出契約](../engineering/application-sessions.md)
- [cxx 橋接依賴依據](../engineering/cxx-admission.md)
- [格式工作程序的宿主日曆與編碼服務](../engineering/host-platform-services.md)

<a name="xdzip-归档读写"></a>

## 歸檔讀寫

- [檔案授權、暫存與提交](../design/file-security.md)
- [只讀瀏覽實現契約](../engineering/readonly-browser.md)
- [ZIP 只讀瀏覽契約](../engineering/zip-readonly-browsing.md)
- [大歸檔索引與按需瀏覽](../engineering/large-index.md)
- [分卷讀取與臨時開啟](../engineering/archive-opening.md)
- [檔名編碼自適應契約](../engineering/filename-encoding.md)
- [主動內容預覽](../engineering/content-preview.md)
- [解壓預檢：後設資料階段](../engineering/extraction-preflight.md)
- [安全 ZIP 解壓實現](../engineering/zip-safe-extraction.md)
- [壓縮檔完整性檢查](../engineering/archive-checking.md)
- [開啟條目、雙向拖放與歸檔修改](../engineering/archive-transfers.md)
- [壓縮檔密碼與加密目錄開啟](../engineering/password-extraction.md)
- [ZIP 建立實現契約](../engineering/zip-creation.md)
- [ISO9660 與 Joliet 建立](../engineering/iso-creation.md)
- [自有 Windows ZIP 自解壓包](../engineering/sfx-creation.md)

<a name="xdzip-格式能力与实现"></a>

## 格式能力與實現

- [內建格式能力、建立配置與檢查強度](../engineering/format-registry.md)
- [格式適配與外部 SDK 試驗設計](../design/codec-extension-sdk.md)
- [外部只讀 SDK 試驗與可執行範圍](../engineering/sdk-trial.md)
- [隔離格式工作程序契約](../engineering/native-codecs.md)
- [自研 Rust 格式與演算法契約](../engineering/independent-rust-codecs.md)
- [演算法輔助工作計量](../engineering/auxiliary-decoding-work.md)
- [ZIP 後設資料依賴准入](../engineering/zip-metadata-admission.md)
- [ZIP 解壓與輸出依賴准入](../engineering/zip-extraction-admission.md)
- [7z 格式來源與獨立實現依據](../engineering/sevenzip-format-sources.md)
- [RAR 獨立實現依據與相容邊界](../engineering/rar-format-sources.md)
- [CAB 格式來源與實現依據](../engineering/cab-format-sources.md)
- [TAR/GZIP 實現契約](../engineering/tar-gzip.md)
- [單檔案壓縮流與 ZIP 衍生包](../engineering/single-file-streams.md)
- [LZ4 壓縮流契約](../engineering/lz4-streams.md)
- [Zstandard 壓縮流契約](../engineering/zstandard-streams.md)
- [Unix compress Z 契約](../engineering/compress-streams.md)
- [普通 Brotli 流契約](../engineering/brotli-streams.md)
- [LHA 資料與演算法契約](../engineering/lha-containers.md)
- [PMA 容器與 PMarc 方法](../engineering/pma-streams.md)
- [ARJ 壓縮流與容器准入](../engineering/arj-streams.md)
- [ARC 容器與壓縮流配置](../engineering/arc-streams.md)
- [BH 容器與准入的 BlakHole 配置](../engineering/bh-containers.md)
- [ACE、ALZ 與 EGG 的資料和許可門檻](../engineering/ace-alz-egg-format-sources.md)
- [AES Crypt 只讀封裝](../engineering/aescrypt-envelopes.md)
- [DEB 容器契約](../engineering/deb-containers.md)
- [ASAR 容器契約](../engineering/asar-containers.md)
- [UU、UUE 與 XXE 文字封裝](../engineering/text-envelopes.md)
- [WIM、MSI 和安裝程式格式的資料與許可](../engineering/wim-msi-installer-format-sources.md)
- [磁碟映象與其他歸檔格式的資料和許可](../engineering/disk-image-archive-format-sources.md)
- [Bandizip 格式清單與 xdzip 可行性評估](../engineering/bandizip-format-assessment.md)

<a name="xdzip-桌面交互与视觉设计"></a>

## 桌面互動與視覺設計

- [Qt Quick 檢視與控制器職責](../design/qt-quick-ui.md)
- [桌面視覺規範](../visual/README.md)
- [介面設計規範](../visual/design-standard.md)
- [介面設計與修正流程](../visual/design-workflow.md)
- [檔案關聯圖示](../visual/file-association-icons.md)
- [HTML 視覺一致性複核](../visual/html-fidelity.md)
- [補充互動與文案規範](../ux/interaction-spec.md)
- [主視窗功能、元件與互動契約](../ux/main-window-components.md)
- [目錄樹設計與接入](../ux/directory-tree.md)
- [主視窗歡迎頁](../ux/welcome.md)
- [歡迎頁讀取遮罩與建立完成行為](../ux/welcome-opening.md)
- [密碼解壓互動](../ux/password-extraction.md)
- [任務視窗與完整性檢查](../ux/task-windows.md)
- [設定視窗與介面語言](../ux/settings-localization.md)
- [簡體中文介面術語](../ux/chinese-terminology.md)
- [系統選單與托盤](../ux/desktop-integration.md)
- [平臺視窗與標題欄規範](../ux/platform-windows.md)
- [高保真狀態與實現契約](../ux/hifi-state-spec.md)
- [v0.1 互動原型與功能對映](../ux/prototype-spec.md)

<a name="xdzip-平台集成发布与信任"></a>

## 平臺整合、釋出與信任

- [安裝器與檔案關聯](../engineering/installers-associations.md)
- [系統支援與維護政策](../engineering/support-policy.md)
- [檔案管理器操作與獨立任務宿主](../engineering/shell-actions.md)
- [使用者主動更新契約](../engineering/software-update.md)
- [資料生命週期與診斷](../engineering/privacy.md)
- [產品版本與構建身份](../engineering/versioning.md)
- [GitHub Release 協議 v1](../engineering/release-protocol.md)
- [本地 macOS 開發 DMG](../engineering/macos-deployment.md)
- [Linux 獨立執行目錄與驗證](../engineering/linux-deployment.md)
- [部署依賴核驗與 SBOM](../engineering/deployment-audit.md)
- [實際元件歸屬與許可材料](../engineering/runtime-attribution.md)
- [原生執行庫來源與通知](../engineering/native-runtime-materials.md)
- [Qt SDK 與原始碼對應核驗](../engineering/qt-sdk-correspondence.md)
- [Windows 開發工具與執行庫許可](../engineering/windows-licensing.md)
- [許可證決策與第三方材料](../engineering/licensing.md)
- [依賴與格式後端准入](../engineering/dependencies.md)
- [對應原始碼與重建材料](../engineering/source-delivery.md)
- [從 CLDR 重建 ICU 73.2 資料](../engineering/icu-cldr-rebuild.md)

<a name="xdzip-构建与命令行参考"></a>

## 構建與命令列參考

- [構建、部署與發行設計](../design/build-release.md)
- [編碼規範與驗證層次](../design/quality-engineering.md)
- [開發者準備指南](../engineering/development.md)
- [開發構建與平臺驗證](../engineering/developer-build.md)
- [工具鏈與平臺凍結表](../engineering/toolchain.md)
- [CLI 命令、JSON、安裝與更新回執](../engineering/command-line.md)
