<a name="xdzip-targzip-实现契约"></a>

# TAR/GZIP 實現契約

[English](../../engineering/tar-gzip.md) | [简体中文](../../zh-CN/engineering/tar-gzip.md) | 繁體中文（台灣）

本契約定義 TAR／TAR.GZ 的瀏覽、建立和安全解壓邊界，平臺執行結果與人工驗收在 issue／CI 工件登記。沿用 [安全解壓](zip-safe-extraction.md)與 [ZIP 建立](zip-creation.md)的輸入授權、暫存、衝突、取消及整體提交。

<a name="xdzip-范围与能力"></a>

## 範圍與能力

支援 TAR、TAR.GZ 瀏覽、建立和安全解壓。TAR 為容器，GZIP 為一層壓縮過濾器，按內容探測，不按字尾選擇讀取後端。ZIP 行為保持相容。目前不啟用巢狀自動展開、加密或分卷 TAR。普通 GZIP 單檔案模式遵守獨立的[壓縮流契約](single-file-streams.md)。

TAR 頭、數字、PAX、流式編碼與重寫由獨立 Rust 記錄層實現；GZIP 封裝、CRC-32、Deflate 的 LZ77 與 Huffman 由共享 `xdzip-compression` 實現。`tar` 和 `flate2` 僅用於開發測試參照，不進入這兩個 crate 的生產依賴閉包。規格、來源及限制見[自研格式契約](independent-rust-codecs.md)。

TAR 接受普通檔案與目錄，解析 PAX 路徑及 GNU 長名稱。符號連結、硬連結、裝置、FIFO、稀疏和未知型別不能解壓。所有輸出名稱繼續經過 PortablePath 與宿主衝突預檢，格式庫不獲得檔案路徑或 unpack 許可權。目錄許可權、所有者與特殊 mode 不恢復。日期作為展示後設資料，不承諾恢復。

<a name="xdzip-有界流式处理"></a>

## 有界流式處理

檔名的傳統編碼、GNU 長名稱及 PAX UTF-8/BINARY 宣告按[編碼契約](filename-encoding.md)統一解析。完整掃描後才釋出索引，掃描期間條目數與總量未知。後設資料沿用 100,000 條、單名稱 4096 位元組及累計名稱預算；流式來源的編碼輸入和解碼工作各最高 24 GiB，單個 GNU／PAX 擴充套件頭最多 64 KiB，每條最多四個擴充套件頭。全域性 PAX 僅接受無檔案語義的 comment 註釋（相容 git archive），按記錄宣告長度完整校驗後丟棄，不繼承到條目。全域性 path、size、mtime、linkpath、稀疏及未知欄位仍明確拒絕；GNU 長連結與稀疏變體不在支援範圍內。全域性註釋同樣計入每條最多四個擴充套件頭及流位元組預算，不使未完成的區域性擴充套件變為有效。常見的前置 ./ 在 TAR 名稱中規範化，根目錄標記不作為輸出項。順序流的讀取與跳過同樣檢查取消和位元組預算，不以 seek 繞過損壞流檢查。GZIP 必須讀至 EOF 驗證 CRC 與 ISIZE，多 member 僅作為同一連續 TAR 位元組流；尾部非零垃圾、截斷及損壞均失敗。拒絕未經驗證的部分索引。

TAR.GZ 缺少逐項壓縮大小，高壓縮比准入使用所選宣告總量與整份凍結來源大小比較；超過 1000 倍且大於 1 MiB 仍需明確允許，允許也不繞過掃描工作和輸出配額。後設資料預檢不偽造逐項壓縮比。

解壓按物理順序處理選擇，跳過未選內容仍計量並可取消，不承諾隨機讀取。PayloadReader 只獲得來源與宿主 sink；來源版本在掃描、解碼和提交邊界仍由宿主驗證。每條提交和整體建立失敗語義沿用現有安全 I/O 契約。

<a name="xdzip-创建与界面"></a>

## 建立與介面

建立視窗啟用已有 TAR、TAR.GZ 分段，不改變視窗密度。格式切換同步副檔名；TAR 禁用壓縮級別，兩個 TAR 格式都禁用密碼及分卷並清空相關值。TAR.GZ 使用 GZIP 0／1／6／9 級別。逐條壓縮後大小未知，顯示為橫線，不偽裝成 0 位元組。同名目標資料夾由 Rust 從來源名稱統一生成，經有界橋接用於預覽；TAR.GZ 去掉整個複合副檔名，大小寫一致處理，空基名回退 archive。UI 不自行推導另一套目標，最終仍由 PortablePath 與授權根複驗。主視窗格式標記按真實格式和文字寬度自適應，TAR 日期以 UTC 展示，日期超出既有顯示範圍時保留未知。建立對 Unicode／長路徑顯式寫 PAX UTF-8 path，大檔案寫 PAX size，避免參考讀取器的本地編碼猜測；凍結輸入樹、流式寫入、終止 TAR/GZIP、重新掃描及逐項解碼校驗，驗證成功後才整體釋出。衝突改名保留完整複合副檔名。

<a name="xdzip-验证与限制"></a>

## 驗證與限制

必須覆蓋系統 tar／Python tarfile 雙向互操作，長路徑、Unicode、空目錄、重複項、PAX/GNU、特殊項、路徑逃逸、截斷、CRC、資源限額、取消及舊檔案保護；Qt 真實建立、重開與解壓複用現有任務和衝突視窗。實際證據只生成到 build/ 或 CI 工件。

桌面操作與輔助功能按[通用驗證方法](../../../developers/qa/README.md)選擇受影響項複驗，原生自動化與離屏檢查不代替人工專項。具體執行結果和限制記錄於對應 issue，既有桌面驗收見 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)，不在本契約保留單輪許可權或環境阻塞。`v0.1.0` 的歷史發行許可複核見 [#4](https://github.com/xiaodaozhi/xdzip/issues/4) 和 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)，未來候選須建立新的發行 issue。


CLI 建立入口接受 .tgz 作為 .tar.gz 的同格式名稱別名，複用相同 GZIP 編碼、驗證和提交，普通非 TAR 的 GZIP 由獨立單檔案格式介面卡處理。
