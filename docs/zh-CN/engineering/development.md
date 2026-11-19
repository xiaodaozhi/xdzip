# 开发者准备指南

[English](../../engineering/development.md) | 简体中文

从[开发与贡献](../contributing/README.md)选择适合这次改动的构建和验证入口。[系统架构](../design/system-architecture.md)与[仓库结构](repository-layout.md)说明 Qt、Rust 运行时和各格式模块的边界。

## 获取与修改

准备干净工作区，按[贡献约定](../../../CONTRIBUTING.md)提交 PR。修改契约时同步更新两种语言的规范及相关验证用例。颜色和尺寸以 specs/visual 中的参数为准。

## 文档构建

撰写两种语言的对应文档，标题下放置语言切换，当前语言显示为纯文字，另一语言链接到同一主题。新增或迁移文档时同步更新目录和已有链接。

文档工具依赖 Python 3.10+、lxml、Pandoc 和 Graphviz，字体推荐系统 CJK。执行 `python3 scripts/build-design-html.py` 生成 build/docs-html 的 16 篇详细软件设计及单文件版，输出下载包在 dist/。其余工程和协作文档从 docs/README.md 阅读。HTML 不作为产品界面技术选择。

执行 `python3 scripts/check-docs.py` 检查仓库 Markdown 文件目标、双语配对、JSON 与设计资产；构建器另检查 HTML 内部链接和结构。Markdown 标题锚点和远端 URL 不在前者检查范围。文档检查通过不代表实际产品验收通过。

## 产品工具链准备

macOS 准备 Xcode Command Line Tools 与 Qt；Windows 准备 MSVC Build Tools、Windows SDK 与相匹配的 Qt；Linux 准备编译器、目标 glibc 环境和 Qt Wayland/X11 依赖。当前开发版本按根目录工具链文件与 CI 锁定，来源与许可按[工具链冻结表](toolchain.md)记录，不依赖开发者机器的“最新版本”。正式系统支持与维护方式见[支持政策](support-policy.md)，具体来源许可复核使用发行许可复核 issue。

从干净 checkout 构建与运行的命令见[Qt 开发应用](../../../apps/desktop/README.md)，包含 CMake 开关、macOS 架构和部署目标、离屏测试边界。构建或安装逻辑变更应复验空格／中文路径。当前个人项目不要求付费签名身份；本地 DMG 的 ad-hoc 完整性与 CI 不部署 Mac 应用的边界见[构建指南](developer-build.md)。

## 独立工程探针

[工具链探针](../../../tools/toolchain-probe/README.md) 提供 Qt Quick 独立窗口的最小构建和 CTest 命令，不构成桌面产品。版本与测试边界见探针 README，实际结论在 [WP-01 issue](https://github.com/xiaodaozhi/xdzip/issues/1)保留。

[统一 CMake/Cargo 桥接探针](../../../tools/bridge-probe/README.md) 已提供根构建入口、锁定依赖与生命周期检查。依赖记录见 [cxx 诊断准入](cxx-admission.md)。
