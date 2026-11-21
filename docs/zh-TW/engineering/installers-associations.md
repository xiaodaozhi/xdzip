<a name="xdzip-安装器与文件关联"></a>

# 安裝器與檔案關聯

[English](../../engineering/installers-associations.md) | [简体中文](../../zh-CN/engineering/installers-associations.md) | 繁體中文 | [日本語](../../ja-JP/engineering/installers-associations.md) | [한국어](../../ko-KR/engineering/installers-associations.md) | [Français](../../fr-FR/engineering/installers-associations.md) | [Deutsch](../../de-DE/engineering/installers-associations.md) | [Español](../../es-ES/engineering/installers-associations.md) | [Português (Brasil)](../../pt-BR/engineering/installers-associations.md) | [Русский](../../ru-RU/engineering/installers-associations.md)

Windows 11 x64 使用者可以在 Microsoft Store 搜尋 **XDZip**，安裝或更新商店版。Windows EXE 安裝器、便攜 ZIP 及 macOS、Linux 安裝包可從 [GitHub 最新發布頁](https://github.com/xiaodaozhi/xdzip/releases/latest)下載。兩條渠道獨立釋出，安裝前可在所選渠道核對版本。

Windows MSIX 使用包清單宣告同一組擴充套件作為開啟方式候選，不覆蓋使用者預設選擇。檔案關聯頁完整顯示可關聯型別，開關為禁用狀態。當前預設應用以 Windows 設定為準。“開啟預設應用設定”使用當前包的 `registeredAUMID` 進入對應頁面，身份查詢失敗時開啟通用預設應用選擇頁。應用不讀取普通安裝版註冊作為包狀態，不寫 UserChoice。實際檔案啟用、圖示、共存和解除安裝行為須在目標系統驗收，見[MSIX 構建](developer-build.md#xdzip-windows-msix-%E5%AE%9E%E9%AA%8C%E6%9E%84%E5%BB%BA)。


逐格式圖示素材及平臺對映見[檔案關聯圖示規範](../visual/file-association-icons.md)。Linux 保留系統 MIME 圖示，安裝不改變預設選擇或現有註冊範圍。

MSIX 檔案關聯引用逐格式 PNG，直接提取自普通 Windows 安裝版使用的同一套 ICO 幀。ZIP、7z、RAR 等已對映格式保留各自的文件圖案、尺寸配置和透明邊距；應用標誌另有獨立的無底板目標尺寸資源。兩組資源均編入包內的 `resources.pri`，修改商店一覽圖片不會替換這些已安裝圖示。圖示快取和預設處理程式由 Windows 管理，安裝不會自動更改預設選擇。

自 `v0.2.0` 起的桌面包及當前原始碼構建將命令列程式放在獨立的 `cli/` 目錄，macOS 位於 `Contents/Helpers/cli/`，保留原 GUI 與檔案關聯入口。更早版本的附件不追溯加入該功能。Windows 安裝器提供預設關閉的當前使用者 PATH 選項，檢測同名命令衝突；解除安裝只撤銷本安裝目錄。Mac/Linux 顯式執行私有 CLI 的 `setup`，獨立 CLI 包也透過 `setup` 安裝使用者級版本目錄和穩定啟動入口，不改 Shell 配置。撤銷用 `setup --remove`，保留版本目錄供核查。詳見 [CLI 安裝契約](command-line.md#xdzip-%E6%9E%84%E5%BB%BA%E4%B8%8E%E5%91%BD%E4%BB%A4%E5%AE%89%E8%A3%85)。

下方打包命令生成實驗性開發包。GitHub 正式發行遵循[釋出協議](release-protocol.md)，商店包使用獨立的 [MSIX 構建流程](developer-build.md#xdzip-microsoft-store-msix)。macOS 直接下載包採用 ad-hoc 完整性簽名，未使用 Developer ID 簽名或公證。適用系統見[支援政策](support-policy.md)。

當前原始碼的 Windows 安裝器發行方、Linux 包維護者及更新策略 vendor 均為 `xiaodaozhi`。這些後設資料不代替發行者證書籤名，已有發行包以其原始後設資料為準。

macOS 手動更新時優先使用對應 Release 的 DMG；使用 app.zip 時，先解壓並將 xdzip.app 移出下載目錄，退出正在執行的舊版，再替換“應用程式”中的副本。下載不代表安裝完成，未公證包可能觸發系統安全提示。具體回退條件見[軟體更新契約](software-update.md)。

系統右鍵選單、獨立操作宿主和自有註冊生命週期見[檔案管理器操作契約](shell-actions.md)。右鍵入口不開啟瀏覽主視窗，完成後預設不開啟目標資料夾。

<a name="xdzip-共用构建入口"></a>

## 共用構建入口

環境準備與操作順序統一見[開發構建指南](developer-build.md)。已有同提交的乾淨構建與測試後，使用 `python3 scripts/desktop.py prepare --packaging` 準備鎖定材料，再執行 package 和 verify，不直接把 CI 全量指令碼用於日常構建。

Windows/Linux 的 `package --kind runtime|installer|all` 共用同一部署與審計鏈，預設 `all`。runtime 先生成部署目錄，installer 從同一 output-dir 已核驗的執行目錄生成安裝器，Linux 同時生成 tar.gz；macOS 只支援本地 `all` 開發 DMG。產物及回執位於指定 output-dir，預設 build/package。準備階段可下載固定打包工具與材料，但不安裝 xdzip，不自動安裝系統編譯器／SDK。Windows 準備 CRT／Inno Setup，Linux 預先安裝 patchelf 與 Debian 包工具，Mac 使用系統部署與 ad-hoc 完整性工具。

普通 `verify` 檢查已有執行時／包，Windows 在獨立副本清除 SDK 搜尋路徑，Linux 使用隔離執行容器，Mac 使用 DMG 重定位與 Cocoa／Metal。它不預設執行安裝、升級／回退或關聯生命週期，不寫使用者預設開啟方式。下面的安裝機制與一次性 CI 專項仍是獨立門檻，不能用執行時透過替代。

<a name="xdzip-windows"></a>

## Windows

安裝器展示隨包的原始 Microsoft VC++ 執行庫協議，許可頁明確適用元件範圍並要求接受，隨後展示雙語 Windows-runtime-NOTICE.txt。便攜包附帶同一原文和說明；專案原創程式碼仍採用 Apache-2.0，Qt 等元件的原始授權和替換權不受此頁影響。編譯前核對原始協議雜湊和完整說明，材料附件保留同一文字，詳見 [Windows 許可](windows-licensing.md)。

便攜包與安裝器使用同一部署目錄，由 [deploy_windows.py](../../../scripts/deploy_windows.py) 呼叫 windeployqt，再按已固定的 Qt Quick Basic 策略移除 FluentWinUI3、Fusion、Imagine、Material、Universal、Windows、僅樣式引用的 NativeStyle／Effects、qmltooling 和 .qmltypes。Basic、Controls impl、Templates、執行時 QML 與 qmldir 保留；Qt Widgets 的 qmodernwindowsstyle 繼續服務原生控制元件。新部署目錄不攜帶 DXC、DXIL、D3DCompiler，任意位置存在這些名稱時部署失敗。`opengl32sw.dll` 不進入新部署目錄；已存在的根目錄副本會在受控暫存中移除，其他位置的同名殘留使部署失敗。增加新樣式或 Effects 功能時必須同時調整部署策略並複驗。

Windows 上 Qt Quick 預設使用 Direct3D 11。Qt 在部分硬體或驅動功能不足的情況下可回退到 Direct3D WARP；這與軟體 OpenGL 是不同路徑。新包不承諾透過 `QT_OPENGL=software` 或 OpenGL 後端提供 Mesa 軟體回退。歷史包及其許可材料保持原樣。[Qt 圖形後端說明](https://doc.qt.io/qt-6.8/windows-graphics.html)列出預設和顯式軟體裝置行為。


VC++ 採用應用目錄內部署，使用 [固定來源與檔案雜湊](../../../scripts/windows-runtime.json) 的微軟官方 x64 14.44.35211 執行庫。[準備指令碼](../../../scripts/prepare-windows-runtime.py) 用固定 WiX 工具解包，再校驗每個 DLL 和許可正文，不執行 EXE 或 MSI 安裝、不從 System32 複製檔案。交付所需 MSVCP／VCRUNTIME DLL 與許可、來源記錄，工具、CAB、MSI 和未執行的 vc_redist 安裝程式不進入產品；Windows 10／11 自帶 UCRT。應用目錄內執行庫須隨產品更新維護，不能依賴系統集中執行庫更新替換它們，參見 [Microsoft 部署方式](https://learn.microsoft.com/en-us/cpp/windows/choosing-a-deployment-method)。

[部署迴歸](../../../scripts/test-windows-deployment.py) 檢查裁剪與執行庫缺失邊界；[實際執行檢查](../../../scripts/verify_windows_runtime.py) 拒絕軟體 OpenGL DLL，在獨立副本中加入測試 DLL，清除 SDK 搜尋路徑，執行系統翻譯、設定重啟、原生標題欄和雙語啟動。它另在清除渲染覆蓋後核對預設 D3D11 場景圖初始化，並顯式請求 WARP、核對實際選中的 Microsoft 軟體裝置。顯式請求不證明故障下的自動回退，也不替代人工視覺驗收。測試檔案不寫入產品目錄，此檢查不替代乾淨 Windows 上的安裝／升級／解除安裝驗收。

build-windows-installer.py 只接受乾淨構建的 deployed 審計，重新核對每個檔案的 SHA-256。固定 Inno Setup 6.7.3、官方安裝程式雜湊、原始碼提交、簡體與繁體中文資源及許可正文見 [工具鏈配置](../../../scripts/installer-toolchain.json)。編譯器不進入產品，許可正文隨 notices 交付。安裝精靈支援共用語言清單中的十種語言。

安裝器使用固定 AppId、PrivilegesRequired=lowest，預設安裝至當前使用者 LocalAppData/Programs/xdzip，不需要管理員許可權。主程式固定為安裝根目錄的 xdzip.exe，Qt 庫位於同目錄，外掛、QML、翻譯與通知保留各自必要子目錄。快捷方式、檔案關聯與解除安裝顯示圖示均指向根目錄，不再建立 versions/提交號-雜湊 的產品層級。提交與二進位制雜湊繼續保留在構建和安裝器回執中，不作為安裝路徑。Inno Setup 的[累計解除安裝日誌](https://jrsoftware.org/ishelp/topic_appendnotes.htm)登記各次安裝擁有的檔案，不遞迴刪除額外內容。設定和最近記錄位於既有使用者配置目錄，不由安裝器刪除。

新佈局寫入有界 install-manifest.txt，記錄審計過的相對路徑和 SHA-256。升級前驗證舊清單的路徑、重複項和大小，拒絕路徑逃逸、裝置別名及清理路徑中的重解析點；待廢棄檔案內容被修改或無法核驗時，在寫入前停止安裝。安裝成功後只清理舊清單中不再交付且雜湊仍匹配的檔案，僅移除空目錄，保留額外內容。同一次安裝即使多次收到完成事件也只執行一次清理。清理期間檔案變化或佔用時保留檔案和重試記錄，返回非零退出碼 20；寫重試記錄失敗需人工修復，不聲稱事務性恢復或掉電安全。此當前使用者安裝流程假設目錄不被外部程序併發修改，路徑與雜湊複檢不是基於控制代碼的惡意競態隔離。

現有版本目錄佈局可直接升級，啟動命令和快捷方式遷移到根目錄，設定及近期記錄位置不變。舊安裝器未交付獨立所有權清單，因此歷史 versions/ 目錄保留至正常解除安裝，由 Inno 累計解除安裝日誌清理原記錄的檔案，不按目錄遞迴刪除；使用者額外檔案繼續保留。回退可重新安裝指定舊版，舊版 QA 夾具仍使用歷史佈局，隨後可再次升級到根目錄。全新安裝不產生 versions/。這不包含舊安裝目錄的全量自動擦除。

產品整個生命週期持有 Local\xdzip.desktop.running 命名互斥量。[AppMutex](https://jrsoftware.org/ishelp/topic_setup_appmutex.htm)阻止安裝／解除安裝，CloseApplications=no，不強制結束應用。FFI 控制代碼建立失敗時啟動失敗，程序關閉時釋放。任務退出仍沿用現有選擇與 Rust join 屏障。

普通 Windows 安裝包在 HKCU 註冊開啟方式候選，寫入自有 ProgID、兩個自有 Applications 項、Capabilities、RegisteredApplications 和 OpenWithProgids 中的自有值。HKLM、副檔名預設值、受保護的 UserChoice／UserChoiceLatest 和其他應用的候選均保留。每個登記字尾有單獨的 ProgID 和對應 ICO，例如 `xdzip.Archive.ZIP`。舊共享 `xdzip.Archive` 保留直接開啟命令，與 `Applications/xdzip.exe` 一起從開啟方式中隱藏。升級移除舊推薦項，保留已有預設選擇的相容性。Windows 為 `.tar.gz` 等複合字尾檔案選擇圖示時使用最後一段副檔名。

普通安裝版在啟動及應用語言切換成功後同步九個核心 ProgID 的型別描述，包括 ZIP、TAR、TGZ、GZ、7Z、RAR、CAB、ISO、UDF。安裝包按應用儲存的語言設定描述，自動模式下使用系統顯示語言。嚮導語言獨立設定。同步前核對安裝標識、程式位置、安裝清單、關聯啟動器及九個已有開啟命令。登記缺失或不符時不寫入。寫入部分成功時，下次正常啟動可重試。便攜、開發和使用隔離設定的測試程式不會執行此同步。其他已登記字尾保留安裝時寫入的描述。Explorer 快取和預設應用選擇可能影響實際顯示。


一次性 Windows CI 安裝檢查使用真實產品驗證啟動時的描述同步，以及執行中簡體中文與英文之間的切換；以便攜版及臨時設定程序驗證不寫入，並核對九類圖示、命令、其他應用候選和預設選擇未變。此自動檢查不能代替 Explorer 或系統設定的人工顯示驗收。


全部格式 ProgID 的帶引號命令啟動 `xdzip-association.exe -- "%1"`，其 Applications 註冊承載完整 SupportedTypes，使系統“開啟方式”只呈現一個 xdzip 應用入口。啟動器嵌入同一品牌應用 ICO 和獨立 OriginalFilename、產品版本資源；應用圖示與逐格式文件圖示分別核驗，不能用文件 DefaultIcon 代替可執行檔案的應用資源。該小型 GUI 啟動器只接受單個檔案引數，先在自己的執行緒初始化 COM 單執行緒單元，再透過 ShellExecuteEx 的明確 `xdzip.Archive` 類委託既有開啟命令，不經過命令直譯器；呼叫返回後釋放 COM。舊版回退會更新共享入口，逐格式候選因此仍指向回退後的程式；COM 初始化失敗、入口不存在、引數無效或系統拒絕呼叫時啟動器返回非零，不嘗試其他應用或修改預設值。該機制依賴回退安裝保留當前安裝目錄內的啟動器，安裝／回退和累計解除安裝須實測。

安裝器在共享 ProgID 和各個已登記的格式 ProgID 上設定 REG_NONE 型別的 `AllowSilentDefaultTakeOver`。Windows 仍可在“開啟方式”中提供這些候選。安裝器用有界原生登錄檔呼叫寫入已經建立的自有鍵，失敗時終止安裝並釋放控制代碼。詳見 [Windows ProgID 約定](https://learn.microsoft.com/en-us/windows/win32/shell/fa-progids)。


普通安裝版和 MSIX 從[格式登記表](format-registry.md)生成 Windows“開啟方式”候選，覆蓋核心的 ZIP、TAR、TGZ、GZ、7z、RAR、CAB、ISO、UDF 以及其他已登記字尾。`.gz` 按內容區分獨立 GZIP 與 TAR.GZ。通用 `.img`、`.001` 和 `.rNN` 不註冊為候選，可透過顯式開啟或適用入口提交。關聯提供系統啟動 xdzip 的入口，讀取器仍探測檔案，並檢查操作所需的格式與來源條件。

<a name="xdzip-ubuntu-2404"></a>

## Ubuntu 24.04

build-linux-installer.py 從同一核驗目錄生成 .deb，私有暫存目錄按輸出路徑區分，允許相同包名在不同候選目錄重新打包，不覆蓋既有暫存。包名 xdzip，版本繫結產品渠道和提交，架構 amd64。系統依賴來自實際部署清單，補充桌面資料庫和鎖工具，glibc 至少 2.39。Qt／QML／外掛及通知位於 /opt/xdzip/versions/；/usr/bin/xdzip 是固定目標的引號化轉發入口。dpkg 負責替換和清除舊包檔案，回退需保留原 .deb。

安裝後的 [xdzip.desktop](../../../apps/desktop/packaging/linux/xdzip.desktop) 宣告軟體包的 MIME 候選，包括 ZIP 及部分別名、TAR 與壓縮 TAR、獨立壓縮流、7z、RAR 和 CAB 等歸檔，以及 ISO／UDF 映像。`%f` 用於單檔案啟動，`--` 標記引數邊界。軟體包提供 `*.udf` 的 MIME 宣告，不接管通用映像或二進位制型別。安裝和解除安裝重新整理 MIME 及桌面資料庫，不改寫 `mimeapps.list`、使用者預設選擇或系統 MIME 圖示。解除安裝清除自有啟動項、圖示及執行檔案，保留使用者設定。

DEB 將 [AppStream 目錄記錄](../../../apps/desktop/packaging/linux/xdzip-local.yml) 安裝至 Ubuntu 應用中心可讀取的 `/var/lib/swcatalog/yaml/xdzip-local.yml`。記錄將包名 `xdzip` 與 `xdzip.desktop` 關聯，使已安裝的軟體包可出現在應用列表中。解除安裝 XDZip 時，dpkg 一併移除該目錄檔案。

安裝的產品持有 /run/xdzip/usage.lock 的共享 flock，維護指令碼先取非阻塞獨佔鎖，活動程序存在就失敗。指令碼在根使用者獨佔的 /run/xdzip 下設定 installing 門，產品取得鎖後再次檢查，避免在 dpkg 鉤子間啟動。符號連結或錯誤所有者／許可權被拒絕；關閉 QFile 釋放鎖。tmpfiles.d 在系統啟動時準備執行鎖。此約束不影響 SDK 開發應用。

成功及 dpkg abort 路徑清理門；中斷／掉電可能留下未完成事務，應執行 sudo dpkg --configure -a 並修復實際錯誤，不能把無法啟動當作安裝成功。指令碼不遞迴清理使用者目錄，產物只生成至 build/ 或 CI。

<a name="xdzip-macos-与文件打开"></a>

## macOS 與檔案開啟

主應用和交付包的 Info.plist 以 Viewer／Alternate 宣告文件和 ICNS，[不宣告預設所有權](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)。開發和測試構建預設使用 Viewer／None，不進入系統開啟方式候選；主開發包或交付構建須顯式設定 `-DXDZIP_MACOS_HANDLER_RANK=Alternate`。預設應用由使用者和 Launch Services 決定，不主動修改預設應用。開發驗證副本的臨時註冊按[Mac 維護方法](macos-deployment.md)收尾，保留已有應用與使用者預設選擇。

CAB、ISO 和 UDF 使用同一個 Viewer 角色，ISO 使用 `public.iso-image`。應用不註冊 `.img` 或通用 `public.disk-image`。Viewer 是 Launch Services 中的關聯角色，實際可執行的操作由[格式後端](native-codecs.md)決定。[ISO 建立配置](iso-creation.md)可單獨寫入新映像。

文件宣告覆蓋[格式登錄檔](format-registry.md)中所有系統關聯候選的最終副檔名，並使用應用包內對應的 ICNS。複合字尾使用最後一段副檔名對應的系統型別。例如，`.tar.gz` 和 `.gz` 共用 GZIP 預設應用，`.tgz` 使用系統的壓縮 TAR 型別。Finder 快取、預設應用選擇和系統型別識別影響實際顯示。

macOS 的 RAR、CAB 和 UDF 匯入型別宣告均須符合 `public.data`，使系統能將實際檔案識別為相應型別。CAB 同時宣告系統使用的 `com.microsoft.cab` 和相容標識 `com.microsoft.cab-archive`；設定頁在系統首選型別是這兩者之一時查詢並更改該實際型別，否則沿用相容標識。UDF 的 Viewer 宣告使用 `.udf` 副檔名而不指定 `LSItemContentTypes`，使舊安裝的無效 UTI 宣告仍被系統採用時，實際檔案也能發現候選；此[副檔名宣告鍵](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)已被 Apple 標記為棄用，必須持續在受支援的 macOS 版本驗證。設定頁按系統實際識別的 `.udf` 型別查詢和更改，型別不可用時回退至既有標識。已有預設選擇不由安裝或註冊主動覆蓋。此前沒有處理器時，Launch Services 可能按[隱式規則](https://developer.apple.com/library/archive/documentation/Carbon/Conceptual/LaunchServicesConcepts/LSCGlossary/LSCGlossary.html)將新候選解析為當前處理器；撤銷臨時註冊後應恢復原狀，此解析不等於應用主動寫入使用者偏好。


FileOpenRouter 在 QML 前接收 [QFileOpenEvent](https://doc.qt.io/qt-6.8/qfileopenevent.html)，只接受本地檔案並複用應用的開啟與啟用路由。GUI 命令列 `--` 後作為檔案路徑，避免誤解為診斷開關。待處理意圖最多 128 個，不搶佔活躍任務。同一登入會話的正常產品程序共用一個例項；多檔案請求按順序開啟並啟用最後一個，標籤和視窗行為見[應用會話契約](application-sessions.md)。

啟動引數在載入 QML 前入隊，主視窗先隱藏，附接控制器時立即提交首個可用請求，不再要求視窗已可見。主視窗在初始事件佇列中顯示，允許平臺啟動事件走同一處理路徑；外部開啟期間先隱藏歡迎入口再改變會話，避免首幀或替換會話閃現歡迎頁。取消／失敗恢復歡迎頁，成功顯示真實列表，詳見[讀取狀態](../ux/welcome-opening.md)。

macOS 原生啟動單獨等待 AppKit 的 NSApplicationDidFinishLaunchingNotification，再在隱藏狀態處理該批排隊的 Qt 檔案事件，隨後顯示視窗。通知與排隊回撥均限定在主視窗生命週期，不替換 Qt 的應用 delegate，不使用固定毫秒延遲；offscreen 等非 Cocoa 後端仍走普通事件佇列。實際 Launch Services 迴歸曾檢出 1 幀歡迎正文，修正後要求為 0。check-macos-open.py 在本地與 Mac CI 執行 argv 和原生 open 兩種路徑，僅儲存 JSON／日誌，不生成或上傳應用包。

<a name="xdzip-自动验证与限制"></a>

## 自動驗證與限制

三平臺 CTest 檢查啟動前檔案事件、非本地 URL 拒絕及讀取忙時排隊。顯式 XDZIP_ASSOCIATION_PROBE 加隔離設定僅供 CI：透過正常開啟流程讀取自有十萬條目 ZIP，儲存計數／構建身份並正常關閉，不記錄來源路徑或執行解壓。

當前關聯探針等待至少一次實際幀呈現，逐幀記錄歡迎正文可見次數；Windows ShellExecuteEx 和 Linux GIO 的當前產品要求 presented_frames 大於 0 且 welcome_frames 為 0。固定舊產品仍按歷史探針能力驗證，不偽造舊二進位制具有新的首幀計數。

Windows 原生 CI 安裝 QA 變體 1、升級 2、回退 1，再安裝真正交付包；ShellExecuteEx 按自有註冊類開啟中文、空格、& 路徑，檢查預設及 UserChoice 不變、忙時拒絕替換、解除安裝保留設定／額外檔案／其他候選。Linux 在無 SDK／系統 Qt／編譯器、無網路容器執行相同 dpkg 生命週期，UID 1000 的產品由 GIO 桌面啟動項實際開啟十萬條目，核對 MIME、預設、鎖、解除安裝和 purge。

Linux 發行驗證還在容器的獨立使用者配置中切換 MIME 預設項。GIO 將樣本的內容型別解析為由 xdzip 開啟時，用 `gio open` 驗證啟動。檢查覆蓋所有宣告的 MIME 預設項，另用獨立處理程式驗證切換生效，結束時恢復原預設配置。Linux 和 macOS 的自動開啟檢查使用無需互動的樣本。AES Crypt 的瀏覽器測試驗證開啟時提示密碼及認證後的讀取。系統設定介面和其他桌面環境需單獨驗證。


QA 變體共用本次核驗二進位制，只改變測試包版本／目錄，驗證安裝機制，不作為發行工件，也不冒充真實不同產品版本的資料遷移。設定 schema 當前為 1，兩平臺安裝後設資料記錄它，舊安裝器拒絕覆蓋宣告更新 schema 的安裝；未來遷移須同步契約。

後續故障注入只允許 QA 變體 2，在 postinst configure 階段失敗一次；正式產品禁止攜帶該分支。容器實際驗證 half-configured 狀態、執行門阻止啟動、設定保持，再執行 dpkg --configure -a 恢復並經 GIO 開啟真實檔案。這是可重複配置失敗，不冒充裝置掉電。設定相容檢查另編譯保留的原始設定元件與當前元件，往返同 schema 配置；原始輸入逐檔案固定雜湊，不依賴舊 main 歷史，不冒充完整安裝器的資料遷移。

完整產品相容另由 prepare-previous-product.py 在一次性 Windows/Linux CI 主機構建固定相容快照（來源為原 4cf0f8c）的完整產品原始碼：使用 detached worktree 保留真實幹淨身份，使用當前固定 SDK 生成歷史產品 QA 安裝器，不下載易過期的舊 Actions 二進位制。先安裝歷史產品並實際開啟，再安裝當前產品並檢查 revision 改變、配置位元組保留，再回退歷史產品重新開啟並檢查資料。它證明兩個完整原始碼提交在同 schema 的相容，不冒充歷史已簽名原包、尚不存在的 schema 2 遷移或裝置重啟。歷史安裝器只作 QA，不上傳為產品；回執繫結兩個二進位制與安裝器 SHA-256。

windows-install-lifecycle.json、linux-install-lifecycle.json 及包雜湊為實際證據。真實 Explorer/Finder 雙擊、預設選擇 UI、重啟後的服務／快取、未正式簽名提示、GPU 與桌面專項按人工清單核驗。自動指令碼不得自行把 INSTALL-01／ASSOC-01 全部標記透過；既有人工結論見 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)，不因重新生成 pending 模板而重開驗收。未來 schema 遷移須另增實際相容用例。

Windows 忙時替換保護的無任務探針使用獨立配置關閉托盤，使 `CloseMainWindow` 按該配置執行正常退出；其他關聯開啟探針保留預設托盤配置並透過顯式退出屏障結束。安裝保護探針不得把“視窗隱藏”誤判為“程序退出失敗”，也不得透過強制終止正常路徑跳過退出驗證。

Windows 更新協議夾具與真實 Qt 更新夾具同樣提供關聯啟動器和全部九份文件 ICO，不能透過缺少必需載荷的安裝器檢查。協議夾具的小型 C 主程式與關聯命令只驗證更新交接；嵌入應用資源、原生候選和真實視窗由完整 Qt 產品鏈分別檢查，不混用結論。


<a name="xdzip-兼容基线的长期输入"></a>

## 相容基線的長期輸入

真實升級／回退使用固定的舊版完整產品原始碼，不從已重寫的 main 歷史獲取。`prepare-previous-product.py` 鎖定獨立 `codex/installer-compat-baseline` 分支中的單根原始碼快照；該分支只儲存構建和迴歸所需的原始原始碼及資源，不攜帶舊 main 父歷史、執行報告或工作流。快照內 `COMPAT-BASELINE.json` 記錄原始來源提交與樹身份，不能合併回 main，也不作為發行包或當前產品的驗收結論。

構建後 CLI 身份必須匹配快照自身的固定 SHA，回執另記原始來源身份，不能偽裝成原發布二進位制。設定元件另以三份原始小型原始碼作為持久測試輸入，逐檔案雜湊固定，雙向讀寫及未來 schema 拒寫不依賴網路或歷史 commit。不要隨當前實現修改這些迴歸輸入；需要新基線時另行評審並保留來源。

該凍結快照早於獨立更新助手，不含助手原始碼或二進位制。只有 `prepare-previous-product.py` 為它傳入 `--qa-historical-revision`：部署／打包指令碼要求一次性 CI、完整固定 SHA 和實際乾淨產品身份同時匹配；Linux 還要求歷史版本模式，禁止與當前 QA 變體混用。Windows 直接執行舊產品的版本命令核對身份，Linux 使用獨立部署審計的身份與完整檔案雜湊。回執明確記錄歷史原始碼不含助手，不注入當前助手，也不為不存在的 Linux 助手安裝 polkit 授權。普通產品和當前 QA 變體仍必須交付並核驗獨立助手；已有 `--qa-legacy-version` 本身不允許缺少助手。

<a name="xdzip-debian-版本迁移"></a>

## Debian 版本遷移

首個公開預釋出開始使用 Debian epoch 1，包版本為 `1:语义版本映射+git提交`，例如 `1:0.1.0~alpha.1+git...`。歷史 `0.1.0~dev.9` 按 dpkg 規則會排在無 epoch 的 alpha.1 之後，不能直接沿用舊對映。epoch 只用於系統包排序，產品介面、版本標籤和 release.json 仍使用原語義版本，後續 Debian 包保持 epoch 1，alpha、beta、rc、stable 的順序繼續由上游版本表達，規則見 [Debian Policy](https://www.debian.org/doc/debian-policy/ch-controlfields.html#version)。

真實歷史產品夾具使用僅 CI 允許的 `--qa-legacy-version` 保留原無 epoch 身份，不能把舊包改成新排序來掩蓋升級問題。當前包必須在實際 dpkg 升級、回退和安裝生命週期驗收中透過；使用者配置與外部預設關聯保持原契約。

<a name="xdzip-设置中的默认应用"></a>

## 設定中的預設應用

設定頁按生成的格式資料對關聯字尾分組，開關能否操作取決於平臺服務和安裝身份。Windows 交由系統“預設應用”處理，不寫 UserChoice。MSIX 禁用開關，改用當前包的系統設定入口。Linux 把支援的字尾對映到 MIME 型別，用 `xdg-mime` 管理使用者預設設定，移除預設時可選擇已安裝的替代程式。

在 macOS 上，設定頁啟用已由安裝包以 Viewer／Alternate 宣告的型別開關。即使型別此前沒有預設應用，也能請求設為 xdzip。macOS 可能要求確認。收到成功回撥後，設定頁會短暫等待 Launch Services 返回新的預設應用。系統查詢確認切換為 xdzip 後，應用按系統內容型別在本地記錄原應用的包標識。關閉開關時，如果系統仍將原應用列為相容候選，則優先請求恢復。沒有相容的歷史應用或恢復失敗時，可以選擇其他相容的已安裝應用。沒有替代應用時，說明直接顯示在開關旁。取消後保留當前選擇，介面以重新查詢的系統狀態為準。Viewer／None 構建禁用關聯變更。


採用自動偵測時，Windows 安裝程式依目前使用者的系統介面語言選擇關聯描述。繁體中文偏好使用繁體中文，不支援的語言使用美式英語。手動選擇受支援的語言時優先採用該語言。安裝程式會保留自動偵測，應用程式啟動後再依系統語言同步自己的描述。
