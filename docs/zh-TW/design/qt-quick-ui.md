<a name="xdzip-qt-quick-桌面界面"></a>

# Qt Quick 桌面介面

[English](../../design/qt-quick-ui.md) | [简体中文](../../zh-CN/design/qt-quick-ui.md) | 繁體中文（台灣）

桌面使用 Qt Quick 呈現介面與互動，C++ 控制器轉換操作意圖並管理檢視狀態。歸檔解釋、任務規則和檔案授權由 Rust 執行時持有。下文呼叫關係見[系統架構](system-architecture.md)和[橋接契約](qt-rust-bridge.md)。

<a name="xdzip-窗口会话与模型"></a>

## 視窗、會話與模型

| 元件 | 目前職責 |
| --- | --- |
| `Browser.qml` 與 `SessionPage.qml` | 承載歡迎頁、歸檔標籤、路徑與搜尋控制元件、目錄導航、列表或網格、屬性和操作入口。頁面展示控制器目前狀態並呼叫命令。 |
| `ApplicationSessions` | 將開啟請求交給獨立瀏覽會話，協調標籤或視窗、共享任務入口與退出。開啟方式設定對後續開啟生效。關閉頁面會釋放瀏覽呈現，任務控制器仍可繼續執行。 |
| `BrowserController` 與 `ArchiveTableModel` | 管理選區、列表狀態和結果身份。小結果完整提交到模型，大結果保留完整邏輯行數，最多常駐 1,024 行並按需載入。未載入行無法提交條目操作。 |
| `TaskListModel` 與操作視窗 | 呈現有界任務快照、進度、結果和目前的待決定事項。任務操作透過桌面的組合身份發往所屬控制器。 |
| 設定與平臺適配層 | 管理外觀、語言、字號和應用行為。由原生適配處理各系統的托盤、啟用、檔案對話方塊、關聯及傳輸交接。 |

[桌面入口](../../../apps/desktop/browser_main.cpp)、[會話頁面](../../../apps/desktop/ui/SessionPage.qml)、[控制器與模型](../../../apps/desktop/BrowserController.h)、[應用會話](../engineering/application-sessions.md)、[設定與本地化](../ux/settings-localization.md)

<a name="xdzip-浏览与操作流程"></a>

## 瀏覽與操作流程

開啟後釋出完整的後設資料快照。新的搜尋、排序或導航請求攜帶自己的結果身份。過期分頁無法替換目前模型。控制器仍認定已提交結果有效時，介面可在等待新查詢期間繼續顯示該結果。核心傳輸每批最多 128 行和 64 KiB 名稱資料。[領域資料](domain-data.md)、[大型歸檔索引](../engineering/large-index.md)

開啟目錄會進入其內容。雙擊或對符合條件的普通條目執行開啟操作，會啟動臨時開啟任務。屬性面板提供單獨的預覽操作。重名與密碼提示保留目前操作身份。進度和結果可透過應用任務入口檢視，瀏覽頁關閉後也可訪問。[瀏覽](../user-guide/browse.md)、[臨時開啟](../engineering/archive-opening.md)、[內容預覽](../engineering/content-preview.md)、[任務視窗](../ux/task-windows.md)

拖入檔案可根據目標開啟歸檔或提供建立輸入。符合條件的歸檔內移動與拖出先準備任務，再按平臺交接。各系統的傳輸能力決定非同步內容與目標寫出的處理方式。[歸檔傳輸](../engineering/archive-transfers.md)

<a name="xdzip-共享控件与原生边界"></a>

## 共享控制元件與原生邊界

操作視窗複用 `OperationButton`、`OperationFooterButton`、`OperationFrame`、`OperationSelect` 和 `OperationSegmentedGroup`。共享控制元件負責視覺引數和互動。離散選擇使用共享意圖控制元件，將滑鼠、鍵盤和輔助功能操作路由到同一狀態模型。介面文字與檔案資料遵循[視覺規範](../visual/design-standard.md)中的字型和本地化規則。歸檔名稱作為資料安全顯示。

獨立操作視窗使用視窗登記與各視窗自己的模態和關閉行為。macOS 使用系統標題欄控制元件，Windows 和 Linux 使用對應的系統適配。[平臺視窗契約](../ux/platform-windows.md)定義真實視窗的驗收，[桌面整合契約](../ux/desktop-integration.md)說明托盤與啟用行為。

<a name="xdzip-设计与验证范围"></a>

## 設計與驗證範圍

產品行為以原始碼和測試為據。高保真 HTML 用於審查佈局及模擬狀態。涉及原生焦點、選單、輸入法、輔助功能操作、顯示縮放和系統檔案管理器時，需要對應平臺證據。擴大桌面平臺配置的支援範圍或新增操作視窗，需要相應實現與驗收。[視覺流程](../visual/design-workflow.md)、[質量工程](quality-engineering.md)
