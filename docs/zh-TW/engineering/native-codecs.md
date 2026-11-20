<a name="xdzip-隔离格式工作进程契约"></a>

# 隔離格式工作程序契約

[English](../../engineering/native-codecs.md) | [简体中文](../../zh-CN/engineering/native-codecs.md) | 繁體中文 | [日本語](../../ja-JP/engineering/native-codecs.md) | [한국어](../../ko-KR/engineering/native-codecs.md) | [Français](../../fr-FR/engineering/native-codecs.md) | [Deutsch](../../de-DE/engineering/native-codecs.md) | [Español](../../es-ES/engineering/native-codecs.md) | [Português (Brasil)](../../pt-BR/engineering/native-codecs.md) | [Русский](../../ru-RU/engineering/native-codecs.md)

7z、RAR、CAB 和 ISO／UDF 的讀取由隔離工作程序內的[內建 Rust 實現](independent-rust-codecs.md)透過共享 CXX 邊界提供。7z 寫入也經該工作程序執行，任務由宿主控制。ZIP、TAR、壓縮流、DEB 和[新建 ISO9660/Joliet 映像](iso-creation.md)在 Rust 宿主程序內處理。格式按內建規則選擇，所有路徑均不會從 PATH 尋找命令列解碼工具。

CAB 產品固定執行獨立 Rust 路由，沒有測試開關、官方處理器或失敗回退。完整幀、共享演算法和資源邊界見[自研契約](independent-rust-codecs.md#xdzip-cab%E5%8F%AA%E8%AF%BB%E8%A7%84%E6%A0%BC)，來源及參考分析角色見[CAB 格式來源](cab-format-sources.md)。

宿主可在執行時啟動前登記一個已驗證的絕對工作程序路徑。重複登記相同路徑成功，相對路徑或衝突路徑被拒絕且保留原選擇。顯式開發環境覆蓋仍優先，其次使用已登記路徑，僅未登記時才查詢可執行檔案旁的工作程序。桌面內建 CLI 透過宿主登記共享桌面工作程序；獨立包保留本地副本，不根據附近目錄猜測其他桌面安裝。

<a name="xdzip-构建与来源"></a>

## 構建與來源

統一準備入口執行 `scripts/prepare-native-codecs.py`。[產品原始碼清單](../../../scripts/native-codec-sources.json)目前為空，準備步驟不下載或解包官方格式原始碼；離線構建同樣不需要 UnRAR 或 7-Zip 歸檔。通用來源工具保留有界解包及摘要校驗，用於明確宣告的來源材料，不能據快取內容自動加入產品。

產品不編譯或連結官方 UnRAR、7-Zip 容器、演算法、通用設施、檔案介面卡或格式登錄檔，7z、RAR、CAB、ISO、UDF 均固定使用 Rust，沒有執行失敗回退。CMake 內部的 `XDZIP_7Z_BACKEND=rust`、`XDZIP_RAR_BACKEND=rust` 與 `XDZIP_CAB_BACKEND=rust` 記錄固定身份。生成的 `native-codec-source-list.txt` 完整記錄七個專案自有膠水檔案，門禁核對路徑與集合，拒絕官方來源和已退出的 `host_file.cpp` 重新進入。`platform_time.cpp` 與 `platform_encoding.cpp` 僅將 OS 本地日曆和字符集對映交回 Rust，不解釋歸檔時間格式或密碼位元組規則。共享 Zstandard/FSE/Huffman/XXH64、LZ4/XXH32、Deflate/Deflate64、LZMA、BZip2、PPMd、LZX、Quantum 按實際格式複用，不重複引入格式庫。

來源對應材料隨實際部署版本交付，不能只交付下載連結。歷史構建與獨立官方參考工具繼續按各自原始條款處理，目前產品原始碼清單為空不改變既有交付材料的義務。

<a name="xdzip-权力与生命周期"></a>

## 權力與生命週期

Rust 核心持有來源授權、分卷身份、目標授權、暫存和提交能力。Rust 7z/RAR/CAB/光碟後端在獨立工作程序執行，透過有界協議讀取授權流及輸出目前批准條目的資料，不獲得使用者目標路徑或任意檔案訪問介面。CXX 同步借用的字串與切片只在呼叫期間有效，不儲存其指標。C++ 異常和 Rust unwind 轉成穩定錯誤；OOM/abort 使工作程序失敗，宿主丟棄不完整暫存。

宿主對初始來源 seek、工作程序隨機讀取、建立輸入及 EOF 探測統一保留 typed 限額、取消、不支援和來源變更錯誤；即使外層 I/O kind 是 Interrupted，也不抹掉其 typed 原因。只有沒有 typed 原因的 Interrupted 可在檢查取消後重試，預先計入的來源工作不退款。Scan 保留 SourceChanged，內容讀取按現有來源錯誤語義返回，建立按來源錯誤拒絕提交。

工作程序不接收命令列密碼，不輸出歸檔提供的任意診斷文字，也不繼承應用憑據和代理環境。錯誤對映為穩定錯誤碼，開啟加密目錄的密碼只保留在目前會話及其關聯任務的歸零容器中，不跨啟動持久化；關閉會話和任務終態分別釋放各自容器。取消停止讀取與輸出，超時後終止工作程序並等待回收；程序失敗不得產生成功終態。程序隔離不等同於作業系統沙箱，不宣稱能阻止已被攻陷的程序訪問所有同使用者資源。

工作程序的 UTF-8 與寬字元轉換嚴格檢查 Unicode scalar。Windows 使用 UTF-16 代理對，macOS 和 Linux 使用 32 位寬字元；非法 UTF-8、孤立代理項和越界 scalar 丟擲 `std::range_error`，由工作程序邊界對映為協議錯誤 2，不釋出成功終態。轉換本身保留嵌入的 NUL，名稱的路徑安全檢查仍由宿主負責。

7z 建立透過 EncodeOutput::write_at 將有界偏移與資料塊寫入宿主私有暫存，原生程序只儲存流位置，不在記憶體中累積整個輸出歸檔。宿主分別累計實際寫入量和歸檔長度高水位，頭部回寫不增加分卷內容長度；偏移、塊大小、累計輸出量和取消均在主機端重新檢查。未提供隨機寫能力的輸出明確拒絕。

任務延續宿主條目、路徑、後設資料和實際位元組預算，預設 native 預算為 1 GiB。Windows 使用 Job Object 提交記憶體限制，Linux 使用絕對虛擬地址空間限制，macOS 使用啟動時虛擬地址空間基線加預算，排除 dyld 預留；Unix 地址空間限制不等同於 RSS 計量。字典及固實工作區在分配前准入，無法在預算內完成的操作明確失敗，不靜默降級為無界磁碟暫存。重試和固實跳過均累計實際工作量。

RAR 現代實際演算法 1 可用[有界精確分頁歷史](rar-format-sources.md#xdzip-%E5%A4%A7%E9%80%BB%E8%BE%91%E5%AD%97%E5%85%B8%E7%9A%84%E7%B2%BE%E7%A1%AE%E5%AD%98%E5%82%A8)表示大邏輯字典，邏輯宣告不等於實際記憶體許可。目錄、保留頁與臨時峰值有獨立上限，原 worker 預算和動態失敗行為不變。

演算法初始化和控制處理的實際輔助工作使用獨立 `AUX_WORK=24`，目前僅允許 RAR 負載使用；宿主與 runtime 分別累計輔助預算，不將其計入檔案解碼、膨脹比、公開進度或預計費信用。完整協議、重試生命週期與必須保持的驗收條件見[演算法輔助工作計量](auxiliary-decoding-work.md)。


<a name="xdzip-能力边界"></a>

## 能力邊界

7z 的瀏覽、解壓、完整性測試和建立分別宣告，建立使用 LZMA2，預設正常級別、非固實、不加密、不分卷。固實、內容加密、檔名加密和編號分卷作為獨立能力驗證。檔名加密必須設定密碼。檔名加密包在後設資料開啟階段請求密碼，未完成掃描不能釋出部分索引。

未加密、非分卷且所有條目可安全保留的 7z 包可複用新增、替換和包內移動。固實內容透過受限重建處理。加密、分卷、連結和不支援特殊記錄繼續拒絕修改，來源重新檢查及完整驗證透過後才允許同卷原子替換。獨立刪除和重新命名入口不屬於本次擴充功能。

RAR 固定使用 `xdzip-codec-rar`，只讀且沒有官方後端回退。舊格式、RAR5 及 RAR 7 擴充功能，密碼、頭部加密、固實和分卷分別驗證，演算法及相容性限制以[自研規格](independent-rust-codecs.md#xdzip-rar%E5%8F%AA%E8%AF%BB%E8%A7%84%E6%A0%BC)為準。不提供建立、修改、修復或密碼恢復。RAR 的初始化和控制工作走輔助通道；真實解碼輸出使用 WORK 與預付交付，不將模型初始化計入檔案大小或膨脹比。

CAB 支援 Store、MSZip、Quantum 和 LZX，單卷與同目錄分卷均只讀。分卷先由 Rust 根據頭部關係驗證和凍結授權集合，再向工作程序傳遞原始卷名及各卷邊界。缺卷和不匹配卷不能釋出部分索引，詳見[分卷契約](archive-opening.md)。CAB 原始名字和 UTF-8 宣告保留，舊編碼採用[統一檔名解析](filename-encoding.md)，不呼叫宿主內碼表猜測輸出路徑。

ISO9660、Joliet、Rock Ridge 和 UDF 的解析由 `xdzip-codec-optical` 執行，不連結官方 Iso/Udf 處理器。混合映象優先 UDF 名稱空間，已識別的 UDF 損壞不靜默回退 ISO。按內容識別，不依賴副檔名授權。ISO/UDF 讀取支援瀏覽、解壓、臨時開啟、匯出和測試，不掛載磁碟、不執行啟動映象。程序內 [ISO9660/Joliet 寫入器](iso-creation.md)可建立普通資料映像，UDF 建立與映象更新仍不支援。啟動映象記錄僅作為可讀取檔案，連結及特殊物件保持不可輸出。映象測試檢查結構、範圍及實際內容可讀取性，不聲稱普通檔案具有內容校驗和保證。分割區映射和不支援擴充功能的明確邊界見[自研規格](independent-rust-codecs.md#xdzip-iso%E4%B8%8Eudf%E5%8F%AA%E8%AF%BB%E8%A7%84%E6%A0%BC)。


<a name="xdzip-验证与交付"></a>

## 驗證與交付

實際支援宣告須由官方工具互操作、已知輸出位元組、損壞和資源限制樣本支撐。僅識別魔數、呼叫返回成功或使用同一後端自讀自寫不足以獨立驗證互操作。三平台核心和 Qt 檢查、部署驗證、原生視覺及桌面拖放分別記錄，階段證據保留在 issue 和本地或 CI 工件。

CTest 的 `native_unicode_conversion` 直接編譯工作程序轉換實現，覆蓋 UTF-8 各長度邊界、非 BMP、嵌入 NUL、非法序列和平台寬字元邊界；`native_unicode_protocol` 向實際工作程序傳送非法建立名稱，核對錯誤響應和失敗退出。Windows、macOS、Linux 分別執行，不能以單一平台代替兩種寬字元寬度的驗證。

RAR 路由驗收必須覆蓋實際 worker 的掃描、測試與安全解壓，包含小檔案、PPM 初始化、密碼和授權分卷。小檔案及 PPM 初始化不得單獨觸發高膨脹比，輔助工作不得增加公開位元組計數；失敗不得釋出檔案。原始碼門禁同時核對空產品清單、固定 Rust 身份、完整編譯來源及薄 RAR 入口，不能只據符號或輸出成功判斷官方程式碼已退出。


Foundation 使用 `scripts/check-rar-rust-interop.py --worker <built-worker> --output build/<evidence>` 執行原有完整 `native_backend_tests` 顯式批次。入口先按 `scripts/rar-rust-fixtures.json` 固定 URL、大小和 SHA-256 準備公開小型 PPM、大 PPM 與舊密碼樣本，再生成僅含原有壓縮流的單成員子集、自有數學未知尺寸和已知空檔案頭，以及自有 20/26 夾具所需環境。網路單響應最多 512 KiB，準備檔案與證據僅進入 build，不生成 2 GiB 明文，不執行作者工具。缺失或身份不符必須失敗，不能以跳過測試替代環境準備。包裝還必須實際執行輔助工作惡意 worker、未知尺寸協議、Native 小檔案計量、固實重讀、自有 E8E9 全位元組摘要和真實 OS 字元映射探針。探針只使用目前 worker 旁固定 Rust 輸出目錄的 CXX 標頭檔案與嚴格唯一的 Release 靜態庫，記錄標頭檔案、靜態庫和探針摘要；缺失或歧義必須失敗。大 26 的元件完整 SHA 驗證單獨執行，runtime 批次透過實際 worker 檢查一次完整大檔案的計數、CRC 和零暫存行為；可顯式追加 `--large-digest` 驗證 worker 完整 SHA。


互操作檢查使用固定 26.05 官方參考工具，獨立於產品工作程序。Windows 從同一官方釋出下載並核對 7zr.exe 摘要，再從已核驗的 7z2605-extra.7z 僅提取 x64/7za.exe，並複核該成員的獨立 SHA-256。此流程不依賴系統 tar 的 LZMA 支援，不執行安裝器，也不從 PATH 借用未固定工具。Linux 與 macOS 從已核驗 tar.xz 僅讀取指定 7zz 成員。下載、快取和提取結果任一摘要不符均停止；這些參考檔案只保留於 build/ 測試目錄，不進入產品部署。

開發包維持實驗性。元件許可、對應原始碼和重連結要求按實際構建單獨複核，本功能完成不自動批准正式發行。

[runtime 工作程序驗證入口](../../../developers/qa/runtime-worker-tests.md)自動準備完整整合清單及固定輸入，代替直接執行未準備前提的 `native_backend_tests --ignored` 批次。可選的完整庫模式要求零 ignored 結果。
