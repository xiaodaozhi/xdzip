<a name="xdzip-原生运行库来源与通知"></a>

# 原生執行庫來源與通知

[English](../../engineering/native-runtime-materials.md) | [简体中文](../../zh-CN/engineering/native-runtime-materials.md) | 繁體中文（台灣）

本契約覆蓋實際包中的非 Qt 原生執行庫，與[執行時歸屬](runtime-attribution.md)及[對應原始碼](source-delivery.md)共同使用。源版本、檔案身份、許可條件和實際履約分別核對，不能將供應方檔案雜湊當作許可批准。`v0.1.0` 的歷史候選結論和補證記錄在 [#4](https://github.com/xiaodaozhi/xdzip/issues/4) 與 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)；未來候選使用新的發行 issue，證據只生成到 build/ 或 CI 工件。

| 元件 | 原始依據與交付材料 | 核對邊界 |
| --- | --- | --- |
| Qt 6.8.2 | SDK SPDX、模組原始原始碼和補丁、Qt 通知、LGPLv3 及 GPLv3 原文、應用重建和相容替換說明 | 執行時採用 LGPL 分支，工具許可與第三方許可分別保留；SDK 檔案對應不等於完整上游構建等價 |
| Rust 執行時程式碼 | Cargo.lock、實際連結輸入、執行依賴候選、完整 vendor 和原始通知、Rust 標準庫原始碼 | 保守交付構建圖的通知，不把 build/dev/proc-macro 誤報為已連結執行時程式碼 |
| Noto Sans SC 衍生字型 | 字型 manifest、原始 OFL、改名後的字型和生成入口 | 字型不適用專案 Apache-2.0，保留修改及命名說明 |
| ICU 73.2 | 實際庫版本、完整 ICU 上游 LICENSE（含第三方通知）、來源記錄 | 不以通用 MIT 文字替代 ICU 自帶通知，不把系統包裝器誤當實際打包檔案 |
| Microsoft VC++ runtime | 固定官方 VC_redist 包及 DLL 雜湊、原始許可、版本及分發清單 | 逐檔案保持未修改；維護者的適用許可資格及分發條件須另行核對 |
| Windows SDK 圖形編譯庫 | 新包不包含；歷史包保留原始 SDK 許可、逐 DLL 雜湊和來源記錄 | 歷史分發條件獨立複核，不因新包移除而改變 |
| Inno Setup 6.7.3 安裝引擎 | 固定編譯器包及來源提交、Inno 原始許可、RemObjects Pascal Script 原始許可和雙語 Installer-CREDITS.txt | 保留安裝引擎既有版權介面；Pascal Script 另有在 About 或文件中顯示名稱及來源地址的要求，不能只交付 Inno 總許可 |

Microsoft 的[執行庫分發說明](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files)與 [VS 2022 可分發清單](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)將分發權與適用軟體許可關聯。安裝許可文字不是維護者已滿足資格的證明。Qt 的[開源義務說明](https://www.qt.io/development/open-source-lgpl-obligations)要求保留使用者取得對應原始碼、修改、重新連結和執行修改庫的權利；不能以“動態連結”代替其他義務。

Windows 的 Community 免費路徑、託管 Enterprise 的獨立依據和再分發條件見 [Windows 許可說明](windows-licensing.md)。部署將 CRT／SDK 原始許可、來源記錄及雙語 Windows-runtime-NOTICE.txt 同時納入執行目錄和 materials，安裝器編譯前核對原始 CRT 協議雜湊並展示許可頁，不將微軟 DLL 納入 Apache-2.0 授權。

<a name="xdzip-生成与验证"></a>

## 生成與驗證

`scripts/native-runtime-sources.json` 固定 ICU 許可的版本、官方 URL 和 SHA-256。`collect-notices.py` 在 Linux 收集 ICU 完整許可，快取和下載均重新核對雜湊。目前 Windows 開發包不攜帶 `opengl32sw.dll`，不再為它收集 Mesa／LLVM 原始通知；如通知輸出目錄含舊的專用材料，收集過程拒絕複用，避免把舊材料誤寫入新候選。

`prepare-source-delivery.py` 附帶 ICU 許可及固定來源清單。材料保持原始位元組，交付總清單重新計算大小和雜湊。歷史包如包含 Mesa 軟體渲染庫，仍須保留對應歷史原始碼、通知和適用義務，不能據新策略改寫歷史交付。

平臺 materials 附件必須包含歸屬報告 `license_materials_base` 引用的目錄，逐個複核許可原文雜湊後再打包；不能僅因執行目錄中另有相同原文，就交付指向缺失相對目錄的報告。

Windows 部署禁用 windeployqt 對軟體 OpenGL、D3D 編譯器和 DXC 的環境搜尋，不再從 Windows SDK 複製三個圖形編譯庫；暫存中出現這些名稱時拒絕部署。受控暫存中殘留的根目錄軟體 OpenGL DLL 會被清除，其他同名殘留會被拒絕。

macOS 打包在最終簽名應用仍存在時輸出 `deployment-audit.json`、`macos-deployment.json`、`runtime-attribution.json`，檔案 SBOM 使用同一審計及歸屬報告。Qt 原生檔案按 arm64 分片、全部 section 位元組、UUID 和匯入版本比較，允許部署需要的 Qt 路徑重定位；簽名、load-command 佈局及 LINKEDIT 不屬於該比較結論。非原生資源按精確位元組關聯。未關聯檔案和原始 SPDX 缺口繼續保留，不能將這個過程宣稱為完整二進位制等價。

公開包發現缺失通知後，補充材料不能改寫為“原包已包含”。原版本資產及校驗和保持不可變，修訂後的包必須使用新版本並重新驗證；補充材料及其繫結物件應明確登記。
