<p align="center">
  <img src="design/assets/brand/xdzip-128.png" width="80" height="80" alt="xdzip 应用图标">
</p>

<h1 align="center">xdzip</h1>

<p align="center"><strong>先浏览，再解压。</strong><br>面向 macOS、Windows 和 Linux 的开源压缩包管理工具。</p>

<p align="center">
  <a href="https://github.com/xiaodaozhi/xdzip/actions/workflows/qt-components.yml"><img src="https://github.com/xiaodaozhi/xdzip/actions/workflows/qt-components.yml/badge.svg?branch=main" alt="Qt 三平台构建"></a>
  <a href="https://github.com/xiaodaozhi/xdzip/actions/workflows/foundation.yml"><img src="https://github.com/xiaodaozhi/xdzip/actions/workflows/foundation.yml/badge.svg?branch=main" alt="Rust 核心检查"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="Apache License 2.0"></a>
</p>

<p align="center">
  <a href="#功能亮点">功能</a> / <a href="#格式支持">格式支持</a> / <a href="#获取与安装">获取与安装</a> / <a href="#从源码构建">源码构建</a> / <a href="docs/README.md">文档</a> / <a href="https://github.com/xiaodaozhi/xdzip/issues">反馈</a>
</p>

> **开发预览**：xdzip 目前处于实验性开发阶段，尚无获批的正式发行版本。试用前请阅读[当前限制](#当前限制)与[已知问题](https://github.com/xiaodaozhi/xdzip/issues)，并为重要文件保留备份。

![xdzip 主界面设计预览](design/hifi/screens/main-light.png)

<p align="center"><sub>主界面高保真设计参考。图中的多标签与图片预览为设计示例，当前产品能力见下文。</sub></p>

## 功能亮点

- **清晰地浏览内容**：目录树、列表与网格、搜索、全结果排序和文件属性，支持最多十万条物理记录。
- **按需要解压**：选择文件或文件夹，确认目标与重名策略，查看真实进度，随时请求取消；已完成的文件有明确结果记录。
- **创建压缩包**：支持 ZIP、TAR 与 TAR.GZ，提供压缩级别、ZIP AES-256 内容加密和编号分卷，写后校验通过后保存。
- **适应你的桌面**：浅色、深色与跟随系统主题，100%／125%／150% 字号、列表密度、中文 (简体) 与 English (US)，配合 macOS 系统菜单和全平台托盘。

打开文件先浏览内容，不自动解压。创建成功后在应用内打开可浏览的单卷结果，不自动弹出 Finder 或 Explorer。

## 格式支持

| 格式 | 浏览 | 解压 | 创建 |
| --- | --- | --- | --- |
| ZIP | ✓ | ✓ | ✓ |
| TAR | ✓ | ✓ | ✓ |
| TAR.GZ | ✓ | ✓ | ✓ |
| AES ZIP | 元数据 | — | ✓ |
| 分卷 ZIP | — | — | ✓ |

✓ 已支持，— 尚不支持；“元数据”表示仅浏览名称与属性。

- **ZIP**：支持单卷 ZIP64，解压与创建支持 Store／Deflate，创建提供四档级别，默认正常压缩。
- **TAR 与 TAR.GZ**：处理普通文件与目录，TAR.GZ 也支持 `.tgz` 扩展名；TAR 仅打包，TAR.GZ 提供四档级别。
- **AES-256 ZIP**：仅加密文件内容，文件名仍可见；当前可以浏览元数据，尚不支持密码解压。
- **编号分卷**：创建为 `.zip.001`、`.zip.002` 等文件，当前不能跨卷读取或解压，可使用兼容外部工具读取。

解压包含路径检查、目标授权、重名决策、暂存与内容校验。POSIX 平台的 ZIP 在限定条件下可还原相对符号链接和可执行权限，不恢复特权位；TAR 链接、Windows ZIP 链接及特殊对象不支持。具体边界见[安全解压](docs/engineering/m3-extraction.md)、[ZIP 创建](docs/engineering/m4-creation.md)与 [TAR/GZIP](docs/engineering/m4-tar-gzip.md)契约。

## 获取与安装

当前可从 [GitHub Actions](https://github.com/xiaodaozhi/xdzip/actions/workflows/qt-components.yml?query=branch%3Amain) 获取通过检查的 Windows、Linux 开发工件，或从源码构建。选择 main 分支中成功完成的运行，在页面底部 **Artifacts** 下载对应平台工件；下载可能需要登录 GitHub，工件通常保留 14 天。

| 平台 | 支持范围 | 开发交付方式 |
| --- | --- | --- |
| macOS | macOS 14+，Apple Silicon arm64 | 本地构建 `.app`，可按[部署指南](docs/engineering/macos-deployment.md)制作开发 DMG；CI 不提供应用包 |
| Windows | 仍受维护的 Windows 11，x86_64 | 当前用户安装器，或含 Qt 运行库的目录包 |
| Linux | Ubuntu 24.04 LTS，x86_64，X11，glibc 2.39+ | `.deb` 安装包，或保留执行权限的 `.tar.gz` 运行目录 |

Windows 安装器未签名；本地 macOS DMG 使用必要的 ad-hoc 完整性签名，未做 Developer ID 签名或公证。系统可能提示未验证的发布者。安装与文件关联说明见[安装指南](docs/engineering/installers-associations.md)，完整平台及维护范围见[支持政策](docs/engineering/support-policy.md)。

## 从源码构建

界面采用 **Qt Quick/QML + 薄 C++ 适配层**，格式、查询与任务由 **Rust 核心**负责，核心不依赖 Qt。

准备仓库锁定的 Rust 工具链、Qt 6.8.2、CMake 3.24+、Ninja、Python 3 与平台编译器，并将 Qt 安装前缀加入 `CMAKE_PREFIX_PATH`。克隆后构建：

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
cmake -S . -B build/ui -G Ninja -DCMAKE_BUILD_TYPE=Debug -DXDZIP_BUILD_DESKTOP_PREVIEW=ON
cmake --build build/ui
ctest --test-dir build/ui --output-on-failure
```

macOS 配置时另加 `-DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_OSX_DEPLOYMENT_TARGET=14.0`。Windows 使用 MSVC x64 开发环境。

macOS 开发与测试构建默认使用 `XDZIP_MACOS_HANDLER_RANK=None`，可正常启动和拖入文件，但不作为系统“打开方式”候选。仅为准备保留的主应用或 DMG 配置 `-DXDZIP_MACOS_HANDLER_RANK=Alternate`，避免多个测试副本出现在菜单中，详见 [Mac 部署指南](docs/engineering/macos-deployment.md)。

| 平台 | 构建后的运行入口 |
| --- | --- |
| macOS | `build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip` |
| Windows | `build/ui/apps/desktop/xdzip-browser.exe` |
| Linux | `build/ui/apps/desktop/xdzip-browser` |

更完整的依赖、离屏测试与平台说明见 [Qt 开发应用](apps/desktop/README.md)和[开发构建指南](docs/engineering/developer-build.md)。仅验证 Rust 核心可运行 `cargo test --workspace --locked`。

## 当前限制

- 当前使用单个真实压缩包会话，尚未实现多标签会话、文件内容预览和独立完整性检查命令。
- 尚不支持 7z／RAR、独立非 TAR 的 GZIP、稀疏 TAR、第三方扩展及百万条磁盘索引。
- [独立发行审查](https://github.com/xiaodaozhi/xdzip/issues/4)尚未完成，开发验证不等于独立安全审查。
- 当前不提供 Intel Mac、Windows／Linux ARM64 包，不承诺其他 Linux 发行版或 Wayland。

版本、渠道和构建身份可在“关于 xdzip”或 `xdzip --version` 查看。项目采取尽力维护，优先处理最新版本反馈，不承诺固定响应时限。

## 文档与参与

| 我想了解 | 入口 |
| --- | --- |
| 产品设计与技术架构 | [文档导航](docs/README.md)、[软件设计](docs/design/index.md)、[目录与依赖](docs/engineering/repository-layout.md) |
| 界面与交互 | [高保真原稿](design/hifi/README.md)、[可操作 HTML 设计](design/html/index.html)、[视觉规范](docs/visual/README.md) |
| 后续方向与变更 | [工作包路线](docs/planning/backlog.md)、[Issues](https://github.com/xiaodaozhi/xdzip/issues)、[变更日志](CHANGELOG.md) |
| 参与开发 | [贡献指南](CONTRIBUTING.md)、[开发指南](docs/engineering/development.md)、[行为规范](CODE_OF_CONDUCT.md) |

欢迎提交可复现的缺陷、功能建议、文档改进和代码贡献。报告问题时请附版本或提交、系统与架构、复现步骤和实际结果，样本应自行生成且不含敏感内容。

**安全问题请私下报告**，使用仓库的 [Report a vulnerability](https://github.com/xiaodaozhi/xdzip/security/advisories/new) 入口，不在公开 issue 上传漏洞样本、密码或用户文件。

## 许可证

项目原创内容采用 [Apache License 2.0](LICENSE)。Qt、编解码器、字体及其他依赖遵守各自许可证，相关来源与说明见[许可证文档](docs/engineering/licensing.md)。
