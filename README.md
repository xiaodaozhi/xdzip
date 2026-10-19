# xdzip

现代、轻量的跨平台压缩包管理工具，采用 Qt Quick/QML、薄 C++ 适配层与 Rust 核心。打开压缩包先浏览内容，由用户明确选择创建或解压。

当前为实验性开发版本。WP-12 开发与验收已完成，见 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)；[独立发行审查 #4](https://github.com/xiaodaozhi/xdzip/issues/4)仍未完成，尚未批准正式发行。

![xdzip 浅色主窗口设计](design/hifi/screens/main-light.png)

## 当前能力

| 格式 | 浏览与解压 | 创建 |
| --- | --- | --- |
| ZIP、单卷 ZIP64 | Store／Deflate，目录树、搜索、全结果排序及选择性解压 | Store／Deflate 四档级别，默认 2-正常压缩 |
| TAR、TAR.GZ | 普通文件与目录，流式扫描和安全提交 | 输入树预检、写后校验、整体保存 |
| AES-256 ZIP、编号分卷 ZIP | 尚不支持读取与解压 | 支持内容加密和 `.zip.001` 编号分卷，可用兼容工具读取 |

- 浏览支持最多十万条物理记录，显示元数据和文件属性；实际内容预览与独立完整性检查命令尚未实现。
- 解压提供目标预检、同名询问／跳过／改名／替换、真实进度和取消。ZIP 在符合安全条件的 POSIX 目标可恢复受限相对符号链接和可执行位；TAR 链接、Windows ZIP 链接及特殊对象不支持。
- 创建成功后在应用内打开受支持的单卷结果，不自动打开 Finder／Explorer。
- 欢迎页支持拖放和最近记录；设置支持跟随系统／浅色／深色、100%／125%／150% 字号、列表密度及中文 (简体)／English (US)。语言保存后可选择重启生效。
- 提供关于窗口、macOS 系统菜单、全平台托盘及活动任务退出保护。Qt 当前使用单个真实压缩包会话，HTML 多标签属于设计示例。

独立非 TAR 的 GZIP、7z／RAR、稀疏 TAR、第三方扩展和百万条磁盘索引不在当前实现范围。详细限制见[浏览](docs/engineering/readonly-browser.md)、[安全解压](docs/engineering/m3-extraction.md)、[ZIP 创建](docs/engineering/m4-creation.md)及 [TAR/GZIP](docs/engineering/m4-tar-gzip.md)契约。已知的文件输入竞态与 macOS ACL 兼容问题继续在[安全修复复验 #6](https://github.com/xiaodaozhi/xdzip/issues/6)跟踪，现有检查不等同独立安全审查。

## 构建与运行

产品入口为 `xdzip-browser`，macOS 构建为 `xdzip.app`；`xdzip-ui-preview` 仅用于组件示例。准备锁定的 Rust、CMake、Ninja 与 Qt，将 Qt 前缀加入 `CMAKE_PREFIX_PATH`，按[Qt 开发应用](apps/desktop/README.md)构建和测试。平台部署、安装器和本地 DMG 见[开发构建指南](docs/engineering/developer-build.md)。

版本由 [VERSION.json](VERSION.json)统一维护，可在“关于 xdzip”或 `xdzip --version` 查看，规则见[版本机制](docs/engineering/versioning.md)。开发工件随 [Actions](https://github.com/xiaodaozhi/xdzip/actions)提供，保存时间及平台差异见 [CI 策略](docs/engineering/ci-policy.md)，不作为正式发行承诺。

## 系统支持与维护

首批支持 macOS 14 及以上的 Apple Silicon arm64、仍受维护的 Windows 11 x86_64，以及 Ubuntu 24.04 LTS x86_64、X11、glibc 2.39 及以上。不提供 Intel Mac 或 Windows／Linux ARM64 包，不承诺其他 Linux 发行版或 Wayland。实际验证边界见[支持政策](docs/engineering/support-policy.md)。

个人项目优先维护最新公开版本，尽力处理反馈，不承诺固定响应时限；v0.1 手动下载更新。付费开发者账号、签名证书和公证不作为发行前提。Windows 提供未签名的当前用户安装器，Ubuntu 提供 .deb；macOS 可按需制作带必要 ad-hoc 完整性签名的本地开发 DMG，CI 只编译和验证，不部署或上传 Apple Silicon 应用包。

## 文档与贡献

- [文档导航](docs/README.md)：产品、架构、工程契约、构建和验证方法。
- [高保真原稿](design/hifi/README.md)、[可操作 HTML 设计](design/html/index.html)与[统一素材库](design/assets/index.html)：界面设计输入；[视觉规范](docs/visual/README.md)约束实现。
- [工作包路线](docs/planning/backlog.md)与 [Issues](https://github.com/xiaodaozhi/xdzip/issues)：交付边界、实际进度和缺陷跟踪。
- [贡献指南](CONTRIBUTING.md)、[开发指南](docs/engineering/development.md)及[目录职责](docs/engineering/repository-layout.md)：参与项目的入口。

仓库保留源码、测试、设计输入和持久文档。阶段报告与验收状态使用 Issues，生成证据放本地或 Actions 工件；build/、target/ 和所有层级的 dist/ 不提交到 Git，详见[内容与历史管理](docs/engineering/repository-hygiene.md)。

项目原创内容采用 [Apache License 2.0](LICENSE)。Qt、编解码器及其他依赖分别遵守自身许可证，见[许可证说明](docs/engineering/licensing.md)。
