# 开发者准备指南

先读产品 README、AGENTS、软件设计 01/02/10 和当前工作包。根 Cargo workspace 已包含领域、格式、I/O、运行时和桥接，CMake 构建真实 Qt 应用、组件预览及独立探针。现有模块见[目录职责](repository-layout.md)，实际能力和限制以根 README 为入口。

## 获取与修改

使用 Git 克隆仓库，先检查工作区和远端 main，保留已有改动。维护者按 AGENTS.md 在 main 推进，相关检查通过后直接推送；外部贡献按[贡献指南](../../CONTRIBUTING.md)提交 PR。新任务先查重并建立或复用 issue，阶段结果不另写入 docs。Markdown 是规范源文件；颜色和尺寸由 docs/visual/tokens.json 集中维护。新增/修改契约先更新相应设计与验收编号。

## 文档构建

文档工具依赖 Python 3.10+、lxml、Pandoc 和 Graphviz，字体推荐系统 CJK。执行 `python3 scripts/build-design-html.py` 生成 build/docs-html 的 16 篇详细软件设计及单文件版，输出下载包在 dist/。其余工程和协作文档从 docs/README.md 阅读。HTML 不作为产品界面技术选择。

执行 `python3 scripts/check-docs.py` 检查仓库 Markdown 文件目标、JSON 与设计资产；构建器另检查 HTML 内部链接和结构。Markdown 标题锚点和远端 URL 不在前者检查范围。文档检查通过不代表实际产品验收通过。

## 产品工具链准备

macOS 准备 Xcode Command Line Tools 与 Qt；Windows 准备 MSVC Build Tools、Windows SDK 与相匹配的 Qt；Linux 准备编译器、目标 glibc 环境和 Qt Wayland/X11 依赖。当前开发版本按根目录工具链文件与 CI 锁定，来源与许可按[工具链冻结表](toolchain.md)记录，不依赖开发者机器的“最新版本”。正式系统支持与维护方式见[支持政策](support-policy.md)，具体来源许可复核使用发行许可复核 issue。

从干净 checkout 构建与运行的命令见[Qt 开发应用](../../apps/desktop/README.md)，包含 CMake 开关、macOS 架构和部署目标、离屏测试边界。构建或安装逻辑变更应复验空格／中文路径。当前个人项目不要求付费签名身份；本地 DMG 的 ad-hoc 完整性与 CI 不部署 Mac 应用的边界见[构建指南](developer-build.md)。

## 独立工程探针

[工具链探针](../../tools/toolchain-probe/README.md) 提供 Qt Quick 独立窗口的最小构建和 CTest 命令，不构成桌面产品。版本与测试边界见探针 README，实际结论在 [WP-01 issue](https://github.com/xiaodaozhi/xdzip/issues/1)保留。

[统一 CMake/Cargo 桥接探针](../../tools/bridge-probe/README.md) 已提供根构建入口、锁定依赖与生命周期检查。依赖记录见 [cxx 诊断准入](cxx-admission.md)。
