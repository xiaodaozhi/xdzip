<a name="xdzip-编码规范与质量工程"></a>

# 編碼規範與質量工程

[English](../../design/quality-engineering.md) | [简体中文](../../zh-CN/design/quality-engineering.md) | 繁體中文（台灣） | [日本語](../../ja-JP/design/quality-engineering.md) | [한국어](../../ko-KR/design/quality-engineering.md) | [Français](../../fr-FR/design/quality-engineering.md) | [Deutsch](../../de-DE/design/quality-engineering.md) | [Español](../../es-ES/design/quality-engineering.md) | [Português (Brasil)](../../pt-BR/design/quality-engineering.md) | [Русский](../../ru-RU/design/quality-engineering.md)

<a name="xdzip-代码与所有权"></a>

## 程式碼與所有權

- **Rust。** 遵循工作區工具鏈、檢查規則與鎖檔案。校驗外部長度、偏移和整數轉換，保留未知大小與型別化錯誤。記錄 unsafe 與 FFI 的前提、所有權、生命週期和失敗行為。縮短同步臨界區，將歸檔 I/O 留在臨界區之外。
- **C++ 與 Qt。** 使用 C++17、RAII 和明確的 QObject 所有權。在介面執行緒修改模型，跨執行緒傳遞自有資料的值，並將排隊通知繫結到存活的接收方。接收執行緒仍需介面取走事件時，以非同步方式關閉。歸檔和檔案規則仍由 Rust 負責。[Qt/Rust 橋接](qt-rust-bridge.md)
- **QML。** 透過控制器表達狀態和操作意圖。複用共享控制元件及生成的視覺和文字資源。列表委託無需訪問磁碟或解釋歸檔。視窗變動遵循登記、原生行為及視覺規範。[Qt Quick 介面](qt-quick-ui.md)、[視覺流程](../visual/design-workflow.md)

記錄不變數及失敗後留下的狀態。例如，檔案程式碼應說明提交許可保護什麼、校驗哪些身份，以及出錯時哪些內容可能已經可見。[檔案提交](file-security.md)

<a name="xdzip-验证层次"></a>

## 驗證層次

| 改動 | 對應證據 |
| --- | --- |
| 領域、查詢與服務 | 用 Rust 測試身份、限額、同名記錄、排序、過期響應、有界交付與取消。 |
| 格式操作 | 使用專案自有樣本，在適用時進行獨立往返核驗，同時覆蓋拒絕、損壞、不支援的配置和額度。 |
| 原生檔案與任務 | 在受影響平臺核對檔案身份和實際位元組、取消與提交競態、已有輸出保護和清理結果。 |
| 桌面、Qt 與輔助功能 | 覆蓋模型和控制器、QML 與共享控制元件，按影響範圍執行原生互動，驗證實際視覺或輔助技術行為。 |
| 效能與分發 | 記錄繫結提交的負載、渲染及資源指標，或對應發行渠道的構建身份、安裝更新結果和依賴回執。 |

[驗收指南](../../../developers/qa/README.md)用於確定本次改動所需的檢查。協議一致性、夾具或離屏驗證、原生自動化和桌面人工觀察各有證據範圍。現有迴歸入口與交付邊界見[開發構建指南](../engineering/developer-build.md)和[追蹤矩陣](requirements-traceability.md)。

<a name="xdzip-现有-ci-与补充验证方法"></a>

## 現有 CI 與補充驗證方法

常規 [CI 工作流](../../../.github/workflows/docs.yml)檢查文件、生成資源、佈局契約、格式和工作流結構。更重的原生 Rust、格式及 Qt 檢查分別手動排程 Foundation 和 Qt 工作流。發行工件由專用發行任務處理。[CI 策略](../../../developers/engineering/ci-policy.md)描述觸發條件和門檻。需要執行哪些本地或手動排程的測試，取決於本次受影響的程式碼路徑和平臺。

模糊測試、執行時檢查工具（Sanitizer）、模型檢查與效能基準是按覆蓋需要選用的方法。使用有界且可再分發的樣本，單獨記錄實際證據。敏感安全報告遵循[安全政策](../../../SECURITY.zh-CN.md)。設計中的計劃檢查或方法示例本身不表示已有定時流水線，也不表示已經驗收。

<a name="xdzip-维护设计"></a>

## 維護設計

實作變更若涉及識別資訊、命令、能力、任務或檔案寫入範圍，請同步更新對應設計、工程參考及驗收連結，並維持各語言版本一致。[架構決策](architecture-decisions.md)記錄取捨，[效能設計](performance.md)定義量測規則。實際結果與限制另行記錄於對應變更。
