<a name="xdzip-本地-macos-开发-dmg"></a>

# 本地 macOS 開發 DMG

[English](../../engineering/macos-deployment.md) | [简体中文](../../zh-CN/engineering/macos-deployment.md) | 繁體中文 | [Français](../../fr-FR/engineering/macos-deployment.md)

專用 Release 工作流允許原生部署、必要的 ad-hoc 簽名、app.zip 與 DMG 打包及上傳，按需 Qt 工作流的 macOS 作業保持 SDK 驗證策略。同一份已部署、簽名並驗證的 .app 用於兩個釋出包，ZIP 解包後另作重定位啟動核驗，詳見[釋出協議](release-protocol.md)。

本地開發 DMG 是允許的開發交付方式；按需 Qt 工作流驗證 Apple Silicon 原生構建／啟動，不打包、簽名或上傳 Mac 應用。本地指令碼不安裝應用、不改預設關聯、不上傳檔案，不讀取發行憑據。

<a name="xdzip-制作与核验"></a>

## 製作與核驗

環境預檢和階段引數以[開發構建指南](developer-build.md)為準。先準備打包工具，從乾淨提交構建獨立 Release 主應用，再依次打包和驗證：

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

各階段不隱式補跑前序階段。Qt SDK 預設位於 `build/toolchain/Qt/6.8.2/macos`，其他位置在各命令傳入 `--qt-prefix`。普通開發／測試構建預設 `None`；上面的主應用顯式選擇 `Alternate`，不改變使用者預設關聯。macOS 本地 package 僅支援 `all`；按需 Qt 工作流不執行此打包階段，也不上傳 Mac 應用。專用 Release 例外見釋出協議。

共用 package 在執行 macdeployqt 前檢查原生 arm64、系統 SDK、部署工具、Rust PATH／鎖定後設資料、Python 校驗依賴、Qt 許可與 schema 材料。build、test 允許未提交原始碼；package 要求乾淨提交且實際 CLI 身份一致，不能借已有編譯快取繞過。系統編譯器、SDK、lipo、codesign 和 hdiutil 由系統工具鏈提供，不由準備階段安裝。

圖片工作程序同樣由 macdeployqt 重定位。Qt 6.8 的預設平臺外掛選擇只包含 Cocoa，打包指令碼須在掃描應用庫之前顯式複製 offscreen 外掛，使其 Qt framework 依賴一起處理；缺少外掛或工作程序時拒絕打包。完整性簽名與匯入審計之後，清除 Qt／QML／DYLD 搜尋覆蓋，使用應用內真實圖片工作程序核對格式集合及自有 PNG 解碼，回執為 `image-preview-deployed.json`。指令碼單元檢查不等同於實際部署包驗收。

底層 [build-macos-dmg.py](../../../scripts/build-macos-dmg.py)和 [verify-macos-dmg.py](../../../scripts/verify-macos-dmg.py)繼續負責映象與原生驗證細節；需要故障定位時使用各自 `--help`，不另外維護日常步驟。底層輸出路徑必須位於 build/ 且尚不存在。DMG 輸入的文件型別必須全部宣告 `Viewer/Alternate`，否則要求重新構建，不在原包上改寫 plist 或簽名。macdeployqt 掃描實際 QML，部署 Qt frameworks、外掛和 QML；補充 Qt 系統控制元件翻譯、專案通知、Rust 上游通知、字型許可、原始碼 SBOM 和固定 Qt 許可材料。保留 framework 內部符號連結，拒絕逃逸應用目錄的連結；移除開發機絕對 RPATH，遞迴檢查全部原生匯入只解析到應用或系統。構建身份不能為 dirty 或 unknown，必須匹配目前乾淨提交以繫結實際通知。

Qt 可選 QML LocalStorage 引入 QtSql 後，部署工具會順帶複製資料庫驅動。本專案不提供伺服器資料庫能力，因此移除除內建 SQLite 外的驅動，避免 PostgreSQL、ODBC、Mimer 等 SDK 外部客戶端依賴進入映象；移除列表寫入回執，不把 unresolved 匯入隱藏為透過。

<a name="xdzip-架构与样式精简"></a>

## 架構與樣式精簡

開發 DMG 只交付 Apple Silicon `arm64`。主程式的 CMake 架構設定不能約束 macdeployqt 複製的 Qt SDK 檔案，因此部署後必須遞迴檢查應用中的全部 Mach-O，包括 framework、動態庫、外掛及 QML 原生外掛。雙架構檔案僅在新建的打包副本中裁剪為 `arm64`；缺少 `arm64`、無法識別的架構或裁剪後仍含其他分片均使打包失敗，不以主程式架構代替巢狀檢查。裁剪必須先於 RPATH 重寫與遞迴 ad-hoc 簽名。

產品固定使用 Qt Quick Controls 的 `Basic` 樣式。部署只移除明確列出的非 Basic 樣式及其專用匯入／實現外掛，保留 Basic、共享控制元件、Templates 和實際 QML 依賴，不透過刪除未知模組縮減體積。非預期的樣式佈局、缺失的保留依賴或殘留的未使用樣式應使審計失敗；更換產品樣式或 Qt SDK 時須同步更新裁剪規則和測試。Qt 許可、對應原始碼材料、系統翻譯與必要執行外掛不因體積精簡而省略。

最佳化僅作用於受限 `build/` 下新建的暫存應用，不修改輸入主應用、Qt SDK 或原始碼。回執保留架構與樣式檢查結果、裁剪的二進位制與移除清單，以及部署後最佳化前、最佳化後的應用邏輯位元組數和 framework、外掛、QML 等分類大小。該數字按實際檔案內容統計，符號連結不重複計入，不等於 Finder 磁碟佔用或 UDZO 映象體積；最終 DMG 大小另行記錄。具體版本的前後對比、驗證結論和剩餘項登記在 issue，原始測量只保留為本地工件。

`scripts/test-macos-bundle.py` 使用自有小型夾具在跨平臺 CI 驗證架構識別、只操作副本、符號連結邊界、樣式保留／移除及失敗行為，不依賴 macOS 部署工具。它不能替代最終包核驗：DMG 驗證還須對掛載後複製的應用再次遞迴確認所有原生檔案僅含 `arm64`、樣式集合符合契約，並執行下述 SDK 搜尋路徑隔離及 Cocoa／Metal 流程。

<a name="xdzip-镜像与运行核验"></a>

## 映象與執行核驗

映象提供 xdzip.app、Applications 連結和安裝說明，使用 UDZO 壓縮並執行 hdiutil verify。拖入 Applications 前應正常退出舊版，未提供自動更新器或安裝中的任務遷移。設定、最近記錄位於使用者目錄，移除應用不自動刪除這些資料。

映象旁保留部署核驗 JSON 和 CycloneDX 檔案級 SBOM，繫結同提交原始碼清單、最終原生檔案 SHA-256 和匯入關係。SBOM 繼續標記 incomplete，不將檔案列舉當作精確元件歸屬或許可批准；清單位於映象外，避免自包含雜湊遞迴。

打包暫存目錄中的新 `xdzip.app` 只在部署、簽名、映象製作及核驗期間存在，結束時自動回收，失敗時也執行清理。DMG、JSON、SBOM 與說明繼續保留，輸入主應用不受影響，不留下可被系統再次發現的鬆散暫存包。

ARM64 的已修改 Mach-O 需要有效程式碼目錄，因此複製／重寫後使用 ad-hoc 本地簽名，並驗證巢狀程式碼。**這不屬於 Developer ID 簽名，也未公證，不表示 Gatekeeper 信任。** 按[個人開源發行政策](../../../developers/engineering/release-runbook.md)，正式簽名與公證不是發行硬性條件，依賴、許可、安全與實際包驗證仍須滿足。 不關閉 SIP，不刪除使用者全域性安全策略。

核驗指令碼只讀掛載映象，核對 Applications 連結，複製到含中文與空格的隔離目錄，遞迴確認 arm64、Basic 樣式及每個原生檔案和應用包的嚴格簽名，移除 SDK／QML／外掛搜尋環境後以 Cocoa／Metal 啟動關閉。再由映象內真實產品開啟自行生成的 ZIP，執行查詢、排序、解壓與退出，獨立核對 Unicode、空檔案、空目錄和全部輸出位元組，並檢查實際首幀的構建身份、架構及 Metal 後端。隨後僅在核驗副本修改 QtCore 的 Mach-O 庫身份並重新 ad-hoc 簽名，再檢查架構、樣式、簽名及啟動。該測試證明部署允許修改映象替換，不冒充從對應原始碼重建 Qt、任意 ABI 相容或法律履約批准。

app.zip 核驗還使用產品的預設同名資料夾選項解壓整個應用，與 `ditto` 輸出逐項比較檔案位元組、連結目標及安全檔案許可權，再對解壓後的真實應用執行深層嚴格簽名檢查和 Cocoa／Metal 啟動。外層目錄不能以 `.app` 結尾；臨時應用和本次新增註冊在驗證結束時清理。此檢查不改變簽名信任或公證邊界。

本機仍安裝 SDK，匯入核驗和清除環境不能代替另一臺乾淨 Mac。Finder 拖入、系統開啟方式、預設選擇、Gatekeeper、真實重啟和簽名公證繼續按[人工清單](../../../developers/qa/private-test-manual.md)驗收。

原生選單編輯命令和 Qt 標準控制元件需要系統 locale 的 qtbase 翻譯。打包除複製 `Contents/Resources/translations/qtbase_*.qm` 外，還須在 `Contents/Resources/qt.conf` 的 `[Paths]` 設定 `Translations = Resources/translations`，保留已有外掛與 QML 路徑，並在完整性簽名前完成寫入。重定位啟動門檻同時檢查原生編輯 responder 和應用選單角色，不能只在 SDK 開發目錄驗證。

<a name="xdzip-本机-finder-sync-签名与启用"></a>

## 本機 Finder Sync 簽名與啟用

Finder Sync 使用同一應用內的原生 appex 與獨立 relay，服務 helper 繼續保留。完整簽名順序是原生映像、framework 及專用 helper、帶精確沙箱 entitlement 的 appex、relay bundle、外層應用，最後嚴格驗證整個巢狀包。不得在 appex 簽名後再次用無 entitlement 的通用簽名覆蓋它。普通 CI 構建不隱式簽名或啟用擴充套件。

需要本機 Finder 驗證時，先完成預設原生構建，再顯式執行：

```sh
python3 scripts/sign-macos-finder.py --app build/desktop/apps/desktop/xdzip.app --output build/review/finder-signing.json
build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip --register-shell-integration
```

簽名指令碼僅接受倉庫 `build/` 下自有應用及輸出，不部署 Qt、不建立 DMG、不讀取發行憑據。其回執只證明 ad-hoc 完整性與巢狀嚴格校驗，不證明 Developer ID、公證、Gatekeeper 信任或發行許可。應用須保留原始位置，在系統設定的登入項與擴充套件中啟用 xdzip 的 Finder 擴充套件。系統版本不同，設定路徑和選單排列可能不同；僅登記成功不等於選單或操作驗收透過。

結束臨時驗證前，用同一應用的 `--remove-shell-integration` 撤銷自有服務和擴充套件登記，核對初始狀態後只清理本次建立的配置及臨時 LaunchServices 登記。不要修改其他應用或預設檔案關聯。若應用已被 Finder 載入，更新後可在系統設定中關閉再開啟自有擴充套件，不重啟 Finder。交接和回退邊界見[檔案管理器操作](shell-actions.md)。

<a name="xdzip-开发与核验副本的打开方式注册"></a>

## 開發與核驗副本的開啟方式註冊

LaunchServices 會記住曾開啟過的應用包，構建樹中相同 bundle id 的多個副本可能同時出現在系統或 Codex 的“開啟方式”選單。不要為此刪除系統資料庫、重啟 Finder、刪除驗證包或更改使用者預設關聯。

新開發包預設在生成 Info.plist 時寫入 `LSHandlerRank=None`，僅保留指定主應用和交付包的 `Alternate`。CMake 引數 `XDZIP_MACOS_HANDLER_RANK` 嚴格限定這兩個值；原始碼離線重建、原始碼 Qt 產品驗證和 Mac 產品 CI 也顯式傳入 `None`，避免以後更改預設值使測試副本重新進入候選。[Apple 對 `None` 的定義](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)保留檔案拖入能力，不把該包選為此型別的開啟應用。這個設定不影響應用內選擇檔案或顯式指定應用的開啟流程，不設定預設關聯。

Mac 產品 CI 執行 `scripts/check-macos-registration.py`，透過原生 `NSWorkspace` 的檔案候選查詢檢查新構建包。候選數量與按 bundle id 列舉的註冊數量分別判斷，不能用後者代替使用者實際的“開啟方式”候選。已有預設處理器在臨時註冊期間須保持，原先沒有處理器的型別可由系統臨時解析到新候選；全部測試型別在登出後須恢復原來的處理器和候選狀態。目前使用者的 `LSHandlers` 偏好只在記憶體中比較註冊前、中、後的快照，不輸出其內容；其他偏好與系統級規則不在此比較範圍。

`scripts/macos_launch_services.py` 只識別 `org.xdzip.desktop`。DMG 重定位、原始碼離線重建、原始碼 Qt 產品驗證、QtCore 替換和系統檔案開啟驗證在受限 `build/` 範圍內記錄執行前的註冊集合，結束時（含失敗）僅撤銷新增的測試副本註冊，保留原有主應用及其他產品。清理失敗使驗證命令失敗，不能靜默寫成成功。

DMG 重定位和 QtCore 替換保留交付包的原始文件宣告與校驗流程，但只允許複製到尚不存在的 `build/` 內新 `.app` 路徑。驗證結束或失敗後自動回收這次新建的包，先在作用域內凍結所需雜湊，JSON、日誌及替換前的主包、DMG 和原始碼 Qt 保留。該清理不接受已有應用或符號連結，不追溯刪除歷史驗證包。單獨退註冊的舊 `Alternate` 包仍可能被系統或其他應用重新發現並註冊，不能保證永久不復發；需要保留舊包時可歸入可校驗、可還原的備份，再按明確授權處理鬆散副本。

歷史殘留用以下工具預覽，再顯式應用；`--keep` 必須指定仍存在且身份正確的主應用，可以是目前開發構建或 Applications 中的安裝版：

```sh
python3 scripts/cleanup-macos-open-with.py --keep build/ui-release/apps/desktop/xdzip.app
python3 scripts/cleanup-macos-open-with.py --keep build/ui-release/apps/desktop/xdzip.app --apply
```

工具只撤銷本倉庫 `build/` 內其他 xdzip 副本及同身份失效路徑的註冊，不刪除包、修改簽名或設定預設 ZIP 應用；不同產品、Applications 中的其他安裝和構建樹外路徑不自動處理。解析符號連結後越界的路徑不清理。完成後重新核對註冊集合，只保留指定主開發包；構建樹外有多個安裝時單獨列為未處理，不冒充全域性已去重。

具體清理進度、數量與執行結論記錄在對應 issue，例如[開發副本維護 #7](https://github.com/xiaodaozhi/xdzip/issues/7)，生成 JSON 和原生探針結果僅存本地／CI 工件。
