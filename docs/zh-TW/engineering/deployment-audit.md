<a name="xdzip-部署依赖核验与-sbom"></a>
<a name="部署依賴核驗與-sbom"></a>

# 部署相依套件驗證與 SBOM

[English](../../engineering/deployment-audit.md) | [简体中文](../../zh-CN/engineering/deployment-audit.md) | 繁體中文（台灣）

部署驗證預設產生開發審查資料，明確標示為 `incomplete`，不會自動證明已可發行。正式 Release 會另外為同一候選版本收集資料，經發行流程審查後隨套件提供。格式使用 [CycloneDX 1.6 官方結構描述](https://github.com/CycloneDX/specification/blob/55343ba19dee1785acf1ce9191540d5fd7b590db/schema/bom-1.6.schema.json)。上游提交與三個結構描述檔的 SHA-256 固定在[設定](../../../scripts/sbom-schemas.json)中；驗證只下載並解析這些資源，快取於 build/，不會從 SBOM 內任意網址下載結構描述。

| 成品 | 資料來源與用途 | 無法證明的事項 |
| --- | --- | --- |
| source-build.cdx.json | Cargo 針對主機解析的工作區相依圖，包含開發與建置工具；宣告的 Qt 模組；字型與品牌原始檔雜湊 | Rust 套件必定靜態連結至產品，或所有 Qt 外掛程式及內嵌第三方元件均已識別 |
| deployment-audit.json | 指定實際成品根目錄下的檔案 SHA-256、原生二進位檔匯入、SDK／系統／未解析相依套件，以及實際 CLI 建置識別 | 所有依名稱動態載入的 QML／外掛程式、乾淨安裝、系統 ABI、簽章或授權合規 |
| artifact-files.cdx.json | 實際成品與觀察到的外部 SDK 檔案，原生匯入的已知關聯，綁定原始碼清單及實際執行檔的版本／提交／dirty 狀態 | 已完整識別部署元件、靜態連結歸屬、Qt 內嵌函式庫授權，或已完成正式產品 SBOM |
| 選用的桌面紀錄 | 依變更與可用環境記錄已執行的桌面檢查及限制 | 未執行的動作、其他平台或發行核准 |

原始碼紀錄不包含 Cargo 路徑型的不透明 ID，也不為工作區套件虛構 crates.io purl。成品路徑以指定根目錄或 Qt SDK 為相對基準；其他外部相依套件依內容雜湊與檔名識別，不輸出私人 SDK 絕對路徑。字型使用 OFL-1.1，原創品牌素材沿用素材庫的 Apache-2.0。宣告 Qt 模組不能據此推斷所有實際部署元件的授權。

<a name="xdzip-自动核验"></a>

## 自動核驗

`deployment_audit.py` 只檢查明確指定、可信的專案建置，不檢查使用者壓縮檔。macOS 使用 otool 分析匯入與 RPATH；Windows 使用 MSVC dumpbin 分析一般及延遲 DLL 匯入；Linux 使用 ldd，不得用於來源不明的執行檔。檔案符號連結若超出指定成品根目錄，驗證即失敗。報告必須寫在根目錄之外，避免清單包含自己的輸出。

- `--mode sdk` 允許解析至外部 Qt SDK，並明確標示此建置不是獨立版本；未解析的匯入仍會失敗。遞迴範圍涵蓋成品及宣告的 Qt SDK；其他外部函式庫只記錄雜湊與未遍歷數量，不視為已驗證完整依賴閉包。
- `--mode deployed` 不允許非系統外部相依套件或未解析匯入。Windows 的 MSVCP、VCRUNTIME 等 VC++ 匯入必須來自套件，不能讓開發電腦的 System32 掩蓋缺失；系統 UCRT 與 API 契約仍由系統提供。仍須清除 Qt 開發搜尋路徑後執行原生啟動冒煙測試。
- 未解析匯入、原始碼與實際二進位檔的版本／提交／dirty 狀態不符、官方結構描述的雜湊或格式不符、重複或懸空的元件參照，均會阻斷相關檢查。
- 原生匯入閉包不能取代依名稱載入的外掛程式覆蓋及乾淨電腦檢查。Windows 已部署目錄的匯入檢查通過也不代表安裝程式驗收通過。

按需執行的三平台 Qt 工作流程保留根目錄中的兩個 cdx.json 和 deployment-audit.json，不會自動產生空白人工表單。macOS 只讀取本機建置的 .app 與 SDK；一般檢查不執行 macdeployqt，也不簽署、壓縮或上傳應用程式套件。Linux 透過[獨立執行階段目錄](linux-deployment.md)提供 Qt／QML／外掛程式，使用 deployed 驗證和不含 SDK 的容器；下載的 tar.gz 保留執行權限位元。目前只宣告 Ubuntu 24.04、x86_64、X11 基線，不宣稱是通用 Linux 安裝程式。

對可信的專案 Windows 成品，請獨立擷取安裝程式，並執行 `python scripts/check-windows-package-payload.py --installer-app <extracted-app-directory> --portable <portable-package.zip> --output build/review/payload-comparison.json`，將其應用程式目錄與同一候選版本的可攜 ZIP 比較。檢查逐檔雜湊、安裝清單及不應包含的 SDK 圖形 DLL；輸出路徑必須是新檔案。此靜態比較不執行安裝程式或更新，也不取代實際安裝驗證。

本機 macOS 範例，從一致的原始碼重新建置後執行：

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

在隔離驗證環境安裝 jsonschema==4.23.0 後，使用該環境的 Python 執行 `scripts/validate-sbom.py`，傳入上述兩個 cdx.json。登錄表的用法見 [jsonschema 4.23 文件](https://python-jsonschema.readthedocs.io/en/v4.23.0/referencing/)。驗證工具不部署至產品。

<a name="xdzip-下一层发行门槛"></a>
<a name="下一層發行門檻"></a>

## 歸屬結果與驗證限制

已整合[實際元件歸屬](runtime-attribution.md)：以 SDK SPDX 關聯實際檔案、Qt 模組與內嵌第三方，交付固定雜湊的官方授權文字，並記錄 SDK 與上游雜湊差異及尚未確認的歸屬。一般 Rust 執行階段候選與建置／開發／程序巨集工具分開。新增元件關係綁定實際稽核並合入成品 SBOM，但仍明確標示為不完整。

在實際安裝目錄中，逐一將交付的 DLL、framework 和外掛程式關聯至其上游元件、版本、授權與對應原始碼。Qt 內嵌元件、靜態 Rust 連結歸屬和工具鏈來源須分別審查。候選版本的判定見[發行審查指南](../../../developers/qa/release-review.md)。
