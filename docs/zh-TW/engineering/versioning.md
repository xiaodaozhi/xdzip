<a name="xdzip-产品版本与构建身份"></a>

# 產品版本與構建身份

[English](../../engineering/versioning.md) | [简体中文](../../zh-CN/engineering/versioning.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/versioning.md) | [한국어](../../ko-KR/engineering/versioning.md) | [Français](../../fr-FR/engineering/versioning.md) | [Deutsch](../../de-DE/engineering/versioning.md) | [Español](../../es-ES/engineering/versioning.md) | [Português (Brasil)](../../pt-BR/engineering/versioning.md) | [Русский](../../ru-RU/engineering/versioning.md)

可選 `prerelease_number` 分離預釋出序號與構建號。例如 alpha.1 對應 build 14，後續 alpha.2 仍繼續遞增 build。預設保留歷史 `渠道.build` 讀取方式。釋出清單和資產命名見[釋出協議](release-protocol.md)。

[VERSION.json](../../../VERSION.json) 是產品版本、渠道與構建號的唯一維護入口。目前值由構建系統讀取，不在本文另行維護。版本變更見[變更日誌](../../../CHANGELOG.zh-CN.md)，各批次交付與驗證見[專案 issue](https://github.com/xiaodaozhi/xdzip/issues)；版本遞增本身不代表發行獲批。

歷史相容檢查使用固定原始碼基線，與目前構建共同驗證安裝升級、回退及設定相容。完整產品基線由[歷史產品準備指令碼](../../../scripts/prepare-previous-product.py)維護，設定基線由[相容檢查指令碼](../../../scripts/check-settings-compat.py)維護；不在本文複製隨實現變化的提交或目前版本。設定 schema 與產品版本分別管理。

- `version` 使用無前導零的 `major.minor.patch`。不相容變更、相容功能與修復分別按版本範圍管理，0.x 階段仍須說明相容邊界。
- `channel` 支援 development、alpha、beta、rc、stable。非 stable 顯示為 `version-渠道缩写.build`，development 縮寫為 dev；stable 顯示基礎版本。
- `build` 為 1～65535 的整數。準備新的對外交付構建時遞增，不按本機每次編譯自動遞增，也不自動建立標籤或釋出。數字版本各段同樣不超過 65535，以相容 Windows 資源。
- CMake 直接讀取 JSON。Cargo workspace 與 HTML 的派生版本由 `python3 scripts/sync-version.py` 更新，`--check` 用於 CI；所有內部 Rust 包繼承 workspace 版本。同步後更新 Cargo.lock，並檢查 `cargo check --workspace --locked`。
- Windows 四段數值資源為基礎版本加構建號，文字版本保留渠道；macOS 的 `CFBundleShortVersionString` 為基礎版本，`CFBundleVersion` 為構建號，長版本保留渠道。格式遵循 [Apple bundle 版本說明](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleversion)；它們使用各平臺允許的數值格式，不把 Git 提交塞入系統數值欄位。
- Windows 的 `FileDescription` 為 `xdzip Archive Manager`，用於工作管理員等系統位置的程式說明；`ProductName` 保持 `xdzip`，可執行檔名與關聯身份不隨顯示名稱改變。構建檢查從實際 EXE 讀取並核對這些資源。
- 每次桌面構建生成 Git 短提交和工作區 dirty 標誌，沒有 Git 時顯示 unknown。沒有機器路徑、使用者名稱或不穩定時間戳；標頭檔案僅在內容變化時更新，提交之後再次構建即可得到準確的乾淨提交身份。

QML 模組、橋接協議和設定 schema 的版本獨立於產品版本，不隨產品版本自動變更。

`xdzip --version` 與“設定 → 關於 xdzip”讀取同一構建資訊，預覽應用也一致。CLI 在建立 GUI 之前處理版本引數。設定的“關於”頁顯示版本、構建號、提交、實際 Qt、系統和架構，開發渠道與未提交修改明確標記。複製結果為 JSON，不包含個人檔案、路徑、機器名或憑據。專案主頁按鈕由使用者點選後開啟瀏覽器。

關於的大圖示使用對應平臺的 512px PNG，HTML 使用同源向量，不放大 72px 的介面小圖。

關於頁沿用設定視窗的邏輯參數，不含頁尾，支援[隨附的介面語言](../ux/settings-localization.md)、淺色和深色主題，以及 100%、125% 和 150% 字級。目前設計見[設定的關於頁](../../../design/hifi/sources/settings.html?tab=about)。HTML 版本資訊由同一份 JSON 產生，無法取得的原生 Qt 版本或提交值顯示為未知。

驗證入口：`scripts/check-build-version.py` 對照實際 CLI 與原生版本資源，`scripts/test-version.py`、`scripts/check-html-about.py`、`BrowserTest::versionAboutWindow`。截圖與測量僅儲存在 `build/review/about/`。原始碼、系統後設資料與實際執行版本一致是檢查條件；版本號本身不證明簽名、人工驗收或相容性已經透過。
