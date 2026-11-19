# 内置格式能力登记

[English](../../engineering/format-registry.md) | 简体中文

内置格式以 [archive-formats.json](../../../specs/engineering/archive-formats.json) 为登记来源，记录稳定格式标识、标签、容器族、压缩外层、既有私有工作进程编号、支持的方法族、创建后缀、检查强度及入口候选。方法族是对应格式契约的摘要，不承诺全部参数、历史变体或组合；实际条目、宿主策略和平台能力仍在任务开始前进一步收敛准入范围。

检查强度分为三类：`supported_content` 验证支持的内容，`structure_and_readability` 不承诺内容校验和，`format_declared_checks` 验证流声明的校验，但允许格式中无校验和的配置。生成的界面说明和 CLI 结果须保留此区别。创建方法编号只能解析到该格式已准入的写方法，登记读过滤器不能隐式启用写入。

`creation_methods` 为方法名指定宿主传输编号，`creation_levels` 按格式限定级别，生成的界面选项与 Rust 描述同时携带这两项。CLI、GUI 和宿主解析同一份登记，不再假定压缩方法只能是 Deflate 或 LZMA2。Store、Copy 保留编号 0，Deflate 保留 8，LZMA2 保留 32；Zstandard 使用宿主编号 93，LZ4 使用 256，仅在其容器内解释，不代表 ZIP 方法。只读配置没有创建级别。生成器拒绝缺失或格式内歧义的方法编号及非法级别。

修改规格后运行 `python3 scripts/generate-format-registry.py`，生成 Rust 格式描述、C++ 拖放与 Shell 候选、JavaScript 界面选项以及 Python 安装图标映射。`--check` 只检查，缺失或过期输出明确失败。`python3 scripts/test-format-registry.py` 检查矛盾能力、重复标识和后缀、生成语言中的非法值及过期消费者；日常 CI 执行这两项检查。

登记表不授予文件、输出或执行权限。读取探测和容器校验仍通过有界授权流进行，后缀与 MIME 只提供操作入口。宿主独立校验输出文件名、任务参数、源身份和最终提交。只读格式不能因新增扩展名获得创建或重写能力。桌面只读说明使用无写入能力格式的生成标签，新增读取器无需维护另一份硬编码格式列表。既有私有工作进程编号保持不变，格式描述不加载外部后端，也不协商第三方扩展。

拖放与 Shell 候选按需分别登记。普通 `.gz` 现提供两个入口及单文件创建，`.tar.gz` 和 `.tgz` 保留 TAR 容器语义。XPI/APK/IPA/CBZ 候选沿用 ZIP 校验和已准入组合，见[单文件压缩流](single-file-streams.md)。分卷命名和源授权继续遵守[归档打开契约](archive-opening.md)，登记表不重新解释编号文件。Windows 按最终扩展名映射图标，`.tar.gz` 使用 `gz` 关联。

只登记已实现且具有对应自动化验收的能力。实现变化按需同步[自研格式契约](independent-rust-codecs.md)、[格式工作进程契约](native-codecs.md)、用户能力说明及[验收条目](../../../developers/zh-CN/qa/acceptance.md)。计划、进度和执行结论保留在 issue，生成测试证据只放在 `build/` 或 CI 工件。

`creation_wrappers` 声明仅限 Windows 的 ZIP SFX 封装及显式 `.sfx.exe` 后缀，复用 ZIP 方法编号，并关闭加密和分卷，不登记通用 EXE 处理。ISO9660 创建使用宿主方法编号 260 和固定级别 0。详见 [ISO 创建](iso-creation.md)与[自有 SFX 创建](sfx-creation.md)。
