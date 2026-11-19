# 构建并参与贡献

[English](../../contributing/README.md) | 简体中文

1. 阅读[贡献约定](../../../CONTRIBUTING.md)，查找已有议题或新建议题。说明希望改变的行为、适用平台以及可实际验证的小场景。
2. 按[构建指南](../engineering/developer-build.md)准备锁定工具。先运行 `python3 scripts/desktop.py doctor`，再构建和测试桌面目标。CLI 改动使用独立的 `scripts/cli.py` 入口。Windows 下按指引在 MSVC 环境中使用 `python`。
3. 从下表中的契约和源码入口入手。行为改动应使用自制样本或能揭示缺陷的定向测试。
4. 在 PR 中说明改了什么、测试覆盖了哪些场景和平台，以及仍待验证的部分。同时更新受影响的两种语言文档。

| 改动类型 | 源码与规范入口 | 验证方式 |
| --- | --- | --- |
| 界面、设置或翻译 | 查看 `apps/desktop/ui`、`resources/i18n` 与 `specs/visual`。修改窗口前阅读[视觉规范](../visual/design-standard.md)和[设计流程](../visual/design-workflow.md)。 | 执行对应组件和本地化检查。涉及原生窗口行为时，在相关系统实际操作受影响窗口并单独记录。 |
| 压缩格式 | 查看 `specs/engineering/archive-formats.json`、对应的 `xdzip-codec-*` 包以及运行时路由，再阅读[格式注册表契约](../engineering/format-registry.md)。 | 用自制样本核对识别、声明的操作和至少一类格式特有的拒绝场景。注册表变动后重新生成并核对产物。 |
| 归档任务 | 从 `BrowserController`、`RuntimeConnection` 追到 `xdzip-runtime` 与 `xdzip-io`，查阅[运行时](../engineering/product-runtime.md)和[文件安全](../design/file-security.md)契约。 | 核对受影响的结果和失败路径，例如取消、重名和已保存文件。调用真实工作进程时参照[运行时工作进程测试](../../../developers/zh-CN/qa/runtime-worker-tests.md)。 |

查看包结构和运行边界，请进入[技术参考](../reference/README.md)。团队的 CI 与发行流程见[维护者手册](../../../developers/zh-CN/README.md)。
