# Windows 开发工具与运行库许可

本文定义 Windows 构建和再分发的适用路径，与[原生运行库材料](native-runtime-materials.md)、[开发构建入口](developer-build.md)和[安装契约](installers-associations.md)共同使用。具体环境、验证提交、材料哈希和验收结论登记在 issue 与本地／CI 工件，本文不保存某台机器的许可证明。

## 免费 Community 路径与本地选择

项目原创内容以 Apache-2.0 公开，属于 OSI 批准的开源许可。Visual Studio Community 2022 的个人条款允许个人开发自己的应用，组织条款也允许用户开发和测试以 OSI 批准许可发布的应用。组织的开源用途与其他商业内部用途必须分别判断，不能把本项目的适用条件扩大到其他项目。

需要自有 Community 开发环境时，维护者从[微软官方安装入口](https://learn.microsoft.com/en-us/visualstudio/install/use-command-line-parameters-to-install-visual-studio?view=vs-2022)取得 Community 2022，阅读并接受[该版本的原始条款](https://visualstudio.microsoft.com/license-terms/vs2022-ga-community/)，在允许范围内使用。C++ 构建选择所需的 `Microsoft.VisualStudio.Component.VC.Tools.x86.x64` 和 Windows SDK，不要求安装全部工作负载。无需购买 Professional／Enterprise，也没有另行申请每个项目运行库分发证书的步骤。

许可复核不要求本机长期安装大型 IDE。只使用下述 GitHub 标准托管环境构建与发布时，适用依据是托管 Visual Studio 使用授权及其 REDIST 条款，无需为托管 CI 另装本地 Community 或购买个人 Enterprise。Community 是维护者需要自有开发环境时的免费选择，不能把本地安装状态当作托管构建合法性的额外门槛。

Community 第 3 节允许在符合条件的构建设备上使用软件文件和 Build Tools；第 4 节授予清单内 Distributable Code 的分发权，并规定功能、下游条款和限制。仅下载 VC_redist、安装 Build Tools 或附带一份许可文本，不单独证明开发工具的适用许可。Build Tools 的无 Visual Studio 许可例外针对第三方开源依赖的构建，不是自行开发所有 C++ 程序的通用例外。使用 VS Code 也不自动取得 MSVC 许可。参见[微软许可指南](https://www.microsoft.com/licensing/guidance/Visual-Studio)和 [Build Tools 条款](https://visualstudio.microsoft.com/license-terms/vs2022-ga-diagnosticbuildtools/)。

本地应核对 `vswhere` 返回的产品、完整版本、非预览状态及 C++ 组件，进入实际实例的开发环境，再检查 `cl`、`link` 和 SDK。已安装的 2019 Build Tools 不记为 Community 2022，也不代替当前 MSVC 2022 工具链验证。通用 `desktop.py prepare` 不隐式安装系统 IDE／SDK，首次安装使用微软安装器，保留既有工具并禁止自动重启。

选用 2022 是当前发行工具链的选择，不是 Community 免费许可只允许使用 2022。[Qt 6.8 支持表](https://doc.qt.io/qt-6.8/supported-platforms.html)列出 Windows MSVC 2022，本仓库使用相应 `msvc2022_64` SDK。2019 工具可以保留用于本地开发；与当前 SDK 混用时，不能只凭跨版本 ABI 兼容推断完整支持。[微软的兼容性规则](https://learn.microsoft.com/en-us/cpp/porting/binary-compat-2015-2017?view=msvc-170)还要求最终链接器通常不旧于输入库所用工具，并分别约束运行库和 `/GL`／`/LTCG`。替代配置需要自己的适用许可及验证，不因此强制给本机安装完整 2022 IDE。

## GitHub 托管构建

项目 Windows 工作流使用 `windows-2022` 标准 GitHub 托管 runner；预装版本是 Visual Studio Enterprise 2022，不把它标记为 Community。runner-images 维护者已明确[确认](https://github.com/actions/runner-images/issues/12945#issuecomment-3348002951)：预装 Enterprise 通过 GitHub 与微软的约定供 GitHub Actions／Azure DevOps 托管 runner 的公开和私有用途使用，用户无需另备个人 Enterprise 许可。本项目据此使用托管构建环境；该公开说明不替代或声称公开了双方完整协议。

根据 [GitHub 托管 runner 文档](https://docs.github.com/en/actions/concepts/runners/github-hosted-runners)，每次 Windows job 获得独立虚拟机，`Set up job` 指向该次运行的镜像与软件清单。工作流只进行本仓库的编译、测试、部署与发布，符合 [Actions 附加条款](https://docs.github.com/en/site-policy/github-terms/github-terms-for-additional-products-and-features#actions)的用途范围。公开仓库的免费计算额度与 Microsoft 组件的再分发权是不同事项。

CI 记录实际 VS 产品、版本、编译器相对位置、工具版本及 SDK 版本到 `windows-toolchain.json`，与同提交包及 materials 一并保存，不包含产品密钥或开发者私人路径。它记录工具身份，不能自动证明许可资格。自托管 runner、从镜像复制到本地的工具、其他 CI 服务及一般共享服务器不继承上述托管许可，应重新核对 Community／Build Tools 的条款和设备条件。

## Microsoft 组件再分发

[VS 2022 REDIST 清单](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)明确包含有效许可的 Community 2022。当前使用 [windows-runtime.json](../../scripts/windows-runtime.json) 固定的正式版 VC_redist x64，原样提取并逐一核对八个 CRT DLL 和原始运行库许可，不打包编译器、调试器或 `debug_nonredist`。开发工具的 Community 免费路径和托管 CI 的 Enterprise 身份均不将微软 DLL 重新授权为 Apache-2.0。

Windows SDK 的 `d3dcompiler_47.dll`、`dxcompiler.dll`、`dxil.dll` 来自显式 SDK 的 `Redist/D3D/x64`。[SDK REDIST 清单](https://learn.microsoft.com/en-us/legal/windows-sdk/redist)允许在有效 SDK 许可下原样随 Classic Windows 应用分发这些文件，禁止随 UWP 应用分发；xdzip 是 Classic Windows 桌面应用。SDK 自己的原始许可及第三方通知另外保留，不用 Community 许可替代 SDK 条款。Mesa 软件回退库另按其原始许可和通知交付。

包内 `share/notices/Windows-runtime-NOTICE.txt` 双语说明各组件许可边界。Microsoft CRT 原始条款位于 `Microsoft-VC-runtime/license.rtf`，SDK 原始条款位于 `Windows-SDK-graphics/sdk_license.rtf`。安装器展示原始 CRT 协议并要求接受，随后展示组件说明；便携包用户同样需要阅读并遵守随包 Microsoft 条款。两种包都保留原始许可及来源，下游再分发者应自行满足适用资格和分发条件，不能仅凭项目 Apache-2.0 许可转发 Microsoft 组件。

安装器编译前核对 CRT 协议的固定哈希和双语说明，materials 附带同一份原始 CRT／SDK 文本和来源记录。许可补正不改写旧公开资产或历史构建身份；修订包沿用[不可变发布协议](release-protocol.md)另发版本。

English: xdzip uses Apache-2.0 for its original work. Eligible maintainers may acquire and accept Visual Studio Community 2022 for their own development environment under its individual or OSI open-source provisions; Community's Distributable Code rights remain subject to its redistribution requirements. A large local IDE installation is not a prerequisite for hosted builds. GitHub-hosted Windows jobs actually use Enterprise 2022, covered by the runner-images maintainer's published confirmation of GitHub's agreement with Microsoft and the applicable REDIST terms, not by relabeling it as Community. That confirmation does not cover self-hosted machines or tools copied off a hosted image. The current 2022 release toolchain matches Qt's supported SDK configuration; retaining 2019 for local development is a separate choice requiring its own licensing and compatibility checks. Tool identity and license eligibility are separate checks. Unmodified, pinned Microsoft CRT and Classic Windows SDK redistributables retain their own terms, outside the project's Apache-2.0 license. The installer presents the original CRT agreement and the bilingual component notice; portable users and downstream distributors must also comply with the applicable supplier terms. Corrected public packages require a new version.
