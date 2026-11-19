<a name="xdzip-7z-格式来源与独立实现依据"></a>

# 7z 格式來源與獨立實現依據

[English](../../engineering/sevenzip-format-sources.md) | [简体中文](../../zh-CN/engineering/sevenzip-format-sources.md) | 繁體中文（台灣）

本檔案記錄可複用的來源和實現邊界。遷移驗收與剩餘能力在[任務 #50](https://github.com/xiaodaozhi/xdzip/issues/50)維護，固定 Rust 路由及官方產品依賴退出不自動構成跨平臺、人工或發行驗收。通用替代規則見[自研契約](independent-rust-codecs.md)。

<a name="xdzip-规范与参考身份"></a>

## 規範與參考身份

容器欄位、變長整數、方法圖、屬性及檔案對映依據官方 `DOC/7zFormat.txt`、`DOC/Methods.txt`。固定參照為 [7-Zip 26.05 官方原始碼歸檔](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz)，SHA-256 為 `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e`。其中兩個文件的 SHA-256 分別為 `8ec976b73ca88710286be1e991e49dbab485d9c0bf2410a80fa87e657850f82b`、`b16ea5fdc50345fa2cb031f06fe410cea0b089f74a4138ac8141b9e2d816ffc7`。

LZMA 位流另參考 Igor Pavlov 2015-06-14 的[官方規格包](https://www.7-zip.org/a/lzma-specification.7z)，歸檔 SHA-256 為 `4529aca392d6b9598f72086e7d136b1e05d2863bf7a757836d3fd4f49e623b58`，其中規格文字 SHA-256 為 `6a4d441c2fa1d8d37b0189c7147a4b5424d5cdb122a7fc1fbebd022864ed54fb`。僅閱讀格式文字、規範數學與偽碼，不將包內參考解碼器作為 Rust 移植輸入；文件及樣本不因下載而獲得新的再分發許可。

通用 Deflate 依據 RFC 1951，與 ZIP、GZIP 和 MSZIP 共用一套實現。Deflate64 在該實現中顯式選擇 64 KiB 歷史、距離碼 30/31 和長度碼 285 的擴充套件規則，普通 Deflate/MSZIP 不改變。[PKWARE APPNOTE](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT) 的增強 Deflate 段只提供概述，具體位流差異透過固定參考的事實分析和獨立讀取驗證補足，不能將概述冒充完整規格。AES-CBC 的數學和獨立向量依據 [NIST SP 800-38A](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf)，7z 的屬性、UTF-16LE 密碼派生和零填充是額外格式規則，不能從 ZIP AES 或 PKCS#7 推導。BZip2 手冊及 PPMd 的格式名稱沒有完整規定歷史位流、模型重啟和結束行為，不能將這些實現描述成只讀手冊即可完成。

<a name="xdzip-参考分析与原创编码"></a>

## 參考分析與原創編碼

分析者在上述固定原始歸檔中定向閱讀下列組，補充文件省略的欄位、數學狀態、容量重啟、結束及錯誤關係。交接只含事實和辨識要求，不含參考程式碼、逐行改寫、類/陣列佈局、函式劃分、控制流程搬運或註釋翻譯。對應 Rust 演算法由未閱讀這些參考實現的實現者獨立組織；參考分析者不再實現其已分析的演算法。容器作者也未閱讀官方容器實現，依據文件和事實交接獨立編碼。

| 分析主題 | 固定歸檔中的參考範圍 | 許可區分 |
| --- | --- | --- |
| LZMA1/LZMA2 | `C/LzmaDec.c/.h`、`C/Lzma2Dec.c/.h`，編碼器中 range、chunk/reset 與屬性相關部分，C++ Lzma/Lzma2 包裝 | 這些 C 檔案宣告 Public domain；C++ 包裝適用歸檔預設條款 |
| 容器細節 | `CPP/7zip/Archive/7z/7zIn.cpp` 的外部屬性、子流校驗繼承與檔案屬性相關部分 | 歸檔預設 LGPL-2.1-or-later |
| 7z AES | `CPP/7zip/Crypto/7zAes.cpp/.h`，MyAes、FilterCoder 及 7z 編解碼包裝中的塊長、密碼、填充和完成相關部分 | 歸檔預設 LGPL-2.1-or-later，不讀取或移植底層 AES 實現 |
| BZip2 | BZip2Decoder、Const、Crc，區分普通 7z 和獨立 bz2 包裝的錯誤處理 | 歸檔預設 LGPL-2.1-or-later，不據格式名稱套用其他實現的許可 |
| PPMd7 | `C/Ppmd7.c/.h`、`Ppmd7Dec.c`、`Ppmd7Enc.c` 的數學狀態、虛擬容量和重啟事實，以及 C++ Ppmd 包裝的屬性與結束約束 | 這些 C 檔案宣告 Public domain；C++ 包裝適用歸檔預設條款，不復制上下文佈局、分配器實現或控制流程 |
| Deflate64 | `CPP/7zip/Compress/DeflateConst.h`、DeflateDecoder 的 64 位模式相關範圍、Deflate64Register 及 7z 屬性准入 | 歸檔預設 LGPL-2.1-or-later，僅交接與普通 Deflate 的格式差異 |
| ISA、Delta、Swap | `C/Bra.c`、`Bra86.c`、`Bra.h`、`Delta.c`、`SwapBytes.c` 的標量語義及 C++ 方法註冊/屬性/尾部包裝 | 這些 C 檔案宣告 Public domain；C++ 包裝單獨適用歸檔預設條款，SIMD 策略不作為格式 |
| BCJ2 | `C/Bcj2.c/.h` 的四流和數學狀態，C++ Bcj2Coder 的解碼完成與尺寸包裝 | C 檔案宣告 Public domain；C++ 包裝適用歸檔預設條款，不分析或搬用編碼器主體 |

固定歸檔及具體檔案的身份、實際閱讀範圍和事實交接摘要應隨開發證據保留，升級參照時重新核對，不能沿用舊行號證明新版本行為。規範常量、標識和數學關係可用於格式實現，內部最佳化及歷史未定義行為不作為產品許可權。Public domain 宣告不擴充套件到整個歸檔、文件、CPP 包裝或其他樣本。

這不是嚴格潔淨室或法律風險清零的宣告。第三方來源、含官方後端的歷史交付及獨立測試工具繼續遵守各自條款；目前產品不構建官方後端，見[許可](licensing.md)與[對應原始碼](source-delivery.md)。

<a name="xdzip-互操作边界"></a>

## 互操作邊界

自有 writer 和 reader 相互成功不能替代獨立參照。測試分別驗證獨立壓縮器的已知輸出、自有壓縮流由官方讀取、所有輸入消費、結束狀態、CRC、損壞、配額、取消及來源錯誤；元件驗證與宿主/程序/平臺驗收分別記錄。

歷史相容寬容必須逐項確認。例如 BZip2 的 7z 節點可接受最終位元組的非零未使用位，以及塊末四個重複位元組恰無額外計數字節，但仍需正確 CRC、宣告大小及完整流結束。額外尾位元組和拼接流不能據此放行。末位元組位與額外位元組不能混為一種填充。

PPMd7 區分外層已知長度與未知長度的結束 profile。7z 已知長度節點在精確輸出後核對 range 終態和 packed 範圍，不額外消費 END 符號；未知長度的共享 reader 要求 END。格式規定的模型容量重啟與宿主記憶體准入是兩回事，記憶體不足不能觸發偽造的格式重啟。獨立對照輸入包含固定官方 C 編碼器產生的容量重啟流，生成器和原始參考原始碼僅作為本地測試工具，不進入生產依賴。
