<a name="xdzip-设置与更新"></a>

# 設定與更新

[English](../../user-guide/settings-and-updates.md) | [简体中文](../../zh-CN/user-guide/settings-and-updates.md) | 繁體中文（台灣） | [日本語](../../ja-JP/user-guide/settings-and-updates.md) | [한국어](../../ko-KR/user-guide/settings-and-updates.md) | [Français](../../fr-FR/user-guide/settings-and-updates.md) | [Deutsch](../../de-DE/user-guide/settings-and-updates.md) | [Español](../../es-ES/user-guide/settings-and-updates.md) | [Português (Brasil)](../../pt-BR/user-guide/settings-and-updates.md) | [Русский](../../ru-RU/user-guide/settings-and-updates.md)

<a name="xdzip-调整工作区"></a>

## 調整工作區

從應用選單或工具欄開啟“設定”。在對應設定中選擇語言、主題、字號及介面密度。自動語言在支援時跟隨系統。還可按平臺和版本設定用標籤頁或視窗開啟壓縮檔、是否記錄最近檔案，以及托盤行為。設定隨選擇即時生效，儲存失敗時請檢視對應頁面的提示。[設定行為說明](../ux/settings-localization.md)

檔案關聯由作業系統管理，右鍵選單動作單獨配置。macOS 的 Finder 擴充套件由系統管理啟用狀態，已登記時還可使用 Finder 服務。[檔案管理器操作](../engineering/shell-actions.md)

<a name="xdzip-更新应用"></a>

## 更新應用

- **Microsoft Store 版。** 透過 Microsoft Store 更新。應用以 MSIX 包身份執行時，“關於”頁沒有 GitHub 更新入口。
- **GitHub 安裝版。** 開啟“關於 xdzip”，主動檢查更新。應用內安裝需要符合條件的 Windows 目前使用者安裝、macOS“應用程式”中的副本或 Ubuntu DEB 安裝，並且釋出包須透過校驗。Windows 可能需要在安裝嚮導中確認，Ubuntu 使用系統授權。若安裝正在等待活動任務結束，可等它們完成，或取消本次更新等待。
- **便攜版或開發構建。** 下載匹配的包。Windows ZIP 和 Linux TAR.GZ 請完整解壓到新目錄。macOS 請先退出舊版，再替換“應用程式”中的副本。詳見[安裝步驟](install.md)。

GitHub 更新由使用者主動檢查，並按平臺支援範圍使用代理設定。如果應用只提供手動下載，請透過對應的釋出連結下載並核對檔案。下載完成後仍須安裝。更新後重新開啟“關於 xdzip”，確認目前執行的版本。Microsoft Store 與 GitHub 獨立釋出。[更新行為與恢復](../engineering/software-update.md)
