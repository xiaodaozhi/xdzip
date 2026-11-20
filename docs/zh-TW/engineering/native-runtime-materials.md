<a name="xdzip-原生运行库来源与通知"></a>

# 原生執行庫來源與通知

[English](../../engineering/native-runtime-materials.md) | [简体中文](../../zh-CN/engineering/native-runtime-materials.md) | 繁體中文（台灣）

本契約涵蓋實際套件中的非 Qt 原生執行階段，並與[執行階段歸屬](runtime-attribution.md)及[對應原始碼](source-delivery.md)說明一同使用。來源版本、檔案識別、授權條件及實際履約須分別核對；供應者檔案的雜湊不能視為授權核准。`v0.1.0` 的歷史候選版本結論與補充證據記錄在 [#4](https://github.com/xiaodaozhi/xdzip/issues/4) 和 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)。後續候選版本使用新的發行 issue；證據只產生在 build/ 或 CI 成品中。

| 元件 | 原始依據與交付資料 | 審查範圍 |
| --- | --- | --- |
| Qt 6.8.2 | SDK SPDX、原始模組原始碼與修補檔、Qt 通知、LGPLv3 和 GPLv3 全文、應用程式重建與相容替換說明 | 執行階段使用 LGPL 分支；工具與第三方授權分別處理。SDK 檔案對應不等同完全符合上游建置 |
| Rust 執行階段程式碼 | Cargo.lock、實際連結輸入、執行階段相依套件候選、完整 vendor 和原始通知、Rust 標準函式庫原始碼 | 保守交付建置相依圖的通知，但不把 build/dev/proc-macro 工具誤報為已連結的執行階段程式碼 |
| Noto Sans SC、TC、JP、KR 和 Noto Sans 衍生字型 | 字型資訊清單、原始 OFL 授權全文、重新命名的字型及產生入口 | 字型不適用專案 Apache-2.0，保留修改與命名說明 |
| ICU 73.2 | 實際函式庫版本、完整上游 ICU LICENSE（含第三方通知）、來源紀錄 | 通用 MIT 文字不能取代 ICU 自身的通知；系統包裝器不能誤認為實際打包檔案 |
| Microsoft VC++ 執行階段 | 固定的官方 VC_redist 套件與 DLL 雜湊、原始授權、版本及散布清單 | 檔案保持未修改；維護者資格及再散布條件須另行審查 |
| Windows SDK 圖形編譯器函式庫 | 新套件不包含；歷史套件保留原始 SDK 授權、逐個 DLL 雜湊與來源紀錄 | 歷史散布條件須獨立審查，不會因新套件移除而改變 |
| Inno Setup 6.7.3 安裝引擎 | 固定的編譯器套件和來源提交、Inno 與 RemObjects Pascal Script 原始授權、雙語 Installer-CREDITS.txt | 保留既有安裝引擎版權介面。Pascal Script 另要求在關於頁面或文件中列出名稱與來源網址；只有一般 Inno 授權不足以滿足此要求 |

Microsoft 的[執行階段再散布說明](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files)與 [VS 2022 可再散布清單](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)將再散布權與適用的軟體授權相連。安裝程式中的授權文字不能證明維護者符合資格。Qt 的[開源義務說明](https://www.qt.io/development/open-source-lgpl-obligations)要求保留使用者取得對應原始碼、修改、重新連結和執行修改後函式庫的權利；「動態連結」不能取代其他義務。

Windows Community 免費使用途徑、託管 Enterprise 的獨立授權依據和再散布條件見 [Windows 授權說明](windows-licensing.md)。部署時在執行階段和資料附件中都納入 CRT／SDK 原始授權、來源紀錄及雙語 Windows-runtime-NOTICE.txt。編譯前，Setup 會核對原始 CRT 協議的雜湊並顯示授權頁。Microsoft DLL 不會重新授權為 Apache-2.0。

<a name="xdzip-生成与验证"></a>

## 生成與驗證

`scripts/native-runtime-sources.json` 固定 ICU 授權版本、官方網址和 SHA-256。`collect-notices.py` 在 Linux 收集完整 ICU 授權，並重新核對快取與下載內容的雜湊。目前 Windows 開發套件不包含 `opengl32sw.dll`，也不再為它收集 Mesa／LLVM 通知。若通知輸出目錄含有舊的專用資料，收集程序會拒絕重用，避免誤將其加入新候選版本。

`prepare-source-delivery.py` 納入 ICU 授權及固定來源清單。資料保留原始位元組，交付資訊清單重新計算大小和雜湊。歷史套件若含 Mesa 軟體算繪函式庫，仍需對應的歷史原始碼、通知及義務；新政策不能改寫歷史交付。

平台資料附件必須包含歸屬報告中 `license_materials_base` 所指的目錄，打包前逐一重新檢查原始授權文字的雜湊。不能因執行階段目錄其他位置有相同文字，就允許報告指向缺少的相對目錄。

Windows 部署停用 windeployqt 對軟體 OpenGL、D3D 編譯器及 DXC 的環境搜尋，不再從 Windows SDK 複製三個圖形編譯器函式庫。暫存區出現這些名稱時會拒絕；受控暫存根目錄殘留的軟體 OpenGL DLL 會移除，其他位置的同名殘留會被拒絕。

macOS 打包會在最終簽署的應用程式仍存在時產生 `deployment-audit.json`、`macos-deployment.json` 和 `runtime-attribution.json`；檔案 SBOM 使用相同的稽核與歸屬報告。Qt 原生檔案依 arm64 切片、所有區段位元組、UUID 和匯入版本比較，允許部署所需的 Qt 路徑重定位。簽章、load-command 配置及 LINKEDIT 不在此比較結論內。非原生資源按精確位元組關聯。未關聯檔案和原始 SPDX 缺口仍然存在；此流程不代表完整二進位等價。

公開套件後若發現缺少通知，不能宣稱補充資料已包含在原套件中。原版本資產與檢查碼保持不可變；修正後的套件須使用新版本並重新驗證。應明確記錄補充資料及其綁定的對象。
