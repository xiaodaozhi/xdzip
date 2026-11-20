<a name="xdzip-zstandard-压缩流契约"></a>

# Zstandard 壓縮流契約

[English](../../engineering/zstandard-streams.md) | [简体中文](../../zh-CN/engineering/zstandard-streams.md) | 繁體中文 | [日本語](../../ja-JP/engineering/zstandard-streams.md) | [한국어](../../ko-KR/engineering/zstandard-streams.md) | [Français](../../fr-FR/engineering/zstandard-streams.md) | [Deutsch](../../de-DE/engineering/zstandard-streams.md) | [Español](../../es-ES/engineering/zstandard-streams.md) | [Português (Brasil)](../../pt-BR/engineering/zstandard-streams.md) | [Русский](../../ru-RU/engineering/zstandard-streams.md)

格式遵循 [RFC 8878](https://www.rfc-editor.org/rfc/rfc8878.html)、作者的[格式 0.4.3](https://github.com/facebook/zstd/blob/v1.5.7/doc/zstd_compression_format.md)及 [XXH64 規格](https://github.com/Cyan4973/xxHash/blob/v0.8.3/doc/xxhash_spec.md)。產品演算法由 Rust 獨立實現。獨立參考庫和程式僅處理專案自有測試內容，不進入產品依賴。宿主許可權與釋出遵循[單檔案壓縮流](single-file-streams.md)。

<a name="xdzip-帧与资源准入"></a>

## 幀與資源准入

驗證標準魔數、保留位、視窗計算、可選字典編號、可選內容尺寸、塊型別與尺寸、終止塊及可選的零種子 XXH64 低 32 位。讀取時忽略規格指定的未使用頭位，寫入時置零。字典編號零不請求字典，非零編號及依賴外部歷史的引用不支援。頭部和歸檔內容都不能授予其他路徑或執行許可權，單段幀同樣須先透過視窗預算再分配。

視窗歷史上限為 64 MiB，總工作預算為 128 MiB。每塊解碼上限取視窗尺寸與 128 KiB 的較小值。分配、索引及複製前須檢查算術溢位，原始塊、RLE 塊和壓縮塊使用相同的實際解碼計量。串聯標準幀與 skippable 幀受物理輸入、任務位元組預算和取消限制，至少需要一個標準幀。每幀最多 100,000 塊，幀與 skippable 項合計最多 100,000 個。發現階段最多讀取 8 MiB 壓縮輸入及 512 位元組解碼字首。強幀或 TAR 識別後損壞不得降級。釋出實際尺寸與虛擬條目前，完整掃描須驗證終止及所有宣告的尺寸和校驗。

<a name="xdzip-熵编码与序列处理"></a>

## 熵編碼與序列處理

反向位流從非零結束標記開始，須精確消費。FSE 歸一化計數、零遊程、表精度、符號邊界、狀態轉移和表複用須明確驗證。Huffman 權重、隱含末權重、完整字首空間以及單流或四流字面量邊界須與實際輸出一致。重複熵表和重複偏移在幀邊界重置，塊間僅保留格式規定的狀態。輸出前檢查序列字面量長度、匹配長度、偏移附加位、零字面量重複偏移調整、重疊及可用歷史。拒絕不可能或不完整狀態，讀取失敗後不可繼續。宿主終止 I/O、取消、超限、非法資料與不支援配置保留各自類別。

<a name="xdzip-创建与产品接入"></a>

## 建立與產品接入

已准入寫入器須對可壓縮的普通內容產生真實壓縮序列塊，不得僅以原始塊或 RLE 塊宣稱功能完成。提供的級別選擇有界匹配搜尋工作量，每級須獨立解碼驗證空輸入、不可壓縮內容和多塊輸入。輸出內容 XXH64，僅按格式使用原始塊回退。在超過每幀塊數策略前開始新標準幀，同時重置歷史、偏移、熵表和校驗。寫入前准入輸入與輸出緩衝，只有明確完成才輸出流尾。

獨立的 Zstandard 壓縮串流只接受一個一般檔案。TAR.ZST 採用共用的 TAR 剖析與寫入實作。登錄前須有對應主程式、實際 CLI 與 Qt 的自動化測試，涵蓋重新驗證已固定的來源、取消、精確位元組限額、暫存、失敗清理及最終寫入。建立方式和壓縮等級取自共用格式登錄表。兩種設定均不支援密碼或分割壓縮。獨立壓縮串流無法改寫。符合[修改壓縮檔規範](archive-transfers.md)的 TAR.ZST 可以改寫，並重新編碼外層 Zstandard 壓縮串流。輸入的資料框可以省略內容總和檢查碼，因此使用 `format_declared_checks` 檢查。產生的驗證證據儲存在 `build/` 或 CI，實際平台涵蓋範圍和發行許可分別記錄。

有界寫入器使用 128 KiB 獨立匹配搜尋塊、原始字面量、規範預定義 FSE 表和顯式非重複偏移。與 LZ4 複用四位元組雜湊鏈匹配器，距離最多 65535 位元組，1/6/9 級分別搜尋最多 1/16/64 個候選。不可壓縮塊回退為原始塊，全同位元組塊可使用 RLE。幀宣告 128 KiB 視窗，不宣告內容尺寸，始終提供 XXH64；宿主仍驗證凍結輸入的精確長度。每幀最多 65536 塊，只有繼續輸入才開始下一幀。這些寫入選擇不縮減讀取器已准入的 Huffman、FSE、重複狀態或視窗配置。
