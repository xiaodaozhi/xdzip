# 发布操作手册

[English](../../engineering/release-runbook.md) | 简体中文

跨平台稳定发行与预发布使用[发布协议 v1](../../../docs/zh-CN/engineering/release-protocol.md)，按完整标签冻结源码，草稿汇集六类平台包、更新材料、通知、许可、SBOM 和部署／验证材料。发行不生成或附带对应源码压缩包；常规发行及验收不运行源码交付收集器，仅用户针对源码收集另行明确要求时可用于本地审查。正式发布在 Release 的单次手动运行中构建三平台工件，每个平台只构建一次。包验收、签名、公开和公开下载复核分别使用该运行的工件。创建标签前完成匹配的预检。全部核验通过后，stable 作为普通 Release 公开并标记 Latest；alpha、beta、rc 作为 Pre-release 公开。公开后下载保留的附件复验哈希。未执行的人工验收按候选实际范围说明，不以自动化测试代替其结论。

开发验收、[发行许可复核](../qa/release-review.md)与发行执行分别记录。许可结论绑定实际候选。公开发布前须解决来源、通知及适用源码与替换义务的具体缺口。

## 选择最少的发行运行

触发长时发行或发行预检工作流前，先核对目标、实际入口及默认输入，以及已有运行、源码提交、包身份、环境和验证证据。在发行 issue 记录必要检查和剩余风险。身份、环境和覆盖范围仍适用的证据必须复用；不得把可选诊断、历史建议或执行者习惯变成发行门槛。

已获授权的常规发行，完成适用的本地预检和发布说明后冻结版本，**直接触发一次正式 Release**。不得仅为增加信心、出于习惯或确认正式运行可能通过，就先运行完整的 `candidate=true` 演练、另一次三平台打包或其他等价完整构建。正式运行本身会在签名和公开前构建并验证全部必要平台。对已知风险先用能覆盖真实入口及输入的最小检查收敛。

只有用户明确要求、适用规范强制要求，或具体变更／失败产生的风险无法由必要的定向检查覆盖时，才允许另行运行候选演练。触发前须在 issue 记录具体条件、覆盖范围、较小检查不足的原因及是否计划正式发行。演练不等于发行批准，不得把它增加为默认前置条件。

当前工作流不能将另一运行的候选包直接晋升为正式发行：正式验证、签名和发布只接收本次运行的工件。选择演练**之前**必须核对这一限制。演练通过的适用证据可以复用，但不能代替实际正式包的验收，也不能省去正式运行的构建。不得先启动演练，事后才发现这一规则会导致第二次完整构建。跨运行晋升必须另行实现并验证工作流；不得为避免重复而绕过同次运行身份、包哈希、签名、许可复核或发行授权。

作业失败时保留日志和已通过工件，先定位受影响边界；冻结源码和输入仍有效且工作流支持时，优先在原运行中定向重跑失败作业。若源码、配置、环境变化、工件缺失或其他实际约束使原运行重试无效，须在 issue 说明原因，并用适当的新运行重验受影响范围。不得违反交接契约混用不同提交或运行的工件，也不得将失败或未执行的检查记为通过。

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

稳定版触发前，在 `release-signing` 配置 [Snap Store 发布凭据](../market/linux/snap-store.md#准备并发布修订)。正式发布任务会将同一运行中已经核验的 amd64 Snap 发布到 `latest/stable` 并记录商店修订。如果 GitHub 已公开而 Snap 步骤失败，在原运行中重试失败的任务即可。重试先核对公开的 GitHub Release，再重试 Snap 上传。

公开后核对发布状态、说明、资产清单及重新下载文件的 `SHA256SUMS`。工作流的 `verify-public` 作业调用[发布脚本](../../../scripts/publish-release.py)单独下载复验。已公开版本保留原资产，需要修正内容时使用新版本。在[变更日志](../../../CHANGELOG.zh-CN.md)登记已公开版本，将待发布的变更保留在[未发布章节](../../../CHANGELOG.zh-CN.md#unreleased)，并同步更新所有语言版本。

## 候选复核

GitHub 安装包遵循项目的分发政策。Windows 安装器未签名，macOS 安装包使用 Apple Silicon 所需的 ad-hoc 完整性签名。当前政策不包含 Developer ID、公证和付费签名账号。具体发行仍需明确授权，并核对版本、提交、包哈希、实际许可义务和安全条件。

1. 按[版本机制](../../../docs/zh-CN/engineering/versioning.md)更新 VERSION.json 并同步派生版本，确认版本范围与能力矩阵，关闭或标记未达门槛的特性。准备 `release-notes/<完整版本>.md`，记录一致的 version/build 元数据与中英文说明，按平台和提交引用真实验收；运行 `python3 scripts/publish-release.py --check-notes`，缺失说明时不能沿用旧版内容。
2. 核对本次运行提供指定提交的桌面包、独立 CLI 包、SBOM、通知、依赖许可证及验证记录。包通过原生核验后由 Linux 的 `sign` 作业生成更新签名。发行附件禁止生成或上传对应源码压缩包。
3. 记录工具链、哈希、features、目标和基准环境；验证源码可重建未签名部分。
4. 核验未正式签名的交付方式：验证 macOS ad-hoc 完整性签名，明确 Windows 安装器未签名，在发布说明中如实说明系统可能出现的安全提示和已验证的安装限制。不要求用户关闭系统安全保护，不宣称获得平台发行者信任。
5. 核对发行工作流实际执行的安装、首开、关联、升级／卸载和依赖检查；按变更选择补充桌面或干净环境检查，并记录未覆盖项。
6. 核对安全、互操作与缺陷回归结果，记录实际执行范围和已知限制。
7. 核对版本专属发布说明、已知限制、完整安装/下载体积及校验和，经发布负责人确认后上传。草稿与说明文件不一致时停止，先显式对齐已审阅输入；不会自动覆盖草稿中的人工修订。
8. 发行后确认下载工件与已验证哈希一致；发生问题撤回问题渠道并说明影响，保留诊断证据。

GitHub 桌面版从“关于”主动检查更新。符合条件的安装版下载并交接已校验的候选包。便携版和不满足安装身份或信任要求的构建手动安装。独立 CLI 与桌面附带 CLI 的更新对象各不相同。Windows 和 Linux 商店版通过各自的商店发布和更新，见 [Microsoft Store](../market/windows/build.md) 和 [Snap Store](../market/linux/snap-store.md) 流程。活动任务使安装等待时，用户可取消等待。回退前核对设置 schema 兼容性并保留可运行旧版。见[桌面更新契约](../../../docs/zh-CN/engineering/software-update.md)和[CLI 更新契约](../../../docs/zh-CN/engineering/command-line.md)。

按[桌面验证方法](../qa/private-test-manual.md)为实际候选选择相关场景。只有身份和范围仍适用时才引用已有证据。用最终包核对[部署 SBOM 与文件清单](../../../docs/zh-CN/engineering/deployment-audit.md)，定位具体组件、通知和再分发缺口。按平台要求核对安装版及便携版行为。macOS 单独验证交付 app 和 DMG 的部署与 ad-hoc 完整性，SDK 模式验证另行记录。

用户明确要求时，可在本地 Mac 生成[开发 DMG](../../../docs/zh-CN/engineering/macos-deployment.md)。只有所有者另行明确请求本地源码审查时才运行[交付收集器](../../../docs/zh-CN/engineering/source-delivery.md)。发布受影响的二进制前，按[发行许可复核](../qa/release-review.md)解决实际组件义务。项目在 Mac 上使用 ad-hoc 完整性签名，当前 GitHub 分发政策不包含 Developer ID 与公证。

## 候选演练与失败续跑

只检查 Qt Network 准备时，对待验证分支运行无签名候选探针：

```sh
gh workflow run release.yml --ref BRANCH -f candidate=true -f qt_network_probe=true
```

探针在 Windows x64 和两种原生 Mac 架构上运行，使用更新回归工作流的 Qt Network 工件查找机制，在验证后结束。`qt-network-prepare.json` 记录命中或源码构建结果。传入 `-f dependencies=cold` 可强制源码编译。

Windows x64、两种 Mac 架构和 Ubuntu x64 的候选与正式构建共用原生依赖准备入口。先生成 Qt Network、Qt ImageFormats 和 Mac 的 Sparkle SDK 工件，生产提交进入 `main` 后，在新 runner 上验证跨运行命中。

```sh
gh workflow run release.yml --ref BRANCH -f candidate=true -f native_dependencies=seed
gh workflow run release.yml --ref main -f candidate=true -f native_dependencies=hit
```

默认 `dependencies=auto` 接收依赖键完全匹配的工件，工件的原生生产作业必须成功，且生产提交须已进入 `main`。使用前校验工件摘要、原始 Qt SDK、ABI 和原生架构。Sparkle 还要验证锁定的框架回执和严格代码签名。Ubuntu 使用宿主 runner 生成的工件。缺失或拒收时从锁定源码构建，`dependencies=cold` 可直接走源码。`dependencies.json` 记录每个依赖的结果。探针验证 PNG 和 TIFF 解码像素，常规候选和正式发行仍在当次运行构建、部署并验证应用和 CLI 包。

在 Windows x64 和原生 macOS Intel 上测量 Qt ImageFormats 复用时，运行图片模块探针：

```sh
gh workflow run release.yml --ref BRANCH -f candidate=true -f qt_image_probe=true
```

每个平台从源码构建补丁版 QtGui 和 TIFF 插件，用预览程序验证后，再在同一轮的新 runner 上恢复两个模块及其许可证。恢复要求 Qt SDK 二进制、图片模块构建输入和工具链一致。预览程序核对 PNG、TIFF 的解码像素，并确认 TIFF 插件来自选定的 SDK。通过 `image-timing.json` 比较模块准备耗时，通过工件下载步骤查看传输耗时。探针在桌面端和 CLI 打包前结束，复用仅在探针中启用。

只有满足上述条件确需演练时，才在包含工作流变更的分支或其他待核查的候选上运行专用 Release 工作流：

```sh
gh workflow run release.yml --ref BRANCH -f candidate=true
```

测试续跑时加入 `-f retry_probe=true`。如果本次运行的首次尝试进入 Windows 包验收，作业会在恢复候选工件后主动中断。等本次运行结束，用 `gh run rerun RUN_ID --job JOB_ID` 指定失败的验收作业。重跑会读取现有构建工件并验哈希。签名、发布和公开下载复核也是独立作业。在原运行中重跑失败作业，GitHub 可以保留已经完成的作业和工件。

组装结果可从 Actions 运行的 `release-candidate` 工件下载。`candidate-download` 作业重新下载并校验每个文件。候选包不含发行更新签名，演练不会创建草稿或公开 Release。正式发行在 `main` 上使用已授权标签，并使用当次运行中核验通过的工件。
