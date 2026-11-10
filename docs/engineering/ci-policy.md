# CI 与合入策略

## 日常检查

[CI](../../.github/workflows/docs.yml) 在 main 推送、pull request 和手动触发时执行一个 Linux 作业。它检查文档链接与 JSON、版本与界面文案、生成资源一致性、窗口与共享控件契约、Rust 与 C/C++ 格式、工作流结构及空白错误。作业不构建桌面产品，不执行完整 Rust、三平台 Qt、压缩格式互操作、打包或安装生命周期。目标是通常在五分钟内完成，排队和下载时间以实际运行记录为准。

每次 PR 更新可以取消同一 PR 的旧运行；main 和手动运行使用独立运行身份。失败或取消不能记为通过。工作流使用只读仓库权限和固定版本的依赖，不使用发布密钥。普通 CI 不启动 Playwright、Chromium 或其他网页浏览器；HTML 设计的静态生成检查仍会执行。

## 提交前本地验证

提交和推送前，在本地对拟提交的改动完成对应构建或验证，并记录实际执行结果。无需安装 Git 钩子；源码或配置发生影响结果的变化后重新验证，不把包含其他未提交改动的测试结果直接描述成干净提交的结果。

| 改动 | 本地验证 |
| --- | --- |
| 文档与静态资源 | 检查链接、JSON 和受影响的生成输出。 |
| Rust 核心与格式 | 检查格式，构建或测试受影响 crate 和相关业务路径；修改安全 I/O、格式或任务语义时覆盖相应失败、取消和保护行为。 |
| C++、Qt 和桥接 | 构建受影响目标，执行相应模型、桥接及交互测试。 |
| QML 窗口 | 检查窗口契约、共享控件以及双语多字号布局；单选和复选行为变化时执行实际辅助功能动作并检查选择模型。 |
| 打包、安装与更新 | 执行受影响的本地测试；需要隔离系统、其他平台或真实安装时按需使用对应工作流。 |

可用命令和工具链前提见[开发构建指南](developer-build.md)。优先使用当前机器可用环境；所需工具缺失时报告未执行的检查，不把预检或其他平台的结果记为本机测试通过。日常检查不要求本地安装完整 IDE，也不自动安装产品或修改文件关联。

## 按需完整检查

[Foundation probes](../../.github/workflows/foundation.yml) 和 [Qt components](../../.github/workflows/qt-components.yml) 仅通过 `workflow_dispatch` 手动运行，不随普通推送、PR 或标签自动触发。两者仍在 Windows、Linux 和 Apple Silicon macOS 运行完整矩阵。Foundation 包括 Rust workspace、Clippy、桥接探针、隔离工作进程和格式互操作；Qt 包括产品构建、组件及会话测试、原生平台检查、部署和适用的安装生命周期。Qt 按需运行保留首轮和十次完整浏览器套件；日常本地验证不重复这十轮。

Qt 按需运行的 `test --skip-rust` 将 Rust workspace 标记为未执行。需要 Foundation 结论时应针对同一提交另行运行并核对，不把仅有的 Qt 结果当作 Foundation 已通过。运行中的失败、取消、缺少平台或原始日志仍使相应完整检查失败；仅使用本次实际执行的运行和工件。

Foundation 和 Qt 使用固定输入缓存，缓存仅包含 Cargo 下载、pip 下载和固定 Qt 对应源码归档，不包含已编译产品、安装的 Qt SDK、测试结论或凭据。可信 main 的手动运行可保存经过哈希封存的缓存；其他分支仅恢复并核验。缓存命中不能代替重新执行检查。

Foundation 在三平台原生构建并执行无 Qt 的 CLI 回归，清除显示会话和 Qt 搜索路径，保留 JSON 回执。Qt 构建同时验证私有 CLI 布局及同用户更新交接。独立包生成要求同一可执行文件、worker 哈希和构建身份的通过回执；三平台 CLI 发行包及独立签名清单仅由已授权的 Release 流程产出。普通 macOS CI 仍只上传日志与 JSON。English: Native CLI checks run without Qt/display settings and bind receipts to both binaries. CLI release assets and signatures remain part of the separately authorized Release workflow.

Windows 和 Linux 的 Qt 手动运行上传实验性开发包和验证工件，通常保留 14 天；macOS 普通检查仅保留日志和 JSON，不部署、签名或上传应用包。日常 CI 不生成这些开发包。按需开发 DMG 和专用 Release 的例外见[Mac 部署](macos-deployment.md)和[发布协议](release-protocol.md)。

## 视觉、桌面与发行

根据改动先运行自动化并由执行者尽量检查实际应用或截图。输入法、真实读屏、系统对话框、窗口装饰、物理多显示器及真实 GPU 等场景按需检查；没有执行或没有对应环境时记录限制，不要求每次由维护者完成固定人工清单，也不把离屏、合成输入或虚拟显示器记为真人或物理设备通过。方法见[桌面验证](../qa/private-test-manual.md)。

[Windows 原生视觉诊断](../../.github/workflows/windows-native-visual.yml)可手动采集自有窗口和菜单图像；它是一次性 hosted runner 自动诊断，不代表真人点击或物理显示器验收。[Qt 源码验证](../../.github/workflows/qt-source.yml)及更新专项工作流同样按各自的手动入口执行。

[Release 工作流](../../.github/workflows/release.yml)独立处理三平台发行构建、包验证、签名和不可变资产，继续遵循[发布协议](release-protocol.md)。发布必须有所有者对具体结果的授权；未执行的检查及许可复核限制如实记录。普通 CI 成功、手动工作流成功和正式发行批准是不同结论。

日志、截图、测试输入副本和生成报告仅放在 `build/`、`dist/` 或 Actions 工件中。项目源文件、设计规格和测试代码保留版本管理；实际工作进展与结果登记在 GitHub issue。
