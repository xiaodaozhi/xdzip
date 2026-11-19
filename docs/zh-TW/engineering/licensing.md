<a name="xdzip-许可证决策与第三方材料"></a>

# 許可證決策與第三方材料

[English](../../engineering/licensing.md) | [简体中文](../../zh-CN/engineering/licensing.md) | 繁體中文（台灣）

狀態：已採用。專案於 2026-10-11 確立 **Apache-2.0** 許可證，適用於專案原創程式碼、文件及其他原創內容，另有宣告的材料除外。此前實驗文件中的 MIT OR Apache-2.0 不適用於本倉庫的許可選擇。

根目錄 [LICENSE](../../../LICENSE) 提供完整 Apache License 2.0 標準正文，SPDX 標識為 `Apache-2.0`。貢獻指南與發行設計已同步。許可證選擇不改變倉庫可見性；2026-10-19 只讀查詢確認倉庫已公開，發行准入仍按實際材料單獨核對。

Qt 動態連結仍需逐模組確認 LGPL/GPL/商業條款；後端、圖示、字型、工具和靜態 Rust 依賴分別審查。許可證相容不代表程式碼沒有安全缺陷。所需通知、原始碼/重連結材料和安裝資訊隨發行產物交付。

7z、RAR、CAB 與 ISO/UDF 使用[獨立 Rust 實現](independent-rust-codecs.md)，官方 UnRAR、7-Zip 原始碼不進入目前產品構建或部署。固定路由與受控 I/O 見[隔離後端契約](native-codecs.md)。分析來源、開發參考工具、歷史官方後端交付分別保留原許可，獨立實現不構成嚴格潔淨室或法律風險清零宣告。包含官方 7-Zip 或 UnRAR 的歷史交付繼續履行各自原始條款、通知和修改材料義務，不能因目前退出依賴而改標 Apache-2.0。目前產品原始碼清單為空不免除歷史交付或獨立參考工具的義務，實驗構建不替代正式許可複核。

RAR 的有限線格式對映及公開測試夾具保留各自的來源、原始通知和許可記錄，見[RAR 實現依據](rar-format-sources.md)。生產引擎退出不將這些材料重新許可為專案原創程式碼。

有限對映另保留[固定參考原文與來源說明](../../../third_party/rar-wire-mappings/README.md)，包括上游內部版本 UnRAR 7.30 beta1 的原始完整許可、下載歸檔 `unrarsrc-7.3.1.tar.gz` 及原文 SHA-256。通知收集器將這些有界材料納入實際通知雜湊，對應原始碼交付也保留獨立副本；它們不是重新引入官方引擎或宣告其為執行依賴，不自動裁定條款適用範圍，也不批准本候選發行。

固定原文與 SPDX 的 `UnRAR` 許可文字統一空白後逐詞一致。為反映參考對映的保守處理，RAR crate 及其原始碼 SBOM 宣告 `Apache-2.0 AND UnRAR`，並非可任選的雙許可；該 crate 不釋出到 crates.io。`UnRAR` 是 SPDX 3.28 及以後清單中的非 OSI 批准許可標識，較舊工具可能不識別；原許可和限制仍應隨材料保留。該宣告不裁定對映的版權性質，原版 alpha.8 清單也不會因此被改寫。

平臺執行庫專屬通知、供應方輸入和許可資格的核對遵循[原生執行庫來源與通知](native-runtime-materials.md)，不以通用許可正文替代供應方版權宣告。

Windows 維護者需要自有開發環境時，可採用符合條件的 Visual Studio Community 免費許可路徑。GitHub 標準託管 runner 實際預裝的 Enterprise 有獨立的託管使用依據，CI 構建與分發不額外要求本機安裝 Community IDE。適用依據、Microsoft CRT／SDK 分發條件及下游說明見 [Windows 開發工具與執行庫許可](windows-licensing.md)。這些微軟元件不適用專案 Apache-2.0 許可，普通執行使用者無需安裝 Visual Studio。

[實際元件歸屬流程](runtime-attribution.md)從固定 SDK 的 SPDX 提取模組、第三方、版權與上游許可表示式，並隨 Windows/Linux 執行目錄交付固定來源的許可文字。上游 licenseConcluded 不代表本專案已選擇或批准對應許可分支；每個候選均須按實際元件核對來源、通知、對應原始碼和替換材料。已識別且適用的義務或必需材料若未解決，會阻斷該候選發行；上游 NOASSERTION 或機器報告 incomplete 本身不能直接推導為許可衝突，也不能當作豁免。

三張視覺圖採用生成式設計工具製作，不是 Bandizip 截圖；參考連結只用於說明研究依據。生成式資產不承諾獨佔性。正式發行的圖示、示例圖片和字型需獨立來源登記，不能直接從設計稿裁剪當作完成資產審計。

首個 ZIP 後設資料後端曾採用 MIT 許可 rawzip，版本與通知見[歷史准入記錄](zip-metadata-admission.md)。目前 ZIP、TAR 和 GZIP/Deflate 採用[獨立 Rust 實現](independent-rust-codecs.md)，格式參照庫僅用於開發測試。舊版交付與測試參照的許可義務繼續保留，通用密碼學基礎庫按[依賴契約](dependencies.md)收集通知。專案原創許可仍為 Apache-2.0。

HTML 與 Qt 固定介面文案採用官方 Noto Sans SC 的衍生子集，許可為 OFL-1.1。家族名、固定來源、版本與雜湊見 [字型材料清單](../../../specs/dependencies/ui-font.json)和[原生 TTF 匯出](../../../design/assets/fonts/README.md)；完整許可為 [OFL.txt](../../../design/assets/fonts/OFL.txt)。字型及其衍生字型不適用專案 Apache-2.0；離線 HTML 包含字型二進位制與完整通知，Qt 分發包的 notices 包含同一 OFL 原文。產品未引入完整上游 CJK 字型。
