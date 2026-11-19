# 发布操作手册

[English](../../engineering/release-runbook.md) | 简体中文

跨平台稳定发行与预发布使用[发布协议 v1](../../../docs/zh-CN/engineering/release-protocol.md)，按完整标签冻结源码，草稿汇集六类平台包、更新材料、通知、许可、SBOM 和部署／验证材料。发行不生成或附带对应源码压缩包；常规发行及验收不运行源码交付收集器，仅用户针对源码收集另行明确要求时可用于本地审查。正式发布使用 Release 工作流的单次手动运行：同一运行只构建一次三平台 native，汇总步骤直接复用这些工件并公开 Release。不要先运行一轮验证再推送标签触发第二轮构建；本地验证和发布前预检应在创建标签前完成。全部核验通过后，stable 作为普通 Release 公开并标记 Latest；alpha、beta、rc 作为 Pre-release 公开。公开后下载保留的附件复验哈希。未执行的人工验收按候选实际范围说明，不以自动化测试代替其结论。

开发验收、[发行许可复核](../qa/release-review.md)与发行执行分别记录。许可结论绑定实际候选。公开发布前须解决来源、通知及适用源码与替换义务的具体缺口。

## 触发并核验发行

创建版本标签前，用候选的锁定输入执行适用本地检查。在仓库根目录校验派生版本文件及已审阅的发布说明：

```sh
python3 scripts/sync-version.py --check
python3 scripts/publish-release.py --check-notes
```

获得明确发行授权后，推送指向目标源码提交的版本标签。从 `main` 触发 [Release](../../../.github/workflows/release.yml)。将 `vX.Y.Z` 换成完整标签，预发布时保留后缀：

```sh
gh workflow run release.yml --ref main -f tag=vX.Y.Z
```

工作流从标签取产品源码，从触发工作流的提交取已审阅说明。记录两处提交 SHA 和运行 ID。核对本次运行的草稿、原生构建和部署回执，以及发行契约检查。在最终汇总前确认六项桌面资产和三平台独立 CLI 包齐全。受保护的 `release-signing` 环境提供更新签名权限，更新签名和 macOS 的 ad-hoc 包完整性验证分别有各自用途。见[候选材料](release-candidate.md)和[签名契约](../../../docs/zh-CN/engineering/release-protocol.md#可信更新附件)。

公开后核对发布状态、说明、资产清单及重新下载文件的 `SHA256SUMS`。[发布脚本](../../../scripts/publish-release.py)在公开后单独下载复验。已公开版本保留原资产，需要修正内容时使用新版本。[更新日志](../../../CHANGELOG.md)只登记实际公开的版本。

## 候选复核

GitHub 安装包遵循项目的分发政策。Windows 安装器未签名，macOS 安装包使用 Apple Silicon 所需的 ad-hoc 完整性签名。当前政策不包含 Developer ID、公证和付费签名账号。具体发行仍需明确授权，并核对版本、提交、包哈希、实际许可义务和安全条件。

1. 按[版本机制](../../../docs/zh-CN/engineering/versioning.md)更新 VERSION.json 并同步派生版本，确认版本范围与能力矩阵，关闭或标记未达门槛的特性。准备 `release-notes/<完整版本>.md`，记录一致的 version/build 元数据与中英文说明，按平台和提交引用真实验收；运行 `python3 scripts/publish-release.py --check-notes`，缺失说明时不能沿用旧版内容。
2. 从确定提交构建各平台未签名包、调试符号、SBOM、NOTICE 与依赖许可证；对应源码压缩包不作为发行附件生成或上传。
3. 记录工具链、哈希、features、目标和基准环境；验证源码可重建未签名部分。
4. 核验未正式签名的交付方式：验证 macOS ad-hoc 完整性签名，明确 Windows 安装器未签名，在发布说明中如实说明系统可能出现的安全提示和已验证的安装限制。不要求用户关闭系统安全保护，不宣称获得平台发行者信任。
5. 核对发行工作流实际执行的安装、首开、关联、升级／卸载和依赖检查；按变更选择补充桌面或干净环境检查，并记录未覆盖项。
6. 核对安全、互操作与缺陷回归结果，记录实际执行范围和已知限制。
7. 核对版本专属发布说明、已知限制、完整安装/下载体积及校验和，经发布负责人确认后上传。草稿与说明文件不一致时停止，先显式对齐已审阅输入；不会自动覆盖草稿中的人工修订。
8. 发行后确认下载工件与已验证哈希一致；发生问题撤回问题渠道并说明影响，保留诊断证据。

GitHub 桌面版从“关于”主动检查更新。符合条件的安装版下载并交接已校验的候选包。便携版和不满足安装身份或信任要求的构建手动安装。独立 CLI 与桌面附带 CLI 的更新对象各不相同，商店版通过 Microsoft Store 发布和更新。活动任务使安装等待时，用户可取消等待。回退前核对设置 schema 兼容性并保留可运行旧版。见[桌面更新契约](../../../docs/zh-CN/engineering/software-update.md)和[CLI 更新契约](../../../docs/zh-CN/engineering/command-line.md)。

按[桌面验证方法](../qa/private-test-manual.md)为实际候选选择相关场景。只有身份和范围仍适用时才引用已有证据。用最终包核对[部署 SBOM 与文件清单](../../../docs/zh-CN/engineering/deployment-audit.md)，定位具体组件、通知和再分发缺口。按平台要求核对安装版及便携版行为。macOS 单独验证交付 app 和 DMG 的部署与 ad-hoc 完整性，SDK 模式验证另行记录。

用户明确要求时，可在本地 Mac 生成[开发 DMG](../../../docs/zh-CN/engineering/macos-deployment.md)。只有所有者另行明确请求本地源码审查时才运行[交付收集器](../../../docs/zh-CN/engineering/source-delivery.md)。发布受影响的二进制前，按[发行许可复核](../qa/release-review.md)解决实际组件义务。项目在 Mac 上使用 ad-hoc 完整性签名，当前 GitHub 分发政策不包含 Developer ID 与公证。
