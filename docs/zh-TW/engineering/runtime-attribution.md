<a name="xdzip-实际组件归属与许可材料"></a>

# 實際元件歸屬與許可材料

[English](../../engineering/runtime-attribution.md) | [简体中文](../../zh-CN/engineering/runtime-attribution.md) | 繁體中文 | [日本語](../../ja-JP/engineering/runtime-attribution.md) | [한국어](../../ko-KR/engineering/runtime-attribution.md) | [Français](../../fr-FR/engineering/runtime-attribution.md) | [Deutsch](../../de-DE/engineering/runtime-attribution.md) | [Español](../../es-ES/engineering/runtime-attribution.md) | [Português (Brasil)](../../pt-BR/engineering/runtime-attribution.md) | [Русский](../../ru-RU/engineering/runtime-attribution.md)

本流程補充檔案級 SBOM，將實際 Qt 庫／外掛與 SDK 的 SPDX 宣告關聯。產物是可複核證據，不是自動許可選擇或合規批准。正式發行完整性繼續為 incomplete。

<a name="xdzip-输入与来源"></a>

## 輸入與來源

Qt 6.8 起提供構建對應的 [SPDX 清單](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html)。工具只讀取本次固定 Qt 6.8.2 SDK 的 sbom/ 中 qtbase、qtdeclarative、qtsvg 和 qttranslations 等執行時宣告，排除 qttools 構建／測試工具。驗證 SPDX 版本、模組名和 Qt 版本，儲存原始文件 SHA-256、namespace 和穩定包引用。輸出只保留必要欄位，不複製包含開發機路徑的上游構建註釋。

`scripts/qt-license-sources.json` 固定官方 Qt 原始碼及 SPDX license-list-data 提交、許可文字 URL 和 SHA-256。下載內容放入 build/ 快取；任何快取或下載的雜湊不符都會失敗。不會跟隨 SPDX 中任意地址下載材料。除 LICENSES/ 的標準文字，還包含 Qt GUI 平滑縮放演算法與 SVG 的原始通知，以及系統依賴宣告涉及的 LGPL-2.1 文字；上游 LicenseRef 原文和版權宣告保留在宣告 JSON 中。

Qt 執行時通知另隨包提供 Qt Network 使用的 Public Suffix List 固定可編輯原始碼、MPL-2.0 原文和獲取說明；收集器核對上游 blob、SHA-256 及許可位元組。原始碼交付還驗證該輸入能重現 Qt 歸檔中的兩種生成資料，見[對應原始碼材料](source-delivery.md)。此項不代替其他 Qt 模組的對應原始碼及替換核驗。


Qt Core 的 Tika MIME 資料通知同時保留 Apache Software Foundation 與英國國家檔案館 PRONOM 的相關署名、Apache-2.0 和 OGL-UK-3.0 許可文字，以及固定版本完整上游 NOTICE。完整 NOTICE 同時描述 Tika 上游的其他元件，不能據此推斷它們隨本產品提供；原始 MIME 資料和 Qt 轉換的核驗見[對應原始碼材料](source-delivery.md)。


<a name="xdzip-文件与组件关联"></a>

## 檔案與元件關聯

1. 對核驗根內檔案重新計算雜湊，拒絕在 deployment-audit.json 之後被修改的檔案。
2. macOS SDK 模式使用實際解析的 Qt SDK 相對路徑和位元組一致性，不據此聲稱獨立部署。
3. Linux 部署階段記錄 SDK 原檔案路徑／SHA-256 與 patchelf 後工件路徑／SHA-256。後續雙向校驗記錄，拒絕過期或篡改的複製證據。
4. Windows 優先按 SDK 檔案位元組身份關聯。檔名唯一但位元組不同的候選僅標記 filename-only-unverified-rewrite，並登記缺口，不能當作來源驗證透過。
5. 透過 SPDX 的 CONTAINS 找到檔案所有者，再遞迴解析 DEPENDS_ON，包括跨 Qt 倉庫引用和內嵌第三方。缺失包引用保留為缺口。不從倉庫總包的 CONTAINS 將全部工具錯誤納入執行時。
6. 非原生 QML／翻譯若沒有逐檔案宣告，只關聯到對應原始碼倉庫範圍，不推斷該檔案的許可證。沒有 SPDX 檔案記錄的 SDK 原生庫、Qt 範圍外原生檔案、NOASSERTION 許可及缺失文字均單獨登記。

SDK 中實際檔案可能與上游構建 SPDX 中的 SHA-1 不同，例如後續安裝處理或簽名；工具保留預期值、對比結果並登記差異，不自行推斷原因。SDK 位元組關聯、部署重寫記錄和上游原始雜湊是三種不同證據。

專用 macOS 打包流程在最終已簽章的應用程式包存在時生成部署關聯，按精確資源位元組或 arm64 section、UUID、匯入版本對應 SDK，並將該報告加入實際檔案 SBOM。比較不覆蓋簽名、load-command 佈局及 LINKEDIT，範圍與原生執行庫專屬通知見[原生執行庫來源與通知](native-runtime-materials.md)。普通 SDK 模式不據此獲得部署結論。

對於從 QML 目錄遷移至 `Contents/PlugIns/quick/` 的外掛，部署關聯允許使用已審計的包內 QML 符號連結定位唯一 SDK 輸入。生成和消費記錄時均重新核對真實連結、審計目標及檔案摘要，再執行相同的位元組或 Mach-O 比較；來源缺失保留未驗證，多個不同來源或連結變更直接拒絕。不能僅憑同名檔案提升歸屬證據，也不消除上游 SDK 摘要差異或許可審查缺口。

Rust 繼續儲存完整宿主構建圖，並從桌面入口 `xdzip-qt-bridge`、格式工作程序入口 `xdzip-codec-bridge`，以及 Windows／Linux 實際構建的 `xdzip-update-helper` 沿普通依賴邊生成 runtime candidates；macOS 不加入獨立更新助手根。構建、開發和 proc-macro 工具及其依賴不屬於候選。這是潛在執行時輸入，不能證明每個包的機器碼最終保留；連結器裁剪、原生 C/C++ 與編譯器執行庫還需進一步核驗。


Cargo 舊式斜槓分隔的簡單許可備選項在原始碼 CycloneDX 的表示式中轉換為 `OR`，並以 `xdzip:cargo_license_raw` 保留上游原文；包含斜槓但無法明確判斷優先順序的寫法直接拒絕。原始構建清單和通知不改寫，標準 `AND`、`OR` 和 `WITH` 表示式保持不變。這是格式規範化，不代表已選擇許可分支或批准發行。


<a name="xdzip-执行与交付"></a>

## 執行與交付

QtNetwork 的已知安全派生庫在檔案記錄中增加 `network_security_variant`，報告同時保留完整、重新核驗過的構建回執。上游 SDK 摘要差異仍保留，發行複核使用固定原始碼、完整補丁、構建回執與部署轉換證據解釋該差異。釋出暫存要求最終 Network 的歸屬與這些位元組相連，檔名匹配不能透過。原始碼與工件 SBOM 分別標記派生身份；不會把全域性 incomplete 改寫成許可批准。

```sh
python3 scripts/prepare-runtime-notices.py --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --output build/review/notices/Qt-runtime
python3 scripts/runtime_attribution.py --audit build/review/deployment-audit.json \
  --root build/review/linux --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --deployment build/review/linux-deployment.json --output build/review/runtime-attribution.json
python3 scripts/sbom.py --audit build/review/deployment-audit.json \
  --source build/review/source-build.cdx.json --attribution build/review/runtime-attribution.json \
  --output build/review/artifact-files.cdx.json
```

Linux 傳入部署重寫記錄；其他平台省略 --deployment。報告寫在核驗根外，避免自身清單遞迴。按需 Qt 和 Release 工作流在部署／最終雜湊之前準備許可材料，Windows 和 Linux 將 notices 放入實際執行目錄，Linux tar.gz 同時交付這些材料。按需 Qt 的 macOS 作業只做 SDK 讀取與日誌／JSON 記錄；經明確授權的專用 Release 工作流另行部署、ad-hoc 簽名、打包並核對實際釋出材料，按[釋出協議](release-protocol.md)保留獨立的部署範圍與雜湊，不以 SDK 記錄替代最終應用包歸屬。

runtime-attribution.json 包含檔案、元件、關係、上游宣告、版權、文字來源與缺口。artifact-files.cdx.json 增加相同元件與已知所有者關係，並繫結該報告和同一實際審計的雜湊／構建身份。官方 CycloneDX schema 與引用檢查繼續執行，compositions 仍為 incomplete。

預設檢查允許明確登記的開發審查缺口，禁止把它們靜默標記透過；身份、路徑、位元組、版本、複製記錄或許可文字雜湊錯誤直接失敗。--require-complete 是發行阻斷檢查，目前會因未完成歸屬或發行材料而失敗。測試覆蓋檔案被改寫、過期複製記錄、路徑逃逸、版本錯誤、跨文件缺失、部分 SBOM 及 Rust 工具排除。

<a name="xdzip-尚未解决的发行门槛"></a>

## 尚未解決的發行門檻

已增加[原始碼交付收集器](source-delivery.md)，儲存實際專案原始碼、Rust vendor、四份執行時 Qt 模組歸檔及兩份構建模組歸檔、重建及動態替換說明。它不能證明 SDK 與上游逐項構建等價。每次發行還須核對精確靜態連結歸屬、作業系統／再分發執行庫、所有內嵌資源逐項許可、開源許可分支選擇及履約。[本地 Mac 部署核驗](macos-deployment.md)中的後設資料修改替換，與[實際原始碼 Qt 構建及載入／替換](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement)是不同流程，分別保留證據；均不能推斷任意 SDK 的位元組級構建等價。源清單與 SPDX 欄位不能替代許可義務。`v0.1.0` 的歷史候選來源與許可結論見 [#4](https://github.com/xiaodaozhi/xdzip/issues/4) 和 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)，後續候選單獨複核，付費簽名身份不作為門檻；既有桌面驗收在 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)保留。

[Qt SDK 對應核驗](qt-sdk-correspondence.md)進一步記錄固定歸檔提交、SDK 原始碼宣告逐檔案雜湊、安裝配置與已知 AGL 消費端補丁。宣告覆蓋範圍內匹配不等於完整 SDK 構建等價，原有發行阻斷保持不變。

自動化收尾補充實際連結器 map／靜態庫成員、執行時資源雜湊、可信原生依賴版本資訊及精確可逆的二進位差異分析，見[完整契約](../../../developers/engineering/candidate-verification.md)。只在重建出的位元組雜湊精確匹配上游宣告時標記轉換已解釋，原始 SDK 差異不刪除；未匹配的釋出者歷史不能憑猜測補寫。
