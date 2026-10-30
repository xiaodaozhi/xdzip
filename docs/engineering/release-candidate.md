# 发行候选材料契约

每个候选绑定干净源码提交、产品版本和实际包哈希。材料生成不自动创建 GitHub Release、标签或上传包，也不批准许可选择；具体状态记录在候选对应 issue。正式签名与公证不作为门槛。

## 交付内容

固定干净提交后生成 Windows x86_64 ZIP／安装器、macOS arm64 app ZIP／DMG、Ubuntu 24.04 x86_64 tar.gz／deb，并收集同提交的项目源码、Rust vendor、Qt 固定源码与补丁、通知、实际文件 SBOM、构建和生命周期回执。候选总清单记录相对文件路径、SHA-256、大小、平台、构建版本和源码提交；不将不同提交的包拼成一个候选。

普通 macOS CI 只保留日志与 JSON。经所有者明确授权，专用 Release 工作流可在 Apple Silicon runner 部署 Qt、执行必要的 ad-hoc 签名，并从同一应用导出 app ZIP 与 DMG，验证完整性和无 SDK 搜索路径下的重定位启动；本地开发 DMG 仍遵循已有授权。Windows/Linux 的包由实际成功的产品 CI 获取，不把 QA-only 安装器当作产品。历史完整源码 Qt／性能记录只作为历史材料引用，不改写提交，也不要求人工验收重做。

源码包的离线重建与实际启动另行核验；这不等于完全可复现的上游 SDK 二进制或已批准许可。macOS DMG 部署后字节与 SDK 模式 CI 字节有差异，两套 SBOM 保留各自审计范围，不混作同一工件。

## 许可与来源核对

已有 runtime-attribution.json、qt-sdk-correspondence.json、qt-binary-differences.json、链接输入和内嵌资源记录。逐项审查输出到 build/，区分已精确解释的二进制转换、上游未声明构建来源、系统包装依赖、实际再分发库、主程序复合代码和待选许可分支。原始 gap 与 incomplete 保留，不把一个技术解释直接改为法律批准。

审查清单须逐平台核对报告的实际可用性与范围：最终文件 SBOM、部署后组件归属和 SDK 参考是三类输入。只有文件哈希或 SDK 模式归属时，将部署后的组件关联登记为缺失；裁剪架构、重定位、签名后的应用不能直接沿用 SDK 文件身份。历史候选与当前候选的观察数量只有在范围、文件与组件均匹配后才能比较，不能把数量变化当作已解决项。

专用 macOS Release 在最终部署应用仍存在时采集 Cocoa／Metal 资源探针、SDK 源码对应与差异分析，提供通用名称的来源／文件 SBOM。链接图和静态库绑定编译应用，资源与审计绑定部署应用；`release-deployment-evidence.json` 核对两者非调试代码／数据节的精确字节和零填充大小，保留不同的整文件哈希。加载命令、调试与签名元数据不计入节等价结论，异常边界、混合身份或节变化直接失败。单次探针不批准参考机性能或真人视觉，原始归属缺口与 incomplete 保持。

English: Dedicated macOS Release packaging captures a Cocoa/Metal resource probe, SDK-source correspondence and difference analysis while the final deployment exists, with conventional source/file SBOM names. Link inputs bind the compiled image; resources and audit bind the deployed image. `release-deployment-evidence.json` requires exact non-debug code/data section bytes and zero-fill sizes while preserving distinct whole-file hashes. Load commands, debug and signing metadata remain outside that equivalence claim. Invalid bounds, mixed identities or changed sections fail. A single probe grants no reference-performance, manual-visual or license approval; original gaps and incomplete status remain.

Qt 动态链接、通知及对应源码／替换材料是核对依据，不自动消除其他许可义务；参见 [Qt 官方义务说明](https://www.qt.io/development/open-source-lgpl-obligations)。Windows 再分发逐文件对照 [Visual Studio 2022 官方可分发清单](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)，并保留工具链及 DLL 来源，不能以“来自 Qt SDK”代替具体授权。字体按现有 OFL 材料，项目原创按 Apache-2.0，生成式图标按来源说明分别登记。

剩余问题必须具体到组件／文件、平台、原始声明、技术核对结果、可接受的补证方式和责任角色，不笼统要求购买商业 Qt、开发者账号或证书。对未经批准的许可选择保留待确认，不代替所有者接受条款。

## 政策与审查材料

- [系统支持与维护政策](support-policy.md)：2026-10-19 所有者批准 macOS 14+ arm64、受维护 Windows 11 x86_64、Ubuntu 24.04 X11 x86_64 的首批范围，以及个人项目尽力维护策略。范围与实际验证边界以该政策为准，冻结材料中的草案保留历史状态。

## 材料完成条件

三平台实际候选包与来源身份核验成功，源码和通知齐全、离线重建成功，许可问题整理为可处理的审查项，材料可独立阅读。待确认的来源与许可问题继续按[发行审查流程](../qa/release-review.md)跟踪，不以材料准备代替批准。

实际包、执行报告和校验和保存在本地或 CI 工件；正式交付材料随获批 Release 提供，不提交生成报告到源码仓库。
