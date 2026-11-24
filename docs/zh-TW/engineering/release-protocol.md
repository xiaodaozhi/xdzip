<a name="xdzip-github-release-协议-v1"></a>

# GitHub Release 協議 v1

[English](../../engineering/release-protocol.md) | [简体中文](../../zh-CN/engineering/release-protocol.md) | 繁體中文 | [日本語](../../ja-JP/engineering/release-protocol.md) | [한국어](../../ko-KR/engineering/release-protocol.md) | [Français](../../fr-FR/engineering/release-protocol.md) | [Deutsch](../../de-DE/engineering/release-protocol.md) | [Español](../../es-ES/engineering/release-protocol.md) | [Português (Brasil)](../../pt-BR/engineering/release-protocol.md) | [Русский](../../ru-RU/engineering/release-protocol.md)

本流程負責 GitHub 下載包。[Microsoft Store MSIX 工作流](../../../.github/workflows/windows-store-msix.yml)可基於同一版本原始碼生成商店包，商店認證和釋出在微軟合作夥伴中心完成。

專用 [Release 工作流](../../../.github/workflows/release.yml)透過 `workflow_dispatch` 手動執行。正式發行從 `main` 輸入已有版本標籤並凍結其提交。四個原生作業分別配置、建置、測試和打包一次。macOS 首次建置就使用發行所需的檔案關聯優先順序，部署 Qt 並進行 ad-hoc 完整性簽名，從同一應用生成 ZIP 和 DMG。普通 macOS Qt CI 使用 SDK 驗證。Developer ID、Apple 公證和 Windows 發行者證書不在目前發行政策範圍內。

Intel 建置另行產生 `macos-x86_64` 應用 ZIP、DMG 與 CLI 封存檔。Intel 桌面與 CLI 分別使用 `release-intel.json` 和 `cli-release-intel.json`，其簽章及 `appcast-intel.xml` 均獨立驗證。既有 ARM64 資訊清單和 feed 保持原格式，以相容舊版用戶端。發行前兩種架構皆須通過原生套件驗證。

產品原始碼取自標籤指向的提交。草稿、公開發行及公開下載複核皆從觸發工作流程時的 `main` 提交讀取同一份已審閱的發行說明。

完整標籤等於 `v` 加 VERSION.json 派生的語義版本。`prerelease_number` 是可選正整數，未設定時仍相容歷史建置號；alpha.1 與 build 14 是不同身份。Windows 數值資源和 macOS CFBundleVersion 仍使用單調遞增建置號。公開同一版本的資產不可替換，修訂必須使用新版本；草稿可重跑。

發行說明是持久且可審閱的版本輸入，預設讀取 `release-notes/<完整版本>.md`；CLI 也可透過 `--notes-file` 指定已審閱檔案。檔案含一個 `<!-- xdzip-release {"version":"0.1.0-alpha.6","build":19} -->` 中繼資料行及對應版本標題，版本與建置號必須匹配 VERSION.json。先執行 `python3 scripts/publish-release.py --check-notes`，該檢查不聯絡 GitHub，專用工作流在建置前執行同樣的預檢。更新版本須新建對應說明，不自動借用舊版功能列表或驗收狀態。

說明按平台、實際提交和證據記錄中英文驗收邊界，已有人工驗收與未執行項分別保留；階段進展仍由 issue 管理。初建草稿和公開使用同一份說明快照。草稿重跑若發現其正文與已審閱檔案不同，會在上傳／刪除資產前停止，要求顯式對齊輸入；公開版本繼續拒絕重跑替換資產。將待發布的變更記在[變更日誌的未發布章節](../../../CHANGELOG.zh-TW.md#unreleased)，公開後再移到對應版本。同步更新所有語言的變更日誌。

釋出器在改動草稿資產前回讀 GitHub 標籤、草稿狀態和通道標記。stable 通道必須保持非預釋出狀態並成為 GitHub Latest；alpha、beta、rc 通道必須保持預釋出狀態且不能取代 Latest。公開操作後及公開附件複核結束後再次核對相應狀態，全部符合才報告成功。公開後的核對失敗時，版本可能已經對外可見，指令碼會停止並要求所有者處理，不自動撤回、修改或替換已公開版本及資產。

正式發行前完成本地預檢並審閱對應版本說明。包驗收在同一原生平台獨立執行，核對交接構件的倉庫、執行編號、提交、平台、架構和歸檔 SHA-256。Windows 和 Linux 各驗證一次已部署執行時，並執行實際安裝器生命週期檢查。macOS 核驗實際交付的 DMG 和應用 ZIP。按需候選執行使用相同作業，組裝後重新下載 Actions 構件並核對校驗和。候選模式使用只讀倉庫權限，無需標籤或發行簽名金鑰。

發行契約作業先建置同一提交的隔離格式工作程序，再透過 `XDZIP_NATIVE_CODEC` 與 `XDZIP_NATIVE_CODEC_WORKER` 執行 Rust 工作區測試。三平台原生作業分別驗證實際應用、部署和安裝生命週期。macOS 開發版檔案關聯測試使用已編譯應用的臨時副本，並為副本設定開發版關聯中繼資料。原生啟動和打包使用帶有正式關聯優先順序的發行應用。

Windows 每次部署核驗使用獨立的臨時測試副本，保留此前日誌；重複核驗不復用或刪除已有目錄，產品執行目錄不包含測試 DLL。

釋出固定附帶 `release.json` 與 `SHA256SUMS`。清單 `schema_version=1`，根欄位為 product、version、channel、build、tag、commit（完整 40 位 SHA）、repository、assets。每個資產含 filename、platform、architecture、package_type、minimum_system、bytes、sha256、url，URL 固定到標籤下載路徑。

| 平台、架構 | 包型別 | 最低系統 |
| --- | --- | --- |
| windows、x86_64 | zip、exe | Windows 11 受維護版本 |
| macos、arm64 | app.zip、dmg | macOS 14.0 |
| linux、x86_64 | tar.gz、deb | Ubuntu 24.04、glibc 2.39、X11 |

六類包必須齊備，清單校驗拒絕缺包、重複目標、不一致通道、可變下載地址、錯誤位元組數或雜湊。發行附件包含平台通知、許可、SBOM 和部署／驗證材料，不包含對應原始碼壓縮檔。Windows 使用 app-local VC++ 執行庫，不隨包分發 Windows SDK 圖形編譯庫。

自 2026-11-05 起，所有者決定從 GitHub Releases 和 Actions 構件中撤下歷史對應原始碼壓縮檔，並停止後續發行流程生成或上傳此類壓縮檔。`scripts/prepare-source-delivery.py` 仍可用於本地審查；釋出清單器和釋出器拒絕檔名含 `corresponding-source` 的歸檔。該釋出政策不改變任何元件的適用許可，也不表示對應原始碼、重連結或其他義務已經滿足。發行審查必須針對實際交付繼續處理這些義務；若條款要求的條件沒有其他可行交付方式，應阻止受影響二進位的分發。GitHub 公共倉庫自動生成的標籤原始碼快照不屬於這些 Release 附件。

常規發行不得自行在本地執行原始碼交付收集器；只有使用者針對原始碼收集另行明確要求時才執行，且仍遵守上傳限制。通用機器待複核標記須結合實際元件及具體義務判斷，不能單憑標記認定出現新的發行缺陷。

macOS 的應用 ZIP、DMG 與通知材料必須在同一已部署、簽名的臨時應用仍存活時匯出，隨後才清理暫存 `.app` 和臨時註冊。不能在暫存清理後重新部署另一份應用來補 ZIP。

順序為本地預檢、凍結版本提交、建立標籤、手動啟動一次 Release 工作流、三平台建置與獨立啟動及安裝生命週期核驗、彙總草稿清單、全部齊備後按通道公開 Release、再次下載並核對所有附件。stable 公開為普通 Release 並顯式標記 Latest；alpha、beta、rc 公開為 Pre-release。任何平台失敗時草稿保持未公開。釋出前後均按 SHA256SUMS 複驗。許可來源複核或人工驗收未完成時必須在說明中保留限制，不以自動化測試冒充其實際結論。

更新客戶端分頁獲取 GitHub Releases 列表，跳過草稿，校驗清單版本，按語義版本、使用者允許通道、平台、架構、包型別選擇。不能按釋出時間排序判斷升級，也不能猜測檔名。預釋出不使用排除預釋出的 `/releases/latest`。穩定通道只接收穩定版本，alpha 允許 alpha、beta、rc 和穩定版，beta 不回到 alpha，rc 僅接收 rc 和穩定版；開發版本比較支援 dev。升級的語義版本與單調 build 均須遞增。下載後校驗大小與 SHA-256，任何缺失或不一致拒絕使用。

<a name="xdzip-可信更新附件"></a>

## 可信更新附件

新發行附帶 `release.json.sig`，格式為 schema_version=1、algorithm=ed25519、key_id 和 128 字元小寫十六進位 signature。簽名覆蓋 `release.json` 的精確原始位元組，不能解析後重新序列化再驗籤。客戶端和無 Qt 安裝助手使用各自內建的 `resources/update-trust.json` 公鑰集；不接受下載的清單或使用者請求攜帶的信任根。未知公鑰、重複 key_id、弱公鑰和空信任集均拒絕安裝。

原生 macOS 作業封裝應用程式 ZIP 並驗證交付套件。Linux 的 `sign` 作業取得已驗證的交付位元組後，執行 `scripts/sign-macos-update.py`，由 `xdzip-update-tool sign-archive` 產生 `macos-update.json`，記錄 schema_version、version、build、filename、bytes、sha256 與 Sparkle base64 `ed_signature`。彙總階段重新核對歸檔及其簽名，生成只含同一個不可變候選的 `appcast.xml`，清單可選欄位 `macos_appcast_url` 固定指向同一標籤下的該檔案。feed 使用 build 作 Sparkle version、完整語義版本作 shortVersionString、macOS 14.0 作最低系統版本，僅內嵌簡短版本說明。完整發行說明仍作為 GitHub Release 正文，客戶端不讀取外部發布說明。

歸檔和 feed 使用獨立 Sparkle 金鑰，feed 自身也必須簽名。跨平台釋出工具按 [Sparkle 上游簽名格式](https://github.com/sparkle-project/Sparkle/blob/2.x/common_cli/Signing.swift)對完整原始 XML 簽名，並在末尾追加 sparkle-signatures 註釋塊，包含 base64 edSignature 與原始位元組 length；這不是隻給 enclosure 新增簽名。生成後再次獨立驗證，修改任意位元組須重新簽名。Mac 包的 SUPublicEDKey 必須與 `resources/sparkle-update-key.json` 中的公鑰一致。

釋出環境透過 `XDZIP_UPDATE_SIGNING_KEY_HEX`、`XDZIP_UPDATE_KEY_ID` 和 `XDZIP_SPARKLE_SIGNING_KEY_HEX` 提供已有金鑰和公開 key ID。兩份生產公鑰資源由所有者提供，隨客戶端和安裝助手納入版本管理，不填測試公鑰，也不在建置時生成私鑰。所有者先將公開信任根納入客戶端版本，再在可信釋出環境配置匹配私鑰；金鑰輪換先交付新公鑰，再改為新金鑰簽名，不能使舊客戶端失去驗證路徑。未內建這些公鑰的舊客戶端須先手動安裝包含公鑰的版本，不能透過下載的清單增加信任根；舊公開發行不補籤或替換資產。

釋出指令碼先在移除兩份私鑰環境變數的子程序中建置簽名工具，僅已編譯的工具程序接收簽名私鑰；Cargo build script、編譯器和依賴解析不獲得釋出簽名權限。

簽名作業在 `release-signing` 環境中為已驗證的應用歸檔和更新清單簽名。獨立釋出作業也需要該環境審批，公開前核對簽名並重新下載草稿附件。最後由只讀作業下載公開附件，與已簽名構件核對。各作業均使用同一凍結提交及同一次執行生成的構件。

[生產更新驗收](../../../.github/workflows/production-update-acceptance.yml)是另一個僅手動執行、固定 `main` 和 `contents: read` 的入口，使用同一 `release-signing` 環境及其所有者審批，不呼叫 Release 工作流，也不建立標籤、草稿或發行。先在無秘密步驟編譯工具，再在獨立步驟派生已有私鑰對應的公鑰，與本次提交中的兩份公鑰和公開 key ID 嚴格比較；條目存在不能代替匹配成功。三平台私有舊版 200 與新版 201 保留生產公鑰，建置不接收秘密。簽名步驟僅接受本次原始碼提交、私有產品身份及完整雜湊繫結的固定候選，隨後在無秘密程序中執行原生升級、失敗恢復和普通重啟。沒有任意輸入清單、資產、遠端構件或釋出引數。

macOS 在無秘密的工具鏈準備步驟中一次安裝 `rust-toolchain.toml` 宣告的 `rustfmt` 和 `clippy`，再執行歸檔准入測試。測試金鑰更新迴歸也覆蓋該準備和准入流程；它不使用或驗證生產私鑰。

生產驗收的安裝包、原始清單、簽名、appcast 和應用僅留在一次性 runner 內，不上傳；有界收集器只上傳明確允許的日誌、狀態、身份及雜湊記錄，並拒絕簽名附件誤入結果槽。生產金鑰匹配、簽名驗證、實際安裝及人工桌面驗收分別記錄。macOS 使用原生 Apple Silicon 和實際 `SUPublicEDKey` 核對；Windows Server 的夾具仍不代替 Windows 11 人工驗收。此流程不會讀取 GitHub Secret 的值到本地，也不授權釋出。

`prepare-release.py --require-update-signature` 在缺少任一金鑰、簽名或原生歸檔中繼資料時停止；`publish-release.py` 獨立複驗清單、同候選 feed 與歸檔後才上傳。簽名、appcast 和狀態附件均進入完整 SHA256SUMS。已有公開版本不補傳或替換這些新附件。







<a name="xdzip-独立-cli-附件"></a>

## 獨立 CLI 附件

從 v0.2.0 起，完整發行額外提供 `xdzip-cli-<version>-windows-x86_64.zip`、`xdzip-cli-<version>-macos-arm64.tar.gz`、`xdzip-cli-<version>-linux-x86_64.tar.gz`。三者由同提交的原生 CLI 建置與驗證產生，保留 worker、實際執行依賴與通知，身份須為乾淨 Release。普通 CI 不因新增 CLI 而部署或上傳 Mac 應用包。

CLI 使用 `cli-release.json` 與 `cli-release.json.sig`。清單 schema_version=1、product=xdzip、component=cli，version、channel、build、tag、commit、repository 與桌面清單完全一致。assets 只包含上述三個目標，欄位與 desktop asset 相同；Mac 包型為 tar.gz，Linux 最低系統不含 display_server，其他系統、架構和大小／摘要要求不變。固定釋出者仍為 xiaodaozhi/xdzip。

桌面 release.json 繼續恰好六類包，不加入 component 或 CLI 條目。舊客戶端忽略新增獨立附件。CLI 清單簽名複用內建 Ed25519 信任根，簽署和獨立驗證使用 sign-cli／verify-cli，覆蓋精確原始位元組。完整彙總和釋出在簽名、身份、三個 CLI 包及全部校驗和齊備後才能公開。已公開舊版本不補傳、改寫或替換附件。
