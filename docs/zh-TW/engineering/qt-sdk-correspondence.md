<a name="xdzip-qt-sdk-与源码对应核验"></a>

# Qt SDK 與原始碼對應核驗

[English](../../engineering/qt-sdk-correspondence.md) | [简体中文](../../zh-CN/engineering/qt-sdk-correspondence.md) | 繁體中文（台灣）

原始碼對應核驗以官方歸檔的實際位元組為準，不以相同版本號推斷 SDK 構建等價。`scripts/qt_sdk_correspondence.py` 將固定原始碼歸檔、SDK 自帶的原始碼 SPDX、構建宣告、已安裝配置與已知消費端補丁關聯，輸出到 build/；不修改 SDK，不批准許可分支。

```sh
python3 scripts/qt_sdk_correspondence.py \
  --qt-prefix build/toolchain/Qt/6.8.2/macos \
  --output build/review/qt-sdk-correspondence.json
```

預設離線讀取 build/source-cache，缺失則失敗；顯式 --fetch 才按 [固定清單](../../../scripts/corresponding-sources.json)下載並校驗官方歸檔。按需三平臺 Qt 工作流安裝固定 SDK 後執行 --fetch，將 JSON 與其他驗證記錄一起保留；Mac 仍只上傳記錄，不部署、簽名或打包應用。原始碼交付收集器傳入 --qt-prefix 時，將該記錄與補丁、固定清單一起納入交付雜湊，見[原始碼交付](source-delivery.md)。

<a name="xdzip-核验层次"></a>

## 核驗層次

1. 歸檔 SHA-256 必須匹配固定值，歸檔根目錄、成員路徑與 .tag 的固定標記必須匹配。標記的 Git 物件型別單獨記錄；若標記是樹物件，先對固定的原始提交物件重算 Git 物件雜湊並核對其 tree 欄位。逐檔案讀取，不解包上游原始碼。
2. 若 SDK 提供模組的 .source.spdx，逐檔案比較其 SHA-1 與歸檔內容。記錄相同數量、不同檔案雙方雜湊、SDK 宣告中缺失於歸檔的檔案及歸檔未覆蓋數量。SHA-1 只複核上游原宣告，整個歸檔另以固定 SHA-256 繫結。沒有宣告、宣告差異及未覆蓋檔案不被補寫為驗證透過。
3. 儲存 SDK 模組構建 SPDX 的雜湊、編譯器宣告與 configure 引數，同時讀取安裝後的 modules/*.json、qconfig*.pri 和 QT_FEATURE 配置頭，儲存內容及雜湊。輸出將上游構建者的絕對目錄替換為佔位符，原始證據透過檔案雜湊繫結。配置不是完整 CMake cache、構建環境或工具鏈材料；SDK 預設架構欄位不代表實際庫的所有架構，不能據此把 universal 庫誤判為 x86_64。
4. [AGL 補丁](../../../scripts/qt-patches/macos-no-agl.patch)只修改 SDK 的 lib/cmake/Qt6/FindWrapOpenGL.cmake，用於本專案消費 Qt 時避免連結舊 AGL。工具從固定 QtBase 歸檔讀取補丁原文，重放統一差異，驗證補丁、原文、結果三個 SHA-256，再檢查實際 SDK 檔案。只接受原版或已知補丁版；Windows SDK 的 CRLF 僅轉換行尾後比較，同時保留實際位元組 SHA-256、LF SHA-256 與轉換標記，其他未知修改直接失敗。它不改變 Qt 執行庫的機器碼，不能當作 Qt 已重編譯的證據。完整 SDK 修改歷史仍待核驗。

原始碼對應記錄使用 schema 2，分別儲存固定的 archive_commit、歸檔實際 archive_marker、archive_marker_type 和 commit_tree_verified。Qt Declarative 6.8.2 的 `.tag` 是 Git 樹 `0a8139c574fe150aba92d5f2484a8f3fdb774977`，釋出標籤對應提交為 `75534f3e7fff24ed7ccb364e2ed9950a73da879f`；倉庫固定的 base64 原始提交物件可離線重算提交雜湊並驗證其 tree 欄位。其他目前固定模組的標記為提交。樹關係只說明這兩個 Git 物件的關係，不證明歸檔包含完整樹或 SDK 與該樹構建等價。

qt-license-sources.json 的 source_commits 僅表示許可文字的固定取回提交，與上述歸檔標記和釋出提交分別儲存，不能冒充 SDK 編譯提交。歷史 schema 1 材料把 Qt Declarative 樹標記寫入 archive_commit，已公開資產保留原樣；後續解釋應以物件型別修正，許可證文字仍按原有 SHA-256 檢查。

<a name="xdzip-结果解释与发行边界"></a>

## 結果解釋與發行邊界

QtNetwork 安全派生庫另記錄 `network_security`，其回執繫結補丁、構建器、原 QtCore 與實際 QtNetwork 摘要、功能及真實傳輸驗證。工具讀取回執時重新核對實際 SDK 位元組。該項證明限定在修改庫，不將上游 SPDX 改寫為派生庫證明，也不改變其他模組的未覆蓋事實。輸入與維護方法見[Qt 補丁說明](../../../patches/qt/README.md)。

模組是否提供原始碼 SPDX、匹配／缺失／未覆蓋數量、配置和補丁狀態均以本次生成的 JSON 為準，不把某個 SDK 的觀測推廣到其他 SDK。沒有逐檔案宣告時不能推斷對應透過，必須保留具體缺口。

報告狀態固定為 evidence-collected-not-build-equivalence。[實際 Qt 原始碼構建和載入／替換](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement)已有獨立流程，但不能由此推斷原 SDK 的位元組級重現。完整工具鏈與系統 SDK 材料、未覆蓋／生成檔案、執行庫來源及許可履約尚未全部核清時，不宣稱全部對應原始碼義務已履行，不宣稱位元組級可重複構建。既有[執行時歸屬](runtime-attribution.md)的 incomplete、許可選擇及平臺執行庫複核繼續保留。
