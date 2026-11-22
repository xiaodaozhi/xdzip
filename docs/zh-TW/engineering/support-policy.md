<a name="xdzip-系统支持与维护政策"></a>

# 系統支援與維護政策

[English](../../engineering/support-policy.md) | [简体中文](../../zh-CN/engineering/support-policy.md) | 繁體中文 | [日本語](../../ja-JP/engineering/support-policy.md) | [한국어](../../ko-KR/engineering/support-policy.md) | [Français](../../fr-FR/engineering/support-policy.md) | [Deutsch](../../de-DE/engineering/support-policy.md) | [Español](../../es-ES/engineering/support-policy.md) | [Português (Brasil)](../../pt-BR/engineering/support-policy.md) | [Русский](../../ru-RU/engineering/support-policy.md)

本頁列出 XDZip 目前支援的系統與維護方式。請從[最新穩定版](https://github.com/xiaodaozhi/xdzip/releases/latest)下載對應平台的安裝套件，並查閱版本說明中的已知限制。

<a name="xdzip-首批支持范围"></a>

## 首批支援範圍

| 平臺 | 首批支援範圍 | 目前工程依據與邊界 |
| --- | --- | --- |
| macOS | macOS 14 及以上，Apple Silicon arm64 | 部署目標為 14.0。GitHub 為 Apple Silicon 提供 DMG 與應用程式 ZIP，兩者包含同一份採用 ad-hoc 完整性簽署的應用程式。 |
| Windows | 仍受 Microsoft 維護的 Windows 11，x86_64 | MSVC 構建、目前使用者安裝器、實際安裝／升級／回退／解除安裝 CI；Server CI 不替代對所有 Windows 11 版本的支援宣告，不提供 ARM64 包 |
| Linux | Ubuntu 24.04 LTS x86_64，X11，glibc 2.39 及以上 | 目前 .deb 和無 Qt SDK 的隔離執行驗證；不宣稱任意發行版、較舊 glibc、Wayland 或 ARM64 可用 |

Mac 最低版本取目前專案部署目標，不反向採用舊設計中未驗證的 13。Windows 10 的常規支援已於 2025-10-14 結束，不列為首批承諾範圍；LTSC／ESU 不自動納入。依據：[Microsoft 生命週期說明](https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support)。Qt 自身支援矩陣只是依賴邊界，不等同於 xdzip 實際驗證，見 [Qt 6.8 支援平臺](https://doc.qt.io/qt-6.8/supported-platforms.html)。

<a name="xdzip-维护方式"></a>

## 維護方式

個人專案採取盡力維護，不設付費服務、固定響應 SLA 或未經執行的安全承諾。優先維護最新公開穩定版，開發預覽可能發生相容變化；新版本說明列出已知問題和回退方式，遇到安全問題可撤回受影響下載。桌面應用支援使用者從關於頁主動檢查並交接更新，不增加背景輪詢或靜默更新器。目前原始碼中的 CLI 也提供主動更新命令，其適用身份與已釋出附件的區別見[命令列契約](command-line.md)。

候選仍固定 Qt 6.8.2 和現有 Rust/Cargo 鎖定版本。凍結用於複驗，不等於宣佈舊補丁無漏洞或永久有免費 LTS 更新。維護者在每次公開發布前檢查 Qt 官方安全通告及可取得的開源更新，優先評估受維護的公開版本；若目前依賴有影響本專案且未修復的安全問題，暫停相關釋出。升級 Qt 必須複驗原始碼、模組許可、Windows 私有介面、動態替換、三平臺構建及實際選單／視窗行為，不購買商業維護作為前置條件。

<a name="xdzip-批准范围与变更"></a>

## 批准範圍與變更

新增架構、系統或發行管道時，先實作並驗證支援，再更新本頁。平台範圍的依據見 [ADR-011](../design/architecture-decisions.md#xdzip-adr-011%E9%A6%96%E6%89%B9%E6%94%AF%E6%8C%81%E8%8C%83%E5%9B%B4%E4%B8%8E%E4%B8%AA%E4%BA%BA%E9%A1%B9%E7%9B%AE%E7%BB%B4%E6%8A%A4%E6%94%BF%E7%AD%96)。

漏洞通報請參閱[安全政策](../../../SECURITY.zh-CN.md)。
