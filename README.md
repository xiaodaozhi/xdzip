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

![xdzip 主界面设计预览](design/hifi/screens/main-capsule-light.png)

<p align="center"><sub>主界面高保真设计参考。图中的图片预览为设计示例，当前产品能力见下文。</sub></p>

## 功能亮点

- **清晰地浏览内容**：目录树、列表与网格、搜索、全结果排序和文件属性，支持最多十万条物理记录。
- **按需要解压**：选择文件或文件夹，确认目标与重名策略，查看真实进度，随时请求取消；已完成的文件有明确结果记录。
- **文件拖放**：文件列表右键打开与双击一致，支持拖出所选文件和目录，拖入本地文件后确认添加到当前目录，以及将所选项目拖到列表文件夹或目录树进行包内移动。普通单文件 ZIP、TAR、TAR.GZ、7z 使用完整校验后的安全替换，加密与分卷包暂不支持添加和移动，RAR、CAB、ISO、UDF 只读，并显示拒绝原因。平台传输与验收边界见[文件拖放契约](docs/engineering/archive-transfers.md)。
- **创建压缩包**：支持 ZIP、TAR、TAR.GZ 与 7z，提供压缩级别、ZIP／7z AES-256 内容加密和编号分卷，7z 还可选择固实压缩与文件名加密。自动排除各层级 `.DS_Store` 文件和 `__MACOSX`／`_MACOSX` 目录，写后校验通过后保存。
- **测试压缩包**：后台检查整包结构和受支持的内容校验，支持密码与取消，显示真实结果，全程不写出文件；TAR、ISO、UDF 仅检查结构与可读性，不提供逐文件内容校验和保证。
- **按习惯打开**：默认新标签页，也可设置为新窗口；每个登录会话共用一个进程和托盘，关闭页面后任务继续运行。
- **适应你的桌面**：浅色、深色与跟随系统主题，100%／125%／150% 字号、列表密度、中文 (简体) 与 English (US)，配合 macOS 系统菜单和全平台托盘。

打开压缩包先浏览内容，不自动解压。双击其中的普通文件会安全解压到私有临时目录，再通过系统关联打开；系统接受打开请求后自动隐藏进度窗口。临时副本的修改不会写回压缩包，缓存保留七天后在后续打开文件时清理。创建成功后在应用内打开可浏览的结果，不自动弹出 Finder 或 Explorer。

## 格式支持

| 格式 | 浏览 | 解压 | 创建 |
| --- | --- | --- | --- |
| ZIP | ✓ | ✓ | ✓ |
| TAR | ✓ | ✓ | ✓ |
| TAR.GZ | ✓ | ✓ | ✓ |
| AES-256 ZIP | ✓ | ✓ | ✓ |
| 分卷 ZIP | ✓ | ✓ | ✓ |
| 7z | ✓ | ✓ | ✓ |
| RAR | ✓ | ✓ | — |
| CAB | ✓ | ✓ | — |
| ISO | ✓ | ✓ | — |
| UDF | ✓ | ✓ | — |

✓ 已支持，— 尚不支持。

- **ZIP**：容器、AES 封装和修改使用自研 Rust 后端，Deflate 与 TAR.GZ 共享一套实现；支持 ZIP64，解压与创建支持 Store／Deflate，创建提供四档级别，默认正常压缩。实现来源与迁移边界见[自研后端契约](docs/engineering/independent-rust-codecs.md)。
- **TAR 与 TAR.GZ**：处理普通文件与目录，TAR.GZ 也支持 `.tgz` 扩展名；TAR 容器及 GZIP/Deflate 使用自研 Rust 后端，TAR 仅打包，TAR.GZ 提供四档级别。迁移边界见[自研后端契约](docs/engineering/independent-rust-codecs.md)。
- **AES-256 ZIP**：WinZip AE-1／AE-2，Store／Deflate，支持密码解压、错误重试和取消；仅加密文件内容，文件名仍可见。AES-128／192、传统 ZipCrypto 尚不支持，详见[密码解压契约](docs/engineering/password-extraction.md)。
- **分卷 ZIP**：读取连续编号 `.zip.001`、`.zip.002` 等，以及 PKZIP `.z01`、`.z02`、最终 `.zip`。将全部卷放在同一目录后可打开任意一卷，最多 1024 卷；支持 Store／Deflate 和 AES-256。创建输出为编号分卷。详见[分卷与临时打开契约](docs/engineering/archive-opening.md)。
- **7z**：使用自研 Rust 后端，支持密码、文件名加密、固实及 `.7z.001` 编号分卷。创建提供四档级别，默认正常级别 LZMA2、非固实、不加密、不分卷，级别 0 仅存储。未加密、非分卷且条目受支持的包可添加、替换和包内移动；修改时重建，保留与新增文件内容的工作集上限为 512 MiB。新建使用流式输入，不适用此重建工作集限制。详见[隔离后端契约](docs/engineering/native-codecs.md)。
- **RAR**：使用自研 Rust 只读后端，处理旧版 RAR、RAR5 及 RAR 7 格式扩展，支持密码、文件名加密、固实和新旧分卷命名，不提供创建或修改。生产构建不编译 UnRAR 源码；历史版本与过滤器的实际覆盖、未知尺寸等兼容边界见[RAR 实现契约](docs/engineering/rar-format-sources.md)。RAR 7 算法 1 可用精确分页历史处理大逻辑字典，保持 worker 的 1 GiB 内存上限，实际页工作集超预算仍明确拒绝；独立健康压缩流的完整验收见[任务 #43](https://github.com/xiaodaozhi/xdzip/issues/43)。完整迁移验收由[任务 #51](https://github.com/xiaodaozhi/xdzip/issues/51)跟踪。
- **CAB**：使用自研 Rust 只读后端，支持 Store、MSZip、Quantum、LZX 和同目录分卷；缺卷或不匹配卷拒绝打开。可浏览、搜索、解压、测试、临时打开普通文件及导出，不提供创建或修改，具体边界见[自研后端契约](docs/engineering/independent-rust-codecs.md#cab只读规格)。
- **ISO 与 UDF**：使用自研 Rust 只读后端，在隔离工作进程中处理 ISO9660、Joliet、Rock Ridge 与 UDF，混合映像优先使用 UDF 命名空间。可浏览、搜索、解压、临时打开普通文件、导出及检查结构与可读性，不挂载或执行光盘映像，不提供创建或修改。链接及特殊记录不可输出，已识别的 UDF 损坏不会静默回退 ISO。打开筛选及关联支持 `.iso`、`.udf`，不注册通用 `.img`，实现边界与尚需验证的扩展见[自研后端契约](docs/engineering/independent-rust-codecs.md#iso与udf只读规格)及[迁移任务](https://github.com/xiaodaozhi/xdzip/issues/47)。

创建和加入文件会排除上述 macOS 元数据，过滤后没有有效内容则失败，原文件和已有压缩包保持不变。新写入的多语言名称使用 UTF-8；读取历史包会优先采用 Unicode 声明，再自适应 GBK/GB18030、Big5、Shift-JIS 等旧编码，并贯通浏览、解压与普通包修改。短名称或混合旧编码无法保证唯一识别，损坏声明及异常路径继续拒绝输出，详见[文件名编码契约](docs/engineering/filename-encoding.md)。

解压包含路径检查、目标授权、重名决策、暂存与受支持的内容校验。POSIX 平台的 ZIP 在限定条件下可还原相对符号链接和可执行权限，不恢复特权位；TAR、7z、RAR、CAB、ISO、UDF 链接、Windows ZIP 链接及特殊对象不支持。具体边界见[安全解压](docs/engineering/m3-extraction.md)、[ZIP 创建](docs/engineering/m4-creation.md)、[TAR/GZIP](docs/engineering/m4-tar-gzip.md)与[隔离格式后端](docs/engineering/native-codecs.md)契约。

## 获取与安装

预发布通过 [GitHub Releases](https://github.com/xiaodaozhi/xdzip/releases)提供六类下载包与校验清单，公开前仍保留为草稿。Windows 包保留应用目录内 VC++ 运行库和图形兼容库，Mac 包仅 ad-hoc 完整性签名。发行材料要求与更新协议见[发布说明](docs/engineering/release-protocol.md)。

当前也可从 [GitHub Actions](https://github.com/xiaodaozhi/xdzip/actions/workflows/qt-components.yml?query=branch%3Amain) 获取通过检查的 Windows、Linux 开发工件，或从源码构建。选择 main 分支中成功完成的运行，在页面底部 **Artifacts** 下载对应平台工件；下载可能需要登录 GitHub，工件通常保留 14 天。

| 平台 | 支持范围 | 开发交付方式 |
| --- | --- | --- |
| macOS | macOS 14+，Apple Silicon arm64 | 本地构建 `.app`，可按[部署指南](docs/engineering/macos-deployment.md)制作开发 DMG；普通 CI 不提供应用包，专用发布流程提供 app.zip 与 DMG |
| Windows | 仍受维护的 Windows 11，x86_64 | 当前用户安装器，或含 Qt 运行库的目录包 |
| Linux | Ubuntu 24.04 LTS，x86_64，X11，glibc 2.39+ | `.deb` 安装包，或保留执行权限的 `.tar.gz` 运行目录 |

Windows 安装器未签名；本地 macOS DMG 使用必要的 ad-hoc 完整性签名，未做 Developer ID 签名或公证。系统可能提示未验证的发布者。安装与文件关联说明见[安装指南](docs/engineering/installers-associations.md)，完整平台及维护范围见[支持政策](docs/engineering/support-policy.md)。

Windows 和 macOS 为受支持的文件类型提供对应文档图标，Linux 沿用系统文件类型图标。安装不更改默认应用；Windows 用户若以前已将 xdzip 设为默认，可能需要重新选择对应格式才能看到新图标。系统对 `.tar.gz` 和 `.tgz` 的区分及图标缓存因平台而异。

“关于 xdzip”提供用户主动检查更新与下载，安装版的更新入口需有效的发布签名和安装身份。生产更新公钥随客户端和安装助手内置，不自动覆盖未验证的安装；便携包和开发构建使用手动安装。状态、隐私和三平台交接边界见[更新契约](docs/engineering/software-update.md)。

## 从源码构建

界面采用 **Qt Quick/QML + 薄 C++ 适配层**，格式、查询与任务由 **Rust 核心**负责，核心不依赖 Qt。

使用 Python 3.10+、Git、CMake 3.24+、Ninja、rustup 和平台编译器／SDK。统一入口负责准备仓库锁定的 Rust 与 Qt 6.8.2，先检查环境，再构建和测试：

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
python3 scripts/desktop.py prepare --install-rust --install-qt --qt-root .cache/Qt
python3 scripts/desktop.py doctor
python3 scripts/desktop.py build
python3 scripts/desktop.py test
```

Windows 在 MSVC x64 开发环境运行，可将 `python3` 换为 `python`；macOS 使用 Apple Silicon 原生终端与有效 Xcode Command Line Tools。已有 Qt 时用 `--qt-prefix` 指定 SDK 前缀，无需重新下载。

同一入口提供 `doctor`、`prepare`、`build`、`test`、`package` 和 `verify`，各阶段独立执行。日常构建允许未提交改动，打包要求干净源码与对应构建身份。环境检查、参数、平台包及完整操作顺序统一见[开发构建指南](docs/engineering/developer-build.md)。单独执行 Rust workspace 测试时，也须先构建隔离格式工作进程并通过 `XDZIP_NATIVE_CODEC` 指定工作进程，再运行 `cargo test --workspace --locked`；统一 `test` 入口会自动提供该路径。

默认运行入口为 `build/desktop/apps/desktop/xdzip`，Windows 加 `.exe`，macOS 为该目录下 `xdzip.app/Contents/MacOS/xdzip`。普通 Mac 开发／测试副本不进入系统“打开方式”候选；主应用与本地开发 DMG 的构建方式见[Mac 部署指南](docs/engineering/macos-deployment.md)。

## 当前限制

- 支持独立标签页或多窗口会话，文件内容预览尚未实现。完整性检查的支持与限制见[检查契约](docs/engineering/archive-checking.md)。
- 尚不支持 RAR、CAB、ISO、UDF 创建与修改、加密或分卷 7z 修改、独立非 TAR 的 GZIP、稀疏 TAR、第三方扩展及百万条磁盘索引。7z 修改重建受上述 512 MiB 工作集限制。
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

Windows 包中的 Microsoft 运行库保留各自原始条款，安装器展示 VC++ 运行库协议，便携包附带同一文本与双语说明。普通用户无需安装 Visual Studio；开发者的 Community 免费许可、GitHub 托管构建依据及再分发要求见 [Windows 许可说明](docs/engineering/windows-licensing.md)。

组件来源与再分发义务按实际发布包逐版核对，复核记录见 [#4](https://github.com/xiaodaozhi/xdzip/issues/4)，不因源码开源或 CI 通过而自动批准后续发行。
