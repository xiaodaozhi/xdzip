<a name="xdzip-命令行运行契约"></a>

# 命令列執行契約

[English](../../engineering/command-line.md) | [简体中文](../../zh-CN/engineering/command-line.md) | 繁體中文 | [Français](../../fr-FR/engineering/command-line.md) | [Español](../../es-ES/engineering/command-line.md) | [Português (Brasil)](../../pt-BR/engineering/command-line.md) | [Русский](../../ru-RU/engineering/command-line.md)

CLI 是獨立的 Rust 宿主，透過同一個 `xdzip-runtime` 服務呼叫格式和安全 I/O，不依賴 Qt、QML 或顯示會話。7z、RAR、CAB、ISO、UDF 繼續使用隨包提供的隔離工作程序，不搜尋 PATH 中的替代解包工具。平臺和架構沿用[支援政策](support-policy.md)，獨立 Linux CLI 不要求 X11；執行需要系統 OpenSSL 3 和證書，構建另需 libssl-dev、pkg-config。

自 `v0.2.0` 起，正式發行提供三平臺獨立 CLI 包，桌面包也包含私有 CLI；更早版本的附件不會因此改變。下載入口見[釋出頁](https://github.com/xiaodaozhi/xdzip/releases)，原始碼構建、安裝與更新邊界見下文及[釋出協議](release-protocol.md)。具體實現與驗收進度登記在 [#254](https://github.com/xiaodaozhi/xdzip/issues/254)。

<a name="xdzip-使用与输出"></a>

## 使用與輸出

```sh
xdzip --help
xdzip version
xdzip list archive.zip --json
xdzip extract archive.zip docs/ -C output
xdzip extract archive.zip --entry-id 3 -C output
xdzip create backup.7z docs/ --level normal
xdzip create data.iso docs/
xdzip create portable.sfx.exe docs/ --level normal
xdzip create private.zip docs/ --encrypt --password-stdin
xdzip add backup.zip notes.txt --into docs
xdzip test backup.zip
xdzip update --check
xdzip update --download --output downloads
xdzip update --yes
xdzip update --status
xdzip gui archive.zip
```

不帶引數或使用 `--help` 時顯示幫助。`version`、`--version` 和 `--version-json` 根據 `VERSION.json` 及構建時的 Git 狀態報告版本身份。`--version-json` 返回獨立物件，包含 product、version、build、revision、dirty、component、channel、configuration、distribution、platform 和 architecture。可用 `--language zh-CN|en-US` 選擇幫助和訊息的語言。預設按 `LC_ALL`、`LC_MESSAGES`、`LANG` 的順序取值，所用值以 `zh` 開頭時使用中文。公開標識和錯誤碼使用英文。`l`、`x`、`c`、`a`、`t` 分別是 `list`、`extract`、`create`、`add`、`test` 的別名。用 `--` 傳遞字面檔案引數。

所有業務命令接受 `--json`。stdout 只輸出一個 JSON 物件，欄位固定為 schema_version（1）、command、status、error_code、data、summary、warnings。未知大小使用 null，包括共享壓縮 TAR 流的子項；原始 DEB ar 成員保留精確壓縮尺寸。列表 data 包含 format 和 entries；每條保留 id、path、display_name、size、packed_size、method、directory、link、payload_supported、encrypted、warning。path 是凍結的有效名稱，異常或未解析名稱為 null；display_name 只用於轉義展示。任務 summary 包含實際讀取、解碼、校驗、提交、跳過、目錄與分卷計數，並保留清理警告及不確定結果。

進度、錯誤和互動提示寫入 stderr。JSON、非終端或 `--non-interactive` 禁止詢問，缺少密碼或衝突決定時取消等待中的任務，等待清理後返回 interaction_required。退出碼為 0（成功或明確交接）、1（失敗、部分完成或清理警告）、2（引數錯誤）、3（缺少互動、授權或安裝條件）、130（已確認取消）。顯式 skip 是已執行的策略，跳過數量仍單獨報告。取消未被接納且提交已完成時保留真實完成結果。

<a name="xdzip-归档行为"></a>

## 歸檔行為

`list` 返回物理記錄並保留重名條目。`extract` 接受完整、精確、區分大小寫的有效包內路徑，使用正斜槓。目錄遞迴展開，保留根目錄下的層級。精確重名返回 `ambiguous_member`。可用 `--entry-id` 傳入 `list` 返回的數字 ID（從 1 開始），選擇多個 ID 時重複該引數。無法解析指定路徑或 ID 時，會在建立目標目錄前失敗。目前不支援萬用字元、從 stdin 讀取整個歸檔，或將解壓內容寫到 stdout。

指定 `--limit` 或 `--offset` 均啟用分頁。偏移量從 0 開始，最大為 1,000,000。只指定 `--limit` 時從 0 開始，只指定 `--offset` 時每頁最多返回 128 條。`--limit` 的取值為 1～4,096。JSON 的 `data` 包含 `offset`、`total` 和可為空的 `next_offset`，`summary.entries` 是實際返回的行數。普通單檔案 ZIP／ZIP64 可索引一百萬條，未分頁的列舉在總量超過十萬條時失敗。兩種模式下，序列化後的 JSON 行與格式化文字合計均限 16 MiB。超限返回 `resource_limit`，不返回部分結果。分頁不改變物理 ID，詳見[大歸檔索引](large-index.md)。

extract 預設輸出到目前目錄，`-C` 指定缺失目錄時，從最近已存在的安全父目錄逐級獲得授權，不使用無條件遞迴建目錄。目標、源身份、連結、Unicode／大小寫碰撞、暫存、校驗及提交繼續遵守[安全解壓](zip-safe-extraction.md)和[檔名編碼](filename-encoding.md)。預設詢問重名；`--conflict ask|skip|rename|overwrite` 顯式選擇策略。`--allow-high-ratio` 只確認高壓縮比，不取消硬資源限制。

`create` 按輸出[字尾](format-registry.md)選擇建立配置，要求目標父目錄已經存在。`--level store|fast|normal|best` 按所選配置校驗。普通 TAR 和 ISO 可省略級別，或顯式使用 `store`。LHA（`.lzh` 或 `.lha`）的 `fast`、`normal`、`best` 對應 LH5 的 1、6、9 級。ZIP 和 7z 支援 `--encrypt`、`--volume-size`。7z 還可按配置使用 `--encrypt-names`、`--solid`。分卷大小接受正整數字節數，或帶 `KiB`、`MiB`、`GiB` 字尾的值。不適用於所選配置的引數會在寫入前被拒絕。

密碼只透過終端隱藏輸入或 `--password-stdin` 的一行 UTF-8 輸入傳遞，移除行結束符但不裁剪密碼本身。建立互動加密包時確認兩次。拒絕明文密碼引數，不讀取密碼環境變數，不寫設定或日誌。密碼長度與歸零規則沿用[密碼契約](password-extraction.md)和[建立契約](zip-creation.md)。

`add` 提交宿主更新，預設寫入包內根目錄。`--into` 按包內相對名稱選擇已有的安全目錄。符合條件的輸入可新增或替換條目，包內移動是桌面操作。CLI 複用[歸檔修改](archive-transfers.md)的格式與記錄限制，以及輸入保護、臨時重建和最終替換流程。`test` 檢查整包，不寫出歸檔條目。檢查成功後，JSON 的 `data.check_kind` 報告實際檢查強度，見[完整性檢查](archive-checking.md)。

每次 CLI 操作擁有獨立執行時，不加入桌面瀏覽會話。Ctrl+C／終止訊號請求協作取消，等待任務終態、清理和執行時 join；先前已提交的檔案保留。共享服務的 submit_wait 僅序列獲得短狀態鎖，不等待佇列容量，不改變 GUI 的非阻塞准入。Catalog 每頁最多 128 個物理條目，SelectMembers 的數量和名稱預算沿用既有上限，所有請求繫結不可變會話身份。

<a name="xdzip-构建与命令安装"></a>

## 構建與命令安裝

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
python3 scripts/cli.py package --output build/package/cli
build/cli/bin/xdzip --help
```

該入口使用倉庫鎖定的 Rust、CMake、平臺 C++ 工具鏈及現有隔離 worker，無需安裝 Qt。Windows 打包還需透過既有 prepare-windows-runtime.py 準備固定、附帶原許可的 app-local VC++ 執行庫。Qt 的統一構建入口也構建 CLI 並放入私有 cli 目錄，GUI 可執行檔案、應用圖示與關聯入口不改名。

macOS 的內建 CLI 可執行檔案位於 `Contents/Helpers/cli/`，普通上下文 JSON 位於 `Contents/Resources/xdzip-cli-context.json`，由應用資源簽名封裝。CLI 只在固定的 Contents／Helpers／cli 佈局查詢該資源，繼續核對 schema、desktop kind 與固定 GUI 相對路徑；缺少標記按便攜身份處理，非法標記拒絕。重複構建清除 Helpers 下的舊生成標記，不能透過放寬簽名驗證解決佈局錯誤。

新的桌面包在上下文中增加 `worker_relative`，內建 CLI 與桌面共享一個格式工作程序。只接受固定平臺相對路徑：Windows 為桌面旁的工作程序，macOS 為 `Contents/MacOS/xdzip-native-codec`，Linux 為構建目錄旁或部署目錄 `bin/` 中的工作程序。目標缺失、為連結或路徑不符時拒絕上下文，不搜尋 PATH 或其他副本。舊上下文沒有該欄位時，以及獨立或管理式 CLI 包，保留原有的同目錄查詢。顯式開發環境覆蓋仍優先。使用 `scripts/check-cli.py --default-worker` 驗證部署後的實際查詢，普通測試路徑覆蓋不能替代該檢查。

獨立包展開後執行 bin/xdzip，Windows 加 .exe。執行 `bin/xdzip setup` 顯式安裝使用者級穩定入口及版本化執行目錄。Unix 預設入口為 ~/.local/bin，Windows 預設位於使用者 LocalAppData 下的 xdzip/cli/bin；管理目錄使用平臺使用者資料位置。--bin-dir 和 --install-dir 可顯式指定位置。使用者自行將目錄加入 PATH，工具不修改 Shell 啟動檔案。已有入口或登記檔案一律拒絕覆蓋。

管理式身份要求有效的 installation.json；目錄恰好名為 versions 而沒有登記檔案時仍視為便攜執行。

Windows 撤銷正在執行的啟動器時，將雜湊仍匹配的自有副本移到同一命令目錄的私有 .xdzip-retired 子目錄，入口立即撤銷，結果記錄 retained_launcher。後續顯式 setup 只按有界回執回收雜湊仍匹配且已退出的副本，修改過的檔案保留並拒絕覆蓋；保留版本目錄中的 CLI 可重新執行 setup 恢復入口。

版本身份的 distribution 區分 source 與 release。普通原始碼構建即使使用 Release 配置並保持乾淨，也只保留檢查與下載；只有專用 Release 管線標記的發行身份、有效管理式安裝和受信目標籤名共同滿足時才自動替換。

桌面安裝器的 Windows PATH 選項預設不勾選，並拒絕與其他 xdzip 命令衝突。Mac/Linux 可執行桌面包私有 CLI 的 setup；它只登記桌面安裝內的 CLI，不複製出獨立可升級核心。`setup --remove` 只刪除經雜湊確認、屬於本工具的入口和登記檔案。版本目錄及恢復資料保留，不刪除使用者輸入或仍可能被程序使用的執行目錄。

穩定啟動器不經過 Shell 轉義，在 Unix exec 實際 CLI，在 Windows 等待控制檯子程序並轉發取消及退出碼。目前執行目錄由原子 current.json 指標選擇；初次安裝、新版自檢或檔案提交失敗不能產生已安裝成功的結果。

<a name="xdzip-更新桌面交接与恢复"></a>

## 更新、桌面交接與恢復

update 針對目前安裝，獨立 CLI 只升級 CLI，桌面包內的 CLI 更新整套桌面版。--check 查詢，--download 下載並驗證，預設確認後安裝，--status 核對回執和實際身份。--yes 只確認本次更新，不替代系統授權。便攜、dirty、原始碼或身份不明構建只允許檢查和下載；獨立自動替換要求管理式、乾淨 Release 安裝。

CLI 使用固定公開 GitHub Releases、渠道與單調 build 規則。CLI 三平臺資產使用單獨的 cli-release.json 與 cli-release.json.sig，內建公鑰驗證原始位元組，再檢查大小與 SHA-256。缺簽名或未受信金鑰只提供人工獲取資訊，錯誤簽名明確失敗。桌面六包 release.json 不加入 CLI 條目，舊客戶端繼續使用原協議。

網路使用系統證書，預設讀取標準代理環境變數，--proxy 選擇無憑據 HTTP／SOCKS5，--no-proxy 直連，--mirror 選擇無憑據、無查詢和片段的 HTTPS 字首。原始 GitHub 地址、重定向主機、響應大小、超時和下載預算繼續有界。網路、TLS、離線、限流與無相容新版本分別報告，不自動輪詢或重試。

獨立更新在使用者擁有的私有目錄暫存，以同一歸檔執行時安全解包，驗證完整固定檔案清單及每檔案摘要，核對產品、元件、本機平臺架構和簽名清單渠道，執行新版 --version-json 自檢後才切換版本指標。事務互斥，回執繫結目標身份；失敗保持原指標或明確進入恢復狀態，不偽造自動回滾。

桌面檢查與下載只查詢主程式的無 GUI 安裝身份入口，不建立 QApplication 或消費安裝回執。安裝意圖透過原有同使用者單例項通道傳遞，只接受有界請求目錄和候選版本；不接受任意命令、包檔案或信任根。請求／結果檔案複用 Rust 原生目錄授權與安全提交。桌面控制器獨立重新檢查並核對候選，複用關於頁、安裝器、Sparkle 與全部任務 join 屏障。桌面 CLI 任務也持有安裝佔用保護，安裝不能破壞它們。

終端收到 accepted 僅報告 handed_off。實際安裝與恢復由桌面回執和實際執行身份判斷，不能把下載或交接記為 installed。無互動模式拒絕需要圖形介面或系統授權的桌面安裝。安裝安全及原生驗收邊界見[更新契約](software-update.md)。

`.iso` 配置以固定的 0 級建立無壓縮 ISO9660/Joliet 資料映像，可省略 `--level` 或使用 `--level store`。Windows x64 的 `.sfx.exe` 配置使用隨包部署的相鄰 `xdzip-sfx.exe` 封裝 Store 或 Deflate ZIP，拒絕加密、固實和分卷。兩種配置均不支援修改現有歸檔。詳見 [ISO 建立](iso-creation.md)與[自有 SFX 建立](sfx-creation.md)。
