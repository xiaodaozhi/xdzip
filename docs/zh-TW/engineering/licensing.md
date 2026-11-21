<a name="xdzip-许可证决策与第三方材料"></a>
<a name="許可證決策與第三方材料"></a>

# 授權決策與第三方資料

[English](../../engineering/licensing.md) | [简体中文](../../zh-CN/engineering/licensing.md) | 繁體中文 | [日本語](../../ja-JP/engineering/licensing.md) | [한국어](../../ko-KR/engineering/licensing.md) | [Français](../../fr-FR/engineering/licensing.md) | [Deutsch](../../de-DE/engineering/licensing.md) | [Español](../../es-ES/engineering/licensing.md) | [Português (Brasil)](../../pt-BR/engineering/licensing.md) | [Русский](../../ru-RU/engineering/licensing.md)

狀態：已採用。專案於 2026-10-11 確立以 **Apache-2.0** 授權專案原創程式碼、文件及其他原創內容，另有聲明的資料除外。先前實驗文件中的 MIT OR Apache-2.0 選擇不適用於本儲存庫。

根目錄 [LICENSE](../../../LICENSE) 提供完整 Apache License 2.0 標準全文，SPDX 識別碼為 `Apache-2.0`。貢獻指南與發行設計已同步更新。授權選擇不改變儲存庫可見性；2026-10-19 的唯讀檢查確認儲存庫公開。發行准入仍須依實際資料另行審查。

Qt 動態連結仍須逐模組確認 LGPL/GPL／商業條款；後端、圖示、字型、工具和靜態 Rust 相依套件分別審查。授權相容不代表程式碼沒有安全缺陷。所需通知、原始碼及重新連結資料與安裝資訊須隨發行成品交付。

7z、RAR、CAB 與 ISO/UDF 使用[獨立 Rust 實作](independent-rust-codecs.md)；官方 UnRAR 和 7-Zip 原始碼不進入目前產品建置或部署。固定路由與受控 I/O 見[隔離後端契約](native-codecs.md)。分析來源、開發參考工具及歷史官方後端交付各自保留原始授權；獨立實作不代表嚴格潔淨室開發，也不宣稱法律風險為零。包含官方 7-Zip 或 UnRAR 的歷史交付仍須履行原始條款、通知及修改資料義務，不能因目前移除相依套件而改標 Apache-2.0。目前產品原始碼清單為空，不免除歷史交付或獨立參考工具的義務；實驗性建置不能取代正式授權審查。

RAR 的有限線路格式對應與公開測試資料保留各自的來源、原始通知及授權紀錄，見 [RAR 實作依據](rar-format-sources.md)。移除正式產品引擎不會將這些資料重新授權為專案原創程式碼。

有限對應資料另保留[固定參考原文與來源](../../../third_party/rar-wire-mappings/README.md)，包括上游內部版本 UnRAR 7.30 beta1 的完整原始授權、下載封存檔 `unrarsrc-7.3.1.tar.gz` 及原文 SHA-256。通知收集器將這些有限資料納入實際通知雜湊，對應原始碼交付也保留獨立副本。這些資料不會重新引入官方引擎或宣告其為執行階段相依套件，也不會自動決定條款適用範圍或核准此候選版本發行。

固定原文與 SPDX `UnRAR` 授權文字在統一空白後逐詞一致。保守處理參考對應資料時，RAR crate 及其原始碼 SBOM 宣告 `Apache-2.0 AND UnRAR`，表示兩者均須遵守，而非可任選的雙重授權；該 crate 不發布至 crates.io。`UnRAR` 是 SPDX 3.28 及後續版本中的非 OSI 核准授權識別碼，較舊工具可能無法辨識。原始授權和限制仍須隨資料保留。此宣告不裁定對應資料的著作權狀態，也不改寫原始 alpha.8 清單。

平台執行階段通知、供應者輸入及授權資格依[原生執行階段來源與通知](native-runtime-materials.md)核對；通用授權文字不能取代供應者的版權通知。

Windows 維護者若需要自己的開發環境，可在符合資格時使用 Visual Studio Community 免費授權途徑。GitHub 標準代管 runner 預先安裝的 Enterprise 有獨立的代管使用授權依據；CI 建置與散布不會額外要求本機安裝 Community IDE。適用依據、Microsoft CRT／SDK 再散布條件與下游說明見 [Windows 開發工具與執行階段授權](windows-licensing.md)。這些 Microsoft 元件不適用專案 Apache-2.0 授權，一般使用者不必安裝 Visual Studio。

[實際元件歸屬流程](runtime-attribution.md)從固定 SDK 的 SPDX 擷取模組、第三方、版權及上游授權表示式，並隨 Windows/Linux 執行階段目錄交付固定來源的授權文字。上游 licenseConcluded 不代表本專案已選擇或核准該授權分支。每個候選版本都須依實際元件檢查來源、通知、對應原始碼與替換資料。已識別且適用的義務或必要資料若未解決，會阻擋該候選版本發行；上游 NOASSERTION 或機器報告標示 incomplete 本身既不能證明授權衝突，也不能視為豁免。

三張視覺圖片由生成式設計工具製作，並非 Bandizip 截圖；參考連結僅說明研究依據。生成式素材不保證獨占性。發行圖示、範例圖片和字型須分別記錄來源，從設計稿裁切不代表完成素材稽核。

第一個 ZIP 中繼資料後端曾使用 MIT 授權的 rawzip，版本與通知見[歷史准入紀錄](zip-metadata-admission.md)。目前 ZIP、TAR 與 GZIP/Deflate 使用[獨立 Rust 實作](independent-rust-codecs.md)，格式參考函式庫僅供開發測試。歷史交付與測試參考的授權義務仍然保留；通用密碼學基礎元件依[相依套件契約](dependencies.md)收集通知。專案原創內容仍使用 Apache-2.0。

Qt 固定介面文字使用官方 Noto Sans SC 衍生子集，並依語言使用 Noto Sans TC、JP、KR 和 Noto Sans 衍生子集。這些字型使用 OFL-1.1，不適用專案的 Apache-2.0。字族名稱、固定來源、版本與雜湊見[字型資訊清單](../../../specs/dependencies/ui-font.json)和[原生 TTF 匯出](../../../design/assets/fonts/README.md)；原始授權全文分別為 [CJK OFL](../../../design/assets/fonts/OFL.txt) 和 [Noto Sans OFL](../../../design/assets/fonts/NotoSans-OFL.txt)。Qt 散布套件的通知包含對應字型授權全文。產品未包含完整的上游 CJK 字型。
