<a name="xdzip-cxx-桥接依赖依据"></a>

# cxx 橋接依賴依據

[English](../../engineering/cxx-admission.md) | [简体中文](../../zh-CN/engineering/cxx-admission.md) | 繁體中文（台灣）

Qt/Rust 採用 `cxx` 生成的型別化所有權橋接；C++ 保留 Qt 物件，Rust 保留領域和執行時。`cxx` 與 `cxx-build` 必須精確同版，以 [Cargo.lock](../../../Cargo.lock) 及[工具鏈策略](toolchain.md)為準。cxx 不作為第三方格式擴充套件的穩定 ABI。手寫 C ABI 雖減少依賴，卻增加字串、釋放函式與所有權不變數維護成本。

<a name="xdzip-来源与许可"></a>

## 來源與許可

來源為 crates.io，上游為 [dtolnay/cxx](https://github.com/dtolnay/cxx)。依賴的來源雜湊、版本、features、構建指令碼和許可宣告從鎖檔案及[橋接依賴清單](../../../specs/engineering/bridge-dependencies.json)取得，發行時重新核對並彙總實際適用通知。鎖檔案包含目標條件不成立的依賴，不等於目前二進位制實際連結清單，也不覆蓋編譯器、Qt 或系統庫。

<a name="xdzip-构建与所有权边界"></a>

## 構建與所有權邊界

cxx-build 生成原始碼和標頭檔案並呼叫 C++ 編譯工具；cxx 編譯包內橋接實現，link-cplusplus 負責 C++ 標準庫連結。依賴獲取由 Cargo 按鎖檔案校驗，新增構建指令碼或可選特性必須重新審查其網路與執行行為，不把歷史檢查當作新版本安全結論。

保持 std/alloc 等實際產品所需特性；裁剪需複驗 UTF-8、錯誤傳播、唯一控制代碼、背壓、關閉與排空行為。[執行時契約](product-runtime.md)和[橋接設計](../design/qt-rust-bridge.md)定義實際介面。發行許可複核按[發行審查流程](../../../developers/qa/release-review.md)執行。

具體探針、CI 執行與驗收結論記錄在 [WP-01 issue](https://github.com/xiaodaozhi/xdzip/issues/1)及其 Actions 工件，不在此累積執行報告。
