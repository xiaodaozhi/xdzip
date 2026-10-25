# 许可证决策与第三方材料

状态：已采用。项目于 2026-10-11 确立 **Apache-2.0** 许可证，适用于项目原创代码、文档及其他原创内容，另有声明的材料除外。此前实验文档中的 MIT OR Apache-2.0 不适用于本仓库的许可选择。

根目录 [LICENSE](../../LICENSE) 提供完整 Apache License 2.0 标准正文，SPDX 标识为 `Apache-2.0`。贡献指南与发行设计已同步。许可证选择不改变仓库可见性；2026-10-19 只读查询确认仓库已公开，发行准入仍按实际材料单独核对。

Qt 动态链接仍需逐模块确认 LGPL/GPL/商业条款；后端、图标、字体、工具和静态 Rust 依赖分别审查。许可证兼容不代表代码没有安全缺陷。所需通知、源码/重链接材料和安装信息随发行产物交付。

7z 使用官方 7-Zip 源码，RAR 使用 UnRAR 源码，固定来源、裁剪和受控 I/O 要求见[原生后端契约](native-codecs.md)。CAB 与 ISO/UDF 使用[独立 Rust 实现](independent-rust-codecs.md)，官方 Cab/Iso/Udf 处理器及 CAB 专用 LZX/Quantum 不进入当前构建；包含它们的历史交付仍需履行原有义务。7-Zip 按实际编译文件保留 LGPL/BSD 等上游条款，UnRAR 保留专有用途限制及修改声明。它们不适用项目原创代码的 Apache-2.0，源码移植或 Rust 包装均不改变这一点。对应源码归档、构建桥接、许可原文与 SBOM 必须随相应交付材料保留，实验构建不替代正式许可复核。

平台运行库专属通知、供应方输入和许可资格的核对遵循[原生运行库来源与通知](native-runtime-materials.md)，不以通用许可正文替代供应方版权声明。

Windows 维护者需要自有开发环境时，可采用符合条件的 Visual Studio Community 免费许可路径。GitHub 标准托管 runner 实际预装的 Enterprise 有独立的托管使用依据，CI 构建与分发不额外要求本机安装 Community IDE。适用依据、Microsoft CRT／SDK 分发条件及下游说明见 [Windows 开发工具与运行库许可](windows-licensing.md)。这些微软组件不适用项目 Apache-2.0 许可，普通运行用户无需安装 Visual Studio。

[实际组件归属流程](runtime-attribution.md)从固定 SDK 的 SPDX 提取模块、第三方、版权与上游许可表达式，并随 Windows/Linux 运行目录交付固定来源的许可文本。上游 licenseConcluded 不代表本项目已选择或批准对应许可分支；未确认归属、源码交付和重链接材料继续阻断正式发行。

三张视觉图采用生成式设计工具制作，不是 Bandizip 截图；参考链接只用于说明研究依据。生成式资产不承诺独占性。正式发行的图标、示例图片和字体需独立来源登记，不能直接从设计稿裁剪当作完成资产审计。

首个 ZIP 元数据后端曾采用 MIT 许可 rawzip，版本与通知见[历史准入记录](zip-metadata-admission.md)。当前 ZIP、TAR 和 GZIP/Deflate 采用[独立 Rust 实现](independent-rust-codecs.md)，格式参照库仅用于开发测试。旧版交付与测试参照的许可义务继续保留，通用密码学基础库按[依赖契约](dependencies.md)收集通知。项目原创许可仍为 Apache-2.0。

HTML 与 Qt 界面字体采用官方 Noto Sans SC 的衍生子集，许可为 OFL-1.1。家族名、固定来源、版本与哈希见 [字体材料清单](../../design/assets/fonts/manifest.json)和[原生 TTF 导出](../../design/assets/fonts/README.md)；完整许可为 [OFL.txt](../../design/assets/fonts/OFL.txt)。字体及其衍生字体不适用项目 Apache-2.0；离线 HTML 包含字体二进制与完整通知，Qt 分发包的 notices 包含同一 OFL 原文。产品未引入完整上游 CJK 字体。
