# 许可证决策与第三方材料

状态：已采用。项目于 2026-10-11 确立 **Apache-2.0** 许可证，适用于项目原创代码、文档及其他原创内容，另有声明的材料除外。此前实验文档中的 MIT OR Apache-2.0 不适用于本仓库的许可选择。

根目录 [LICENSE](../../../LICENSE) 提供完整 Apache License 2.0 标准正文，SPDX 标识为 `Apache-2.0`。贡献指南与发行设计已同步。许可证选择不改变仓库可见性；2026-10-19 只读查询确认仓库已公开，发行准入仍按实际材料单独核对。

Qt 动态链接仍需逐模块确认 LGPL/GPL/商业条款；后端、图标、字体、工具和静态 Rust 依赖分别审查。许可证兼容不代表代码没有安全缺陷。所需通知、源码/重链接材料和安装信息随发行产物交付。

7z、RAR、CAB 与 ISO/UDF 使用[独立 Rust 实现](independent-rust-codecs.md)，官方 UnRAR、7-Zip 源码不进入当前产品构建或部署。固定路由与受控 I/O 见[隔离后端契约](native-codecs.md)。分析来源、开发参考工具、历史官方后端交付分别保留原许可，独立实现不构成严格洁净室或法律风险清零声明。包含官方 7-Zip 或 UnRAR 的历史交付继续履行各自原始条款、通知和修改材料义务，不能因当前退出依赖而改标 Apache-2.0。当前产品源码清单为空不免除历史交付或独立参考工具的义务，实验构建不替代正式许可复核。

RAR 的有限线格式映射及公开测试夹具保留各自的来源、原始通知和许可记录，见[RAR 实现依据](rar-format-sources.md)。生产引擎退出不将这些材料重新许可为项目原创代码。

有限映射另保留[固定参考原文与来源说明](../../../third_party/rar-wire-mappings/README.md)，包括上游内部版本 UnRAR 7.30 beta1 的原始完整许可、下载归档 `unrarsrc-7.3.1.tar.gz` 及原文 SHA-256。通知收集器将这些有界材料纳入实际通知哈希，对应源码交付也保留独立副本；它们不是重新引入官方引擎或声明其为运行依赖，不自动裁定条款适用范围，也不批准本候选发行。

固定原文与 SPDX 的 `UnRAR` 许可文本统一空白后逐词一致。为反映参考映射的保守处理，RAR crate 及其源码 SBOM 声明 `Apache-2.0 AND UnRAR`，并非可任选的双许可；该 crate 不发布到 crates.io。`UnRAR` 是 SPDX 3.28 及以后清单中的非 OSI 批准许可标识，较旧工具可能不识别；原许可和限制仍应随材料保留。该声明不裁定映射的版权性质，原版 alpha.8 清单也不会因此被改写。

English: Current worker formats use independent Rust; official UnRAR and 7-Zip source code is absent from product builds and deployment. Historical deliveries, finite RAR wire mappings, public fixtures and separate reference tools retain their respective provenance and licensing obligations. The finite-mapping notice set retains the exact original license for upstream internal version UnRAR 7.30 beta1, from the pinned archive unrarsrc-7.3.1.tar.gz, in package notices and corresponding-source delivery. The notice matches the SPDX `UnRAR` text after whitespace normalization. As a conservative package-level statement, the RAR crate and source SBOM use `Apache-2.0 AND UnRAR`, requiring both rather than offering a choice; the crate is not published to crates.io. The non-OSI-approved `UnRAR` identifier appears in SPDX 3.28 and later, so older tools may not recognize it. This does not decide the mappings' copyright status or alter the alpha.8 inventory. These bounded materials neither reintroduce the official engine nor constitute clean-room certification, legal clearance or release approval.

平台运行库专属通知、供应方输入和许可资格的核对遵循[原生运行库来源与通知](native-runtime-materials.md)，不以通用许可正文替代供应方版权声明。

Windows 维护者需要自有开发环境时，可采用符合条件的 Visual Studio Community 免费许可路径。GitHub 标准托管 runner 实际预装的 Enterprise 有独立的托管使用依据，CI 构建与分发不额外要求本机安装 Community IDE。适用依据、Microsoft CRT／SDK 分发条件及下游说明见 [Windows 开发工具与运行库许可](windows-licensing.md)。这些微软组件不适用项目 Apache-2.0 许可，普通运行用户无需安装 Visual Studio。

[实际组件归属流程](runtime-attribution.md)从固定 SDK 的 SPDX 提取模块、第三方、版权与上游许可表达式，并随 Windows/Linux 运行目录交付固定来源的许可文本。上游 licenseConcluded 不代表本项目已选择或批准对应许可分支；每个候选均须按实际组件核对来源、通知、对应源码和替换材料。已识别且适用的义务或必需材料若未解决，会阻断该候选发行；上游 NOASSERTION 或机器报告 incomplete 本身不能直接推导为许可冲突，也不能当作豁免。

三张视觉图采用生成式设计工具制作，不是 Bandizip 截图；参考链接只用于说明研究依据。生成式资产不承诺独占性。正式发行的图标、示例图片和字体需独立来源登记，不能直接从设计稿裁剪当作完成资产审计。

首个 ZIP 元数据后端曾采用 MIT 许可 rawzip，版本与通知见[历史准入记录](zip-metadata-admission.md)。当前 ZIP、TAR 和 GZIP/Deflate 采用[独立 Rust 实现](independent-rust-codecs.md)，格式参照库仅用于开发测试。旧版交付与测试参照的许可义务继续保留，通用密码学基础库按[依赖契约](dependencies.md)收集通知。项目原创许可仍为 Apache-2.0。

HTML 与 Qt 固定界面文案采用官方 Noto Sans SC 的衍生子集，许可为 OFL-1.1。家族名、固定来源、版本与哈希见 [字体材料清单](../../../design/assets/fonts/manifest.json)和[原生 TTF 导出](../../../design/assets/fonts/README.md)；完整许可为 [OFL.txt](../../../design/assets/fonts/OFL.txt)。字体及其衍生字体不适用项目 Apache-2.0；离线 HTML 包含字体二进制与完整通知，Qt 分发包的 notices 包含同一 OFL 原文。产品未引入完整上游 CJK 字体。
