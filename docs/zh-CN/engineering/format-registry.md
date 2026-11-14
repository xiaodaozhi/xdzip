# 内置格式能力登记

内置格式以 [archive-formats.json](../../../specs/engineering/archive-formats.json) 为登记来源，记录稳定格式标识、标签、容器族、压缩外层、既有私有工作进程编号、支持的方法族、创建后缀及入口候选。方法族是对应格式契约的摘要，不承诺全部参数、历史变体或组合；实际条目、宿主策略和平台能力仍在任务开始前进一步收敛准入范围。

修改规格后运行 `python3 scripts/generate-format-registry.py`，生成 Rust 格式描述、C++ 拖放与 Shell 候选以及 Python 安装图标映射。`--check` 只检查，缺失或过期输出明确失败。`python3 scripts/test-format-registry.py` 检查矛盾能力、重复标识和后缀、生成语言中的非法值及过期消费者；日常 CI 执行这两项检查。

登记表不授予文件、输出或执行权限。读取探测和容器校验仍通过有界授权流进行，后缀与 MIME 只提供操作入口。宿主独立校验输出文件名、任务参数、源身份和最终提交。只读格式不能因新增扩展名获得创建或重写能力。既有私有工作进程编号保持不变，格式描述不加载外部后端，也不协商第三方扩展。

拖放与 Shell 候选有意分别登记。例如，既有 `.gz` Shell 候选能够进入 TAR.GZ 读取链路，但普通非 TAR 的 GZIP 仍不支持，`.gz` 也不是独立创建后缀。分卷命名和源授权继续遵守[归档打开契约](archive-opening.md)，登记表不重新解释编号文件。Windows 按最终扩展名映射图标，`.tar.gz` 使用 `gz` 关联。

只登记已实现且具有对应自动化验收的能力。实现变化按需同步[自研格式契约](independent-rust-codecs.md)、[格式工作进程契约](native-codecs.md)、用户能力说明及[验收条目](../qa/acceptance.md)。计划、进度和执行结论保留在 issue，生成测试证据只放在 `build/` 或 CI 工件。
