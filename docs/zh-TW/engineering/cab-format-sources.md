<a name="xdzip-cab-格式来源与实现依据"></a>

# CAB 格式來源與實現依據

[English](../../engineering/cab-format-sources.md) | [简体中文](../../zh-CN/engineering/cab-format-sources.md) | 繁體中文 | [Français](../../fr-FR/engineering/cab-format-sources.md) | [Español](../../es-ES/engineering/cab-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/cab-format-sources.md)

本文記錄 CAB 容器、MSZIP、LZX 和 Quantum 的來源、適用範圍及獨立實現約束。產品能力、資源治理和逐格式替代規則分別由[隔離格式工作程序契約](native-codecs.md)、[自研 Rust 契約](independent-rust-codecs.md)及[分卷契約](archive-opening.md)定義。來源記錄不代替互操作驗收或發行許可複核。

<a name="xdzip-微软技术文档"></a>

## 微軟技術文件

優先使用 Microsoft 的格式文件。下表固定實際閱讀的副本，日期是文件版本標識，不表示該 URL 永遠返回相同位元組。下載副本須核對 SHA-256，不能僅按相同標題判斷版本相同。

| 文件與入口 | 固定版本 | 使用範圍與限制 |
| --- | --- | --- |
| [MS-CAB 官方歸檔入口](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprotlp/5641c1b2-35ed-4542-9058-a07ccab5b0a1)，[PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/Archive_Exchange/%5BMS-CAB%5D.pdf) | 頁尾 `v20110304`，Release 2011-03-04 | §2 的 CFHEADER、CFFOLDER、CFFILE、CFDATA，§3.1 的校驗方法。Microsoft 將其列為歸檔文件，並明確提示不應依賴歸檔材料；不能稱為現行受維護規範。它不定義各壓縮演算法的內部位流。 |
| [MS-MCI](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-mci/27f0a9bf-9567-4e40-ad66-6ae9ab9d2786)，[PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-MCI/%5bMS-MCI%5d.pdf) | 2016-10-17，revision 7.3 | §2 的 MSZIP 資料結構、CK 標記、完整 Deflate 流和塊大小。Deflate 位流另依據 [RFC 1951](https://www.rfc-editor.org/rfc/rfc1951.html)。 |
| [MS-PATCH](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-patch/cc78752a-b4af-4eee-88cb-01f4d8a4c2bf)，[PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-PATCH/%5bMS-PATCH%5d.pdf) | 2025-08-19，revision 14.1 | §2.1 位流、視窗和距離，§2.2 頭與 E8，§2.3 塊，§2.4 Huffman 樹，僅用於比較共同機制及差異。它描述 LZX Delta，不是 CAB LZX 的替代規範。 |

| 固定 PDF | SHA-256 |
| --- | --- |
| MS-CAB | `09f796a493547697ed7d4bc36222ac132bf3ca1bd6b1da76fbdc3b38b5307709` |
| MS-MCI | `cb097c434e1cfccb368f0d7e9084ab4d47a65809a2eea94bbf41f9722c1a0343` |
| MS-PATCH | `ea17466b97c9a9d4e2c815538387bf442fb2393ed29632c6f58b2c47ef1f29fe` |

Microsoft Open Specifications 文件帶有自己的版權及智慧財產權條款，包括實現所需文件使用許可和專利提示。它們不因此成為本專案 Apache-2.0 材料，也不能從文件可下載推定所有專利或再分發問題均已解決。

<a name="xdzip-1997-年-cab-sdk-文档"></a>

## 1997 年 CAB SDK 文件

原始標題為 Microsoft Cabinet File Format、Microsoft MSZIP Data Compression Format 和 Microsoft LZX Data Compression Format。所讀副本正文包含 Microsoft 1997 年版權宣告，由第三方託管：

| 映象原件 | 閱讀範圍 | SHA-256 |
| --- | --- | --- |
| [CABFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/CABFMT.DOC) | §2 容器佈局與跨卷語義，§3 固定容器示例，§4.1 校驗定義 | `5a220658b76f8a66e9013e342b4eb236f057b597a8e6f3939c86f05efc51e83a` |
| [MSZIPFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/MSZIPFMT.DOC) | Introduction、Implementation Details，特別是獨立 Deflate 塊與保留歷史 | `046d283fa3b697e46c35b6a329bb90c86cbc9f4dedbed24c3ae43bd9b8ccf239` |
| [LZXFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/LZXFMT.DOC) | Concepts、LZX compressed data format、樹編碼、距離/長度及 E8 預處理各節 | `68ba4df86e08bece3259574133f1c61606d143d85f749b388f5f1e8ea46f8bef` |

這些 `.DOC` 檔案實際為 RTF。正文標題、版權和 CAB 定位支援其作為微軟作者文件副本使用，但託管站不是 Microsoft，尚未與微軟釋出的 CAB SDK 歸檔逐位元組比對。[Microsoft 知識庫 310618 存檔](https://mskb.pkisolutions.com/kb/310618)列出原 SDK 的內容，並記錄舊下載已經下線。該存檔無法驗證映象檔案的真實性。閱讀範圍是格式文件，不含該 SDK 的 FCI/FDI 實現或示例程式碼。

公開 TCOMP 引數另核對 Windows SDK 10.0.19041.0 的 `um/fdi_fci_types.h` 常量宣告及 [TCOMPfromLZXWindow 文件](https://learn.microsoft.com/en-us/windows/win32/api/fdi_fci_types/nf-fdi_fci_types-tcompfromlzxwindow)。該標頭檔案 SHA-256 為 `27a1f6f7728f5db64d11e434d101819d646ca49270930606bb76c7989ebe0254`。只使用公共引數範圍與位定義，沒有據此呼叫或分析 FCI/FDI 演算法實現。

LZXFMT 的部分概覽、表格和正文存在矛盾。MS-PATCH 可輔助辨認共同機制，但其參考資料、擴充套件長匹配、128 KiB 至 32 MiB 視窗及額外塊頭不能直接套入 CAB。CAB LZX 的視窗範圍為 15 至 21 位，須單獨核對原文、參考行為和辨識向量。

<a name="xdzip-quantum-资料范围"></a>

## Quantum 資料範圍

已取得的原演算法廠商材料為 [Cinematronics QUANTUM 使用者手冊映象](https://carrero.es/wp-content/uploads/2025/07/QUANTUM.pdf)，正文版權為 Cinematronics 1993、1994、1995，保留全部權利；PDF SHA-256 為 `a2d6ed51e47db407ea5849b42004750f2f36c82a380f263b983c5282838a8212`。§6 說明獨立 `.Q` 歸檔容器，沒有定義 CAB Quantum 所需的算術位流、機率模型更新和幀終止，不能把 `.Q` 的整流狀態或檔案校驗移作 CAB 規則。

[Matthew Russotto 的 Quantum 格式研究地址](http://www.russotto.net/quantumcomp.html)及歷史存檔未取得可核驗正文。因此沒有將其內容作為已讀依據，也不能判斷原文是否足以完整定義演算法，或推定其許可條件。

另只閱讀了 [David Carrero 的 UnQuantum README](https://github.com/dcarrero/unquantum/blob/main/README.md)，副本 SHA-256 為 `187af81f8231cfe4cc31aec0c76476b1e33179fe517a365c820fc49c1b8106e0`，未閱讀該專案解碼原始碼。它是 `.Q` 專案的非官方演算法概覽，並宣告參考 Russotto、libmspack 和二進位制研究。它不是 Russotto 原文，也不能完整替代 CAB 位流規範；專案標註 MIT 不足以推定其參考資料和演算法來源均按 MIT 授權。

上述缺口以固定官方參考實現的行為分析補齊，不把非官方材料一概排除，也不把參考實現稱為 Microsoft 格式規範。

<a name="xdzip-固定参考源码与分析范围"></a>

## 固定參考原始碼與分析範圍

參考來源為 [7-Zip 官方 26.05 釋出](https://github.com/ip7z/7zip/releases/tag/26.05)中的 [7z2605-src.tar.xz](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz)，歸檔 SHA-256 為 `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e`。構建來源清單另見 [native-codec-sources.json](../../../scripts/native-codec-sources.json)；該清單描述完整官方來源，下面的記錄僅說明 CAB 參考分析實際使用的檔案。

按歸檔中的 `DOC/License.txt`，下列解碼器及通用檔案適用預設 LGPL-2.1-or-later，未發現這些檔案帶有替代許可宣告。歸檔還包含適用於其他檔案的 BSD 條款，不能把整個歸檔簡化為一個無例外許可，也不能將這些解碼檔案誤標為 BSD。

| 歸檔內檔案 | 實際分析範圍 | SHA-256 |
| --- | --- | --- |
| `CPP/7zip/Compress/QuantumDecoder.cpp` | 算術位流、模型、token、幀終止和字典狀態 | `1a22429d792493573f542782a45a7f702b6e636e5aee1d47bdb3d1cf75ef11e2` |
| `CPP/7zip/Compress/QuantumDecoder.h` | 模型字母表及 selector 常量 | `7571ee0360a7abe8f297f4cd4e34df1863003f252c6fab66c61f8d1bd25a21b4` |
| `CPP/7zip/Compress/Lzx.h` | LZX 位流常量 | `779b32ecaecfc5b8c135e0051394001786d680fca4cfda57a480884342500e99` |
| `CPP/7zip/Compress/LzxDecoder.h` | 跨幀保留狀態的邊界 | `18141528bc8587ade4f7b1e330ee35dc8e31d6c6ab34d86a2dff59d362d0d6c9` |
| `CPP/7zip/Compress/LzxDecoder.cpp` | CAB 模式的樹、raw 塊、補齊、重複距離、E8、歷史和結束行為；不以最佳化程式碼或記憶體佈局作為實現設計 | `e78bf50f27f21e04966785fafe7f94fddc533a9e6103f79b81c4e7d7613d1aab` |
| `CPP/7zip/Compress/HuffmanDecoder.h` | 僅完整樹/空樹的准入部分，固定副本約第 109 至 164、430 至 497 行 | `22f93e29b2408ce7e1c1bca5f5f807c1383a8d1634fe3ae6cfcab15f084109c0` |
| `CPP/7zip/Archive/Cab/CabHandler.cpp` | 僅壓縮方法、CFDATA 幀和歷史接續部分，固定副本約第 1020 至 1252 行 | `77b82387eea40eda02929b90d3006877c08765089d6b96c5c12b9cfd3f56b8f7` |
| `C/7zVersion.h` | 固定版本身份 | `f7dc58b3a3c01569e061eda9cfe633ad8983667eb0fa10ab94a6c364ddd7e699` |
| `DOC/License.txt` | 檔案許可預設規則和例外 | `9ac2b4a97ab5d523965534d8b2d5868e511b39096d51fff458ab72c38b80fccc` |
| `DOC/copying.txt` | LGPL 原文的身份摘要，未作為法律意見解釋 | `1e7e6bae5a5bde32f1ae5a7c37a082d1ab03cf89354f7f936ac40be9e39a6531` |

行號僅用於限定該固定副本的閱讀範圍，檔案身份以摘要為準。升級參考版本時須重新記錄差異，不能沿用行號認定行為不變。

<a name="xdzip-分析与独立实现的分工"></a>

## 分析與獨立實現的分工

前期準備只閱讀格式文件及專案自有介面。隨後參考分析者明確閱讀了上表指定原始碼，將缺失或矛盾的位流、數學更新、重置、終止和錯誤條件整理為不含程式碼表達的行為規格，再交給未閱讀這些參考原始碼的 LZX、Quantum 實現者。

兩個演算法的 Rust 實現由各自未讀參考原始碼的實現者獨立組織狀態、型別、控制流程及測試。分析者不承擔這兩個演算法的 Rust 實現，另行負責 CAB 容器。由此不能對整個 CAB 工作聲稱無人接觸參考原始碼，不能將本流程稱為嚴格潔淨室。行為規格不得包含參考程式碼、逐行改寫、結構搬運或註釋翻譯；規範名稱、常量和數學關係須保留來源。

參考閱讀、獨立編碼和黑盒互操作分別回答不同問題。自有編碼器與自有解碼器相互成功不能證明獨立相容；參考工具成功也不能證明其所有寬容行為都適合本專案完整性契約。參考原始碼、文件、工具和樣本各自保留原許可，Rust 重寫不構成法律風險已經清零的結論。許可與發行要求另見[許可契約](licensing.md)及[對應原始碼契約](source-delivery.md)。

<a name="xdzip-必须保留的技术区分"></a>

## 必須保留的技術區分

以下是格式文件和限定參考分析共同約束的行為邊界，不規定 Rust 內部資料結構或最佳化方式。

- CAB 的 folder 是壓縮歷史單元，檔案是其輸出區間。檔案邊界、CFDATA 邊界及跨卷延續不自動重置歷史；只有新邏輯 folder 重置。`cbUncomp=0` 的跨卷壓縮片段按同一邏輯幀拼接，每個物理 CFDATA 的校驗仍獨立驗證。該校驗是位元組/字的異或，含資料保留區，不能用 CRC 或拼接後的單次校驗代替。後設資料、長度、卷關係與負載範圍須分別驗證。
- MSZIP 的每幀由 CK 和完整裸 Deflate 流組成。樹和位流隨幀重置，至多 32 KiB 歷史隨 folder 保留；幀最多 32,780 壓縮位元組、32,768 輸出位元組。複用專案共享 Deflate 的預置歷史介面，不能新建第二套 Deflate。零填充或強行補齊截斷流不能代替嚴格終止檢查。
- CAB LZX 使用小端 16 位字、字內最高位優先的位流。視窗 15 至 21 位的 position slot 數依次為 30、32、34、36、38、42、50；舊文概覽中的大視窗計數須結合完整距離表校正。樹長差值按模 17 做減法，aligned 樹先於 main/length 樹。main、pretree、aligned 樹要求完整，length 樹可全空，但實際請求該空樹時必須失敗。
- LZX raw 塊仍有通用塊長，進入 raw 資料前跳過 1 至 16 個零位，即使恰好按字對齊也須跳過一個字。奇數 raw 塊的零補位元組可以在下一幀開始消費；僅在 folder 最終結束且沒有下一塊時相容省略該補位元組。普通壓縮幀只允許 0 至 15 個零位補齊，不能接受額外完整字或任意尾位元組。LZX 塊可以跨 CFDATA 幀，最終 `finish` 必須拒絕仍有宣告輸出尚未完成的塊，不沿用舊 CAB 接入遺漏結束檢查的寬容。
- LZX 的 LZ 歷史儲存 E8 還原前的位元組；新 folder 沒有可回溯的有效零歷史。E8 在每幀末十位元組前掃描，translation size 按完整無符號 32 位解釋，位置按 folder 累計。舊文末六位元組的描述和僅適用於正有符號大小的比較不能直接用於實現。
- Quantum 每幀重置 16 位閉區間算術碼狀態，九個機率模型和實際字典歷史隨 folder 保留。selector 的七種取值不等於只有七個模型。模型符號頻率每次增加 8，總頻率嚴格大於 3800 時才在下次使用前重標定；每個模型獨立的累計頻率減半、第四次及其後每五十次重排、同頻順序均屬於位流行為，不能隨意替換為語言預設排序。字典視窗為 10 至 21 位，level 為 1 至 7，level 不改變解碼語義。
- Quantum 沒有獨立結束 token，精確輸出長度由容器提供，同一壓縮位元組串可能滿足不同合法宣告長度。到達期望輸出後仍須消費至少兩個零位，取使輸入達到位元組邊界的最少位數，即 2 至 9 位，然後恰好耗盡壓縮輸入。追加零位元組、非零終止位、截斷及跨幀 match 均不能成功。新 folder 也不能回溯尚未生成的視窗內容。

宿主繼續負責凍結卷集合和源身份、名稱解析、條目身份、路徑預檢、任務預算、取消、暫存及提交。格式層只讀取宿主授權的流，不按 CAB 內部名字自行開啟檔案。原始名字與 UTF-8 宣告分別保留，普通重名記錄不合並，僅規範明確的 continued 檔案合併。演算法錯誤不得掩蓋宿主的取消、配額或 I/O 錯誤。具體介面和資源上限以共享契約為準，不從參考緩衝區尺寸或 HRESULT 分類推導新的產品許可權。
