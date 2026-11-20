<a name="xdzip-外部只读-sdk-试验"></a>

# 外部只讀 SDK 試驗

[English](../../engineering/sdk-trial.md) | [简体中文](../../zh-CN/engineering/sdk-trial.md) | 繁體中文（台灣） | [日本語](../../ja-JP/engineering/sdk-trial.md) | [한국어](../../ko-KR/engineering/sdk-trial.md) | [Français](../../fr-FR/engineering/sdk-trial.md) | [Deutsch](../../de-DE/engineering/sdk-trial.md) | [Español](../../es-ES/engineering/sdk-trial.md) | [Português (Brasil)](../../pt-BR/engineering/sdk-trial.md) | [Русский](../../ru-RU/engineering/sdk-trial.md)

`xdzip-plugin-protocol` 和 `xdzip-plugin-host` 實現[格式擴充功能設計](../design/codec-extension-sdk.md)的獨立只讀試驗。它不進入桌面／CLI 的格式發現、安裝、打包或自動啟動路徑。使用者顯式執行試驗命令並選擇包與來源；產品繼續禁用第三方擴充功能。目前僅協商 `xdp1`，不提供建立、密碼、分卷、連結、GUI 或任意宿主路徑許可權。這不是完整外掛生態或正式 SDK 釋出。

<a name="xdzip-可重现入口"></a>

## 可重現入口

```sh
python3 scripts/sdk-trial.py test
python3 scripts/sdk-trial.py example --output-dir build/sdk-example
target/debug/xdzip-plugin-host scan build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp
mkdir build/sdk-example/output
target/debug/xdzip-plugin-host read build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp 0 build/sdk-example/output
```

Windows 使用 `python` 和 `.exe`。`test` 構建原生參考程式，執行 Rust framing 迴歸、獨立 Python 互通、惡意協議、取消、源變化、衝突與核心許可權迴歸。Python 只由編譯為測試的受信 conformance 入口啟動；釋出宿主不接受直譯器、引數、shell 或“不使用沙箱”開關。例子只在 `build/` 生成自有包、manifest 與 XDP1，不觸發發行或第三方下載。日誌去除工作區／使用者主目錄，JSON 繫結提交、dirty 狀態和二進位雜湊，均為本地／CI 工件。Foundation 的同一原生矩陣呼叫此入口；獨立 [SDK trial 工作流](../../../.github/workflows/sdk-trial.yml)用於先收斂沙箱平台相容，不重複執行無關格式與 Qt 管線。也可在 Foundation 工作流中選擇 `sdk_only=true`，呼叫同一可複用試驗。該模式跳過完整 Foundation gate，不產生完整 Foundation 結論。Windows 證明的是拒絕執行。

<a name="xdzip-包准入"></a>

## 包準入

嚴格 JSON manifest 使用 `deny_unknown_fields`。欄位為 `manifest_version=1`、`plugin_id`、`version`、`author`、`protocol_majors=[1]`、`targets`、包內 `entrypoint`、小寫十六進位 `sha256`、`formats=["xdp1"]`、`permissions`、`resources`、`license_files`、HTTPS `source_url`。參考 manifest 由上面的 `example` 生成，是目前 schema 的可執行入口。

`permissions` 恰為 `broker_source_read=true`、`broker_output_write=true`、`network=false`；`resources` 恰為 `memory_bytes=536870912`、`threads=1`。manifest ≤32KiB，原生程式 ≤64MiB，許可檔案最多四個、各 ≤128KiB。入口與許可檔案不能絕對定址、包含父目錄或連結／reparse 逃逸；每個包成員均透過持有的 cap-std 目錄逐層 no-follow 開啟，核對後不再按原字串重開。宿主透過持有的普通檔案複製、校驗實際位元組數、SHA-256 與前後檔案版本，複製到私有臨時目錄後才執行，保持副本至程序與 IPC 結束。雜湊宣告證明選擇的包內容一致，不認證釋出者，也不證明程式碼安全。

<a name="xdzip-线协议与状态"></a>

## 線協議與狀態

依據 [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html)實現專案自有的有界 CBOR 子集，不引入第三方 CBOR 實現。參考 Rust peer 複用 framing crate；[Python peer](../../../examples/sdk/reference.py)獨立手寫編碼、解碼與 XDP1 解析，不匯入 Rust 或生成的協議程式碼。互通覆蓋多塊實際內容與宿主摘要。

幀為 `u32 LE length + CBOR`，先檢查長度再分配，幀 ≤1MiB，單個 bytes/text ≤256KiB，深度 ≤16，陣列 ≤128，map ≤32，解碼節點 ≤8192。只接受 UInt、bytes、嚴格 UTF-8 text、array、UInt 鍵 map、bool；拒絕非最短整數、重複鍵、不定長、負數、標籤、浮點及尾隨資料。包絡 map 的鍵 `1..5` 依次為 kind、request、operation、sequence、payload。每方向 sequence 從 1 連續遞增；request 在每次呼叫遞增，operation 為宿主隨機非零身份。每次響應必須精確匹配目前身份。payload 未知鍵 `<128` 拒絕，`>=128` 為可忽略可選欄位。

| kind | 訊息 | payload 數字鍵 |
| --- | --- | --- |
| 1 | Hello | 1 最小 major，2 最大 major，3 包 ID，4 `xdp1` |
| 2 | Welcome | 1 major，2 block 上限，3 單塊信用，4 總讀取預算，5 輸出預算 |
| 10 | Probe | 1 source token，2 來源長度 |
| 11 | Open | 1 source token |
| 12 | Scan | 1 session token |
| 13 | ReadEntry | 1 session，2 entry，3 sink，4 信用數 1，5 輸出預算，6 首塊信用 token |
| 14 | Close | 1 session |
| 20、21 | ReadAt、ReadData | 1 source token，2 offset，3 length 或 bytes |
| 30 | EntryBatch | 1 session，2 entries；每記錄 1 token、2 raw path、3 型別 0 檔案／1 目錄、4 已知大小 |
| 31 | ScanFinished | 1 實際記錄數 |
| 40 | WriteData | 1 sink，2 從 0 遞增塊序號，3 bytes，4 目前信用 token |
| 41 | FinishOutput | 1 sink，2 宣告輸出數，3 SHA-256 bytes |
| 42 | WriteAck | 1 sink，2 下一塊序號，3 新隨機信用 token |
| 50、51、52 | Cancel、CancelAck、Shutdown | 空 map |
| 61 | OperationFinished | 1 outcome，0 成功／1 取消；Probe 另有 2 格式，Open 另有 2 session |

其他 kind 即使 framing 認識也不在本試驗狀態機中接受。Hello 必須先到達，雙方選擇 major 1；握手身份為零，後續呼叫均非零。ScanFinished 後仍必須唯一成功終態。每次只處理一個操作，無背景擴充功能重試。目前單塊信用比設計最大四塊更嚴格，每次實際接受輸出後重新整理不可預測的信用 token，不能預支或重放。

<a name="xdzip-broker预算与提交"></a>

## Broker、預算與提交

Source token、sink 和每塊信用均為隨機 128 位值，不能複用到其他操作。peer 不接收歸檔與目標路徑。宿主每次只返回授權來源範圍內最多 256KiB，偏移加法溢位、越 EOF、零長和累計讀取 >64MiB 均失敗；每次讀取前後及最終提交前檢查持有物件和路徑對應的 `FileVersion`。這沿用現有身份／時間／長度指紋，不宣稱能識別同帳戶攻擊者恢復全部後設資料的任意併發改寫。

後設資料最多 1000 物理記錄，路徑與 token 合計 ≤256KiB，記錄 token 唯一；raw path 在宿主按 PortablePath 嚴格驗證，重複名稱保留不同身份。一次顯式選擇一個普通條目，宣告大小 ≤4MiB；拒絕目錄作為檔案輸出。宿主持有目標 Grant、私有 Workspace 和 Stage，按實際位元組計量並計算 SHA-256，不向 peer 交付檔案控制代碼或提交權。

所有輸出塊、FinishOutput 的宣告數／摘要、唯一終態、Close、Shutdown 後正常退出及純 EOF、源身份與最終取消檢查都透過，才允許既有安全 I/O 無覆蓋提交。任何異常、重複終態、崩潰或超時都先撤銷輸出；析構先移除 stage 再清理 workspace。既有目標保持原樣。輸入解析和預期摘要仍來自擴充功能，宿主計算不能證明惡意擴充功能輸出忠實於歸檔。

握手 2 秒，每次試驗呼叫總時限 5 秒，不以 progress 重新整理。單程序 CPU 上限 3 秒，檔案描述符 32，不允許直接檔案輸出，core dump 關閉。IPC 接收佇列最多四幀，傳送佇列一幀，寫管道有 2 秒上限；stderr 直接丟棄，預算為零，不轉發不可信內容。取消即時撤銷 sink，給 2 秒合作期限後殺死並回收程序和管道，不將 Ack 當成提交許可。資源預算是小型試驗約束，不是普通大包格式的效能承諾。

<a name="xdzip-平台门禁"></a>

## 平台門禁

| 平台 | 試驗策略與失敗行為 |
| --- | --- |
| macOS | 固定 `/usr/bin/sandbox-exec` deny-default，僅允許精確的私有可執行副本、系統庫／dyld cache、根目錄載入器讀取及 `/var` 後設資料。禁止使用者檔案、寫入、網路和 fork；清空環境，繼承僅 stdio。原生 rlimit 加 20ms RSS 監督，超過 512MiB 或計數讀取失敗即終止。RSS 為定期監督，取樣間短暫峰值可超過閾值；該介面不承諾可用於未來正式分發。 |
| Linux x86_64／aarch64 | Landlock ABI ≥3，處理全部 v3 檔案權利，無寫規則，僅程式與系統載入庫讀取；`no_new_privs` 加架構核對後的 seccomp syscall 允許表，含 Rust 標準管道所需的 readv／writev，拒絕 socket、fork/clone、ptrace、io_uring 等。CPU 上限 3 秒、位址空間上限 512 MiB，並套用其他 rlimit。規則、ABI、seccomp 或所需系統呼叫不可用，exec 前失敗。原生 CI 才能建立該平台的實際結論。 |
| Windows 及其他平台 | 固定拒絕執行。未實現並驗證 AppContainer、Job 和控制代碼白名單，不用普通子程序代替沙箱。 |

Linux API 依據[核心 Landlock 文件](https://docs.kernel.org/userspace-api/landlock.html)與 UAPI，Windows 後續依據 [AppContainer 隔離](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation)。宿主 native unsafe 限於沙箱模組，註明佈局、呼叫前提和失敗行為；pre-exec 不做分配或日誌。參考 peer 僅在顯式審計模式增加受控 fork／_exit／waitpid 探針，子程序不執行 Rust 清理或格式邏輯。平台迴歸以自有可讀／可寫標記、自有 localhost 監聽器和 self-exec 先作正向對照，再驗證核心拒絕讀取、寫入、聯網、直接 fork 和 self-exec，核對標記、檔案和連線真實狀態。沒有測試的平台不記透過，也不啟用桌面第三方執行。

<a name="xdzip-自有-xdp1-示例格式"></a>

## 自有 XDP1 示例格式

XDP1 只用於互通，不替代任何真實歸檔格式。`XDP1` 四位元組後為 `u32 LE` 記錄數，各記錄為 `u16 LE` UTF-8 名稱位元組數、`u32 LE` 內容數、名稱與未壓縮內容。最多 1000 條，不能有尾隨資料，名稱和內容按宿主預算讀取。條目 token 為原物理序號的四位元組 LE，不以路徑去重。Python 的故障模式只在 conformance 測試顯式指定，用於驗證越權、序號、重放、重複身份、路徑、信用、摘要、崩潰、取消和重複終態。
