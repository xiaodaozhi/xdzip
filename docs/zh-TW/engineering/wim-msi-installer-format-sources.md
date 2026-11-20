<a name="xdzip-wimmsi-和安装程序格式的资料与许可"></a>
<a name="wimmsi-和安裝程式格式的資料與許可"></a>

# WIM、MSI 和安裝程式格式的參考資料與授權

[English](../../engineering/wim-msi-installer-format-sources.md) | [简体中文](../../zh-CN/engineering/wim-msi-installer-format-sources.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/wim-msi-installer-format-sources.md) | [한국어](../../ko-KR/engineering/wim-msi-installer-format-sources.md) | [Français](../../fr-FR/engineering/wim-msi-installer-format-sources.md) | [Deutsch](../../de-DE/engineering/wim-msi-installer-format-sources.md) | [Español](../../es-ES/engineering/wim-msi-installer-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/wim-msi-installer-format-sources.md) | [Русский](../../ru-RU/engineering/wim-msi-installer-format-sources.md)

本文介紹 WIM、MSI 和安裝程式格式的技術參考資料、授權限制，以及支援這些格式所需的條件。參考資料保留在 `build/toolchain/references` 或連結至公開規範，均非產品依賴。

<a name="xdzip-wim"></a>

## WIM

- 參考實作：`ebiggers/wimlib`，提交版本 `cd5e231c348c255ae5088873b5a66ee0eb96fa07`。
- 專案使用 GPLv3+，函式庫部分另有附帶條件的 LGPL-2.1+ 選擇。選擇 LGPL 的條件包括 `libntfs-3g` 等第三方依賴不得使最終組合必須遵循 GPL 條款。
- WIM 是映像容器，而非一般壓縮檔。受限制的讀取器必須處理資源表、中繼資料串流、分塊資源、Xpress/LZX/LZMS 壓縮、完整性表、硬連結中繼資料及映像選擇，且不得掛載映像或還原安全權限。

參考實作可供格式和樣本研究，但依目前不引入第三方執行期依賴的政策，不能連結至 xdzip。產品決定**不支援 WIM**：專案不承擔其所需的映像專用讀取器、解壓縮放大模型、完整性資源、硬連結語義及完整測試樣本。除非明確重新評估此產品決定，否則不得新增登錄表、MIME、執行環境或 CLI 入口。

<a name="xdzip-compound-file-与-msi"></a>

## Compound File 與 MSI

- Microsoft [MS-CFB] 是主要公開規範：https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b 。該頁提供 PDF/DOCX 修訂版本與智慧財產權聲明，允許實作所述技術；專利授權仍受適用的 Microsoft 方案約束。
- 在解讀 MSI 內容之前，CFB 讀取器必須處理磁區和 FAT/DIFAT 配置、目錄紅黑樹遍歷、mini-stream 配置、串流大小限制及循環偵測。

獨立 CFB 中繼資料讀取器在技術上可行，目前已實作為 `xdzip-codec-cfb` 的唯讀部分。它會在公開儲存物件和串流記錄前驗證 CFB 3.0/4.0 標頭，包括 512 位元組和 4096 位元組的磁區基準、FAT/DIFAT 鏈、目錄樹、mini-stream 配置、串流大小限制和循環。只有不可變的中繼資料快照相符後，才能複製原始串流。MSI 資料表、內嵌 cabinet、自訂動作和轉換仍不在契約範圍內：不執行安裝程式、不解讀自訂動作、不安裝套件，也不回寫；`.msi` 副檔名不會授予執行權限。編解碼器和執行環境測試已自動化驗證測試樣本與執行行為。

<a name="xdzip-nsis-与其他-sfx"></a>

## NSIS 與其他 SFX

- NSIS 參考原始碼：`kichik/nsis`，提交版本 `a96303eae3115221471bb127fa916a13d8bf9ecd`。
- `COPYRIGHT` 記錄大部分原始碼和 zlib 壓縮模組使用 zlib/libpng 授權，bzip2 模組使用 bzip2 授權，LZMA 模組使用 Common Public License 1.0。
- NSIS 檔案是具有指令碼和內嵌資料行為的可執行安裝程式。解析器必須保持在執行邊界之外，限制標頭與資料範圍，並將壓縮方法與安裝程式行為分開。
- 公開原始碼定義了固定的 `firstheader` 記錄：`flags`、`0xDEADBEEF`、`NullsoftInstallSystem` 簽章字、`length_of_header` 和 `length_of_all_following_data`。靜態讀取器可在不開啟 PE 啟動程式的情況下定位並驗證該記錄，但仍需要獨立的壓縮標頭模型和完整的專案自有安裝程式樣本。

NSIS **已納入僅限靜態中繼資料的支援範圍**。編解碼器只掃描受限 PE overlay 中按 512 位元組對齊的記錄，接受已記錄的兩個 `firstheader` 魔數，限制旗標和長度，並要求宣告的後續區間落在實際輸入範圍內，然後公開一筆 `NSIS/firstheader` 中繼資料。不解壓縮安裝程式標頭，也不公開任何內嵌資料位元組。登錄的 `.exe` 刻意不列為拖放候選或系統關聯，以內容偵測結果為準。解析器獨立於 Qt 和安裝程式執行 API，絕不執行 PE 啟動程式、指令碼、自訂頁面、外掛程式或解出的資料。其他 SFX 格式仍僅在內部壓縮檔通過內容驗證時作為別名處理。

<a name="xdzip-准入门槛"></a>
<a name="准入門檻"></a>

## 納入支援的條件

1. 使用參考工具前，固定原始碼版本及授權文字的雜湊值。
2. 每個納入支援的容器和壓縮方法都必須有完整、可重新散布的測試樣本；只有標頭的樣本不足以驗證。
3. CFB、WIM、NSIS 解析必須獨立於 Qt，也不得依賴安裝程式、掛載、登錄或指令碼執行 API。
4. 只有在精確實體識別、重複項目、循環限制、累計輸入／工作／輸出限額、取消、來源變更與最終失敗測試通過後，才能新增執行環境、CLI 和桌面入口。
5. GPL/LGPL/CPL 參考元件須保留在 `build/toolchain`，除非另有依賴與授權決策明確改變產品政策。
