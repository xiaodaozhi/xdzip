<a name="xdzip-构建并参与贡献"></a>

# 構建並參與貢獻

[English](../../contributing/README.md) | [简体中文](../../zh-CN/contributing/README.md) | 繁體中文（台灣） | [日本語](../../ja-JP/contributing/README.md) | [한국어](../../ko-KR/contributing/README.md) | [Français](../../fr-FR/contributing/README.md) | [Deutsch](../../de-DE/contributing/README.md) | [Español](../../es-ES/contributing/README.md) | [Português (Brasil)](../../pt-BR/contributing/README.md) | [Русский](../../ru-RU/contributing/README.md)

先閱讀[貢獻約定](../../../CONTRIBUTING.zh-CN.md)，查詢已有議題或為本次改動新建議題。寫明預期行為、適用平臺和可實際驗證的最小復現場景。使用專案自制或允許再分發的樣本。各模組的原始碼位置見[倉庫佈局](../engineering/repository-layout.md)。

<a name="xdzip-构建需要验证的部分"></a>

## 構建需要驗證的部分

在倉庫根目錄執行命令。Windows 請在 x64 MSVC 開發環境中使用 `python`。macOS 請使用 Apple Silicon 原生終端和 Xcode Command Line Tools。Ubuntu 請按平臺清單準備依賴。[構建指南](../engineering/developer-build.md#xdzip-%E7%8E%AF%E5%A2%83%E4%B8%8E%E5%85%A5%E5%8F%A3)列出了前置工具，以及倉庫鎖定的 Rust 和 Qt 工具鏈。

首次構建桌面版可執行以下命令。

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

準備階段的開關會透過已有的 rustup 安裝鎖定的 Rust 工具鏈，並在需要時下載鎖定的 Qt SDK。已有共享依賴可複用 `build/toolchain/`。如果使用預設位置之外的相容 Qt SDK，在相關階段傳入相同的 `--qt-prefix`。需要執行桌面測試套件時，請先完成同一配置的構建，再執行 `python3 scripts/desktop.py test`。工作程序的完整測試套件需要按[構建指南](../engineering/developer-build.md)準備測試輸入。

開發獨立 CLI 時，先構建，再用生成的 CLI 和工作程序執行驗證。

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
```

預設輸出在 `build/cli`。獨立 CLI 需要鎖定的 Rust 工具鏈、CMake 和平臺 C++ 工具鏈，Ubuntu 還需要 `libssl-dev` 與 `pkg-config`。平臺細節見[命令列構建契約](../engineering/command-line.md#xdzip-%E6%9E%84%E5%BB%BA%E4%B8%8E%E5%91%BD%E4%BB%A4%E5%AE%89%E8%A3%85)。

<a name="xdzip-验证受影响的行为"></a>

## 驗證受影響的行為

有匹配的構建時可以複用。純文件改動先執行以下檢查，並驗證受影響的生成產物。改動或必需檢查涉及桌面構建時，再執行相應構建。

```sh
python3 scripts/check-docs.py
git diff --check
```

| 改動型別 | 原始碼與契約 | 執行或核對 |
| --- | --- | --- |
| 文件和範例 | 同步更新譯文、連結和章節錨點。根 README 和 `docs/` 有十種語言版本，`developers/` 下的指南使用美式英語和簡體中文。對照實作契約，用小型自製測試檔驗證變更後的指令。 | 執行上方的文件檢查，開啟改動後的章節連結，核對相關生成輸出。 |
| CLI | 從 `crates/xdzip-cli` 追到共享 Rust 執行時。[命令列契約](../engineering/command-line.md) | 基於目前構建執行獨立 CLI 驗證。改動命令或輸出時，檢查 stdout、stderr、退出狀態和寫出的檔案，並覆蓋適用的失敗場景。 |
| Rust 領域邏輯、格式或歸檔任務 | 從 `xdzip-domain`、對應編解碼模組、`xdzip-runtime` 或 `xdzip-io` 入手。查桌面命令時，先看 `BrowserController` 與 `RuntimeConnection`。[格式登記](../engineering/format-registry.md)、[執行時](../engineering/product-runtime.md)、[檔案安全](../design/file-security.md) | 執行 `cargo fmt --all -- --check` 和受影響的包測試。驗證實際執行時入口及相關的失敗、取消或提交行為。呼叫原生格式工作程序時使用[工作程序測試入口](../../../developers/qa/runtime-worker-tests.md)。登記表改動後執行 `python3 scripts/generate-format-registry.py` 重新生成，再加 `--check` 核對，並執行 `python3 scripts/test-format-registry.py`。 |
| QML 視窗、控制元件或本地化 | 檢視 `apps/desktop/ui`、`resources/i18n` 和 `specs/visual`。改視窗前閱讀[視覺規範](../visual/design-standard.md)與[設計流程](../visual/design-workflow.md)，在[視窗登記表](../../../specs/visual/window-registry.json)登記契約。 | 視窗改動執行 `python3 scripts/check-window-contracts.py --self-test`、`python3 scripts/check-operation-components.py` 和受影響的雙語多字號佈局測試。本地化改動執行 `python3 scripts/check-ui-i18n.py`。按要求在相關平臺驗證原生互動或視覺表現。 |
| 安裝、系統選單或更新 | 查閱[構建與部署](../engineering/developer-build.md)、[檔案關聯](../engineering/installers-associations.md)與[軟體更新](../engineering/software-update.md)。 | 核對受影響的安裝或註冊入口、實際交付包身份、錯誤及恢復流程。按 [CI 策略](../../../developers/engineering/ci-policy.md)選擇對應平臺工作流。 |

<a name="xdzip-说明验证结果"></a>

## 說明驗證結果

在 PR 中說明改動後的行為，列出執行過的命令、驗證平臺及仍待覆蓋的檢查。明確哪些結果來自離屏測試、原生桌面或實際安裝。[驗證指南](../../../developers/qa/README.md)說明了各類驗證的適用範圍。各模組契約見[技術參考](../reference/README.md)，發行流程見[維護者手冊](../../../developers/README.md)。
