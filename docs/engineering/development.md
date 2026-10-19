# 开发者准备指南

先读产品 README、AGENTS、软件设计 01/02/10 和当前工作包。根 Cargo workspace 已包含 xdzip-domain，CMake 可选择构建 Qt 组件预览和独立探针；不要把文档准备描述为“应用已能运行”。

## 获取与修改

使用 Git 克隆仓库，建立短期分支。Markdown 是规范源文件；颜色和尺寸由 docs/visual/tokens.json 集中维护。新增/修改契约先更新相应设计与验收编号。

## 文档构建

文档工具依赖 Python 3.10+、lxml、Pandoc 和 Graphviz，字体推荐系统 CJK。执行 `python3 scripts/build-design-html.py` 生成 build/docs-html 的 16 篇详细软件设计及单文件版，输出下载包在 dist/。其余工程和协作文档从 docs/README.md 阅读。HTML 不作为产品界面技术选择。

执行 `python3 scripts/check-docs.py` 检查仓库 Markdown 文件目标、JSON 与设计资产；构建器另检查 HTML 内部链接和结构。Markdown 标题锚点和远端 URL 不在前者检查范围。文档检查通过不代表实际产品验收通过。

## 产品工具链准备

macOS 准备 Xcode Command Line Tools 与 Qt；Windows 准备 MSVC Build Tools、Windows SDK 与相匹配的 Qt；Linux 准备编译器、目标 glibc 环境和 Qt Wayland/X11 依赖。当前开发版本按根目录工具链文件与 CI 锁定，来源与许可按[工具链冻结表](toolchain.md)记录，不依赖开发者机器的“最新版本”。正式系统支持与维护方式见[支持政策](support-policy.md)，具体来源许可复核使用独立发行审查 issue。

首次产品实现必须提供从干净 checkout 构建的命令，并验证路径含空格/中文。签名凭据不用于普通开发或 PR；本地调试与签名发布是不同配置。

## 独立工程探针

[工具链探针](../../tools/toolchain-probe/README.md) 提供 Qt Quick 独立窗口的最小构建和 CTest 命令，不构成桌面产品。版本与测试边界见 [WP-01 证据（本地／CI 工件）](../qa/README.md)。

[统一 CMake/Cargo 桥接探针](../../tools/bridge-probe/README.md) 已提供根构建入口、锁定依赖与生命周期检查。依赖记录见 [cxx 诊断准入](cxx-admission.md)。
