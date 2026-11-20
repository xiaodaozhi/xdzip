<a name="xdzip-格式适配器与扩展试验"></a>
<a name="格式介面卡與擴充套件試驗"></a>

# 格式配接器與擴充功能試驗

[English](../../design/codec-extension-sdk.md) | [简体中文](../../zh-CN/design/codec-extension-sdk.md) | 繁體中文（台灣）

<a name="xdzip-三条执行路径"></a>

## 三條執行路徑

| 路徑 | 實作與可用性 |
| --- | --- |
| 主程式中的內建配接器 | 桌面應用程式和 CLI 使用相同的固定格式登錄表與執行階段路由。ZIP、TAR、串流過濾器及 DEB、ASAR 等受支援容器的 Rust 實作在主程序中呼叫。 |
| 隨附的壓縮檔工作程序 | 主程式中的 `xdzip-codec-native` 啟動 `xdzip-native-codec`。其 C++ 協定層透過 `xdzip-codec-bridge` 呼叫 Rust 7z、RAR、CAB 和光碟映像後端。適用的 7z 建立與改寫也使用此路徑；ISO 建立使用程序內的光碟映像寫入器。 |
| 外部 SDK 試驗 | `xdzip-plugin-host` 和 `xdzip-plugin-protocol` 組成獨立、須明確執行的 XDP1 試驗。桌面與 CLI 產品建置不會探索、安裝或執行第三方擴充功能。 |

前兩條路徑是產品內建元件。隨附工作程序為獨立程序，使用專用協定及資源監督。外部試驗有自己的套件驗證、版本協商與平台權限門檻。[系統架構](system-architecture.md)、[壓縮檔工作程序契約](../engineering/native-codecs.md)、[SDK 試驗契約](../engineering/sdk-trial.md)

<a name="xdzip-产品路由与操作能力"></a>

## 產品路由與操作能力

[格式登錄表原始檔](../../../specs/engineering/archive-formats.json)宣告副檔名、容器與過濾器類型、支援的操作、方法和建立選項。產生器同步 Rust 登錄表、桌面清單及設計成品。讀取時，執行階段偵測並驗證來源位元組，再選擇後端。建立時，指定的輸出名稱選擇建立設定，執行階段依設定驗證選項。副檔名清單用於探索和介面提示；成功偵測及掃描後才能確定開啟的格式。

讀取、預覽、解壓縮、整包檢查、建立和改寫各有限制。列出的項目可能可見，但缺少受支援的內容解碼方法。密碼、分割檔案與特殊記錄也會影響准入。執行階段會處理格式差異，例如 ISO/UDF 結構檢查與不支援改寫的設定。[執行階段路由](../../../crates/xdzip-runtime/src/formats.rs)、[格式登錄表契約](../engineering/format-registry.md)、[壓縮檔檢查](../engineering/archive-checking.md)

<a name="xdzip-内部-rust-适配器接口"></a>
<a name="內部-rust-介面卡介面"></a>

## 內部 Rust 配接器介面

| 介面 | 職責 |
| --- | --- |
| `ReadSource` 與 `MetadataCodec` | 讀取或定位主程式授權的來源，包含適用的分割檔案中繼資料，並產生受限的格式記錄。 |
| `PayloadCodec`、`PayloadReader` 與 `DecodedSink` | 解析內容並將符合條件的實體項目解碼至主程式接收端。後端有報告時，區分一般位元組、略過所需的解碼工作和輔助工作。 |
| `CreationCodec`、`EncodeInput` 與 `EncodeOutput` | 使用凍結的輸入中繼資料及索引串流編碼，寫入主程式擁有的接收端；寫入器需要時可進行受限的隨機寫入。 |
| `RewriteCodec` | 依保留的原始項目、變更和已授權的新增內容，將支援的壓縮檔改寫為新的暫存壓縮檔。 |

路徑、來源重新驗證、取消、密碼、配額與最終發布由主程式負責。編解碼器透過操作所需介面取得能力。壓縮檔工作程序請求允許的來源範圍，並提供受限的記錄或輸出區塊。主程式動作前會驗證協定狀態和位元組配額；工作程序故障或回應無效不能授權發布。工作程序邊界提供故障與資源隔離；任意外部程式碼的核心權限限制是此試驗的獨立要求。[中繼資料契約](../../../crates/xdzip-codec-api/src/lib.rs)、[內容解碼契約](../../../crates/xdzip-codec-api/src/payload.rs)、[編碼與改寫契約](../../../crates/xdzip-codec-api/src/encode.rs)、[檔案授權](file-security.md)

這些 Rust trait 是內部原始碼介面。產品配接器須納入工作區並明確接入執行階段路由；允許的操作需要分別測試有效檔案、不支援的功能、資源限制、損壞和輸出行為。參見[獨立 Rust 編解碼契約](../engineering/independent-rust-codecs.md)及[開發指南](../contributing/README.md)。

<a name="xdzip-已实现的外部-sdk-试验"></a>
<a name="已實現的外部-sdk-試驗"></a>

## 已實作的外部 SDK 試驗

試驗接收明確選取的原生套件資訊清單和主程式選取的來源，支援掃描專案自有的 XDP1 測試資料並讀取一個一般項目。套件准入會驗證第 1 版清單、可執行目標及雜湊、套件內路徑、固定權限設定和限制。主程式執行經驗證的私人副本。宣告的雜湊只能確立套件一致性；發布者身分驗證需要獨立信任政策。

第 1 版試驗協定使用帶長度前綴且受限的 CBOR，並檢查操作識別碼與順序。主程式透過不透明權杖授予來源讀取及一個輸出接收端；對端取得位元組範圍和單一區塊的輸出額度。主程式計量並驗證內容、檢查協定完成情況與來源狀態，只有對端正常結束後才以不覆寫方式發布。範例不向對端提供壓縮檔密碼、分割檔案、建立、更新、連結或主程式路徑。

試驗在 macOS 與 Linux 上有明確的原生權限門檻；平台要求及失敗即拒絕的行為依試驗契約執行。Windows 拒絕執行。原生互通與拒絕存取測試獨立於桌面應用程式。實際結構、限制與測試入口見[試驗執行與通訊契約](../engineering/sdk-trial.md)。[試驗主程式](../../../crates/xdzip-plugin-host/src/lib.rs)、[訊框實作](../../../crates/xdzip-plugin-protocol/src/lib.rs)

<a name="xdzip-尚未实现的设计选项"></a>

## 尚未實現的設計選項

產品內的擴充功能管理、發布者信任與升級政策、試驗協定之外的相容性，以及外部寫入或密碼能力仍屬設計工作。納入產品前，須針對每個支援平台與能力驗證權限限制、互通性和故障行為。隨附工作程序與 XDP1 試驗都不會啟用此功能。[架構決策](architecture-decisions.md)
