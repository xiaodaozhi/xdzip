<a name="xdzip-内置格式能力登记"></a>

# 內建格式能力登記

[English](../../engineering/format-registry.md) | [简体中文](../../zh-CN/engineering/format-registry.md) | 繁體中文（台灣）

內建格式以 [archive-formats.json](../../../specs/engineering/archive-formats.json) 為登記來源，記錄穩定格式標識、標籤、容器族、壓縮外層、既有私有工作程序編號、支援的方法族、建立字尾、檢查強度及入口候選。方法族是對應格式契約的摘要，不承諾全部引數、歷史變體或組合；實際條目、宿主策略和平臺能力仍在任務開始前進一步收斂准入範圍。

檢查強度描述成功完成[完整性檢查](archive-checking.md)所提供的保證。`supported_content` 校驗受支援的負載，`structure_and_readability` 檢查結構和可讀取性，`format_declared_checks` 校驗流實際攜帶的校驗欄位，也允許無校驗欄位的配置。能否執行檢查由獨立的能力欄位決定。例如 NSIS 只識別有界的安裝器後設資料，無法執行檢查。建立方法編號只在對應格式獲准的寫入方法中解析。

`creation_methods` 為方法名指定宿主傳輸編號，`creation_levels` 按格式限定級別，生成的介面選項與 Rust 描述同時攜帶這兩項。CLI、GUI 和宿主解析同一份登記，不再假定壓縮方法只能是 Deflate 或 LZMA2。Store、Copy 保留編號 0，Deflate 保留 8，LZMA2 保留 32；Zstandard 使用宿主編號 93，LZ4 使用 256，僅在其容器內解釋，不代表 ZIP 方法。只讀配置沒有建立級別。生成器拒絕缺失或格式內歧義的方法編號及非法級別。

| 操作 | 內建支援範圍 |
| --- | --- |
| 建立新歸檔 | ZIP、7z、TAR、TAR.GZ（`.tar.gz`、`.tgz`）、獨立 GZIP、XZ/TAR.XZ、LZ4/TAR.LZ4、Zstandard/TAR.ZST、LHA（`.lha`、`.lzh`）、ISO9660/Joliet，以及 Windows x64 的 ZIP 自解壓封裝。獨立壓縮流只接收一個普通檔案。詳見[歸檔建立](zip-creation.md)和[單檔案壓縮流](single-file-streams.md)。 |
| 改寫現有歸檔 | 符合條件的 ZIP、7z、TAR 和 TAR.GZ。加密、分卷，以及包含不支援的條目或儲存形式的歸檔不在範圍內，詳見[歸檔修改契約](archive-transfers.md)。 |
| 瀏覽、解壓、開啟條目、匯出或檢查 | 每項操作分別核對格式與條目資格。例如，部分安裝器只能檢視後設資料。詳見[只讀瀏覽](readonly-browser.md)和[完整性檢查](archive-checking.md)。 |

修改規格後執行 `python3 scripts/generate-format-registry.py`，生成 Rust 格式描述、C++ 拖放與 Shell 候選、JavaScript 介面選項以及 Python 安裝圖示對映。`--check` 只檢查，缺失或過期輸出明確失敗。`python3 scripts/test-format-registry.py` 檢查矛盾能力、重複標識和字尾、生成語言中的非法值及過期消費者；日常 CI 執行這兩項檢查。

開啟時先按[歸檔開啟契約](archive-opening.md)處理入口和來源，再探測並校驗已授權的位元組。檔案字尾或 MIME 型別只提供入口。宿主仍須檢查來源身份、操作引數、輸出授權和最終提交。`worker_id` 用於選擇內建的工作程序讀取路徑。例如，ISO 讀取經工作程序執行，新 ISO 映像由 Rust 宿主寫入。登記表分別記錄建立和修改現有歸檔兩種能力。

拖放與 Shell 候選按需分別登記。普通 `.gz` 現提供兩個入口及單檔案建立，`.tar.gz` 和 `.tgz` 保留 TAR 容器語義。XPI/APK/IPA/CBZ 候選沿用 ZIP 校驗和已准入組合，見[單檔案壓縮流](single-file-streams.md)。分卷命名和源授權繼續遵守[歸檔開啟契約](archive-opening.md)，登記表不重新解釋編號檔案。Windows 按最終副檔名對映圖示，`.tar.gz` 使用 `gz` 關聯。

只登記已實現且具有對應自動化驗收的能力。實現變化按需同步[自研格式契約](independent-rust-codecs.md)、[格式工作程序契約](native-codecs.md)、使用者能力說明及[驗收條目](../../../developers/qa/acceptance.md)。計劃、進度和執行結論保留在 issue，生成測試證據只放在 `build/` 或 CI 工件。

`creation_wrappers` 宣告 Windows x64 的 ZIP SFX 封裝與顯式 `.sfx.exe` 字尾，複用 ZIP 方法編號，禁用加密和分卷，並且不註冊通用 EXE 關聯。ISO9660 建立使用宿主方法編號 260 和固定級別 0。詳見 [ISO 建立](iso-creation.md)與[自有 SFX 建立](sfx-creation.md)。
