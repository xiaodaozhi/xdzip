# CI 与合入策略

[English](../../engineering/ci-policy.md) | 简体中文

[macOS Intel 原生 CI](../../../.github/workflows/macos-intel.yml) 是独立的实验性构建与测试，可手动触发，修改其入口时也会在拉取请求上运行。它在 `macos-15-intel` 核对产品、Rust 桥接和修补依赖的原生架构，只保留验证记录，不生成发行包，也不代表发行或人工桌面验收通过。专用更新回归另行使用测试密钥验证 Intel 产品。

## 日常检查

[CI](../../../.github/workflows/docs.yml) 在 main 推送、pull request 和手动触发时执行一个 Linux 作业。它检查文档链接与 JSON、版本与界面文案、生成资源一致性、窗口与共享控件契约、Rust 和原生源码格式、工作流结构及空白错误。Rust 输入发生变更时，执行与发行验证相同的工作区严格 Clippy 命令，并核对 Linux 的生产依赖边界。CI 比较推送或 pull request 的完整差异，手动触发或无法获得对比基线时也执行这两项检查。作业不构建桌面产品，不执行完整 Rust、三平台 Qt、压缩格式互操作、打包或安装生命周期。目标是通常在五分钟内完成，排队和下载时间以实际运行记录为准。

日常 CI 和 Foundation 共同使用 `scripts/check-native-format.py` 和 clang-format 18.1.8。该检查覆盖纳入版本管理的手写 C、C++、Objective-C 和 Objective-C++ 源码。格式注册表生成器检查其生成的原生头文件，历史偏好设置兼容性夹具保留原始源码。

每次 PR 更新可以取消同一 PR 的旧运行；main 和手动运行使用独立运行身份。失败或取消不能记为通过。工作流使用只读仓库权限和固定版本的依赖，不使用发布密钥。普通 CI 不启动 Playwright、Chromium 或其他网页浏览器。

### 变更分类与回退

[ci-plan.py](../../../scripts/ci-plan.py) 按[仓库中的分类策略](../../../scripts/ci-plan-policy.json)，以已核验的基准提交为起点比较完整 Git 文件树。目前只允许把 `docs/README.md` 的原位 UTF-8 修改归入 `plain-docs`，且须满足差异与文件大小限额。其他路径、增删、重命名、模式变化、无法核验的历史及手动触发均得到 `classification=full`。当前 `shadow` 模式下，即使判定为 `plain-docs`，`run_heavy` 仍为 `true`。

Foundation 和 Qt 各保留一份 `build/ci-plans/*-plan.json` 工件。普通运行没有取消时，分类缺失或失败会回退到完整检查。Foundation 的显式诊断参数选择自己的作业。日常 CI 独立运行轻量作业。修改分类或工作流时，在本地运行 `python3 scripts/test-ci-plan.py` 和 `python3 scripts/test-ci-workflows.py`。

## 提交前本地验证

提交和推送前，在本地对拟提交的改动完成对应构建或验证，并记录实际执行结果。无需安装 Git 钩子；源码或配置发生影响结果的变化后重新验证，不把包含其他未提交改动的测试结果直接描述成干净提交的结果。

| 改动 | 本地验证 |
| --- | --- |
| 文档与静态资源 | 检查链接、JSON 和受影响的生成输出。 |
| Rust 核心与格式 | 检查格式，构建或测试受影响 crate 和相关业务路径；修改安全 I/O、格式或任务语义时覆盖相应失败、取消和保护行为。 |
| C++、Qt 和桥接 | 构建受影响目标，执行相应模型、桥接及交互测试。 |
| QML 窗口 | 检查窗口契约、共享控件以及双语多字号布局；单选和复选行为变化时执行实际辅助功能动作并检查选择模型。 |
| 打包、安装与更新 | 执行受影响的本地测试；需要隔离系统、其他平台或真实安装时按需使用对应工作流。 |

修改原生桥接构建时，检查桌面端和隔离进程的构建目标。两者分别跟踪 Cargo 为本机目标解析的本地常规依赖和构建依赖。确认共享源码改动触发相关桥接重建，无关 crate 的源码改动不触发重建。检查 CLI 增量复制时，先用相同输入重复构建，核对已复制的程序、构建身份和桌面上下文保留时间戳，再确认大小和时间戳相同但字节变化的输入也能更新产物。

原生附属程序以显式构建产物声明。检查副本缺失时自动恢复，只改动附属程序时更新副本，并确认这两种情况均无需重新链接桌面主程序。worker、桥接库、内置 CLI 和更新 helper 使用同一 CMake 配置映射。Debug 对应 Cargo dev，Release 对应 Cargo release，RelWithDebInfo 对应保留调试信息的优化配置。受支持的桌面和 CLI 构建入口共用源码身份。直接调用 Cargo 且未传入源码身份时，版本信息报告未知提交。

桌面产品与浏览、Shell 和更新测试链接共享会话目标及其 `Xdzip.Native` QML 注册。专用夹具和编译定义由使用它们的可执行目标管理。修改共享目标时，确认公共源码只编译一次，并验证产品及受影响测试入口能够加载 QML。

可用命令和工具链前提见[开发构建指南](../../../docs/zh-CN/engineering/developer-build.md)。优先使用当前机器可用环境；所需工具缺失时报告未执行的检查，不把预检或其他平台的结果记为本机测试通过。日常检查不要求本地安装完整 IDE，也不自动安装产品或修改文件关联。

## 按需完整检查

[MSIX 菜单集成检查](../../../.github/workflows/windows-msix-shell.yml)无需 Qt，编译生产菜单 DLL 和小型独立激活夹具；可直接运行，也可在 [MSIX 生命周期](../../../.github/workflows/windows-msix.yml)中选择 `shell_only`。默认使用 Windows 11 ARM 的 x64 仿真；`windows-2025` 提供原生 x64 COM 诊断，不代表 Windows 11 桌面验收。报告记录实际系统和架构。

验证项包括包注册、双语传统菜单、开关变化、禁用后拒绝执行、包激活、现代菜单签名拒绝、失效注册目标和默认选择。脚本使用一次性普通账户，通过 AUMID 激活已安装的夹具程序，并拒绝提升权限。加载 WindowsApps 中的 DLL 需要夹具自身的包身份，不能用未打包的副本代替。每次调用先移除旧回执，再检查本次包激活的新回执。托管会话限制和使用其他账户凭据激活可能使检查无法完成；失败的探针不提供菜单激活通过结论，须保留未收口记录。测试证书、包注册、开发安装策略和临时账户会清理或恢复。自签名夹具不代表真实 Store 签名放行或 Explorer 显示验收。

[基础验证](../../../.github/workflows/foundation.yml)和 [Qt 组件验证](../../../.github/workflows/qt-components.yml)在 Windows、Linux 和 Apple Silicon macOS 上手动运行。基础验证覆盖 Rust 工作区、Clippy、桥接探针、隔离工作进程和格式互操作。Qt 组件验证覆盖产品构建、组件及会话测试、原生平台检查、部署和适用的安装生命周期。Qt 组件验证和正式发行各执行一轮完整浏览器测试。归档浏览、窗口会话时序或 Qt 变更需要重复验证时，在候选提交上运行[浏览器稳定性验证](../../../.github/workflows/browser-stability.yml)。该工作流核对同一提交的十轮完整结果并保留测试日志。

Qt 按需运行的 `test --skip-rust` 将 Rust workspace 标记为未执行。需要 Foundation 结论时应针对同一提交另行运行并核对，不把仅有的 Qt 结果当作 Foundation 已通过。运行中的失败、取消、缺少平台或原始日志仍使相应完整检查失败；仅使用本次实际执行的运行和工件。

Foundation 和 Qt 使用固定输入缓存，缓存仅包含 Cargo 下载、pip 下载、固定 Qt 对应源码归档，以及 Qt 检查所需的已核对哈希的上游界面字体，不包含已编译产品、生成字体子集、安装的 Qt SDK、测试结论或凭据。可信 main 的手动运行可保存经过哈希封存的缓存；其他分支仅恢复并核验。缓存命中不能代替重新执行检查。

[界面字体生成](../../../.github/workflows/ui-fonts.yml) 是轻量的三平台手动检查，覆盖新增字形、离线缓存失败、并发生成和 TTF/WOFF2 输出哈希一致性。它只上传 JSON，不构建或部署 Qt 应用。普通 CI 在 Linux 执行字体回归。准备和预览命令见[字体指南](../../../design/assets/fonts/README.md)。

Foundation 在三平台原生构建并执行无 Qt 的 CLI 回归，清除显示会话和 Qt 搜索路径，保留 JSON 回执。Qt 构建同时验证私有 CLI 布局及同用户更新交接。独立包生成要求同一可执行文件、worker 哈希和构建身份的通过回执；三平台 CLI 发行包及独立签名清单仅由已授权的 Release 流程产出。普通 macOS CI 仍只上传日志与 JSON。

Foundation 的 CLI 检查使用统一构建入口和实际二进制，另验证用户级入口的创建、运行、防覆盖与撤销，并使用私有更新夹具。安装夹具只复制自有测试二进制，不生成发行归档，不部署桌面应用，上传内容仅限 JSON 回执。

Windows 和 Linux 的 Qt 手动运行上传实验性开发包和验证工件，通常保留 14 天；macOS 普通检查仅保留日志和 JSON，不部署、签名或上传应用包。日常 CI 不生成这些开发包。按需开发 DMG 和专用 Release 的例外见[Mac 部署](../../../docs/zh-CN/engineering/macos-deployment.md)和[发布协议](../../../docs/zh-CN/engineering/release-protocol.md)。

普通开发与发行验收不自动生成项目源码归档，也不调用源码交付收集器。源码收集必须有所有者单独明确要求；GitHub 标签自带的源码快照与项目流水线附件分开说明。

[SDK trial](../../../.github/workflows/sdk-trial.yml)同样仅手动运行，先执行独立只读协议与原生内核门禁，不下载 Qt、不部署应用，仅上传日志和 JSON。Foundation 的 `sdk_only=true` 手动选项调用同一 SDK 工作流，并跳过完整 Foundation 作业与 gate，不能据此声称完整 Foundation 通过。SDK 平台失败优先用该入口收敛；Foundation 与 Qt 中仍适用的其他证据按验证方法复用。

Foundation 的 `image_only=true` 在原生 macOS 用系统 sips／ImageIO 转换固定自有 HEIC，并通过同一共享工具链准备方法安装固定 Qt，单独编译真实图片工作进程及测试对照。对照不施加限制的 Qt 读取器，以及逐项省略一个限制的真实进程代码，区分系统／插件能力与资源预算；测试拦截器不进入产品构建。`--verify-worker` 另要求真实父进程和默认工作进程对两个输入的尺寸／像素正确，运行时文件额度以内可增长且超额返回 EFBIG，否则此检查失败。此入口不构建桌面应用、不部署或签名，只保存 JSON 和日志，不把观察成功或受影响组件回归等同于完整产品／Foundation 通过。两个 preflight 选项不能同时选择。

### 更新专项验证

[原生更新回归](../../../.github/workflows/update-regression.yml)使用隔离夹具，可选单个平台排障，或以 `platform=all` 跑完整矩阵。[生产更新验收](../../../.github/workflows/production-update-acceptance.yml)是独立的手动工作流，在 `main` 上通过受保护的 `release-signing` 环境核对生产信任根并验证签名夹具。发布决定仍依据专用 Release 工作流和实际候选包。议题应记录运行、平台及覆盖范围。长时原生作业默认至少间隔五分钟查询，按需读取相关日志片段。

## 视觉、桌面与发行

根据改动先运行自动化并由执行者尽量检查实际应用或截图。输入法、真实读屏、系统对话框、窗口装饰、物理多显示器及真实 GPU 等场景按需检查；没有执行或没有对应环境时记录限制，不要求每次由维护者完成固定人工清单，也不把离屏、合成输入或虚拟显示器记为真人或物理设备通过。方法见[桌面验证](../qa/private-test-manual.md)。

[Windows 原生视觉诊断](../../../.github/workflows/windows-native-visual.yml)可手动采集自有窗口和菜单图像；它是一次性 hosted runner 自动诊断，不代表真人点击或物理显示器验收。[Qt 源码验证](../../../.github/workflows/qt-source.yml)及更新专项工作流同样按各自的手动入口执行。

Windows 视觉入口会先运行 `test-windows-capture.ps1`，检查[通用截图工具](windows-screenshots.md)的 PNG 像素保真、异常回执拒绝和演示输入可复现性。该自检不操作桌面，也不向日常 CI 增加 Windows 作业。

[Windows MSIX 生命周期](../../../.github/workflows/windows-msix.yml)在一次性 GitHub 托管 Windows Server 中构建实验包，以临时自签名证书签署测试副本，检查侧载安装、默认关联、中英文关于页按渠道显示更新入口、设置保留、便携版同时运行、文件激活、拒绝在 MSIX 身份下运行普通安装版的右键注册命令，以及卸载，并清理证书。升级夹具只递增专用 CI 身份的 MSIX 清单版本，逐文件核对其余产品内容不变，验证升级后的设置、更新入口隔离和默认关联；它不代表新产品版本或商店更新验收。该工作流仅手动运行，不使用真实商店签名或修改开发者电脑；服务器侧载结果不代表 Windows 11 桌面、商店来源、普通安装版共存或人工视觉验收。

[Windows Store MSIX](../../../.github/workflows/windows-store-msix.yml) 是正式商店身份的手动打包入口。它构建触发时 `main` 已包含的指定提交，执行桌面与命令行测试，校验已部署的运行目录与未签名 MSIX，并将包和回执保存为 Actions 工件。桌面测试阶段跳过 Rust 工作区测试和工作进程用例清单检查。商店提交、认证、签名和发布在合作伙伴中心进行。参见[商店打包步骤](../../../docs/zh-CN/engineering/developer-build.md#microsoft-store-msix)。

仅测试脚本或生命周期工作流有变化时，可手动运行 [MSIX 生命周期重验](../../../.github/workflows/windows-msix-recheck.yml)，指定原构建的运行 ID 和完整提交。重验核对包摘要、生产回执和提交祖先关系，并仅允许明确列出的测试及文档文件发生变化；产品或打包输入发生变化必须重新构建。结果分别标识构建提交与测试提交，不将旧包记为新产品构建。

重验可选 GitHub 托管 `windows-11-arm`，以 Windows 11 的 x64 仿真运行同一 x64 包。结果必须标明系统与架构，不代表 x64 原生环境、ARM64 原生构建或新增支持承诺。

[MSIX 与安装版共存检查](../../../.github/workflows/windows-msix-coexistence.yml)复用通过同一来源约束的构建和运行时审计，在一次性托管主机生成当前用户安装器，并检查安装版与 MSIX 的同时运行、相互卸载及默认关联。可选择 Windows Server 或 Windows 11 ARM 的 x64 仿真；该检查不修改维护者电脑，也不替代 Windows 11 x64 原生与商店验收。

[Release 工作流](../../../.github/workflows/release.yml)独立处理三平台发行构建、包验证、签名和不可变资产，继续遵循[发布协议](../../../docs/zh-CN/engineering/release-protocol.md)。发布必须有所有者对具体结果的授权；未执行的检查及许可复核限制如实记录。普通 CI 成功、手动工作流成功和正式发行批准是不同结论。

已获授权的常规发行，在适用本地预检后直接触发一次正式 Release。除非用户明确要求、适用规范强制要求，或具体风险无法由定向检查覆盖，不得先运行等价的完整候选演练或三平台打包。例外触发前须记录必要性，并确认当前正式工作流不能晋升其他运行的工件。保留适用证据，条件有效时优先在原运行中重跑受影响作业；具体决策规则与同次运行边界必须遵守[发布操作手册](release-runbook.md#选择最少的发行运行)。

日志、截图、测试输入副本和生成报告仅放在 `build/`、`dist/` 或 Actions 工件中。项目源文件、设计规格和测试代码保留版本管理；实际工作进展与结果登记在 GitHub issue。
