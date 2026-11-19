# 构建并参与贡献

[English](../../contributing/README.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/contributing/README.md)

先阅读[贡献约定](../../../CONTRIBUTING.zh-CN.md)，查找已有议题或为本次改动新建议题。写明预期行为、适用平台和可实际验证的最小复现场景。使用项目自制或允许再分发的样本。各模块的源码位置见[仓库布局](../engineering/repository-layout.md)。

## 构建需要验证的部分

在仓库根目录运行命令。Windows 请在 x64 MSVC 开发环境中使用 `python`。macOS 请使用 Apple Silicon 原生终端和 Xcode Command Line Tools。Ubuntu 请按平台清单准备依赖。[构建指南](../engineering/developer-build.md#环境与入口)列出了前置工具，以及仓库锁定的 Rust 和 Qt 工具链。

首次构建桌面版可运行以下命令。

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

准备阶段的开关会通过已有的 rustup 安装锁定的 Rust 工具链，并在需要时下载锁定的 Qt SDK。已有共享依赖可复用 `build/toolchain/`。如果使用默认位置之外的兼容 Qt SDK，在相关阶段传入相同的 `--qt-prefix`。需要执行桌面测试套件时，请先完成同一配置的构建，再运行 `python3 scripts/desktop.py test`。工作进程的完整测试套件需要按[构建指南](../engineering/developer-build.md)准备测试输入。

开发独立 CLI 时，先构建，再用生成的 CLI 和工作进程执行验证。

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
```

默认输出在 `build/cli`。独立 CLI 需要锁定的 Rust 工具链、CMake 和平台 C++ 工具链，Ubuntu 还需要 `libssl-dev` 与 `pkg-config`。平台细节见[命令行构建契约](../engineering/command-line.md#构建与命令安装)。

## 验证受影响的行为

有匹配的构建时可以复用。纯文档改动先执行以下检查，并验证受影响的生成产物。改动或必需检查涉及桌面构建时，再执行相应构建。

```sh
python3 scripts/check-docs.py
git diff --check
```

| 改动类型 | 源码与契约 | 运行或核对 |
| --- | --- | --- |
| 文档和示例 | 同步编辑英文与简体中文页面，并更新引用。对照实现契约，用小型自制样本验证改动后的命令。 | 执行上方的文档检查，打开改动后的章节链接，核对相关生成输出。 |
| CLI | 从 `crates/xdzip-cli` 追到共享 Rust 运行时。[命令行契约](../engineering/command-line.md) | 基于当前构建运行独立 CLI 验证。改动命令或输出时，检查 stdout、stderr、退出状态和写出的文件，并覆盖适用的失败场景。 |
| Rust 领域逻辑、格式或归档任务 | 从 `xdzip-domain`、对应编解码模块、`xdzip-runtime` 或 `xdzip-io` 入手。查桌面命令时，先看 `BrowserController` 与 `RuntimeConnection`。[格式登记](../engineering/format-registry.md)、[运行时](../engineering/product-runtime.md)、[文件安全](../design/file-security.md) | 运行 `cargo fmt --all -- --check` 和受影响的包测试。验证实际运行时入口及相关的失败、取消或提交行为。调用原生格式工作进程时使用[工作进程测试入口](../../../developers/zh-CN/qa/runtime-worker-tests.md)。登记表改动后运行 `python3 scripts/generate-format-registry.py` 重新生成，再加 `--check` 核对，并运行 `python3 scripts/test-format-registry.py`。 |
| QML 窗口、控件或本地化 | 查看 `apps/desktop/ui`、`resources/i18n` 和 `specs/visual`。改窗口前阅读[视觉规范](../visual/design-standard.md)与[设计流程](../visual/design-workflow.md)，在[窗口登记表](../../../specs/visual/window-registry.json)登记契约。 | 窗口改动执行 `python3 scripts/check-window-contracts.py --self-test`、`python3 scripts/check-operation-components.py` 和受影响的双语多字号布局测试。本地化改动运行 `python3 scripts/check-ui-i18n.py`。按要求在相关平台验证原生交互或视觉表现。 |
| 安装、系统菜单或更新 | 查阅[构建与部署](../engineering/developer-build.md)、[文件关联](../engineering/installers-associations.md)与[软件更新](../engineering/software-update.md)。 | 核对受影响的安装或注册入口、实际交付包身份、错误及恢复流程。按 [CI 策略](../../../developers/zh-CN/engineering/ci-policy.md)选择对应平台工作流。 |

## 说明验证结果

在 PR 中说明改动后的行为，列出执行过的命令、验证平台及仍待覆盖的检查。明确哪些结果来自离屏测试、原生桌面或实际安装。[验证指南](../../../developers/zh-CN/qa/README.md)说明了各类验证的适用范围。各模块契约见[技术参考](../reference/README.md)，发行流程见[维护者手册](../../../developers/zh-CN/README.md)。
