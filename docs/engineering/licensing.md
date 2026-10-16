# 许可证决策与第三方材料

状态：已采用。项目于 2026-10-11 确立 **Apache-2.0** 许可证，适用于项目原创代码、文档及其他原创内容，另有声明的材料除外。此前实验文档中的 MIT OR Apache-2.0 不适用于本仓库的许可选择。

根目录 [LICENSE](../../LICENSE) 提供完整 Apache License 2.0 标准正文，SPDX 标识为 `Apache-2.0`。贡献指南与发行设计已同步。许可证选择不改变仓库可见性；当前仍为私有筹备阶段。

Qt 动态链接仍需逐模块确认 LGPL/GPL/商业条款；后端、图标、字体、工具和静态 Rust 依赖分别审查。许可证兼容不等于安全审计通过。所需通知、源码/重链接材料和安装信息随发行产物交付。

[实际组件归属流程](runtime-attribution.md)从固定 SDK 的 SPDX 提取模块、第三方、版权与上游许可表达式，并随 Windows/Linux 运行目录交付固定来源的许可文本。上游 licenseConcluded 不代表本项目已选择或批准对应许可分支；未确认归属、源码交付和重链接材料继续阻断正式发行。

三张视觉图采用生成式设计工具制作，不是 Bandizip 截图；参考链接只用于说明研究依据。生成式资产不承诺独占性。正式发行的图标、示例图片和字体需独立来源登记，不能直接从设计稿裁剪当作完成资产审计。

首个 ZIP 元数据后端采用 MIT 许可 rawzip，版本与通知见 [准入记录](zip-metadata-admission.md)。项目原创许可仍为 Apache-2.0。

HTML 与 Qt 界面字体采用官方 Noto Sans SC 的衍生子集，许可为 OFL-1.1。家族名、固定来源、版本与哈希见 [字体材料清单](../../design/assets/fonts/manifest.json)和[原生 TTF 导出](../../design/assets/fonts/README.md)；完整许可为 [OFL.txt](../../design/assets/fonts/OFL.txt)。字体及其衍生字体不适用项目 Apache-2.0；离线 HTML 包含字体二进制与完整通知，Qt 分发包的 notices 包含同一 OFL 原文。产品未引入完整上游 CJK 字体。
