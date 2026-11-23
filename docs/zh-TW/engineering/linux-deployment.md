<a name="xdzip-linux-独立运行目录与验证"></a>

# Linux 獨立執行目錄與驗證

[English](../../engineering/linux-deployment.md) | [简体中文](../../zh-CN/engineering/linux-deployment.md) | 繁體中文 | [日本語](../../ja-JP/engineering/linux-deployment.md) | [한국어](../../ko-KR/engineering/linux-deployment.md) | [Français](../../fr-FR/engineering/linux-deployment.md) | [Deutsch](../../de-DE/engineering/linux-deployment.md) | [Español](../../es-ES/engineering/linux-deployment.md) | [Português (Brasil)](../../pt-BR/engineering/linux-deployment.md) | [Русский](../../ru-RU/engineering/linux-deployment.md)

目前開發部署基線為 Ubuntu 24.04、x86_64、X11。目標是脫離開發 Qt SDK 執行目前產品，不據此承諾其他發行版、舊 glibc、Wayland 或完整安裝器支援。macOS CI 保持原生編譯、測試與啟動驗證，本地開發 DMG 按[Mac 部署指南](macos-deployment.md)處理，不受本流程影響。

<a name="xdzip-布局和生成"></a>

## 佈局和生成

`scripts/deploy_linux.py` 只接受可信自有 ELF 和明確的 Qt 6.8.2 SDK，在新的 build/ 或 dist/ 目錄生成：

- `xdzip` 為啟動入口，使用目前目錄的庫、外掛和 QML，不繼承開發 SDK 搜尋路徑。
- `bin/xdzip` 為產品，旁邊 `qt.conf` 將 Qt 路徑改為相對佈局。
- `lib/` 為實際匯入的 Qt／SDK 共享庫，`qml/` 保留 Basic 控制元件樣式和必要實現子模組。在收集原生依賴前移除其他控制元件樣式、NativeStyle、Effects、樣式專用對話方塊變體、設計器檔案和 QML 型別後設資料，避免部署其獨佔依賴庫。
- `plugins/` 包含 xcb、離螢幕驗證、圖片、圖示、平台輸入、xcb 圖形整合和必需的 `tls/libqopensslbackend.so`；缺少 OpenSSL 後端時部署失敗。不把 EGLFS、幀緩衝或 Wayland 平台外掛作為已驗證交付。
- `translations/` 保留 Qt 標準控制元件翻譯，`share/notices/` 交付目前通知和原始碼依賴材料。
- `share/deployment.json` 記錄 QML、系統執行依賴、複製的 SDK 庫和邊界，README.md 提供使用方法。

掃描器使用固定 Qt 版本的 [qmlimportscanner](https://doc.qt.io/qt-6.8/qtqml-tooling-qmlimportscanner.html)。Xdzip.Settings／Xdzip.Native 是產品內建註冊模組，不複製開發靜態庫。缺少普通依賴、QML 路徑逃逸 SDK、同名庫衝突或未知外部庫均失敗。原生匯入遞迴收集只複製 SDK 庫，不捎帶宿主 glibc 或圖形驅動；系統依賴按實際 Debian 包歸屬列出。

裁剪先驗證完整複製樹、必要 Basic 外掛、保留元件的強依賴和已知樣式佈局。發現意外連結、未知樣式或對話方塊變體，以及對被省略模組的強依賴時，在刪除前拒絕。預設對話方塊、輸入法、輔助功能、圖片外掛、TLS 和系統翻譯沿用現有部署規則。`share/deployment.json` 記錄移除的 QML 檔案與位元組數。私有 `cli/` 透過固定上下文路徑共享 `bin/xdzip-native-codec`，獨立 CLI 包繼續自包含。

HTTPS 另顯式要求 Ubuntu 的 `libssl3t64` 與 `ca-certificates`，驗證 `libssl.so.3`、`libcrypto.so.3` 的系統包歸屬及非空系統 CA 信任庫。Qt OpenSSL 後端可能在執行時載入這些庫，不能僅憑 ldd 推斷依賴完整。部署清單保留已安裝的二進位包版本、源包與源包版本；安裝器宣告兩項依賴，實際檔案 SBOM 將其記錄為必需 OS 包，不把系統庫打包進應用。

每個 ELF 使用 patchelf 設定相對 `$ORIGIN` RPATH，外掛按自身目錄到 lib/ 的相對距離計算。Qt 的路徑重定向按 [qt.conf 官方規則](https://doc.qt.io/qt-6.8/qt-conf.html)執行，允許整個目錄遷移。庫保持動態連結和可替換，不限制使用者更換相容 Qt。通用 Linux 部署邊界見 [Qt Linux 部署說明](https://doc.qt.io/qt-6.8/linux-deployment.html)。

相對 RPATH 改寫後，部署移除無用的原生符號表，並核對每個 ELF 的 DT_NEEDED 和 RPATH／RUNPATH 項保持不變。`symbol_stripping` 逐檔案記錄工具版本、前後雜湊和位元組數，SDK 輸入及原始構建二進位不變。需要保留符號的診斷副本可對底層部署指令碼傳入 `--keep-symbols`。最終 SDK 歸屬與應用摘要對應轉換後的檔案。這是載荷體積最佳化，不代表已經測得記憶體或吞吐提升。

日常使用[統一構建入口](developer-build.md)分階段生成和核驗；先完成同配置 build／test 與打包準備，再執行：

```sh
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

預設構建目錄為 build/desktop，交付輸出為 build/package，Qt SDK 預設 `build/toolchain/Qt/6.8.2/gcc_64`；其他位置在各階段傳入共用引數。`package --kind runtime` 生成執行目錄與審計材料；隨後在同一 output-dir 執行 `package --kind installer`，生成 tar.gz 與 .deb。預設 `all` 依次完成兩部分。底層 [deploy_linux.py](../../../scripts/deploy_linux.py)、[package_linux.py](../../../scripts/package_linux.py)和 [verify_linux_runtime.py](../../../scripts/verify_linux_runtime.py)保留平台邏輯，需要定位故障時參考各自 `--help`。

部署只在原生 Linux x86_64 執行，不覆蓋既有目錄。打包前預檢 qmlimportscanner、ldd、patchelf、dpkg-query 與安裝器所需的 dpkg-deb；這些構建工具不進入產品。執行容器驗證另外需要 Docker 和可訪問的 daemon，其構建只針對固定 Ubuntu 映像和記錄的 OS 包。Docker 缺失不阻止普通 build／test，verify 則明確失敗，不把未執行隔離檢查寫成透過。普通 verify 不在宿主安裝 .deb 或修改檔案關聯，安裝生命週期仍由一次性 CI 環境執行。

<a name="xdzip-experimental-strict-snap"></a>

## 實驗性 strict Snap

採用 strict confinement 的 `core24` Snap 是獨立的實驗性交付。受限應用程式不會從主機繼承 `$SNAP/usr/share/X11/xkb` 的 XKB 設定。啟動程式將 `XKB_CONFIG_ROOT` 設為該路徑時，Snap 必須納入固定版本 Ubuntu 建置輸入中的完整 XKB 目錄。若缺少資料，Qt Wayland 外掛仍可能載入並連接合成器，卻在建立鍵盤上下文時因 `xkbcommon` 包含路徑錯誤而失敗。載荷須保留 `xkb-data` 版權聲明，並在候選回執中記錄套件版本、確定性目錄雜湊、檔案數與聲明雜湊。解開最終 Snap 後進行驗證，再分別執行離屏與 Wayland 冒煙測試。這些檢查不能取代實際 strict Snap 桌面工作階段中的安裝與互動檢查。

<a name="xdzip-脱离-sdk-的自动化证据"></a>

## 脫離 SDK 的自動化證據

容器基於固定 digest 的 Ubuntu 24.04，安裝實際 OS 執行依賴和 Xvfb／字型／locale 測試支援，不安裝 Qt、Rust 或編譯器，並再次檢查其不存在。容器不掛載開發 SDK 或整個原始碼樹，只掛載只讀產品目錄、單獨的自有十萬條目 ZIP 和驗證指令碼。壓縮檔執行驗證關閉網路、禁止提權，僅臨時配置目錄可寫。

應用被掛載到帶空格的新路徑，在清除開發搜尋變數後，經啟動入口檢查：

1. 實際 `--version` 可執行。
2. X11 下真實 QML 視窗啟動並安全關閉。
3. 原生產品探針完成首幀、十萬條目讀取、活動請求取消與退出，計數和取消結果須符合契約。
4. 宿主清除開發變數後，以 deployed 模式核驗全部打包 ELF；不允許回退 SDK。實際檔案 SBOM 保持同一版本／提交身份並透過官方 schema。
5. 在相同無 SDK 映像的新程序中啟用網路，以隔離的 `XDZIP_SETTINGS_PATH` 執行產品 `--update-probe`，實際請求 GitHub HTTPS。必須報告 OpenSSL 後端可用並完成更新檢查，單獨保留 `.https.json` 與 `.https.log`；網路、證書或 TLS 失敗均阻止驗證透過。

HTTPS 探針分別記錄 OpenSSL 後端、系統 CA 檔案與真實 GitHub 請求結果；真實請求成功不單獨證明簽名驗證或原生更新安裝。簽名解析與 SDK-free 更新閉環由[獨立更新迴歸工作流](../../../.github/workflows/update-regression.yml)驗證，兩類證據不得互相代替。

每次探針嘗試的原始日誌和結構化報告分別保留為 `.https.attempt-N.log` 與 `.https.attempt-N.json`，最終嘗試另保留 `.https.log`。報告缺失、報告無效、超時及程序退出均保留失敗狀態。僅當產品控制器確認匿名 GitHub 釋出列表限流，且狀態、請求階段、響應頭及診斷記錄完整時，才等到服務端指定時間或按有界退避重試。與 macOS 驗收探針一樣，最多嘗試三次，累計等待最多 3,700 秒。期限超過剩餘預算、等待中斷、TLS 或網路失敗及其他錯誤均立即失敗。探針必須完成真實更新請求才能透過。每次等待決定和實際時長保留在 `.https-waits.json`。

再次使用同一輸出路徑前，上一輪報告和日誌會移入同目錄下以報告名命名的 `.history` 目錄，每輪使用獨立子目錄。新一輪中途失敗時，不保留上一輪的成功報告在目前輸出路徑，避免混淆兩次執行；歷史工件仍可單獨核對。

linux-runtime.json 記錄實際事件與可執行檔案 SHA-256；Xvfb／Qt Quick 軟體渲染僅用於執行閉環，不能證明物理 GPU、中文候選視窗、真實螢幕閱讀器或完整桌面安裝透過。

<a name="xdzip-下载交付与剩余门槛"></a>

## 下載交付與剩餘門檻

CI 生成 xdzip-linux-x86_64.tar.gz 並記錄 SHA-256 和可執行檔案雜湊。tar.gz 保留執行許可權，避免 Actions ZIP 解壓丟失入口執行位；歸檔路徑、UID/GID 和時間規範化，不寫入私人目錄。下載後解壓並執行目錄內 `./xdzip`，不能只搬走 bin/xdzip。原始 linux/ 不重複上傳。

該節生成的 `build/package` 執行目錄、tar.gz 和 .deb 是實驗性開發交付，不自動安裝或修改預設關聯；正式 Release 的 Ubuntu tar.gz 和 .deb 由專用 Release 工作流生成，並按[釋出協議](release-protocol.md)提供。平台範圍以[支援政策](support-policy.md)為準，其他發行版與桌面、Wayland 等擴充套件另建 issue。已有桌面驗收保留於 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)，後續變更按受影響項複驗真實輸入法／螢幕閱讀器及乾淨安裝升級解除安裝，不因重建目錄重複要求既有驗收。`v0.1.0` 的歷史元件審查見 [#4](https://github.com/xiaodaozhi/xdzip/issues/4) 和 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)，目錄執行透過不自動完成未來發行門檻。

Ubuntu .deb 的安裝／升級／回退／解除安裝、執行鎖與桌面 MIME 自動驗證見[安裝與關聯](installers-associations.md)，執行容器的透過不替代真實桌面專項與發行許可複核。
