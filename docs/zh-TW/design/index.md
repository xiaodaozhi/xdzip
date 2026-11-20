<a name="xdzip-xdzip-设计指南"></a>

# XDZip 設計指南

[English](../../design/index.md) | [简体中文](../../zh-CN/design/index.md) | 繁體中文 | [日本語](../../ja-JP/design/index.md) | [한국어](../../ko-KR/design/index.md) | [Français](../../fr-FR/design/index.md) | [Deutsch](../../de-DE/design/index.md) | [Español](../../es-ES/design/index.md) | [Português (Brasil)](../../pt-BR/design/index.md) | [Русский](../../ru-RU/design/index.md)

從[系統架構](system-architecture.md)瞭解 Qt Quick 桌面與獨立 Rust CLI 如何呼叫歸檔執行時。這套文件說明目前實現契約、架構決策和擬議工作的範圍。安裝與日常操作見[使用者手冊](../user-guide/README.md)。需要構建或修改專案時，見[開發與貢獻](../contributing/README.md)。

<a name="xdzip-按主题阅读"></a>

## 按主題閱讀

| 主題 | 文件 |
| --- | --- |
| 產品行為 | [產品要求](product-requirements.md)定義預期操作和範圍，[需求追蹤](requirements-traceability.md)提供對應的驗證入口。 |
| 歸檔資料 | [領域模型](domain-data.md)描述會話、來源版本、物理條目、查詢及操作輸入。 |
| 桌面 | [Qt Quick 介面](qt-quick-ui.md)說明介面、控制器職責和互動，[Qt/Rust 橋接](qt-rust-bridge.md)列出現行命令、身份、事件及生命週期。 |
| 格式與擴充套件邊界 | [格式介面卡與擴充套件試驗](codec-extension-sdk.md)介紹內建登錄檔、隨包工作程序和獨立 XDP1 試驗。 |
| 寫出與背景任務 | [檔案授權與安全寫出](file-security.md)覆蓋暫存、驗證和提交，[任務排程](task-scheduling.md)說明准入、提示、進度及取消。 |
| 效能與質量 | [效能設計](performance.md)說明預算、測量與目標，[質量工程](quality-engineering.md)將變更關聯到對應驗證。 |
| 交付與決策 | [構建、部署與發行](build-release.md)涵蓋平臺、打包和更新。[架構決策](architecture-decisions.md)記錄取捨緣由，[架構演進](architecture-evolution.md)說明原型選擇如何形成現行產品。 |
| 術語 | [術語與實現限額](glossary.md)解釋用語，並連結各子系統的目前限額。 |

<a name="xdzip-如何理解这些参考"></a>

## 如何理解這些參考

目前行為附實現入口。設計選項及試驗工具各自說明適用範圍。[技術參考](../reference/README.md)提供各個工程契約的入口。[專案首頁](../../../README.zh-TW.md)和對應發行說明描述支援的分發方式及已交付功能。耗時、資源和原生驗證結論適用於當時記錄的構建、平臺與工作負載。
