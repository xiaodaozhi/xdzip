<a name="xdzip-单文件压缩流与-zip-衍生包"></a>

# 單檔壓縮串流與 ZIP 衍生包

[English](../../engineering/single-file-streams.md) | [简体中文](../../zh-CN/engineering/single-file-streams.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/single-file-streams.md) | [한국어](../../ko-KR/engineering/single-file-streams.md) | [Français](../../fr-FR/engineering/single-file-streams.md) | [Deutsch](../../de-DE/engineering/single-file-streams.md) | [Español](../../es-ES/engineering/single-file-streams.md) | [Português (Brasil)](../../pt-BR/engineering/single-file-streams.md) | [Русский](../../ru-RU/engineering/single-file-streams.md)

[內建能力表](format-registry.md)將普通 GZIP 與 TAR.GZ 分開登記。`xdzip-codec-stream` 將獨立 `xdzip-compression` 中的 [RFC 1952](https://www.rfc-editor.org/rfc/rfc1952) 實現接入中繼資料、內容解碼和建立契約，生產依賴閉包不引入參考壓縮庫。

<a name="xdzip-探测与读取边界"></a>

## 探測與讀取邊界

GZIP 探測最多讀取 256 KiB 壓縮輸入和 512 位元組解碼前綴。有效的傳統 TAR 頭或明確的 `ustar` 標識選擇 TAR.GZ，已識別的 TAR 損壞時直接失敗，不降級為普通檔案。其他內容先完整掃描 GZIP，再釋出一個普通檔案條目。完整解碼流全零且至少 1024 位元組時，空 TAR 與全零普通檔案存在位元組歧義。只有完整驗證壓縮流後，才用同一外層過濾器下最長已登記字尾選擇檢視：明確普通字尾保留檔案，TAR 字尾保留容器，未知或缺失名稱繼續空 TAR 優先順序。編輯壓縮檔產生空的 TAR.GZ 時，會寫入固定的 GZIP 註解。在完整驗證 GZIP 串流並確認解壓內容恰好是兩個全零的 TAR 結束區塊後，即使檔案使用一般的 GZIP 副檔名，也會選用 TAR.GZ 檢視。只有零前綴、隨後出現非零普通資料時仍為 GZIP。提示不能覆蓋 TAR 強識別、壞壓縮、校驗、配額或源許可權。

掃描時，物理輸入長度和實際解碼位元組分別限制為 24 GiB。`ControlledStream` 在讀取前檢查取消並保留第一個明確型別的錯誤。解碼量恰好到達邊界時，可在內部檢查額外一個解碼位元組以區分 EOF 和超限，該位元組不會交給呼叫方。TAR 使用同一受控流，選擇原樣讀取或 GZIP 過濾。必須完整到達 EOF，逐成員校驗 CRC32 與 ISIZE，包括空成員；串聯成員形成一個邏輯檔案。頭標誌、有界的選用欄位、頭校驗、截斷及非成員尾隨位元組繼續由獨立 GZIP 解碼器檢查。

條目尺寸使用完整解碼後觀察到的位元組數，不使用 32 位 ISIZE 代替。宿主可透過 `ReadSource::source_name_hint` 提供檔案基本名，格式層忽略 ASCII 大小寫移除結尾 `.gz`，拒絕分隔符、控制字元和特殊名稱，否則使用 `content`。忽略頭部 FNAME，展示文字另行轉義，最終名稱仍透過宿主的可移植路徑和目標檢查。名稱提示不授予檔案系統許可權。內容解碼繫結不可變條目、壓縮長度和觀察尺寸，成功前再次校驗 EOF，失敗或完成後不得繼續解碼。任務仍限制每個檔案 8 GiB、整個任務 20 GiB。

<a name="xdzip-创建与提交"></a>

## 建立與提交

普通 `.gz` 建立只接受一個普通檔案，使用 Deflate 0/1/6/9 級別。拒絕目錄、多個檔案、密碼、分卷、固實和檔名加密。編碼器透過 EOF 讀取凍結輸入，檢查精確尺寸和取消，並在宿主私有暫存中完成 GZIP 尾部。宿主校驗暫存壓縮流、源版本和輸出事務後才釋出。GZIP 不保留輸入檔名，重新開啟的虛擬名稱取自歸檔基本名，因此建立校驗要求普通檔案身份和尺寸一致，不強求與原輸入名稱相同。普通 GZIP 不提供歸檔修改能力，TAR.GZ 的建立與重寫沿用獨立容器契約。

桌面建立選項、歡迎頁標籤和關聯候選從穩定格式標識生成，共享格式下拉框使用既有控制元件尺寸引數。CLI 建立讀取同一字尾登記，優先匹配最長字尾，保證 `.tar.gz` 不被 `.gz` 遮蔽。衝突改名保留實際登記字尾，包括 `.tgz`。

<a name="xdzip-zip-衍生入口"></a>

## ZIP 衍生入口

`.xpi`、`.apk`、`.ipa` 和 `.cbz` 接入既有 ZIP 讀取鏈路，繼續遵守 ZIP 的內容校驗、Store/Deflate、已支援的 WinZip AES-256 設定檔、路徑處理和資源邊界。副檔名不授予新演算法或加密能力，這些操作不解釋包清單、安裝行為或包簽名。別名不增加建立字尾。Windows 複用 ZIP 圖示，macOS 登記副檔名並查詢該副檔名實際分配的型別，不修改所有 ZIP 的預設處理程式；Linux 使用對應可用 MIME 候選，IPA 沿用通用 ZIP 關聯。

<a name="xdzip-xz-准入与线格式契约"></a>

## XZ 准入與線格式契約

XZ 能力須遵循 [Tukaani 格式 1.2.1](https://tukaani.org/xz/xz-file-format-1.2.1.txt)。分配解碼器前驗證流標誌和頭部 CRC，拒絕保留位、非規範或溢位的九位元組變長整數、不支援的過濾器和超預算字典。塊頭為 8–1024 位元組，包含已驗證 CRC、可選非零壓縮尺寸、可選解碼尺寸、最多三個保持尺寸的 Delta/BCJ 過濾器後接 LZMA2，以及全零頭部填充。LZMA2 結束標記決定精確壓縮邊界，可選尺寸須與實際處理量一致。原有 7z 嚴格構造器與未知尺寸成員解碼分開，僅驗證結束後允許交還未消費的輸入。

驗證塊填充、宣告的內容校驗（None、CRC32、CRC64 或 SHA-256）、索引各記錄與實際未填充尺寸和解碼尺寸、索引填充和 CRC、流尾反向尺寸、標誌、CRC 和魔數，以及串聯流和四位元組全零流填充。None 沒有內容校驗和，不得展示為內容校驗保證。流數和每流塊數分別限制為十萬，固定解析和過濾緩衝計入 128 MiB 工作記憶體准入，並保留取消及獨立輸入輸出預算。失敗後流不可繼續，完整驗證及源檢查前解碼內容只能保留在宿主暫存區。建立須執行真實 LZMA2 壓縮，使用有界且獨立重置的塊並宣告內容校驗和；獨立工具解碼並核對輸出後才能認定編碼器驗收透過。

ZIPX 沿用 ZIP 記錄語法與物理身份。按 [APPNOTE 6.3.10](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT) 分方法准入，包括 BZip2 12、LZMA 14、XZ 95。LZMA 屬性頭與獨立 LZMA 不同，通用標誌第 1 位控制 EOS 要求。已准入的資料描述符配置必須使用 EOS，本地記錄已知尺寸時可省略。仍須驗證精確負載範圍、本地與中央記錄一致性、描述符、解碼尺寸和 ZIP CRC。沒有已驗證實現的方法保持逐記錄不支援，字尾不能啟用其他演算法或加密配置。

XZ（`.xz`）與 TAR.XZ（`.tar.xz`、`.txz`）依上述設定檔提供瀏覽、檢查和解壓。建立時使用 CRC64、不超過 16 MiB 的獨立區塊，以及 LZMA2 第 1/6/9 級。第 0 級、密碼和分卷不受支援。獨立 XZ 只接收一個普通輸入檔，虛擬輸出名稱取自來源檔名。TAR.XZ 透過既有 TAR 寫入器接收普通檔案和資料夾，並支援修改。`.zipx` 是 ZIP 偵測入口。使用未加密 Store、Deflate 或方法 9、12、14、95 的合格 ZIP 和 ZIPX 可讀取並修改。改寫時保留現有項目的壓縮資料，新項目使用 Deflate。ZIP 的 AES-256 讀取限於既有的 Store/Deflate 設定檔。PPMd I、JPEG 和 WavPack 仍不支援。

<a name="xdzip-新增只读过滤器的准入契约"></a>

<a id="新增只讀過濾器的准入契約"></a>

## 壓縮篩選器的支援範圍

BZip2（`.bz2`、`.bz`）、LZMA Alone（`.lzma`）和 Lzip v1（`.lz`）作為獨立單檔串流時為唯讀，TAR 組合接受 `.tar.bz2`、`.tar.bz`、`.tbz`、`.tbz2`、`.tar.lzma`、`.tlz` 和 `.tar.lz`。字尾只提供入口，歧義縮寫由實際位元組決定。適配複用獨立演算法，在分配前准入演算法工作記憶體，限制為 128 MiB，沿用既有流位元組預算，串聯成員最多 100,000 個。BZip2 校驗逐塊和組合 CRC，允許串聯完整流。LZMA Alone 遵守 [13 位元組格式](https://github.com/tukaani-project/xz/blob/master/doc/lzma-file-format.txt)，前五位元組為屬性和字典，後八位元組為小端原始尺寸。全一尺寸必須有 EOS，已知尺寸允許合法 range 終止或 EOS，拒絕尾隨資料。此格式只提供結構和可讀性校驗，不提供內容校驗和。Lzip v1 遵守[格式規範](https://datatracker.ietf.org/doc/draft-diaz-lzip/02/)，檢查固定 LZMA 屬性、編碼字典邊界、EOS 以及尾部 CRC32、解碼尺寸和完整成員尺寸，其他版本明確拒絕。串聯 Lzip 成員組成一個邏輯檔案。

演算法的成員讀取器只有在 EOF 驗證成功後，才可交還未消費輸入，不得吞掉後續容器位元組。獨立構造入口必須保留既有 7z 讀取器對完整壓縮範圍和已知尺寸的嚴格約束，並用前綴發現及畸形輸入測試證明兩種入口隔離。TAR 使用這些篩選器支援讀取和修改，壓縮 TAR 解析失敗不得降級為普通檔案成功。LZMA Alone 沒有強魔數，不得搶佔有效 TAR 頭、ZIP SFX 或其他強容器識別。能力表及產品入口宣告須有對應實現與自動化驗收，並登記檢查強度與壓縮尺寸未知狀態，為 CLI 和介面生成一致資訊。弱識別排除已知原始尺寸達到 256 GiB 的 LZMA，實際工作預算仍採用更小的既有邊界。Windows 和 macOS 為新增候選複用既有無格式字樣的通用歸檔圖示。

<a name="xdzip-自动化验收"></a>

## 自動化驗收

流格式測試使用獨立參考讀寫器驗證成員校驗、截斷、尾隨位元組、輸入變化、取消、精確預算和虛擬名稱。執行時測試覆蓋分類、空 TAR 優先順序、單檔案建立、檢查、解壓和拒絕時不釋出。真實 CLI 迴歸覆蓋四檔級別和 ZIP 別名。Qt 測試覆蓋穩定標識選擇、單檔案准入、實際建立與重新開啟、原生輔助功能中繼資料、鍵盤選擇及中英文 100/125/150% 字號佈局。生成宣告、安裝對映、視窗和共享控制元件契約分別自動檢查。實際執行結果寫入工作 issue，不將這些檢查擴大為未執行平台或發行許可的透過結論。

LZ4 與 TAR.LZ4 遵循[標準幀契約](lz4-streams.md)，讀取獨立塊、關聯塊、宣告的校驗、串聯和 skippable 幀。建立使用真實 LZ4 壓縮，輸出 256 KiB 獨立塊、塊與內容 XXH32，提供 1/6/9 級。普通 LZ4 只接受一個普通檔案，TAR.LZ4 複用共享 TAR 寫入器，且支援修改。兩種格式均不支援密碼或分卷。檢查僅驗證流宣告的校驗和，舊式幀和外部字典尚不支援。

Zstandard（`.zst`）與 TAR.ZST（`.tar.zst`、`.tzst`）遵循 [Zstandard 壓縮流契約](zstandard-streams.md)。讀取准入有界標準幀原始/RLE/壓縮塊、Huffman/FSE、重複偏移、視窗歷史、宣告的校驗、串聯和 skippable 幀。建立使用真實 LZ/FSE，提供 1/6/9 級、128 KiB 視窗和必需 XXH64，普通 Zstandard 僅接受一個普通輸入檔案。TAR.ZST 支援修改。兩種格式均不支援密碼或分卷。非零字典編號、不可用外部歷史和超預算視窗明確拒絕，檢查僅驗證宣告的校驗，不承諾格式中缺失的校驗保證。

<a name="xdzip-rewriting-compressed-tar-archives"></a>

## 改寫帶外層壓縮的 TAR

修改時保留支援的 TAR 記錄，並依原格式重建外層串流。TAR.XZ、TAR.LZ4 和 TAR.ZST 以一般壓縮等級使用既有的資料框寫入器。TAR.BZ2 寫入 BZh1 串流，每次處理 64 KiB 原始輸入，並記錄區塊 CRC 與合併 CRC。TAR.LZMA 和 TAR.LZ 使用帶有 8 MiB 字典及結束標記的 LZMA1，Lzip v1 另外附上 CRC 和大小欄位。TAR.Z 使用區塊模式 LZW，及早執行 CLEAR，將碼寬維持在 9 位元。TAR.BR 使用 [RFC 7932](https://www.rfc-editor.org/rfc/rfc7932) 定義的未壓縮中繼區塊，輸出可能比原本的 Brotli 串流更大。

主機重新掃描並解碼暫存結果，驗證成功後才會取代原壓縮檔。複合 TAR 副檔名可讓空壓縮檔繼續以 TAR 開啟。如果使用的副檔名會讓空壓縮檔被誤認成單檔串流，刪除最後一筆記錄的操作會在提交前失敗。詳見[壓縮檔修改規則](archive-transfers.md)。
