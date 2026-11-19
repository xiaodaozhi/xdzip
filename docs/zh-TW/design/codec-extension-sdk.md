<a name="xdzip-格式适配器与扩展试验"></a>

# 格式介面卡與擴充套件試驗

[English](../../design/codec-extension-sdk.md) | [简体中文](../../zh-CN/design/codec-extension-sdk.md) | 繁體中文（台灣）

<a name="xdzip-三条执行路径"></a>

## 三條執行路徑

| 路徑 | 實現與可用性 |
| --- | --- |
| 宿主程序內的內建介面卡 | 桌面與 CLI 使用同一套固定格式登錄檔和執行時路由。ZIP、TAR、壓縮流過濾器以及 DEB、ASAR 等受支援容器的 Rust 實現在宿主程序內呼叫。 |
| 隨包提供的歸檔工作程序 | 宿主中的 `xdzip-codec-native` 啟動 `xdzip-native-codec`。其 C++ 協議層透過 `xdzip-codec-bridge` 呼叫 Rust 的 7z、RAR、CAB 和光碟映象後端。符合條件的 7z 建立與重寫也走這條路徑。ISO 建立使用程序內的光碟映象寫入器。 |
| 外部 SDK 試驗 | `xdzip-plugin-host` 與 `xdzip-plugin-protocol` 組成獨立且需要顯式執行的 XDP1 試驗。桌面和 CLI 產品構建不會發現、安裝或執行第三方擴充套件。 |

前兩條路徑屬於產品內建元件。隨包工作程序使用專用協議與資源監督。外部試驗有自己的包驗證、版本協商和平臺許可權門檻。[系統架構](system-architecture.md)、[歸檔工作程序契約](../engineering/native-codecs.md)、[SDK 試驗契約](../engineering/sdk-trial.md)

<a name="xdzip-产品路由与操作能力"></a>

## 產品路由與操作能力

[格式登錄檔原始檔](../../../specs/engineering/archive-formats.json)宣告副檔名、容器和過濾器型別、支援的操作、方法及建立選項。生成器同步 Rust 登錄檔、桌面列表及設計產物。讀取時，執行時探測並驗證來源資料，再選擇後端。建立時，指定的輸出名稱選定建立配置，執行時依配置校驗選項。副檔名列表用於發現和介面提示，成功探測和掃描才能確定開啟的格式。

讀取、預覽、解壓、整包檢查、建立和重寫各有限制。可瀏覽的條目可能缺少受支援的內容解碼方法。密碼、分卷和特殊記錄也影響准入。執行時還處理格式間的差異，例如 ISO/UDF 的結構檢查，以及無法重寫的配置。[執行時路由](../../../crates/xdzip-runtime/src/formats.rs)、[格式登錄檔契約](../engineering/format-registry.md)、[歸檔檢查](../engineering/archive-checking.md)

<a name="xdzip-内部-rust-适配器接口"></a>

## 內部 Rust 介面卡介面

| 介面 | 職責 |
| --- | --- |
| `ReadSource` 與 `MetadataCodec` | 讀取或定位宿主授權的來源，使用適用的分卷後設資料，產生受限的格式記錄。 |
| `PayloadCodec`、`PayloadReader` 與 `DecodedSink` | 定位內容並將符合條件的物理條目解碼到宿主接收器。後端支援時分別報告實際位元組、為跳過資料付出的解碼工作量和輔助工作量。 |
| `CreationCodec`、`EncodeInput` 與 `EncodeOutput` | 使用凍結的輸入後設資料和按編號訪問的資料流進行編碼。寫入宿主持有的接收器，寫入器需要時可執行受限的隨機寫入。 |
| `RewriteCodec` | 依據保留的原條目、修改及已授權的新增內容，將支援的歸檔重寫到新的暫存檔案。 |

路徑、來源複核、取消、密碼、額度及最終提交由宿主負責。編解碼器透過操作所需的介面獲得能力。歸檔工作程序請求已獲准的來源範圍，並返回有界記錄或輸出塊。宿主在處理前驗證協議狀態及位元組額度。工作程序異常或響應無效時無法授權釋出。工作程序邊界用於故障與資源隔離，對任意外部程式碼的核心許可權限制屬於試驗的獨立要求。[後設資料契約](../../../crates/xdzip-codec-api/src/lib.rs)、[內容解碼契約](../../../crates/xdzip-codec-api/src/payload.rs)、[編碼與重寫契約](../../../crates/xdzip-codec-api/src/encode.rs)、[檔案授權](file-security.md)

這些 Rust trait 是內部原始碼介面。產品介面卡顯式接入工作區與執行時路由。允許的操作需要分別驗證有效檔案、不受支援的功能、資源限額、資料損壞及寫出行為。參見[獨立 Rust 編解碼契約](../engineering/independent-rust-codecs.md)和[開發指南](../contributing/README.md)。

<a name="xdzip-已实现的外部-sdk-试验"></a>

## 已實現的外部 SDK 試驗

試驗接收顯式選定的原生包清單和宿主選定的來源，支援掃描專案自有的 XDP1 樣本並讀取一個普通條目。包準入驗證版本為 1 的清單、可執行目標與雜湊、包內路徑、固定許可權配置和限額。宿主執行已驗證的私有副本。宣告的雜湊用於確認包內容一致，釋出者認證需要獨立的信任策略。

版本 1 的試驗協議使用長度字首和有界 CBOR，並核對操作身份與順序。宿主透過不透明令牌授權讀取來源或寫入一個接收器。對端接收允許的位元組範圍和單塊寫入額度。宿主計量並驗證內容，檢查協議完成狀態和來源狀態，並在對端正常退出後以無覆蓋方式提交。試驗不向對端開放歸檔密碼、分卷、建立、修改、連結或宿主路徑。

試驗在 macOS 和 Linux 上有顯式的原生許可權門檻，平臺要求和失敗即拒絕行為以試驗契約為準。Windows 拒絕執行。互操作與拒絕訪問測試獨立於桌面執行。實際清單格式、限額和測試入口見[試驗執行與通訊契約](../engineering/sdk-trial.md)。[試驗宿主](../../../crates/xdzip-plugin-host/src/lib.rs)、[資料幀實現](../../../crates/xdzip-plugin-protocol/src/lib.rs)

<a name="xdzip-尚未实现的设计选项"></a>

## 尚未實現的設計選項

面向產品的擴充套件管理、釋出者信任與升級策略、試驗協議之外的相容性以及外部寫入或密碼能力，仍屬於設計工作。進入產品前，各支援平臺與能力都需要驗證許可權限制、互操作性和故障行為。隨包工作程序和 XDP1 試驗都不會啟用這項產品功能。[架構決策](architecture-decisions.md)
