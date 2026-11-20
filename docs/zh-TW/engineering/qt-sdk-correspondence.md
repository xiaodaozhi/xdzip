<a name="xdzip-qt-sdk-与源码对应核验"></a>

# Qt SDK 與原始碼對應核驗

[English](../../engineering/qt-sdk-correspondence.md) | [简体中文](../../zh-CN/engineering/qt-sdk-correspondence.md) | 繁體中文（台灣）

原始碼對應驗證以官方封存檔的實際位元組為準，相同版本號不能證明 SDK 的建置等價。`scripts/qt_sdk_correspondence.py` 關聯固定的原始碼封存檔、SDK 提供的原始碼 SPDX、建置宣告、已安裝設定及已知的使用端修補，並將結果寫入 build/。此工具不修改 SDK，也不核准授權分支。

```sh
python3 scripts/qt_sdk_correspondence.py \
  --qt-prefix build/toolchain/Qt/6.8.2/macos \
  --output build/review/qt-sdk-correspondence.json
```

預設離線讀取 build/source-cache，缺少資料時即失敗；只有明確指定 --fetch 才會依[固定清單](../../../scripts/corresponding-sources.json)下載並驗證官方封存檔。按需執行的三平台 Qt 工作流程在安裝固定 SDK 後執行 --fetch，並將 JSON 與其他驗證紀錄一同保留。Mac 仍只上傳紀錄，不部署、簽署或打包應用程式。原始碼交付收集器收到 --qt-prefix 時，會將此紀錄、修補檔與固定清單納入交付雜湊；見[原始碼交付](source-delivery.md)。

<a name="xdzip-核验层次"></a>

## 核驗層次

1. 封存檔 SHA-256 必須符合固定值，封存檔根目錄、成員路徑與固定的 `.tag` 標記也須相符。標記的 Git 物件類型分開記錄；若標記是樹物件，先重新計算固定的原始提交物件雜湊並驗證其 tree 欄位。逐一讀取檔案，不解壓上游原始碼。
2. 若 SDK 提供模組的 .source.spdx，逐檔比較其宣告的 SHA-1 與封存檔內容。記錄相符數量、差異檔案的雙方雜湊、SDK 宣告但封存檔缺少的檔案，以及封存檔未涵蓋的數量。SHA-1 只用來檢查上游原始宣告；整個封存檔另由固定 SHA-256 綁定。缺少宣告、差異與未涵蓋檔案均不得補記為通過。
3. 保留 SDK 模組建置 SPDX 雜湊、編譯器宣告與 configure 參數，同時讀取已安裝的 modules/*.json、qconfig*.pri 和 QT_FEATURE 設定標頭，保留內容及雜湊。輸出以佔位符取代上游建置者的絕對目錄；檔案雜湊綁定原始證據。這些設定不是完整 CMake 快取、建置環境或工具鏈資料。SDK 預設架構欄位不代表實際函式庫的所有架構，不能據此將通用函式庫誤判為 x86_64。
4. [AGL 修補檔](../../../scripts/qt-patches/macos-no-agl.patch)僅修改 SDK 的 lib/cmake/Qt6/FindWrapOpenGL.cmake，讓本專案使用 Qt 時避免連結舊版 AGL。工具從固定的 QtBase 封存檔讀取原文、套用統一差異，驗證修補檔、原文及結果的 SHA-256，再檢查實際 SDK 檔案。只接受原版或已知修補版。Windows SDK 的 CRLF 須先正規化行尾再比較，同時保留實際位元組 SHA-256、LF SHA-256 及轉換標記；其他未知變更會失敗。這不會改變 Qt 執行階段的機器碼，也不能證明 Qt 已重新建置。完整 SDK 修改歷史仍須驗證。

原始碼對應紀錄使用 schema 2，分別保留固定的 archive_commit、封存檔實際的 archive_marker、archive_marker_type 和 commit_tree_verified。Qt Declarative 6.8.2 的 `.tag` 是 Git 樹 `0a8139c574fe150aba92d5f2484a8f3fdb774977`，發行標籤對應提交 `75534f3e7fff24ed7ccb364e2ed9950a73da879f`。儲存庫中固定的 base64 原始提交物件可供離線重算提交雜湊並驗證 tree 欄位。其他目前固定模組的標記是提交。此樹關係只證明這兩個 Git 物件之間的關係，不證明封存檔包含完整樹，也不證明 SDK 與其建置等價。

qt-license-sources.json 的 source_commits 只識別取得授權文字時固定的提交，與封存檔標記和發行提交分開儲存，不能宣稱是 SDK 建置提交。歷史 schema 1 資料將 Qt Declarative 樹標記放在 archive_commit；已發布資產保持不變。後續說明應更正物件類型，並繼續以原始 SHA-256 驗證授權文字。

<a name="xdzip-结果解释与发行边界"></a>

## 結果解釋與發行邊界

QtNetwork 安全衍生函式庫另記錄 `network_security`；其回執綁定修補檔、建置工具、原始 QtCore 與實際 QtNetwork 的摘要、功能及實際傳輸驗證。工具讀取回執時會重新檢查實際 SDK 位元組。此證據只適用於修改後的函式庫，不會將上游 SPDX 改寫為衍生版本的證明，也不改變其他模組未涵蓋的事實。輸入與維護方式見 [Qt 修補說明](../../../patches/qt/README.md)。

模組是否提供原始碼 SPDX、相符／缺少／未涵蓋的數量、設定和修補狀態，均以本次產生的 JSON 為準。不要將某個 SDK 的觀察結果推及其他 SDK。沒有逐檔宣告時，不得推斷對應驗證通過；須保留具體缺口。

報告狀態固定為 evidence-collected-not-build-equivalence。[實際 Qt 原始碼建置和載入／替換](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement)有獨立流程，但不能由此證明原 SDK 已逐位元組重現。在完整工具鏈與系統 SDK 資料、未涵蓋或產生的檔案、執行階段來源及授權履約全部釐清之前，不得宣稱已履行所有對應原始碼義務，也不得宣稱可逐位元組重現建置。既有[執行階段歸屬](runtime-attribution.md)的 incomplete 狀態、授權選擇和平台執行階段複核仍然保留。
