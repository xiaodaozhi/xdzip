<a name="xdzip-文件管理器操作与独立任务宿主"></a>

# 檔案管理器操作與獨立任務宿主

[English](../../engineering/shell-actions.md) | [简体中文](../../zh-CN/engineering/shell-actions.md) | 繁體中文 | [Français](../../fr-FR/engineering/shell-actions.md) | [Español](../../es-ES/engineering/shell-actions.md) | [Русский](../../ru-RU/engineering/shell-actions.md)

系統入口將選擇和動作交給同一登入會話中的桌面程序。Windows 11 的普通安裝版擴充套件位於“顯示更多選項”，其 COM 類同時提供 `IContextMenu` 和 `IExplorerCommand`。MSIX 單獨宣告經典處理程式，商店構建另外宣告現代選單處理程式，詳見下文。Ubuntu 24.04 的 Files 使用 Nautilus 4.0 擴充套件。macOS 提供 Finder Sync 子選單、“服務”命令及獨立操作面板。

各平臺實際驗證情況與剩餘項見 [#270](https://github.com/xiaodaozhi/xdzip/issues/270)。原始碼能力不追溯加入既有釋出附件。

Finder Sync 的零付費實現與本機驗證見 [#273](https://github.com/xiaodaozhi/xdzip/issues/273)。Apple 將 Finder Sync 定位於同步應用，本專案接受其用於通用壓縮選單的適用性限制，不據此宣稱 App Store 稽核可接受。本機 ad-hoc 完整性簽名不要求付費賬號、Developer ID、App Groups 或公證，也不提供 Gatekeeper 發行者信任。依據為 [Finder Sync 擴充套件文件](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/Finder.html)。

<a name="xdzip-macos-finder-sync"></a>

## macOS Finder Sync

`Contents/PlugIns/xdzip-finder-sync.appex` 是不連結 Qt、Rust 或格式庫的原生擴充套件。選單覆蓋全部本地 Finder 目錄，觀察根為 `/`，僅處理 Finder 提供的選擇，不遍歷目錄，不新增工具欄、徽標或同步任務。單選普通項提供三項建立操作，多選增加逐項 ZIP，全部為歸檔候選時再增加四項解壓操作。空選擇、非本地 URL 或超過 128 項時不生成選單。原生選單使用系統尺寸，獨立操作繼續使用共享正常密度視窗。

擴充套件只讀獨立 JSON 配置，位於使用者 Application Support 下的 `xdzip/finder-sync.json`，包含目前應用位置、開關、語言、登記修訂與本機隨機交接令牌。檔案限 8 KiB、目前使用者所有、0600 普通檔案，寫入使用原子替換，拒絕連結和外來配置。它不是 CFPreferences 檔案。沙箱僅增加該精確檔案的只讀臨時例外，不共享整個使用者目錄、不使用 App Groups 或網路監聽。路徑 entitlement 必須以 `/` 起始，見 [Apple 臨時例外規範](https://developer.apple.com/library/archive/documentation/Miscellaneous/Reference/EntitlementKeyReference/Chapters/AppSandboxTemporaryExceptionEntitlements.html)。

Finder 不保留選單項的 `representedObject`，因此擴充套件透過單調遞增 tag 保留一個有界選擇快照，舊選單 tag 失效後不能對應新選擇。擴充套件透過顯式 `NSWorkspace` 目標向 `Contents/Helpers/xdzip-shell-relay.app` 傳送 `xdzip-shell://v1/` 的 base64url JSON 事件，不能依賴沙箱呼叫者的啟動引數。relay 為不連結 Qt 的獨立原生應用，先校驗目前使用者私有配置中的隨機令牌，再驗證 URL、UUID、動作、路徑與 JSON 邊界，保持 UUID，呼叫現有私有請求檔案與單例項入口；接納失敗顯示原生錯誤。令牌不進入請求檔案、桌面協議或產品日誌，外部構造的無令牌或錯誤令牌 URL 不執行檔案操作。URL 僅用於本機交接。

擴充套件按固定 bundle 佈局定位所屬應用，不在沙箱中讀取外層應用後設資料。relay 每次重新讀取外層 `Info.plist`，同時要求產品身份、`XdzipShellProtocol=1`、`XdzipFinderSyncProtocol=1` 和合法可執行檔名，回退或不相容目標拒絕執行。仍載入的舊擴充套件可能暫時保留選單，點選必須被 relay 拒絕；服務入口獨立核對其原有協議。

安裝版首次啟動登記自有 appex 與服務，系統設定中的 Finder 擴充套件仍需使用者啟用。設定開關同步管理兩種入口，語言只切換擴充套件選單，系統服務標題遵循系統語言。重複儲存設定不重複新增未變化的 appex，避免斷開 Finder 連線；內容摘要變化或登記缺失才重新登記。關閉時僅撤銷本應用精確路徑的登記，保留禁用配置；舊副本解除安裝不禁用較新應用的配置。顯式登記的開發副本關閉後仍可透過設定重新啟用，未經登記的開發副本不自動啟用。

更新可能受 Finder 持有舊擴充套件例項影響，必要時在系統設定中關閉再啟用自有擴充套件，不強制結束 Finder。回退和解除安裝只操作自有登記及檔案，不改變預設檔案關聯。Services 的可見位置由系統管理，可能提升到右鍵選單頂層。

<a id="windows-package-boundary"></a>

<a name="xdzip-windows-应用包边界"></a>

## Windows 應用包邊界

MSIX 構建透過 `desktop9:Extension` 宣告傳統 `IContextMenu` 處理程式，用於 Windows 11 的“顯示更多選項”。商店構建另行宣告獨立的 `IExplorerCommand` 類和 `windows.fileExplorerContextMenus`，用於現代選單。兩個類使用不同身份，分別只暴露各自選單介面。側載構建僅宣告傳統選單類，普通安裝版保留既有傳統選單。兩種選單複用檔案操作及私有請求校驗。

`--msix-channel store|sideload` 選擇包內不可變渠道標記和對應清單，預設為 `sideload`。現代選單要求執行於 Windows 11、商店構建已註冊，且擴充套件自身包的系統簽名型別為 `Store`；簽名查詢失敗及 Developer、Enterprise、None 均隱藏現代入口。商店簽名的商店構建即使被複制側載，仍視為商店構建。這是構建渠道策略，不聲稱能識別相同簽名位元組的安裝途徑，參見[微軟對該限制的說明](https://github.com/microsoft/WindowsAppSDK/issues/4993)。

擴充套件根據自身 DLL 所在包目錄及目前使用者註冊判斷身份，不使用 Explorer 程序身份；每次執行前重新核對註冊及可執行檔案的選單協議，再啟用該包的 AUMID。註冊已移除或被新版本替代時拒絕執行。包內選單語言和開關使用按包族隔離的鍵，不借用普通安裝版後設資料；包身份下不寫普通 COM 註冊，清單登記及解除安裝由 Windows 管理。真實商店簽名、Explorer 顯示、升級與共存驗收獨立於自簽名夾具檢查。Windows 10 沒有 Windows 11 現代選單，且不在目前支援承諾內。

[GetStagedPackageOrigin](https://learn.microsoft.com/en-us/windows/win32/api/appmodel/nf-appmodel-getstagedpackageorigin) 報告暫存包來源，不是已驗證的逐使用者安裝途徑；該介面保留用於診斷，不參與現代選單放行。

需要採集來源對照時，在目前使用者已註冊目標包的 Windows 環境執行以下只讀診斷；包名稱僅為示例，輸出必須是 `build/` 下尚不存在的 JSON 檔案。診斷同時記錄系統簽名型別、暫存來源和匿名化包完整名稱摘要，拒絕沒有唯一登記版本的結果。場景引數是執行者標籤，不證明安裝途徑；系統查詢失敗、未知值或相同返回值均不能單獨批准啟用選單。無需修改證書、安裝或系統設定。

```powershell
python scripts/msix-origin-probe.py --name xdzip.Example --scenario store-install --output build/store-origin-observation.json
```

<a name="xdzip-选择与输出"></a>

## 選擇與輸出

右鍵建立確認直接檢查預填的源目錄或指定目標，不先檢查預設下載目錄，避免不相關的 macOS 隱私訪問等待阻塞操作。正常主視窗建立仍使用預設目標。

壓縮提供快捷 ZIP、7z、建立確認及多選逐項 ZIP。單個普通檔案保留完整名稱，例如 `a.txt.zip`；資料夾使用 `foo.zip`；同目錄多選合包使用所在目錄名。跨目錄合包進入獨立建立視窗確認目標。快捷建立採用正常級別，不加密、不分卷，目標重名使用現有詢問機制。

全部所選項是受支援字尾或分卷命名的歸檔候選時，顯示解壓操作。選單不掃描檔案內容，真正開啟時仍由 Rust 發現完整卷組、識別內容、校驗來源和格式能力。相同完整來源組在一批解壓中只執行一次；來源組標記是持有檔案身份的程序內不透明標記，不是寫入授權。

- 目前目錄：各歸檔內容直接輸出到各自所在目錄。
- 智慧目錄：完整已驗證輸出圖僅含一個頂層目錄時不增加外層目錄，隱式頂層目錄也參與判斷；根目錄檔案或多個頂層項需要歸檔同名外層目錄。不剝除或重新命名包內目錄。
- 逐包目錄：始終增加歸檔同名外層目錄。新增目錄透過持有的目標能力排他建立，已有檔案、目錄或連結均不復用，按 `name (2)` 等編號選擇新名稱，最多嘗試一萬次。
- 指定目錄：獨立解壓視窗選擇共同目標，預設逐包目錄，可改為智慧或直接合並。

目錄派生繼續遵循[安全解壓](zip-safe-extraction.md)的 portable 名稱與 `.app` 字尾規則。智慧判斷不能授權輸出；每個任務仍需重新驗證目標、來源、CRC／認證、衝突和提交許可權。某個包內部的失敗仍按既有整包停止策略處理，批次協調器隨後處理下一個包。

<a name="xdzip-进程请求与窗口"></a>

## 程序、請求與視窗

私有入口為桌面程式 `--shell-request <request.json>`，不屬於獨立 CLI 的指令碼介面。原生擴充套件只生成選擇清單並啟動桌面入口，不載入 Qt 或格式庫。清單位於系統臨時根下排他建立的 `xdzip-shell-<32位随机十六进制>` 私有目錄，固定葉名稱為 `request.json`。Windows 在寫入前設定目前使用者與 SYSTEM 的受保護 DACL，POSIX 使用 0700 目錄與 0600 檔案。桌面使用共享 Rust no-follow 與目錄能力核對根、所有權和許可權，限讀 256 KiB，拒絕連結、相對路徑、無效 Unicode、未知動作及超過 128 個選擇。

請求包含 schema 1、UUID、動作和絕對路徑。可選布林值 `archive_candidates` 僅傳遞檔案管理器已有的 MIME 或 UTI 型別提示，與共享字尾、分卷規則共同決定選單可見性，不授權讀寫或繞過 Rust 內容驗證。Windows 讀取副檔名已登記的 Content Type，Nautilus 讀取 FileInfo MIME，macOS helper 讀取 URL Content Type，不在檔案管理器中解析歸檔。單例項協議保留原開啟與 CLI 操作入口，新增結構化 `shell_action`；接納與執行成功分別判斷。應用層保留至多 128 個已接納請求身份，相同身份與內容只接納一次，同身份不同內容拒絕。清單接納後只刪除固定檔案和空目錄，不遞迴清理外部輸入。

冷啟動操作只建立 `ShellHost`，不建立隱藏主視窗。正常開啟檔案與顯式“顯示主視窗”按需建立瀏覽視窗。已有例項接收右鍵操作不切換瀏覽標籤、不恢復隱藏主視窗。操作會話佔一個既有有界會話槽，批次內部序列復用控制器，不按歸檔數量建立瀏覽頁面。

`OperationTasks` 為瀏覽入口和獨立入口共享進度、結果、密碼及衝突視窗。頂層 `OperationWindow` 允許沒有 owner；有 owner 時跟隨呼叫視窗，無 owner 時約束在所選螢幕。模態提示繫結實際操作視窗，退出確認複用 `OperationConfirmationWindow`。佈局、共享控制元件、雙語和輔助功能仍遵循[視覺流程](../visual/design-workflow.md)。

macOS 單例項身份使用目前使用者與 Security 框架的實際登入會話 ID，查詢失敗拒絕選舉，不依賴 helper 可能缺失的環境變數。普通啟動、Finder relay 和服務入口因此進入同一登入會話的例項，見 [SessionGetInfo](https://developer.apple.com/documentation/security/sessions)。

右鍵建立與解壓沿用 `TaskWindow` 的進度和結果，不切換獨立的簡化結果窗。批次使用同一結果佈局呈現每項狀態及實際錯誤，提交前失敗沒有任務回執時僅建立結果檢視，不虛構任務。右鍵操作視窗按目前顯示器可用區域居中，提示仍關聯實際呼叫視窗。Windows 經典入口使用系統支援的 `Position=Top` 置頂登記，確保在“開啟方式”之前顯示，不依賴 Explorer 新增“開啟方式”與動態選單查詢的先後順序。入口及列舉動作使用應用、格式與系統資料夾圖示，原生尺寸由 Windows 控制。


<a name="xdzip-批次完成与退出"></a>

## 批次、完成與退出

批次逐項記錄實際成功、失敗、取消、未執行與卷組去重。單項失敗繼續處理其餘專案；取消停止後續專案，並向正在準備或執行的任務傳送既有取消意圖。已經提交的輸出保留，不聲稱整批迴滾。準備、等待密碼、衝突、取消和清理仍計入全域性活動數及退出屏障。

獨立建立視窗尚未取得任務回執時，提交被拒絕應保留輸入、引數與錯誤，允許修改後重試，不生成任務終態或消費批次索引。快捷操作沒有配置視窗，提交失敗則按實際失敗繼續批次。

右鍵建立成功不提交應用內開啟意圖，解壓的“完成後開啟資料夾”預設關閉。成功保留獨立結果，使用者可主動開啟目標目錄。任務完成不搶焦點；主動關閉視窗後，從任務入口可重開批次結果。關閉托盤設定或托盤宿主不可用時，已關閉結果保持隱藏。可見或最小化的主視窗提供任務入口；沒有該入口時，僅恢復隱藏的活動任務視窗。有托盤或可訪問的主視窗時，關閉進度後任務可繼續；兩者均無則先顯示等待或取消決定。沒有可訪問視窗、托盤和活動任務後，等待全部 Rust 連線結束再退出；已開啟的設定視窗繼續可用，直到使用者關閉。

<a name="xdzip-安装与兼容"></a>

## 安裝與相容

安裝版預設啟用，在通用設定中可關閉。註冊只管理自有入口，不更改預設檔案關聯。便攜與開發副本不自動註冊。顯式桌面入口 `--register-shell-integration`、`--remove-shell-integration` 用於受控註冊與移除，均不顯示主視窗。

安裝器使用精確的單引數同步或移除命令，只更新自有整合後設資料，在安裝事務啟動鎖釋放前執行；普通檔案、右鍵任務及主視窗啟動仍須取得執行時准入。Windows 安裝身份同時核對目前使用者登記的產品身份、安裝位置和檔案清單，便攜目錄即使帶有部署清單也不自動註冊。

交接前核對桌面程式支援協議 1，防止回退後仍載入的擴充套件把請求檔案交給舊程式。Windows 讀取 GUI 可執行檔案的版本資源 `XdzipShellProtocol`，並核對自有 `Applications/xdzip.exe` 登記命令指向同一個實際檔案；關聯啟動器不宣告協議能力。回退到舊的分版本佈局後，即使新版根目錄程式被保留，也不再作為右鍵目標，不讀取或改變使用者預設檔案關聯。macOS helper 每次重新讀取目標 Info.plist 的產品身份、協議和單個可執行檔名，不使用 NSBundle 的快取決定回退狀態；Ubuntu 安裝包擁有 `/usr/share/xdzip/shell-protocol`，回退到舊包及解除安裝均移除標記。缺失、未知、不相容或已經不屬於目前登記入口的目標拒絕啟動。能力標記僅管理交接相容性，不替代請求和 Rust 內容、目標驗證。

Windows 的自有 CLSID 與經典命令寫入 HKCU。命令採用獨立的類標識，避免 Explorer 快取的舊類工廠使升級後的入口消失。啟用或關閉整合時，僅移除匹配的舊選單處理器及自有舊類登記。歷史回退恢復舊處理器後，仍被持有的新命令隱藏自身，避免重複入口。安裝器為命令及其類保留解除安裝所有權記錄，覆蓋歷史回退後的解除安裝。執行目錄包含按內容摘要命名的擴充套件 DLL 與校驗清單，避免升級覆蓋 Explorer 已載入的同名 DLL。清理舊檔案仍按安裝器所有權與佔用策略執行，不強制結束 Explorer。macOS 安裝應用首次正常啟動時將自有服務安裝到使用者 Library/Services，服務 helper 只收集選擇並交接；關閉整合只移除核對過的自有檔案，額外內容、連結或所有權不符時保留並報錯。服務顯示和啟用狀態仍由系統控制。

Ubuntu 的原生擴充套件隨安裝包登記到 Nautilus 4.0 擴充套件目錄，讀取同一使用者設定。執行中的 Files 可能繼續持有舊模組，重開 Files 後更新，不自動結束檔案管理器。獨立執行目錄不自動寫入系統擴充套件目錄，需顯式安裝對應整合元件。
