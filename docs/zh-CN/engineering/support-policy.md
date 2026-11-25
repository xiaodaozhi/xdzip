# 系统支持与维护政策

[English](../../engineering/support-policy.md) | 简体中文 | [繁體中文](../../zh-TW/engineering/support-policy.md) | [日本語](../../ja-JP/engineering/support-policy.md) | [한국어](../../ko-KR/engineering/support-policy.md) | [Français](../../fr-FR/engineering/support-policy.md) | [Deutsch](../../de-DE/engineering/support-policy.md) | [Español](../../es-ES/engineering/support-policy.md) | [Português (Brasil)](../../pt-BR/engineering/support-policy.md) | [Русский](../../ru-RU/engineering/support-policy.md)

本页列出 XDZip 当前支持的系统和维护方式。可在[最新稳定版](https://github.com/xiaodaozhi/xdzip/releases/latest)下载对应平台的安装包，并在版本说明中查看已知限制。

## 首批支持范围

| 平台 | 首批支持范围 | 当前工程依据与边界 |
| --- | --- | --- |
| macOS | macOS 14 及以上，Apple Silicon（arm64），v0.3.2 起支持 Intel（x86_64） | 部署目标为 14.0。GitHub 为每种架构分别提供 DMG 和应用 ZIP，两种包包含同一份采用 ad-hoc 完整性签名的应用。 |
| Windows | 仍受 Microsoft 维护的 Windows 11，x86_64 | MSVC 构建、当前用户安装器、实际安装／升级／回退／卸载 CI；Server CI 不替代对所有 Windows 11 版本的支持声明，不提供 ARM64 包 |
| Linux | Ubuntu 24.04 LTS x86_64，X11，glibc 2.39 及以上 | 当前 .deb 和无 Qt SDK 的隔离运行验证；不宣称任意发行版、较旧 glibc、Wayland 或 ARM64 可用 |

Mac 最低版本取当前项目部署目标，不反向采用旧设计中未验证的 13。Windows 10 的常规支持已于 2025-10-14 结束，不列为首批承诺范围；LTSC／ESU 不自动纳入。依据：[Microsoft 生命周期说明](https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support)。Qt 自身支持矩阵只是依赖边界，不等同于 xdzip 实际验证，见 [Qt 6.8 支持平台](https://doc.qt.io/qt-6.8/supported-platforms.html)。

## 维护方式

个人项目采取尽力维护，不设付费服务、固定响应 SLA 或未经执行的安全承诺。优先维护最新公开稳定版，开发预览可能发生兼容变化；新版本说明列出已知问题和回退方式，遇到安全问题可撤回受影响下载。桌面应用支持用户从关于页主动检查并交接更新，不增加后台轮询或静默更新器。当前源码中的 CLI 也提供主动更新命令，其适用身份与已发布附件的区别见[命令行契约](command-line.md)。

候选仍固定 Qt 6.8.2 和现有 Rust/Cargo 锁定版本。冻结用于复验，不等于宣布旧补丁无漏洞或永久有免费 LTS 更新。维护者在每次公开发布前检查 Qt 官方安全通告及可取得的开源更新，优先评估受维护的公开版本；若当前依赖有影响本项目且未修复的安全问题，暂停相关发布。升级 Qt 必须复验源码、模块许可、Windows 私有接口、动态替换、三平台构建及实际菜单／窗口行为，不购买商业维护作为前置条件。

## 批准范围与变更

新增架构、系统或发行渠道时，先实现并验证支持，再更新本页。平台范围的依据见 [ADR-011](../design/architecture-decisions.md#adr-011首批支持范围与个人项目维护政策)。

漏洞反馈见 [安全政策](../../../SECURITY.zh-CN.md)。
