<a name="xdzip-windows-开发工具与运行库许可"></a>

# Windows 開發工具與執行庫許可

[English](../../engineering/windows-licensing.md) | [简体中文](../../zh-CN/engineering/windows-licensing.md) | 繁體中文（台灣）

本文定義 Windows 構建和再分發的適用路徑，與[原生執行庫材料](native-runtime-materials.md)、[開發構建入口](developer-build.md)和[安裝契約](installers-associations.md)共同使用。具體環境、驗證提交、材料雜湊和驗收結論登記在 issue 與本地／CI 工件，本文不儲存某臺機器的許可證明。

<a name="xdzip-免费-community-路径与本地选择"></a>

## 免費 Community 路徑與本地選擇

專案原創內容以 Apache-2.0 公開，屬於 OSI 批准的開源許可。Visual Studio Community 2022 的個人條款允許個人開發自己的應用，組織條款也允許使用者開發和測試以 OSI 批准許可釋出的應用。組織的開源用途與其他商業內部用途必須分別判斷，不能把本專案的適用條件擴大到其他專案。

需要自有 Community 開發環境時，維護者從[微軟官方安裝入口](https://learn.microsoft.com/en-us/visualstudio/install/use-command-line-parameters-to-install-visual-studio?view=vs-2022)取得 Community 2022，閱讀並接受[該版本的原始條款](https://visualstudio.microsoft.com/license-terms/vs2022-ga-community/)，在允許範圍內使用。C++ 構建選擇所需的 `Microsoft.VisualStudio.Component.VC.Tools.x86.x64` 和 Windows SDK，不要求安裝全部工作負載。無需購買 Professional／Enterprise，也沒有另行申請每個專案執行庫分發證書的步驟。

許可複核不要求本機長期安裝大型 IDE。只使用下述 GitHub 標準託管環境構建與釋出時，適用依據是託管 Visual Studio 使用授權及其 REDIST 條款，無需為託管 CI 另裝本地 Community 或購買個人 Enterprise。Community 是維護者需要自有開發環境時的免費選擇，不能把本地安裝狀態當作託管構建合法性的額外門檻。

Community 第 3 節允許在符合條件的構建裝置上使用軟體檔案和 Build Tools；第 4 節授予清單內 Distributable Code 的分發權，並規定功能、下游條款和限制。僅下載 VC_redist、安裝 Build Tools 或附帶一份許可文字，不單獨證明開發工具的適用許可。Build Tools 的無 Visual Studio 許可例外針對第三方開源依賴的構建，不是自行開發所有 C++ 程式的通用例外。使用 VS Code 也不自動取得 MSVC 許可。參見[微軟許可指南](https://www.microsoft.com/licensing/guidance/Visual-Studio)和 [Build Tools 條款](https://visualstudio.microsoft.com/license-terms/vs2022-ga-diagnosticbuildtools/)。

本地應核對 `vswhere` 返回的產品、完整版本、非預覽狀態及 C++ 元件，進入實際例項的開發環境，再檢查 `cl`、`link` 和 SDK。已安裝的 2019 Build Tools 不記為 Community 2022，也不代替目前 MSVC 2022 工具鏈驗證。通用 `desktop.py prepare` 不隱式安裝系統 IDE／SDK，首次安裝使用微軟安裝器，保留既有工具並禁止自動重啟。

選用 2022 是目前發行工具鏈的選擇，不是 Community 免費許可只允許使用 2022。[Qt 6.8 支援表](https://doc.qt.io/qt-6.8/supported-platforms.html)列出 Windows MSVC 2022，本倉庫使用相應 `msvc2022_64` SDK。2019 工具可以保留用於本地開發；與目前 SDK 混用時，不能只憑跨版本 ABI 相容推斷完整支援。[微軟的相容性規則](https://learn.microsoft.com/en-us/cpp/porting/binary-compat-2015-2017?view=msvc-170)還要求最終連結器通常不舊於輸入庫所用工具，並分別約束執行庫和 `/GL`／`/LTCG`。替代配置需要自己的適用許可及驗證，不因此強制給本機安裝完整 2022 IDE。

<a name="xdzip-github-托管构建"></a>

## GitHub 託管構建

專案 Windows 工作流使用 `windows-2022` 標準 GitHub 託管 runner；預裝版本是 Visual Studio Enterprise 2022，不把它標記為 Community。runner-images 維護者已明確[確認](https://github.com/actions/runner-images/issues/12945#issuecomment-3348002951)：預裝 Enterprise 透過 GitHub 與微軟的約定供 GitHub Actions／Azure DevOps 託管 runner 的公開和私有用途使用，使用者無需另備個人 Enterprise 許可。本專案據此使用託管構建環境；該公開說明不替代或聲稱公開了雙方完整協議。

根據 [GitHub 託管 runner 文件](https://docs.github.com/en/actions/concepts/runners/github-hosted-runners)，每次 Windows job 獲得獨立虛擬機器，`Set up job` 指向該次執行的映象與軟體清單。工作流只進行本倉庫的編譯、測試、部署與釋出，符合 [Actions 附加條款](https://docs.github.com/en/site-policy/github-terms/github-terms-for-additional-products-and-features#actions)的用途範圍。公開倉庫的免費計算額度與 Microsoft 元件的再分發權是不同事項。

CI 記錄實際 VS 產品、版本、編譯器相對位置、工具版本及 SDK 版本到 `windows-toolchain.json`，與同提交包及 materials 一併儲存，不包含產品金鑰或開發者私人路徑。它記錄工具身份，不能自動證明許可資格。自託管 runner、從映象複製到本地的工具、其他 CI 服務及一般共享伺服器不繼承上述託管許可，應重新核對 Community／Build Tools 的條款和裝置條件。

<a name="xdzip-microsoft-组件再分发"></a>

## Microsoft 元件再分發

[VS 2022 REDIST 清單](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)明確包含有效許可的 Community 2022。目前使用 [windows-runtime.json](../../../scripts/windows-runtime.json) 固定的正式版 VC_redist x64，原樣提取並逐一核對八個 CRT DLL 和原始執行庫許可，不打包編譯器、偵錯程式或 `debug_nonredist`。開發工具的 Community 免費路徑和託管 CI 的 Enterprise 身份均不將微軟 DLL 重新授權為 Apache-2.0。

新開發包不攜帶 Windows SDK 的 `d3dcompiler_47.dll`、`dxcompiler.dll` 和 `dxil.dll`，也不交付 Mesa 軟體 OpenGL 庫。歷史包中這三份檔案來自顯式 SDK 的 `Redist/D3D/x64`。[SDK REDIST 清單](https://learn.microsoft.com/en-us/legal/windows-sdk/redist)允許在有效 SDK 許可下原樣隨 Classic Windows 應用分發這些檔案，禁止隨 UWP 應用分發；xdzip 是 Classic Windows 桌面應用。歷史包原有 SDK 許可與通知及其分發條件繼續獨立處理，不因未來包移除檔案而改變。

包內 `share/notices/Windows-runtime-NOTICE.txt` 雙語說明各元件許可邊界。Microsoft CRT 原始條款位於 `Microsoft-VC-runtime/license.rtf`。安裝器展示原始 CRT 協議並要求接受，隨後展示元件說明；便攜包使用者同樣需要閱讀並遵守隨包 Microsoft 條款。兩種包都保留原始許可及來源，下游再分發者應自行滿足適用資格和分發條件，不能僅憑專案 Apache-2.0 許可轉發 Microsoft 元件。

展示或附帶 SDK 原文不自動證明外部分發者和終端使用者已同意 SDK 要求的保護條款，也不能代替發行者的有效許可與責任判斷。目前安裝器的接受頁僅針對 CRT；如後續繼續分發 SDK 圖形 DLL，安裝器和便攜包應分別核對適用條款及接受方式，並在對應發行記錄儲存實際證據，歷史版本見 [#135](https://github.com/xiaodaozhi/xdzip/issues/135)。

安裝器編譯前核對 CRT 協議的固定雜湊和雙語說明，materials 附帶同一份原始 CRT 文字和來源記錄。歷史包中的 SDK 原始條款位於 `Windows-SDK-graphics/sdk_license.rtf`。許可補正不改寫舊公開資產或歷史構建身份；修訂包沿用[不可變釋出協議](release-protocol.md)另發版本。
