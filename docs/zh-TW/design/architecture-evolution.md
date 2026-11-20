<a name="xdzip-架构演进与当前边界"></a>

# 架構演進與目前邊界

[English](../../design/architecture-evolution.md) | [简体中文](../../zh-CN/design/architecture-evolution.md) | 繁體中文（台灣） | [日本語](../../ja-JP/design/architecture-evolution.md) | [한국어](../../ko-KR/design/architecture-evolution.md) | [Français](../../fr-FR/design/architecture-evolution.md) | [Deutsch](../../de-DE/design/architecture-evolution.md) | [Español](../../es-ES/design/architecture-evolution.md) | [Português (Brasil)](../../pt-BR/design/architecture-evolution.md) | [Русский](../../ru-RU/design/architecture-evolution.md)

<a name="xdzip-从框架试验到现行产品"></a>

## 從框架試驗到現行產品

早期框架試驗使用合成歸檔列表和模擬任務。產品選用 Qt Quick 與型別化 CXX 橋接，再由 Rust 實現歸檔、任務及檔案操作。原型的 JSON/C ABI 和耗時資料屬於當時的試驗。[架構決策記錄](architecture-decisions.md)儲存了選擇依據及複核條件。

產品使用有界行模型，並校驗查詢身份，避免舊結果覆蓋新檢視。在此基礎上實現物理條目身份、顯式格式能力、檔案授權、暫存和驗證後釋出。獨立 CLI 無需 Qt 即可呼叫同一核心。[系統架構](system-architecture.md)、[領域模型](domain-data.md)

<a name="xdzip-当前实现独立试验与拟议工作"></a>

## 目前實現、獨立試驗與擬議工作

| 範圍 | 目前狀態 |
| --- | --- |
| 執行時與索引 | 已在宿主實現。普通單檔案 ZIP/ZIP64 可遷移到私有磁碟索引，任務限額和後設資料限額分別計算。[效能設計](performance.md) |
| 格式隔離 | 內建路由選擇程序內 Rust 實現或隨包工作程序。操作是否支援，由登錄檔配置及執行時准入共同決定。[格式介面卡](codec-extension-sdk.md) |
| 外部擴充套件 | 獨立的只讀 XDP1 試驗有受限協議與平臺許可權門檻，透過單獨宿主執行。產品內的擴充套件發現、安裝與啟用仍屬擬議工作。[SDK 試驗](../engineering/sdk-trial.md) |
| 長期狀態與併發 | 目前排程一次只准入一個桌面重操作。跨啟動的持久索引和任務恢復、通用的共享排程仍是設計選項。[任務排程](task-scheduling.md)、[檔案提交](file-security.md) |

目前原始碼能力見[格式支援表](../../../README.zh-TW.md#xdzip-%E6%A0%BC%E5%BC%8F%E6%94%AF%E6%8C%81)。具體發行版本以對應發行說明和附件為準。原生實現、試驗工具及測量結果各有適用範圍。
