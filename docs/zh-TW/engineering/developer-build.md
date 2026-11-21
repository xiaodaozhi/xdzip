<a name="xdzip-开发构建与平台验证"></a>

# 開發構建與平臺驗證

[English](../../engineering/developer-build.md) | [简体中文](../../zh-CN/engineering/developer-build.md) | 繁體中文 | [日本語](../../ja-JP/engineering/developer-build.md) | [Español](../../es-ES/engineering/developer-build.md)

本文是三平臺環境準備、構建、測試、打包和驗證的統一入口。共用編排由 [scripts/desktop.py](../../../scripts/desktop.py) 實現；平臺文件僅補充部署佈局、工具行為和系統限制，不再另維護一套日常命令。

獨立命令列版使用 `python3 scripts/cli.py build`、`test` 和 `package`，無需 Qt、X11 或圖形會話。Linux 編譯另需 `libssl-dev pkg-config`，執行使用系統 OpenSSL 與證書。桌面構建自動生成私有 CLI 目錄。命令、安裝與受信更新見 [CLI 契約](command-line.md)。

開發構建仍標記實驗性，不能當作正式 Release 安裝包。能力和限制見根目錄 [README](../../../README.zh-TW.md)，支援範圍見[支援政策](support-policy.md)。正式 `v0.1.0` 的歷史發行審查見 [#4](https://github.com/xiaodaozhi/xdzip/issues/4) 和 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)；未來版本須按具體候選重新建立發行審查記錄。

<a name="xdzip-环境与入口"></a>

## 環境與入口

先安裝 Python 3.10+、Git、CMake 3.24+、Ninja、rustup 和平臺編譯器／SDK。Rust 版本讀取根目錄 rust-toolchain.toml，Qt 固定為 6.8.2，依賴與更新規則見[工具鏈策略](toolchain.md)。命令均在倉庫根目錄執行。Windows 可將示例中的 `python3` 改為可呼叫的 `python`，並在 x64 MSVC 開發環境執行。

<a name="xdzip-本地工具链目录"></a>

## 本地工具鏈目錄

透過 PowerShell 載入 Visual Studio 開發環境時，應保留 `USERPROFILE`、`LOCALAPPDATA` 和 `APPDATA` 的 Unicode 值；如果載入器改變了它們的編碼，在繼續執行前恢復原值。Qt 使用這些位置確定預設快取和配置目錄，因此即使編譯器工作正常，目錄變數損壞也可能導致檔案操作失敗。

`prepare` 會在原 SDK 中構建並驗證 QtNetwork 安全補丁，修復 CVE-2026-116337 與 CVE-2026-105946，其他執行模組不被重建庫替換。另需 Perl、make（Windows 為 nmake）和 OpenSSL 命令列工具；Ubuntu 需 `zlib1g-dev libpcre2-dev libicu-dev libglib2.0-dev libzstd-dev libkrb5-dev`。原始碼、生成宣告及構建快取統一在 `build/`，預設複用 `build/toolchain/qt-network-security/<平台>`。輸入和實際庫摘要不匹配時，產品構建與打包直接失敗，重新執行 `prepare`；來源、ABI、TLS 功能及失敗恢復見[補丁維護說明](../../../patches/qt/README.md)。


統一入口的本地依賴只使用 `build/toolchain/`，該目錄及其內容被 Git 忽略，不提交到倉庫：

| 內容 | 固定目錄 | 說明 |
| --- | --- | --- |
| Qt SDK 6.8.2 | `build/toolchain/Qt/6.8.2/<平台子目录>` | `<平台子目录>` 為 `macos`、`msvc2022_64` 或 `gcc_64`，`qmllint`、`qmake`、CMake 配置和 Qt 執行庫都來自這裡 |
| Python 工具環境 | `build/toolchain/desktop-tools` | `aqtinstall`、打包和校驗指令碼使用的固定虛擬環境 |
| Qt 下載暫存 | `build/toolchain/qt-install-tool` | 僅儲存 Qt 安裝器下載和臨時輸入 |

`scripts/desktop.py prepare --install-qt`、`doctor`、`build` 和 `test` 預設共享這些目錄。不要為每個開發管線另建 `.cache/Qt`、`.cache/desktop-tools` 或同類目錄；只有需要隔離不同 Qt 版本、平臺或實驗構建時，才顯式傳入 `--qt-prefix`、`--qt-root` 和 `--tools-dir`。CI 可以把這些引數顯式指向 runner 臨時目錄，但本地預設佈局不隨 CI 回退路徑改變。CMake、Ninja、rustup、Rust 編譯器和平臺 C++ 編譯器屬於宿主工具鏈，按環境預檢要求安裝和呼叫，不由 Qt 工具目錄複製。

研發週期完成、分支和 worktree 清理後，必須清理該週期在 `build/` 下產生的臨時目錄和檔案。保留 `build/toolchain/` 等仍需複用的共享工具鏈，刪除週期專用構建、診斷和下載暫存；具體規則和手動兜底見[倉庫內容與歷史管理](../../../developers/engineering/repository-hygiene.md#development-cycle-artifact-cleanup)。

| 平臺 | 先準備的系統環境 | Qt SDK 子目錄 |
| --- | --- | --- |
| macOS | Apple Silicon 原生終端，Xcode Command Line Tools 與有效 macOS SDK | macos |
| Windows | MSVC 2022 x64 C++ 工具與 Windows SDK，x64 Developer PowerShell／命令列 | msvc2022_64 |
| Ubuntu 24.04 | x86_64 C++ 編譯器，OpenGL、EGL、xkbcommon 開發包和 X11 執行庫 | gcc_64 |

macOS 可用 `xcode-select --install` 準備命令列工具。Ubuntu 基礎開發包包含 `libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev libx11-dev libxext-dev libnautilus-extension-dev pkg-config`；打包還需要 patchelf、binutils（readelf 和 strip）與 Debian 包工具，隔離執行驗證另需可連線的 Docker。Windows 僅安裝 Visual Studio 但未進入開發環境時，cl、link、rc 或 dumpbin 仍可能不可用，預檢會指出實際缺失項。共用入口不自動安裝系統編譯器、SDK 或需要管理員許可權的系統包。

Windows 可按 [Community 免費許可路徑](windows-licensing.md)準備自有開發工具，閱讀並接受該版本條款；只裝 Build Tools 或 VC_redist 不單獨證明資格。GitHub 標準託管 runner 的 Enterprise 使用及相應再分發另有公開依據，不要求維護者為託管構建購買 Enterprise 或在本機安裝 Community IDE，也不將該依據用於自託管機器。現有 2019 工具無需為許可複核而解除安裝；本地替代配置與當前 2022 發行 SDK 的相容性須分別驗證。

所有命令共用以下引數：

| 引數 | 預設值與用途 |
| --- | --- |
| `--qt-prefix` | 先取環境變數 `XDZIP_QT_PREFIX`，否則使用 `build/toolchain/Qt/6.8.2/<平台子目录>`；應指向 SDK 字首而不是 bin 目錄 |
| `--build-dir` | `build/desktop`，CMake 構建與測試目錄 |
| `--output-dir` | `build/package`，交付工件與驗證輸出目錄 |
| `--tools-dir` | `build/toolchain/desktop-tools`，隔離的 Python 工具環境與準備材料 |
| `--configuration` | `Release`，也可顯式選擇開發所需的構建配置 |

build-dir 與 output-dir 必須使用倉庫 build/ 內的獨立子目錄，以匹配底層打包和驗證工具的輸出邊界。同一次分階段執行應保持上述引數一致。顯式 `--qt-prefix` 優先於環境變數；不需要全域性修改 CMAKE_PREFIX_PATH。入口為子程序準備 Qt 與工具搜尋環境，普通桌面啟動不要沿用離屏測試變數。生成物只寫入忽略的 build/、target/、dist/ 或工具快取，不提交 Git。

<a name="xdzip-分阶段执行"></a>

## 分階段執行

各階段可以單獨呼叫，不隱式補跑前面的準備、構建或測試。缺少前置輸入時會報錯，先執行相應階段再重試。每個階段在開始工作前檢查其所需環境，`doctor` 可在不構建或打包的情況下提前檢視問題。

Windows 的受保護構建按現有控制檯內的程序協調，獨立控制檯可以並行，共享同一現有控制檯的構建依次執行。構建期間不得由其他程式碼將受保護程序從該控制檯分離或重新附加。無控制檯程序自行建立的臨時控制檯供其自身和子程序使用，外部程序不得主動附加。舊版指令碼對無視窗控制代碼的控制檯使用統一回退鎖，不能與新版協調，執行中的舊版構建結束前請不要在同一無視窗控制檯啟動新版構建。不能確定程序列表時會停止，不改變內碼表。

對於不需要互動的構建，也可顯式使用獨立控制檯包裝器，並將輸入、輸出和錯誤流全部重定向。例如在 PowerShell 中：

```powershell
New-Item -ItemType Directory -Force build\isolated-logs | Out-Null
cmd /c "python scripts\windows_detached_console.py --timeout 7200 -- python scripts\desktop.py build <NUL >build\isolated-logs\desktop.log 2>&1"
$LASTEXITCODE
```

該包裝器為每次命令建立獨立的隱藏控制檯，並在超時或監督程序退出時結束其子程序；完成訊號缺失時按失敗處理。並行構建仍須分別使用不同的 `--build-dir`，包裝器不為構建目錄提供互斥。原有桌面構建和 `windows_ninja_safety.py` 的依賴檢查仍執行，直接包裝其他命令不會自動補上該檢查。此入口只適用於非互動命令，不支援將提示、密碼輸入或偽控制檯互動交給被包裝程序。

`prepare` 核驗當前產品格式原始碼清單。ZIP、TAR/GZIP、7z、RAR、CAB 和光碟容器使用獨立 Rust，官方產品原始碼清單為空，不下載或編譯官方 UnRAR、7-Zip 引擎。獨立測試參照按各自固定來源準備，與產品執行依賴分開；RAR 有限對映的原始來源通知隨打包和對應原始碼交付保留，見[許可證契約](licensing.md)。`build` 只使用已準備輸入，不浮動下載。只測 Rust 時，需要先構建 `xdzip-native-codec`，將 `XDZIP_NATIVE_CODEC` 指向該工作程序，再執行 workspace 測試。原生後端任務閉環另執行 `cargo test -p xdzip-runtime --locked native_backend_tests -- --ignored`，統一 `test` 與 Foundation CI 均強制執行這部分，忽略狀態不能記作格式驗證透過。原始碼、流授權及許可邊界見[原生後端契約](native-codecs.md)。

| 命令 | 作用與前置條件 |
| --- | --- |
| `doctor --for-stage build` | 檢查 PATH、鎖定 Rust、Qt、編譯器、SDK 和版本；預設檢查 build 所需環境 |
| `prepare` | 準備共用工具和依賴；下載／安裝 Qt、Rust 工具鏈及打包材料須使用下面的顯式開關 |
| `build` | 配置並構建真實產品、測試與診斷程式；依賴環境應已就緒，允許原始碼有未提交改動 |
| `test` | 預設執行 Rust workspace 測試、qmllint 和 CTest；要求已完成同配置構建，允許未提交改動 |
| `package` | 部署執行時、收集許可與來源材料、執行部署審計並生成所選工件；要求乾淨原始碼、同提交構建與測試收據及打包工具 |
| `verify` | 核驗已有執行時或包，不重新構建，不安裝產品、不改預設檔案關聯；依賴待核驗工件與平臺驗證環境 |

預檢報告區分必需缺失、版本不符與當前階段不需要的工具，不用 Docker 或 Inno Setup 缺失阻止普通開發構建。打包預檢會實際檢查 Rust/Cargo 呼叫與鎖定依賴後設資料，避免到許可收集階段才發現 PATH 或依賴快取缺失。失敗的命令和驗證限制保留在本地／CI 輸出中，不把部分完成寫成透過。

首次準備固定 Qt 與 Rust 工具鏈：

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt --qt-root build/toolchain/Qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

`--install-rust` 使用已安裝的 rustup 準備倉庫鎖定的工具鏈，不替代 rustup 安裝。`--install-qt` 顯式下載固定 Qt SDK；預設安裝到 `build/toolchain/Qt`，已有 SDK 時無需重新下載，直接將其路徑交給每個階段：

```sh
python3 scripts/desktop.py doctor --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
python3 scripts/desktop.py build --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
```

完成構建後，顯式準備完整 `native_backend_tests` 批次使用的公共測試輸入。CAB、ISO/UDF 和基礎 RAR 夾具位於 `tests/fixtures/cab-iso/` 與 `tests/fixtures/native-archives/`，隨專案原始碼保留；7z 測試使用剛構建的 Rust worker 建立自己的包。這一 runtime 批次不要求官方 7-Zip／UnRAR 工具或掛載映像，Foundation 的其他互操作參照另按[原生後端契約](native-codecs.md)執行。

首次獲取固定公共 RAR PPM、舊密碼等輸入時，顯式執行：

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs
```

已有經大小和 SHA-256 核對的固定快取時，顯式採用離線模式，替換快取路徑並保留清單規定的 `junrar/` 子目錄佈局：

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs --archives /path/to/fixed-rar-cache --offline
```

準備器的固定來源和通知見 `scripts/rar-rust-fixtures.json` 與其公共相容清單，輸出 `sources.json.environment` 使用原始碼根目錄相對路徑。當前對應原始碼交付不包含這七個公共快取檔案；Cargo 的 `--offline` 不會補齊它們。準備器不會匯出 shell 環境，完整 `test` 前按下面的明確交接啟動子程序：

```sh
python3 -c "
import json, os, pathlib, subprocess, sys
root = pathlib.Path.cwd()
worker = pathlib.Path(sys.argv[1]).resolve(strict=True)
prepared = json.loads((root/'build/native-test-inputs/sources.json').read_text(encoding='utf-8'))
environment = os.environ.copy()
environment.update({name: str(root/value) for name, value in prepared['environment'].items()})
environment.update(XDZIP_NATIVE_CODEC=str(worker), XDZIP_NATIVE_CODEC_WORKER=str(worker), XDZIP_CAB_BACKEND='rust')
environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'
subprocess.run([sys.executable, 'scripts/desktop.py', 'test', *sys.argv[2:]], env=environment, check=True)
" build/desktop/tools/native-codec/xdzip-native-codec
```

Windows 為 worker 路徑加 `.exe`。使用自定義 SDK、build-dir 或 tools-dir 時，將同一組引數附在 worker 路徑後，例如 `--qt-prefix /path/to/Qt/6.8.2/gcc_64`，並將 worker 路徑調整到實際構建目錄。`XDZIP_CAB_BACKEND=rust` 對應固定 Rust CAB 專項；`password` 是公共 fixture 既有的測試值，只交給測試子程序。該交接保留預設 workspace 測試和完整 ignored 批次，不下載輸入或隱式構建 worker。


test 僅對子程序啟用 offscreen／software，不汙染後續命令的桌面環境。日常修改按變更範圍重新構建並驗證；只驗證 Rust 核心可執行相應 crate 的 `cargo test --locked`，不需要 Qt。`test --skip-rust` 執行 qmllint 與 Qt CTest，但跳過 Rust workspace 和完整原生後端測試，收據將 Rust 記為 `not-run`；需要 Rust 結論時另行執行並記錄實際結果。

瀏覽器測試可直接執行或由現有套件入口分組執行。每個套件程序在專案 `build/test-tmp/` 內建立獨立短路徑臨時根，退出時在測試物件銷燬後、Qt 應用銷燬前恢復 TEMP、TMP、TMPDIR 並清理；預設設定檔案同樣隔離。Unix 按實際 sun_path 容量檢查臨時根加既有 IPC 名稱的位元組長度，路徑過長明確報告前置失敗，需縮短檢出路徑，不轉移到專案外或修改產品 IPC。同組輔助程序繼承父臨時根，保持單例項 IPC 的目錄身份，使用私有前置探針並在父程序清理前結束；不同分組不共享。語言重啟及開啟方式子程序顯式傳入的測試配置繼續共享，不改用新的空配置。執行前實際驗證子檔案建立、鎖、暫存、原子提交與設定讀取；失敗以 `Fixture precondition` 和非零退出碼報告，不跳過測試或放寬視窗隱藏等斷言。目錄存在及 QTemporaryDir::isValid 不單獨證明可寫，也不透過調整系統許可權使夾具透過。

測試中的配置路徑使用 `tests/test_environment` 作用域儲存和恢復，Windows 保持寬字元值，並在系統環境塊中區分未設定和顯式空值；Unix 保留原環境位元組。MSVC CRT 的設定 API 會刪除空值，原生環境塊另行恢復，因此空值的存在性使用 helper 的 processValue 或 QProcessEnvironment 檢查，不以 Qt 的 CRT 空值／null 區別作保證。該 helper 僅限序列測試主執行緒，在相關引擎／工作執行緒建立前設定，在銷燬／join 後恢復，不用於產品程式碼或併發環境修改。ASCII、中文及 ANSI 不可表示字元須核對原路徑實際檔案、語言恢復子程序和失敗時視窗／草稿保留。自動化結果仍與真人設定操作分開記錄。


真實產品在所有平臺統一使用 `xdzip` 名稱。預設構建後的入口為：

| 平臺 | 入口 |
| --- | --- |
| macOS | `build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip` |
| Windows | `build/desktop/apps/desktop/xdzip.exe` |
| Linux | `build/desktop/apps/desktop/xdzip` |

元件示例 `xdzip-ui-preview` 不用於真實檔案驗收。更換 build-dir 後相應調整路徑；版本、渠道、提交與 dirty 狀態由實際構建身份記錄。應用模組與低層 CMake 除錯見[Qt 開發應用](../../../apps/desktop/README.md)。

<a name="xdzip-界面字体生成"></a>

## 介面字型生成

`prepare` 在共享工具環境安裝固定版本的 FontTools/Brotli，並快取經過雜湊驗證的上游字型。每次桌面構建自動收集支援的語言目錄和 QML/C++ 固定文字，在 `<构建目录>/generated/fonts/` 生成字型子集。增加文字不需要提交字型二進位制。準備完成後可以離線構建，源快取缺失或損壞時明確失敗。直接使用 CMake 時，透過 `Python3_EXECUTABLE` 選擇同一工具環境。

僅進行 HTML 預覽、離線匯出、匯入現有源字型和字型檢查時，參閱[共享字型指南](../../../design/assets/fonts/README.md)。打包檢查對應產品構建的字型清單，不重新生成字型。固定介面文案和使用者資料系統字型的既有分工保持不變。

<a name="xdzip-打包与验证"></a>

## 打包與驗證

test 與 package 會檢查構建收據、原始碼指紋、配置、SDK 身份及實際產品檔案；修改原始碼或切換配置後先重新 build。打包前提交待交付原始碼，再從該乾淨提交構建和測試。package 不透過忽略髒狀態、修改產物身份或沿用舊報告來繞過檢查。許可材料、Rust 後設資料與實際 CLI 版本必須對應這次原始碼。已有打包目錄的使用與覆蓋限制由各平臺底層工具檢查，重建候選時優先使用新的 output-dir，避免混入舊檔案。

Windows／Linux 先準備打包工具，再執行統一階段：

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py build
python3 scripts/desktop.py test
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

`prepare --packaging` 準備隔離 Python 校驗依賴、鎖定許可正文和 schema 快取，以及平臺打包材料；Windows 包括固定 CRT 和 Inno Setup。系統工具仍由開發者預先安裝。`package --kind runtime` 只准備部署執行目錄及其審計材料；隨後可用同一 output-dir 執行 `package --kind installer`，從已核驗執行目錄生成安裝器，Linux 同時生成保留執行許可權的 tar.gz。`--kind all` 為預設值，在一次命令內依次完成這兩部分。Windows 打包使用 Release 或 RelWithDebInfo，不部署 Debug Qt 庫；Windows 提供部署目錄和當前使用者安裝器，Linux 提供執行目錄、tar.gz 和 .deb。Windows 核驗需要同配置的瀏覽測試程式和 Qt Test／offscreen 測試輸入，Linux 核驗需要構建測試生成的 m2-flat.zip；測試檔案不會進入產品目錄。verify 的 `--kind runtime` 核驗執行時，預設 `--kind all` 核驗已選交付工件；普通核驗不執行安裝、升級／回退或使用者檔案關聯變更。

macOS DMG 須單獨選擇允許註冊文件候選的主構建，避免普通開發／測試副本進入“開啟方式”：

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

普通 build 的 macOS handler rank 預設為 `None`，不更改使用者預設關聯；僅長期保留的主應用或 DMG 輸入選擇 `Alternate`。macOS 本地 package 只支援 `all`，生成 arm64 開發 DMG 並執行必要 ad-hoc 完整性簽名，verify 使用重定位後的 Cocoa／Metal 及真實解壓檢查。普通 CI 禁止執行 Mac package，不部署、簽名、壓縮或上傳應用包；專用 Release 例外見[釋出協議](release-protocol.md)。完整限制與副本註冊維護見[Mac 部署指南](macos-deployment.md)。

部署佈局和底層工具分別見[Linux 部署](linux-deployment.md)、[Mac DMG](macos-deployment.md)及[安裝與檔案關聯](installers-associations.md)。Windows/Linux 安裝器未正式簽名，macOS 未做 Developer ID 簽名或公證。安裝和執行環境問題使用實際產品與對應平臺驗證，不以 SDK 下可執行代替獨立包核驗。

<a name="xdzip-与-ci-的关系"></a>

## 與 CI 的關係

提交或推送前，先對擬提交的內容執行與變更匹配的本地構建或驗證。文件改動執行 `python3 scripts/check-docs.py` 並檢查涉及的生成輸出；Rust 改動至少檢查格式並執行受影響 crate 的構建或測試。涉及隔離格式後端的測試按上文準備實際工作程序和所需夾具。C++、橋接與 QML 改動使用 `python3 scripts/desktop.py build` 構建受影響產品，再執行相關測試；無需為每次改動執行全部格式互操作或重複十次完整瀏覽器套件。

新增或修改 QML 視窗時，執行 `python3 scripts/check-window-contracts.py --self-test`、`python3 scripts/check-operation-components.py` 和受影響的雙語多字號佈局迴歸。已完成構建後，`python3 scripts/desktop.py test --skip-rust` 可執行完整 Qt 元件與 CTest；也可在配置 Qt SDK 路徑和離屏環境後使用 `ctest --test-dir build/desktop -R qt_task_components --output-on-failure` 等現有入口執行相關用例。單選、複選控制元件變化還要執行 `accessibleRadioActionsCommitSelection`、`accessibleCheckboxActionsCommitIntent` 等與實際改動相符的 QAccessibleActionInterface 迴歸。

[日常 CI](../../../.github/workflows/docs.yml)只執行輕量靜態檢查，不代替本地產品構建。[Foundation](../../../.github/workflows/foundation.yml) 和 [Qt 工作流](../../../.github/workflows/qt-components.yml)僅按需手動執行，Qt 工作流透過同一 desktop.py 執行構建、Qt 測試以及 Windows/Linux 打包和執行時核驗。[ci-desktop.ps1](../../../scripts/ci-desktop.ps1)保留按需執行的重複、視覺、互操作和安裝相容檢查，不作為日常本地驗證的必選入口。

歷史完整產品、真實安裝升級／回退及關聯生命週期只在一次性 Windows/Linux CI 主機或隔離容器執行，不因統一入口就開放為普通本地 verify 的預設行為。按需 Qt 工作流的 macOS 作業使用 SDK 模式部署審計和 Cocoa 啟動／檔案開啟檢查，僅儲存日誌和 JSON。完整檢查與證據邊界見[CI 策略](../../../developers/engineering/ci-policy.md)。

<a name="xdzip-文件与任务边界"></a>

## 檔案與任務邊界

開啟只瀏覽，不自動解壓。普通同名檔案支援詢問、跳過、改名及明確替換；內容校驗和同步後才整檔案釋出。提交前取消或寫入失敗保留舊檔案，早前已提交的解壓項繼續保留。關閉任務視窗僅隱藏詳情，退出由活動任務決策和非同步 shutdown 屏障處理。

ZIP 在受限 POSIX 條件下可恢復相對符號連結和可執行位，不恢復特權位、擴充套件屬性或原始修改時間，不代表應用包安裝或簽名驗證。來源與目標必須滿足[安全解壓契約](zip-safe-extraction.md)。崩潰暫存只提示並保留供檢查，不自動恢復或刪除未知內容。

<a name="xdzip-microsoft-store-msix"></a>

## Microsoft Store MSIX

商店正式構建與提交流程見[維護者手冊](../../../developers/market/windows/build.md)。

<a name="xdzip-windows-msix-实验构建"></a>

## Windows MSIX 實驗構建

MSIX 複用同一提交的 Windows Release 桌面構建、測試與已審計執行目錄。先按本文完成 `prepare --packaging`、`build`、`test`，再使用相同引數執行：

```powershell
python scripts/desktop.py package --kind runtime
python scripts/desktop.py package --kind msix --msix-name xdzip.Development --msix-publisher 'CN=xdzip development' --msix-publisher-display-name xdzip
python scripts/desktop.py verify --kind msix
```

身份示例僅供本地開發，正式商店身份的包名稱、釋出者和釋出者顯示名稱必須與 Partner Center 提供的值一致。輸出位於 `build/package/xdzip-windows-x64.msix`，僅是未簽名實驗包，不可直接視為可分發的側載包或已透過商店認證。正常側載需要與釋出者匹配、裝置信任的簽名。Windows 11 的受限未簽名開發測試模式另要求釋出者包含指定 OID，含可執行檔案時通常還需要管理員許可權；上面的示例身份不滿足該模式。指令碼不建立證書、不簽名、不安裝、不提交商店。包的技術最低系統為 Windows 10 22H2 x64，不構成新增平臺支援承諾。版本 `1.0.<产品 build>.0` 與顯示版本分離，必須在 build 用盡前重新確定商店版本策略。

構建要求乾淨來源、匹配的構建與測試回執、已審計的完整執行目錄，以及 Windows SDK 的 MakeAppx 和 MakePRI。指令碼複核執行目錄及其更新、檔案關聯和選單能力標記，在獨立暫存生成清單和圖示資源，打包後逐檔案核對內容並記錄摘要；不會覆蓋已有包或暫存。包清單宣告格式登錄檔中的既有擴充套件、full-trust 桌面入口及對應渠道的選單類。

應用圖示包含縮放變體、精確目標尺寸，以及預設、深色背景無底板、淺色背景無底板三種資源形式。MakePRI 將這些資源編入 `resources.pri`，供 Windows 選擇對應尺寸，避免自動新增系統底板。檔案關聯單獨宣告格式圖示，逐位元組複用現有 Windows ICO 中的 16–256 畫素 PNG，不改變圖案、可見尺寸、邊距或位置。資源引數見 [msix-icons.json](../../../specs/visual/msix-icons.json)。這些資源隨安裝包交付，與 Partner Center 的商店一覽徽標分開管理。`verify --kind msix` 會按新圖示回執核對實際包中的尺寸、變體集合、格式圖片及資源索引。已安裝使用者需要透過更新後的程式包獲得這些修改。

示例使用預設的 `--msix-channel sideload`，僅登記 Windows 11 傳統選單。商店上傳時使用 `package --kind msix --msix-channel store` 及 Partner Center 中的準確身份，同時宣告現代、傳統選單；現代入口在包實際獲得 Store 系統簽名前保持隱藏。渠道標記在打包時加入，無須因此重編已審計執行目錄；兩種輸出須使用不同目錄，參見[選單渠道策略](shell-actions.md#windows-package-boundary)。許可、真實安裝、檔案關聯、解除安裝、商店簽名及目標系統實際驗收獨立於打包成功。


MSIX 清單宣告共用語言清單中的十種語言，與應用內建目錄對應，回執同時記錄並校驗語言列表。Qt 自有翻譯資源不會由本專案的 MakeAppx 流程自動轉換為包語言宣告。Partner Center 的“軟體包中支援的語言”以已上傳包為準，修正原始碼後須重新構建並替換草稿中的包才會更新；“其他 Store 一覽語言”可獨立配置，不能代替包清單宣告。參見 [Microsoft 清單語言說明](https://learn.microsoft.com/windows/apps/design/globalizing/manage-language-and-region)。


<a name="xdzip-证据与复验"></a>

## 證據與複驗

按[通用驗證方法](../../../developers/qa/README.md)和[按需桌面驗證](../../../developers/qa/private-test-manual.md)選擇受影響項，不要求日常填寫人工驗收表。構建與測試收據分別儲存在 build-dir 下的 `desktop-build.json`、`desktop-test.json`，prepare、package 和 verify 回執位於 output-dir 的 `<stage>.json`，並關聯底層驗證證據。自動化收據記錄實際階段與來源；離屏／軟體渲染、原生 Cocoa 檢查和已執行的人工專項分別登記，不相互替代。既有驗收在 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)保留，不因重新生成候選清單而重做。

Windows/Linux 工件交付同提交原始碼和許可通知，Actions 儲存 14 天；過期後按身份和鎖定輸入重建。deployment-audit.json、原始碼及檔案級 CycloneDX 清單用於來源核驗，覆蓋仍可能為 incomplete，不能當作許可批准。原始碼重建與 Qt 替換方法見[原始碼交付](source-delivery.md)及[自動驗證契約](../../../developers/engineering/candidate-verification.md)。

所有構建、截圖、量測、報告與包只保留為忽略的本地或 CI 工件。不要把生成的 pending 表、離屏檢查或部分 SBOM 寫成乾淨環境與全部發行門檻透過。

<a name="xdzip-macos-更新框架"></a>

## macOS 更新框架

原生構建透過 `prepare-sparkle.py` 準備 `build/toolchain/Sparkle/2.10.0-xdzip-proxy.2`。框架主庫從鎖定上游原始碼和代理補丁編譯，安裝助手、資源及簽名工具保留官方 SDK 來源，詳見[依賴契約](dependencies.md#xdzip-macos-%E6%9B%B4%E6%96%B0%E6%8F%90%E4%BE%9B%E8%80%85)。已有有效構建回執可複用，舊官方 SDK 不能代替帶代理介面的派生框架。CI 可顯式指定臨時 SDK 目錄。

`python3 scripts/prepare-sparkle.py --output build/toolchain/Sparkle/2.10.0-xdzip-proxy.2` 使用現有 Xcode Command Line Tools 或 Xcode 的原生 Clang，不需要另建 Qt SDK。原始 SDK 和原始碼歸檔快取於 `build/source-cache/`，離線時可分別傳入 `--archive` 和 `--source-archive`。構建不執行下載原始碼中的指令碼，不修改系統代理，也不讀取生產簽名金鑰。


`check-macos-proxy.py` 預設只執行自有迴環下載器檢查。`--system-conflicts` 僅供專項工作流中的一次性 hosted Apple Silicon runner 使用，受限助手暫存、恢復並核驗系統代理和固定測試域名的 hosts 設定；禁止在本機新增該引數。
