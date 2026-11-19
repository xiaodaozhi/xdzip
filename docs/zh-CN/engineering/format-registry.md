# 内置格式能力登记

[English](../../engineering/format-registry.md) | 简体中文

内置格式以 [archive-formats.json](../../../specs/engineering/archive-formats.json) 为登记来源，记录稳定格式标识、标签、容器族、压缩外层、既有私有工作进程编号、支持的方法族、创建后缀、检查强度及入口候选。方法族是对应格式契约的摘要，不承诺全部参数、历史变体或组合；实际条目、宿主策略和平台能力仍在任务开始前进一步收敛准入范围。

检查强度描述成功完成[完整性检查](archive-checking.md)所提供的保证。`supported_content` 校验受支持的负载，`structure_and_readability` 检查结构和可读取性，`format_declared_checks` 校验流实际携带的校验字段，也允许无校验字段的配置。能否运行检查由独立的能力字段决定。例如 NSIS 只识别有界的安装器元数据，无法执行检查。创建方法编号只在对应格式获准的写入方法中解析。

`creation_methods` 为方法名指定宿主传输编号，`creation_levels` 按格式限定级别，生成的界面选项与 Rust 描述同时携带这两项。CLI、GUI 和宿主解析同一份登记，不再假定压缩方法只能是 Deflate 或 LZMA2。Store、Copy 保留编号 0，Deflate 保留 8，LZMA2 保留 32；Zstandard 使用宿主编号 93，LZ4 使用 256，仅在其容器内解释，不代表 ZIP 方法。未提供创建能力的格式没有创建级别。改写和创建是独立能力。生成器拒绝缺失或格式内歧义的方法编号及非法级别。

| 操作 | 内置支持范围 |
| --- | --- |
| 创建新归档 | ZIP、7z、TAR、TAR.GZ（`.tar.gz`、`.tgz`）、独立 GZIP、XZ/TAR.XZ、LZ4/TAR.LZ4、Zstandard/TAR.ZST、LHA（`.lha`、`.lzh`）、ISO9660/Joliet，以及 Windows x64 的 ZIP 自解压封装。独立压缩流只接收一个普通文件。详见[归档创建](zip-creation.md)和[单文件压缩流](single-file-streams.md)。 |
| 改写现有归档 | 符合条件的 ZIP/ZIPX、7z、2 级头部 LHA/LZH、TAR、TAR.GZ、TAR.XZ、TAR.ZST、TAR.LZ4、TAR.BZ2、TAR.LZMA、TAR.LZ、TAR.Z 和 TAR.BR。加密、分卷，以及包含不支持的条目或存储形式的归档不在范围内，详见[归档修改契约](archive-transfers.md)。 |
| 浏览、解压、打开条目、导出或检查 | 每项操作分别核对格式与条目资格。例如，部分安装器只能查看元数据。详见[只读浏览](readonly-browser.md)和[完整性检查](archive-checking.md)。 |

修改规格后运行 `python3 scripts/generate-format-registry.py`，生成 Rust 格式描述、C++ 拖放与 Shell 候选、JavaScript 界面选项以及 Python 安装图标映射。`--check` 只检查，缺失或过期输出明确失败。`python3 scripts/test-format-registry.py` 检查矛盾能力、重复标识和后缀、生成语言中的非法值及过期消费者；日常 CI 执行这两项检查。

打开时先按[归档打开契约](archive-opening.md)处理入口和来源，再探测并校验已授权的字节。文件后缀或 MIME 类型只提供入口。宿主仍须检查来源身份、操作参数、输出授权和最终提交。`worker_id` 用于选择内置的工作进程读取路径。例如，ISO 读取经工作进程执行，新 ISO 映像由 Rust 宿主写入。登记表分别记录创建和修改现有归档两种能力。

拖放与 Shell 候选按需分别登记。普通 `.gz` 现提供两个入口及单文件创建，`.tar.gz` 和 `.tgz` 保留 TAR 容器语义。XPI/APK/IPA/CBZ 候选沿用 ZIP 校验和已准入组合，见[单文件压缩流](single-file-streams.md)。分卷命名和源授权继续遵守[归档打开契约](archive-opening.md)，登记表不重新解释编号文件。Windows 按最终扩展名映射图标，`.tar.gz` 使用 `gz` 关联。

只登记已实现且具有对应自动化验收的能力。实现变化按需同步[自研格式契约](independent-rust-codecs.md)、[格式工作进程契约](native-codecs.md)、用户能力说明及[验收条目](../../../developers/zh-CN/qa/acceptance.md)。计划、进度和执行结论保留在 issue，生成测试证据只放在 `build/` 或 CI 工件。

`creation_wrappers` 声明 Windows x64 的 ZIP SFX 封装与显式 `.sfx.exe` 后缀，复用 ZIP 方法编号，禁用加密和分卷，并且不注册通用 EXE 关联。ISO9660 创建使用宿主方法编号 260 和固定级别 0。详见 [ISO 创建](iso-creation.md)与[自有 SFX 创建](sfx-creation.md)。
