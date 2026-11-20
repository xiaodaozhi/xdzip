# LZ4 压缩流契约

[English](../../engineering/lz4-streams.md) | 简体中文 | [繁體中文](../../zh-TW/engineering/lz4-streams.md) | [日本語](../../ja-JP/engineering/lz4-streams.md) | [한국어](../../ko-KR/engineering/lz4-streams.md) | [Français](../../fr-FR/engineering/lz4-streams.md) | [Deutsch](../../de-DE/engineering/lz4-streams.md) | [Español](../../es-ES/engineering/lz4-streams.md) | [Português (Brasil)](../../pt-BR/engineering/lz4-streams.md) | [Русский](../../ru-RU/engineering/lz4-streams.md)

标准帧配置依据作者的[帧格式 1.6.5](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Frame_format.md)、[块格式，2026-10-26](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Block_format.md)及 [XXH32 算法规格](https://github.com/Cyan4973/xxHash/blob/v0.8.3/doc/xxhash_spec.md)。产品算法使用独立 Rust 实现，参考工具和运行证据留在本地 `build/`，只对自有、可重现的输入模式进行版本管理。宿主授权和提交边界见[单文件压缩流](single-file-streams.md)。

## 准入与帧结构

标准版本 1 帧验证保留位、BD 取值 4–7、头部 XXH32、可选内容尺寸、声明的逐块及内容 XXH32，以及结束标记。块数据长度不得超过声明的 64 KiB–4 MiB 上限。独立块重置历史，关联块仅保留前 64 KiB。原始块和零长度原始块同样执行校验与计量。外部字典配置和旧式帧不支持，字典声明不授权读取其他文件。

串联标准帧及中间的 skippable 帧受到物理输入、任务预算、取消和十万帧／块策略的约束。成功的独立文件必须包含至少一个标准数据帧，任意尾随内容和仅有 skippable 帧的输入不能成功。发现阶段最多实际读取 8 MiB 压缩输入以产生 512 字节内容前缀。强识别或已识别的 TAR 损坏不能因失败降级为其他格式，完整扫描验证全部帧后才能发布实际大小和虚拟条目。

## 块处理与创建

长度运算须检查溢出。匹配偏移必须非零且位于有效历史内，重叠字节按递增顺序复制，并满足声明的输出上限。最后一个序列只有字面量，其未使用的匹配半字节忽略。含匹配的流至少保留五个末尾字面量，最后一次匹配须在块的解码末尾前至少十二字节开始。读取按有界块交付，失败后不可继续；仅精确验证块 EOF 后允许移交历史。宿主 I/O、取消、超限和不支持配置须保留不同错误类别。

已准入写入器须产生真实 LZ4 匹配，仅在压缩不能减小时使用原始块，并提供逐块与内容校验和。提供的级别选择有界匹配工作量，每个级别均须用独立工具验证多块内容。分配前准入帧与匹配缓冲，创建仍执行冻结输入精确尺寸、宿主暂存、完整验证及最终源和输出检查。普通 LZ4 仅接受一个普通输入文件；TAR 组合只有在共享 TAR 解析、写入及相应自动化覆盖后才能登记。后缀登记不授予修改、加密或分卷能力。

检查使用 `format_declared_checks`，因为输入可省略内容校验和。不得把无内容校验和的帧描述为具备内容验证保证。资源限制、实际平台验收、格式有效性和正式发行许可须分别记录。
