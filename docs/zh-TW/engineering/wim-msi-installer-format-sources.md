<a name="xdzip-wimmsi-和安装程序格式的资料与许可"></a>

# WIM、MSI 和安裝程式格式的資料與許可

[English](../../engineering/wim-msi-installer-format-sources.md) | [简体中文](../../zh-CN/engineering/wim-msi-installer-format-sources.md) | 繁體中文（台灣）

本文介紹 WIM、MSI 和安裝程式格式的技術資料、許可限制及支援這些格式所需的條件。參考資料保留在 `build/toolchain/references` 或連結到公開規範，不作為產品依賴。

<a name="xdzip-wim"></a>

## WIM

- 參考實現：`ebiggers/wimlib`，提交 `cd5e231c348c255ae5088873b5a66ee0eb96fa07`。
- 倉庫對專案提供 GPLv3+，對庫部分提供有條件的 LGPL-2.1+ 選擇。LGPL 選擇取決於 `libntfs-3g` 等第三方依賴沒有把最終組合強制到 GPL 條件。
- WIM 是映象容器，不是普通歸檔。受控讀取器需要處理資源表、後設資料流、分塊資源、Xpress/LZX/LZMS 壓縮、完整性表、硬連結後設資料和映象選擇，同時不能掛載或恢復安全許可權。

參考實現可用於線格式和樣本研究，但按目前不引入第三方歸檔執行時的政策不能連結進 xdzip。WIM **按產品決定明確不支援**：專案不投入其所需的映象專用讀取器、解壓放大模型、完整性資源、硬連結語義和完整語料庫。除非明確重新作出產品決定，不允許增加 registry、MIME、runtime 或 CLI 入口。

<a name="xdzip-compound-file-与-msi"></a>

## Compound File 與 MSI

- Microsoft [MS-CFB] 是主要公開規範：https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b 。頁面釋出 PDF/DOCX 版本，幷包含允許實現所述技術的智慧財產權說明，專利許可仍受適用 Microsoft 計劃約束。
- CFB 讀取器需要在解釋 MSI 之前完成扇區、FAT/DIFAT、目錄紅黑樹、mini-stream 分配、流長度邊界和環檢測。

獨立 CFB 後設資料讀取器技術上可行，目前已實現為 `xdzip-codec-cfb` 只讀切片。它會在釋出儲存和流記錄前驗證 CFB 3.0/4.0 頭部，包括 512 位元組和 4096 位元組扇區基址、FAT/DIFAT 鏈、目錄樹、mini-stream 分配、流長度邊界和環。只有不可變後設資料快照匹配後才允許複製原始流。MSI 表、內嵌 CAB、自定義動作和變換仍不在契約中，不執行安裝器、自定義動作，不解釋安裝包指令碼，也不回寫，`.msi` 字尾不能授予執行許可權。codec 和 runtime 的夾具及執行證據均為自動化測試。

<a name="xdzip-nsis-与其他-sfx"></a>

## NSIS 與其他 SFX

- NSIS 參考原始碼：`kichik/nsis`，提交 `a96303eae3115221471bb127fa916a13d8bf9ecd`。
- `COPYRIGHT` 記錄大部分原始碼和 zlib 壓縮模組使用 zlib/libpng，bzip2 模組使用 bzip2 許可，LZMA 模組使用 Common Public License 1.0。
- NSIS 檔案是帶指令碼和內嵌載荷語義的可執行安裝器。解析器必須停留在執行邊界之下，限制頭部和載荷跨度，並把壓縮方法與安裝器行為分開。
- 公開原始碼定義了固定的 `firstheader` 記錄：`flags`、`0xDEADBEEF`、`NullsoftInstallSystem` 簽名字、`length_of_header` 和 `length_of_all_following_data`。靜態讀取器可以在不開啟 PE stub 的情況下定位並驗證它，但仍需要獨立的壓縮頭部模型和完整自有安裝器樣本。

NSIS **已按只讀靜態後設資料配置准入**。codec 只掃描有界 PE overlay 中按 512 位元組對齊的記錄，接受已記錄的兩個 `firstheader` 魔數，限制 flags 和長度，並要求宣告的後續區間落在輸入的物理範圍內，然後釋出一個 `NSIS/firstheader` 後設資料行。不解壓安裝器頭部，也不暴露任何載荷位元組。登記的 `.exe` 故意不參與拖放和系統關聯，仍以內容識別為準。解析器獨立於 Qt 和安裝器執行 API，絕不執行 PE stub、指令碼、自定義頁面、外掛或提取出的載荷。其他 SFX 仍只能在內部歸檔透過內容驗證時作為別名處理。

<a name="xdzip-准入门槛"></a>

## 准入門檻

1. 使用參考工具前固定原始碼提交和許可文字雜湊。
2. 每個准入容器和壓縮方法都要有完整、可再分發樣本，只有頭部樣本不夠。
3. CFB、WIM、NSIS 解析必須獨立於 Qt，也不能呼叫安裝、掛載、登錄檔或指令碼執行 API。
4. 只有在準確物理身份、重複項、環邊界、累計輸入／工作／輸出限額、取消、來源變化和終止失敗測試透過後，才接入 runtime、CLI 和桌面入口。
5. GPL/LGPL/CPL 參考元件繼續保留在 `build/toolchain`，除非另有依賴與許可決策明確改變產品政策。
