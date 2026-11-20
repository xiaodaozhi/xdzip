<a name="xdzip-bandizip-格式清单与-xdzip-可行性评估"></a>

# Bandizip 格式清單與 xdzip 可行性評估

[English](../../engineering/bandizip-format-assessment.md) | [简体中文](../../zh-CN/engineering/bandizip-format-assessment.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/bandizip-format-assessment.md) | [한국어](../../ko-KR/engineering/bandizip-format-assessment.md) | [Français](../../fr-FR/engineering/bandizip-format-assessment.md) | [Deutsch](../../de-DE/engineering/bandizip-format-assessment.md) | [Español](../../es-ES/engineering/bandizip-format-assessment.md) | [Português (Brasil)](../../pt-BR/engineering/bandizip-format-assessment.md) | [Русский](../../ru-RU/engineering/bandizip-format-assessment.md)

本文比較 Bandizip 列出的壓縮格式與 xdzip 的實現範圍，並說明需要透過原始碼、樣本和自動化驗證的能力邊界。清單來自 Bandizip 產品頁及格式幫助頁，核對日期為 2026-11-15：

- [Bandizip 產品頁](https://www.bandisoft.com/bandizip/) 列出：`7Z`、`ACE`、`AES`、`ALZ`、`ARJ`、`BH`、`BIN`、`BR`、`BZ`、`BZ2`、`CAB`、`Compound(MSI)`、`DAA(1.0)`、`DEB`、`EGG`、`GZ`、`IMG`、`ISO`、`ISZ`、`LHA`、`LZ`、`LZH`、`LZMA`、`PMA`、`RAR`、`RAR5`、`SFX(EXE)`、`TAR`、`TBZ/TBZ2`、`TGZ`、`TLZ`、`TXZ`、`UDF`、`WIM`、`XPI`、`XZ`、`Z`、`ZIP`、`ZIPX`、`ZPAQ`、`PEA`、`UU`、`UUE`、`XXE`、`ASAR`、`ZSTD`、`NSIS` 和 `LZ4`。
- 同一頁面說明 Bandizip 可建立 `ZIP`、`7Z`、`ZIPX`、`EXE (SFX)`、`TAR`、`TGZ`、`LZH`、`ISO`、`GZ` 和 `XZ`，較長的清單主要是解壓清單。
- [Bandizip ALZ 和 EGG 幫助](https://www.bandisoft.com/bandizip/help/how-to-open-alz-and-egg-files/) 說明 ALZ 的 Deflate、混淆 Deflate、BZip2 與 ZipCrypto，以及 EGG 的 Deflate、BZip2、LZMA、AZO、AES/LEA 加密、固實和分卷。
- [Bandizip ZPAQ 幫助](https://cn.bandisoft.com/bandizip/help/what-is-a-zpaq/) 將 ZPAQ 說明為獨立的日誌式歸檔格式，並列出 Bandizip 可處理的壓縮方法。
- [磁碟映像及其他歸檔格式資料](disk-image-archive-format-sources.md)記錄了 DAA、ISZ、ZPAQ、PEA、BOND 和 J2J 的公開資料、許可、樣本和淨室檢查。BOND 與 J2J 尚未確認是目前官方 Bandizip 歸檔條目，只保留為歷史線索。

<a name="xdzip-xdzip-当前结论"></a>

## xdzip 目前結論

表格中的含義如下。“已准入”表示目前登記、runtime、CLI 和桌面路徑已有自動化證據，且只承諾表中配置。“別名/配置”表示透過現有容器入口准入，仍以內容驗證為準。“候選”表示後續入口，目前不宣告解碼。“阻塞”表示在公開、可獨立測試的線格式規範以及相容實作或自有演算法可用前，不能負責任地准入。“明確不支援”表示專案已經明確決定不支援該格式，除非重新作出產品決定，不登記入口或探測器。

| Bandizip 名稱 | xdzip 狀態 | 目前可負責的範圍或原因 |
| --- | --- | --- |
| 7Z | 已准入 | 自研 Rust 7z 後端，覆蓋已宣告的密碼、固實和分卷配置。 |
| ACE | 明確不支援 | 產品決定永久放棄 ACE，不增加解碼器、探測器、字尾、MIME 或關聯入口。 |
| AES | 已准入 | AES Crypt 0–3 版、密碼和 HMAC 配置；這是 `.aes` 封裝，不是任意加密歸檔。 |
| ALZ | 已准入 | 只讀普通單卷 Store 和原始 Deflate，驗證準確尺寸與 CRC-32；改造 BZip2、ZipCrypto 和分卷仍不在准入範圍。 |
| ARJ | 已准入 | 只讀 Store 和方法 1–4，完整頭部、尺寸和成員 CRC-32。 |
| BH | 已准入 | 只讀 `BH 05 07` 普通成員、方法 8 raw Deflate、準確尺寸和 CRC-32，不含 Store、加密、分卷及其他方法。 |
| BIN | 候選 | 副檔名可能表示磁碟、可執行檔案或普通資料，只能在驗證內部格式後處理，不能註冊通用 BIN 歸檔解碼。 |
| BR | 已准入 | 有界 Brotli 流和 TAR.BR。 |
| BZ / BZ2 | 已准入 | BZip2 單檔案流及登記的 TAR.BZ2 別名。 |
| CAB | 已准入 | 自研 Rust CAB 後端，覆蓋 Store、MSZip、Quantum、LZX 和已授權卷集合。 |
| Compound(MSI) | 已准入 | 對經內容驗證的 `.msi` 提供只讀 CFB 結構、儲存/流名稱和有界原始流複製。MSI 表、內嵌 CAB 解釋、變換、自定義動作和安裝仍不在能力範圍。 |
| DAA(1.0) | 排除 | 按產品決定永久不支援，不準入探測器、codec、登記、讀取器或寫入器。 |
| DEB | 已准入 | 普通 `ar` 封裝、control/data 檢視及已准入壓縮成員。 |
| EGG | 已准入 | 只讀無加密、非固實普通單卷 Store 和原始 Deflate，驗證準確塊尺寸與 CRC-32；AZO、BZip2、LZMA、加密、分卷和其他配置仍不在範圍。 |
| GZ | 已准入 | GZIP 成員，包括串聯流和 TAR.GZ。 |
| IMG | 候選 | 磁碟映像副檔名含義不唯一，ISO/UDF 按內容准入不代表通用 IMG 支援。 |
| ISO | 已准入 | 支援 ISO9660、Joliet、Rock Ridge 讀取，以及普通 ISO9660 + Joliet 資料映像建立。 |
| ISZ | 明確不支援 | 產品決定放棄 ISZ，不增加探測器、登記項或金鑰處理。 |
| LHA / LZH | 已准入 | LHA 0–3 級頭部、Store、LH1–LH7、LZS/LZ5，並支援 LH5 建立。 |
| LZ | 已准入 | Lzip v1 和 TAR.LZ。 |
| LZMA | 已准入 | LZMA Alone 和 TAR.LZMA。 |
| PMA | 已准入 | PM0/PM1/PM2 只讀配置、CP/M 結構和 CRC 檢查。 |
| RAR / RAR5 | 已准入 | 自研 Rust RAR 後端，覆蓋宣告範圍內的舊版、RAR5、RAR7 擴充功能、密碼、固實和分卷。 |
| SFX(EXE) | 別名/配置 | ZIP SFX 前綴由 ZIP 讀取器驗證，不允許通用可執行檔提取或執行；自有 Windows x64 Store/Deflate ZIP 建立透過顯式 `.sfx.exe` 配置提供，不自動執行載荷。 |
| TAR | 已准入 | TAR 及登記的 TAR.GZ、TAR.BZ2、TAR.LZMA、TAR.LZ、TAR.XZ、TAR.Z、TAR.BR、TAR.LZ4、TAR.ZST 配置。 |
| TBZ / TBZ2 | 別名/配置 | BZip2 過濾的 TAR 別名。 |
| TGZ | 別名/配置 | GZIP 過濾的 TAR 別名。 |
| TLZ | 別名/配置 | LZMA 過濾的 TAR 別名。 |
| TXZ | 別名/配置 | XZ 過濾的 TAR 別名。 |
| UDF | 已准入 | UDF 唯讀映像，包括混合映像的命名空間規則。 |
| WIM | 明確不支援 | 產品決定不投入 WIM 所需的映像資源模型、讀取器和完整語料庫。 |
| XPI | 別名/配置 | XPI 字尾下的 ZIP 內容，仍以內容驗證為準。 |
| XZ | 已准入 | XZ 和 TAR.XZ，覆蓋 LZMA2、宣告校驗及已准入過濾器。 |
| Z | 已准入 | Unix compress LZW 和 TAR.Z/TAZ 別名。 |
| ZIP | 已准入 | ZIP/ZIP64、AES-256、支援的分卷、方法及修改配置。 |
| ZIPX | 別名/配置 | ZIPX 透過已驗證的 ZIP 方法（如 XZ）准入，不是獨立容器；目前不提供 ZIPX 建立。 |
| ZPAQ | 明確不支援 | 產品決定不實現該固實、日誌式格式所需的 ZPAQL 虛擬機。 |
| PEA | 已准入 | PEA 1.x 單卷、UTF-8、無加密 PCOMPRESS0 儲存檔案與目錄只讀配置，校驗、加密、壓縮、分卷、建立和修改仍不在範圍。 |
| UU / UUE | 已准入 | 有界 UU/UUE 封裝、物理成員和行結構驗證。 |
| XXE | 已准入 | 有界 XXE 封裝。 |
| ASAR | 已准入 | 有序後設資料、儲存範圍和宣告 SHA-256 檢查。 |
| ZSTD | 已准入 | 標準 Zstandard 幀和 TAR.ZST/TZST。 |
| NSIS | 已准入 | 只讀 `firstheader` 靜態後設資料。按內容驗證有界 PE overlay，`.exe` 不參與拖放或系統關聯，絕不執行 PE、指令碼、外掛或載荷。 |
| LZ4 | 已准入 | 標準 LZ4 幀和 TAR.LZ4。 |

因此不能僅因為副檔名加入通用 `BIN`、`IMG` 或 `SFX(EXE)` 處理器。這些名稱可能指向可執行檔案、磁碟映像或安裝器，副檔名不足以建立歸檔能力。NSIS 僅增加按內容驗證的靜態後設資料讀取器，不授予執行或載荷許可權，登記的 `.exe` 也不參與拖放和系統關聯。Compound(MSI) 已具備按內容驗證的 CFB 讀取器，但 `.msi` 字尾仍不授予執行許可權。

DAA、ISZ、ZPAQ、PEA、BOND 和 J2J 的判斷依據見[磁碟映像及其他歸檔格式資料](disk-image-archive-format-sources.md)。PEA 僅支援文件所述的保守只讀配置。ACE、DAA、ISZ 和 ZPAQ 按產品決定不予支援。BOND 和 J2J 尚未確認為 Bandizip 歸檔格式，依產品決定不予支援。

<a name="xdzip-详细推进方案"></a>

## 詳細推進方案

1. **清單與登記門檻。** 每個已准入配置在 `specs/engineering/archive-formats.json` 中保留一個機器可讀條目，生成 Rust、C++、JavaScript 和關聯入口消費者，然後執行兩項格式登記檢查。每個新格式同時增加格式契約和驗收條目。
2. **先定線格式契約。** 固定公開原始碼 revision 和 SHA-256，記錄頭部欄位、名稱編碼、物理結束、方法、校驗和、重複身份和資源限額。參考工具只有在產出完整自有樣本時才作為證據，崩潰或單一 Store 樣本只能記錄為限制。
3. **先做原始演算法自動測試。** Qt 之外實現有界讀取器，覆蓋空和邊界載荷、每個准入方法、準確輸入/輸出/工作/記憶體限額、取消、型別化 I/O、來源變化、截斷、尾隨位元組、CRC 和失敗終止。未知方法保留可見性，不獲得載荷許可權。
4. **再做容器和產品准入。** 增加 codec crate、runtime 強/弱探測、CLI list/test/extract、桌面標籤和關聯 fixture。驗證凍結後設資料與物理重複選擇，再執行必要的自動 Qt 和佈局檢查。本次研究與格式切片無需人工視覺驗收。
5. **候選優先順序。** ALZ 的保守 Store/raw Deflate 切片、EGG 的無加密且非固實 Store/raw Deflate 切片、CFB/Compound(MSI) 後設資料切片、PEA 保守儲存配置和 NSIS 靜態 `firstheader` 後設資料切片已經准入。ACE、DAA、WIM、ISZ、ZPAQ、BOND 與 J2J 按產品決定永久不支援。ZIPX 建立仍不在目前寫入器範圍。[自有 SFX 寫入器](sfx-creation.md)已單獨實現，不會因讀取別名自動獲得。
6. **發行邊界。** 候選格式在 codec、runtime、CLI 和桌面證據完成前不進入公開能力表。未知方法和參考工具限制記錄在 issue 中，不能靜默回退到 Store 或其他容器。

目前 BH 切片遵循這套方案：只讀、無密碼和分卷路徑，使用獨立有界的 Deflate 讀取器，強制 CRC-32 和準確物理結束。Compound(MSI) 切片遵循同一邊界：CFB FAT/DIFAT、目錄樹、mini-stream 和環檢測獨立於 Qt，只有在不可變後設資料快照匹配後才複製原始流。NSIS 切片停在固定的 firstheader 後設資料記錄，並對載荷訪問返回不支援，絕不執行 MSI 或 NSIS 安裝行為。
