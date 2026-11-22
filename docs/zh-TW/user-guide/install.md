<a name="xdzip-安装-xdzip-与打开压缩包"></a>

# 安裝 XDZip 與開啟壓縮檔

[English](../../user-guide/install.md) | [简体中文](../../zh-CN/user-guide/install.md) | 繁體中文 | [日本語](../../ja-JP/user-guide/install.md) | [한국어](../../ko-KR/user-guide/install.md) | [Français](../../fr-FR/user-guide/install.md) | [Deutsch](../../de-DE/user-guide/install.md) | [Español](../../es-ES/user-guide/install.md) | [Português (Brasil)](../../pt-BR/user-guide/install.md) | [Русский](../../ru-RU/user-guide/install.md)

從 [GitHub 最新穩定版](https://github.com/xiaodaozhi/xdzip/releases/latest)選擇對應系統的下載包。Windows 使用者也可透過 Microsoft Store 安裝。GitHub 釋出頁列出檔名、版本和 `SHA256SUMS`，可用來核對直接下載的檔案。具體要求見[支援政策](../engineering/support-policy.md)。

| 桌面系統 | 安裝或啟動 |
| --- | --- |
| Windows 11 x64 | 在 Microsoft Store 搜尋 **XDZip**，或執行 GitHub 的 `-setup.exe` 安裝到目前使用者。GitHub 安裝器未簽名。使用便攜 ZIP 時，先解壓整個包，再從解壓目錄執行 `xdzip.exe`。 |
| macOS 14 或更新版本，Apple Silicon | 開啟 DMG，將 `xdzip.app` 拖到“應用程式”。若下載的是 `.app.zip`，先解壓，再將應用移到該位置。從“應用程式”啟動副本。GitHub 版採用 ad-hoc 完整性簽名，未經過 Apple 公證。系統出現提示時，先核對釋出來源和校驗和。 |
| Ubuntu 24.04 LTS x64，X11，glibc 2.39 或更新版本 | 用包管理器安裝 `.deb`。使用便攜 `.tar.gz` 時，完整解壓後在執行目錄執行 `./xdzip`，保留啟動器及配套執行檔案。 |

桌面支援範圍不涵蓋 Windows ARM64、Intel Mac、Linux ARM64 和 Wayland。Linux 獨立 CLI 無需 X11，執行需要系統 OpenSSL 3 和證書。可選用以 `xdzip-cli-` 開頭的下載包，並按[命令列入門](../integration/README.md)啟動。

使用 DEB 套件安裝後，可在 Ubuntu 應用程式中心的「已安裝」清單找到並移除 XDZip。

<a name="xdzip-打开压缩包"></a>

## 開啟壓縮檔

在 XDZip 中選擇“開啟壓縮檔”，或把壓縮檔拖到歡迎頁。已註冊檔案型別也可從檔案管理器的“開啟方式”選擇 XDZip。要設為某種型別的預設應用，請在系統設定中挑選。Windows 商店版可從檔案關聯頁選擇“在 Windows 中設定預設應用”。預設應用與右鍵壓縮解壓動作分別設定。詳見[從檔案管理器解壓](extract.md#xdzip-%E4%BB%8E%E6%96%87%E4%BB%B6%E7%AE%A1%E7%90%86%E5%99%A8%E8%A7%A3%E5%8E%8B)與[安裝和檔案關聯說明](../engineering/installers-associations.md)。

如果 macOS 仍將 XDZip 視為某種檔案類型的唯一開啟方式，關閉該類型的關聯開關時會顯示提示，系統仍會使用 XDZip。

接下來可以[瀏覽和預覽檔案](browse.md)。
