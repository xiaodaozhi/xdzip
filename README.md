# xdzip

现代、轻量、可扩展的跨平台压缩包管理工具。

**当前阶段：WP-12 开发与验收完成，剩余事项见[独立发行审查](https://github.com/xiaodaozhi/xdzip/issues/4)，尚未批准正式发行。WP-11 已接入 TAR／TAR.GZ 浏览、创建与安全解压；ZIP 创建支持 Store／Deflate、四档压缩级别、AES-256 内容加密和编号分卷；默认 2-正常压缩。与既有安全解压共用任务、冲突、取消和异步退出。自动化与桌面验收状态见各阶段记录，开发构建不作为正式签名发行。** 界面路线为 Qt Quick/QML，薄 C++ 适配层连接 Rust 核心；先浏览压缩包，再由用户明确选择压缩或解压。

![xdzip 已确认的浅色主窗口设计](design/hifi/screens/main-light.png)

版本统一维护在 [VERSION.json](VERSION.json)，当前开发渠道和构建号见“设置 → 关于 xdzip”或 `xdzip --version`，规则见[版本机制](docs/engineering/versioning.md)。

## 从这里开始

- [文档总入口](docs/README.md)：按角色阅读全部设计和开发准备文档。
- [详细软件设计](docs/design/index.md)：需求、架构、领域、桥接、格式 SDK、安全、性能和实施路线。
- [HTML 高保真设计](design/html/index.html)：可操作界面、原稿对照和独立窗口预览；[目录职责](design/README.md)。
- [主窗口组件与交互](design/html/components.html)：共享菜单、按钮状态和键盘操作；[通用验证方法](docs/qa/README.md)。
- [高保真设计统一入口](design/hifi/README.md)：当前主界面、操作窗口与设计参数；图标和原像素切片在[统一素材库](design/assets/index.html)。
- [开发就绪评审](docs/planning/readiness.md)：已确定事项、待验证项和实施门槛。
- [实施工作清单](docs/planning/backlog.md)：可直接拆为开发 issue 的工作包与验收条件。
- [贡献指南](CONTRIBUTING.md)与[编码规范](docs/design/10-quality-engineering.md)。

界面支持独立设置窗口，可保存主题、文字大小及中文 (简体)／English (US) 语言选择；语言下次启动生效，详见[设置与国际化](docs/ux/settings-localization.md)。

开发构建入口：[Qt 开发应用](apps/desktop/README.md)。`xdzip-browser` 读取真实 ZIP、TAR、TAR.GZ 目录，支持搜索、真实目录树、全结果排序、当前目录选择及解压元数据预检，`xdzip-ui-preview` 使用明确标识的示例任务。`xdzip-browser` 已执行真实 ZIP 解压，支持从文件／目录创建普通、AES-256 加密或 `.zip.001` 编号分卷 ZIP。TAR/TAR.GZ 通过 [WP-11 契约](docs/engineering/m4-tar-gzip.md)复用安全提交；加密／分卷 ZIP 读取、独立非 TAR 的 GZIP、稀疏／特殊 TAR 和实际内容预览仍未实现；目前加密／分卷输出可使用 7-Zip 等兼容工具打开。创建边界见 [WP-10 契约](docs/engineering/m4-creation.md)，解压边界见 [M3 实现契约](docs/engineering/m3-extraction.md)。

macOS 保留原生编译与测试，经用户明确要求可制作[本地开发 DMG](docs/engineering/macos-deployment.md)，CI 不生成或上传 Apple Silicon 应用包。build/、target/ 和所有层级的 dist/ 均不提交到 Git；详细规则见 [CI 策略](docs/engineering/ci-policy.md)。

## 系统支持与维护

首批支持 macOS 14 及以上的 Apple Silicon arm64、仍受维护的 Windows 11 x86_64，以及 Ubuntu 24.04 LTS x86_64、X11、glibc 2.39 及以上。当前不提供 Intel Mac、Windows／Linux ARM64 包，不承诺其他 Linux 发行版或 Wayland。范围和实际验证边界见[支持政策](docs/engineering/support-policy.md)。

个人项目优先维护最新公开版本，尽力处理反馈，不承诺固定响应时限；v0.1 手动下载更新，开发预览继续标记实验性。付费签名、公证与开发者账号不作为发行前提，许可来源与独立安全审查继续单独评审。

## 产品方向

首版计划支持 ZIP Store/Deflate 和 TAR/TAR.GZ 的浏览、创建和安全解压；提供真实进度、取消、路径保护和一致的跨平台交互。大索引、受限预览、额外格式及第三方进程扩展按后续里程碑逐项验证。性能预算与方法见[性能规范](docs/design/09-performance.md)，实际执行记录见对应 issue 与 Actions 工件。

Rust 核心独立于 Qt；内置格式通过源码接口扩展，第三方扩展通过版本化进程协议隔离。所有真实输出路径、冲突处理和提交由宿主统一控制。

## 仓库现状

`docs/design/` 为规范源文档，`design/hifi/` 为高保真原稿，`docs/visual/` 为视觉规范，`docs/engineering/` 为工程准备，`docs/planning/` 为阶段和风险，`docs/qa/` 为验收计划，`.github/` 为协作模板。文档构建工具仅用于生成 HTML，不构成产品代码。

仓库只保留源码、测试、设计输入和持久文档；阶段报告与验收状态使用 Issues，生成证据放本地或 Actions 工件，规则见[内容与历史管理](docs/engineering/repository-hygiene.md)。

项目原创内容采用 [Apache License 2.0](LICENSE)，详情见 [许可证说明](docs/engineering/licensing.md)。Qt、编解码器和其他依赖分别遵守自身许可证。
