<a name="xdzip-部署依赖核验与-sbom"></a>

# 部署依賴核驗與 SBOM

[English](../../engineering/deployment-audit.md) | [简体中文](../../zh-CN/engineering/deployment-audit.md) | 繁體中文（台灣）

部署核驗預設生成開發審閱材料，完整性明確標記 `incomplete`，不自動提升為可發行結論；正式 Release 會為同一候選另外收集並隨包提供經過釋出流程核對的材料。格式使用 [CycloneDX 1.6 官方 schema](https://github.com/CycloneDX/specification/blob/55343ba19dee1785acf1ce9191540d5fd7b590db/schema/bom-1.6.schema.json)。schema 的上游提交與三個檔案 SHA-256 固定在 [配置](../../../scripts/sbom-schemas.json)，驗證時只下載並解析這些資源，快取到 build/，不會從 SBOM 中的任意 URL 下載 schema。

| 工件 | 資料來源與用途 | 不能證明的事項 |
| --- | --- | --- |
| source-build.cdx.json | Cargo 宿主解析的工作區依賴圖，含開發／構建工具；宣告的 Qt 模組；字型和品牌原始檔雜湊 | 某個 Rust 包一定靜態連結進產品、所有 Qt 外掛／內嵌第三方均已識別 |
| deployment-audit.json | 指定實際工件根的檔案 SHA-256、原生二進位制匯入、SDK／系統／未解析依賴以及實際 CLI 構建身份 | 所有按名稱動態載入的 QML／外掛、乾淨安裝、系統 ABI、簽名或許可證合規 |
| artifact-files.cdx.json | 實際工件及觀察到的外部 SDK 檔案，按原生匯入建立已知邊；繫結原始碼清單和實際可執行檔案版本／提交／dirty | 完整部署元件身份、靜態連結歸屬、Qt 內嵌庫許可、正式產品 SBOM 已完成 |
| 可選桌面記錄 | 按實際改動和可用環境記錄執行過的桌面檢查及限制 | 未執行的操作、其他平臺或發行批准 |

原始碼不把 Cargo 的路徑型 opaque ID 寫入記錄，工作區包不偽造 crates.io purl。工件路徑相對於指定根或 Qt SDK 記錄，其他外部依賴以內容雜湊和檔名標識，不輸出私人 SDK 絕對路徑。字型使用 OFL-1.1，原創品牌源沿用素材庫的 Apache-2.0；宣告 Qt 模組不據此推斷所有實際部署元件的許可證。

<a name="xdzip-自动核验"></a>

## 自動核驗

`deployment_audit.py` 只檢查明確指定的自有可信構建，不接收使用者壓縮檔。macOS 使用 otool 解析匯入和 RPATH；Windows 使用 MSVC dumpbin 解析普通及延遲 DLL 匯入；Linux 使用 ldd，禁止用於來源不明的可執行檔案。檔案符號連結若逃逸指定工件根，核驗失敗；報告必須寫到根外，避免把輸出包含進自身清單。

- `--mode sdk` 允許解析到外部 Qt SDK，並明確顯示不獨立；未解析匯入仍失敗。遞迴範圍為工件及宣告的 Qt SDK，其他外部庫僅記錄雜湊與未遞迴計數，不視為閉包已核驗。
- `--mode deployed` 不允許非系統外部依賴或未解析匯入。Windows 的 MSVCP、VCRUNTIME 等 VC++ 匯入必須來自包內，不允許開發機 System32 掩蓋缺失；Windows 系統自帶的 UCRT 和 API 合約仍由系統提供。同時繼續執行清除 Qt 開發搜尋路徑後的原生啟動冒煙。
- 未解析匯入、原始碼與實際二進位制版本／提交／dirty 不符、官方 schema 雜湊或格式不符、重複／懸空元件引用均阻斷相關檢查。
- 原生匯入閉合不能替代按名稱載入外掛的覆蓋和乾淨機器檢查。Windows 已部署目錄的匯入透過也不等於安裝器驗收。

按需三平臺 Qt 工作流儲存根目錄下的兩個 cdx.json 和 deployment-audit.json，不自動生成待填寫的人工表。macOS 只讀取本地編譯 .app 與 SDK，普通檢查不執行 macdeployqt、簽名、壓縮或上傳應用包。Linux 透過[獨立執行目錄](linux-deployment.md)提供 Qt／QML／外掛，使用 deployed 核驗及無 SDK 容器驗證；下載 tar.gz 保留執行位。目前只宣告 Ubuntu 24.04、x86_64、X11 基線，不稱為通用 Linux 安裝器。

Windows 可對自有可信安裝器的獨立提取結果和同一候選的便攜 ZIP 執行 `python scripts/check-windows-package-payload.py --installer-app <提取的应用目录> --portable <便携包.zip> --output build/review/payload-comparison.json`。檢查逐檔案雜湊、安裝清單和不應攜帶的 SDK 圖形 DLL；輸出路徑須是新檔案。此靜態對照不執行安裝器或更新，也不代替實際安裝驗證。

本地 macOS 示例，先從一致原始碼重新構建，然後執行：

```sh
python3 scripts/collect-notices.py build/review/wp12/notices
python3 scripts/deployment_audit.py \
  --root build/ui/apps/desktop/xdzip.app \
  --app build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --mode sdk \
  --output build/review/wp12/deployment-audit.json
python3 scripts/sbom.py \
  --audit build/review/wp12/deployment-audit.json \
  --source build/review/wp12/notices/source-build.cdx.json \
  --output build/review/wp12/artifact-files.cdx.json
```

在隔離驗證環境安裝 jsonschema==4.23.0 後，使用該環境的 Python 執行 `scripts/validate-sbom.py`，傳入上述兩個 cdx.json。參考的登錄檔用法見 [jsonschema 4.23 文件](https://python-jsonschema.readthedocs.io/en/v4.23.0/referencing/)。驗證環境的工具不部署到產品中。

<a name="xdzip-下一层发行门槛"></a>

## 下一層發行門檻

已接入[實際元件歸屬](runtime-attribution.md)：使用 SDK SPDX 關聯實際檔案、Qt 模組與內嵌第三方，交付固定雜湊的官方許可文字，記錄 SDK 與上游雜湊差異及未確認歸屬。Rust 普通執行時候選與構建／開發／過程宏工具分開。新增元件關係繫結實際審計併合入工件 SBOM，仍明確標記不完整。

在實際安裝目錄中逐一關聯 DLL、framework 和外掛的上游元件、版本、許可與對應原始碼。Qt 內嵌第三方、靜態 Rust 連結及工具鏈來源也納入歸屬檢查。發行材料如何判定和登記，見[維護者許可複核指南](../../../developers/qa/release-review.md)。
