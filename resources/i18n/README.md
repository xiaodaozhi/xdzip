# 产品界面语言资源

`zh-CN.json` 与 `en-US.json` 是 Qt 产品可见静态文案的唯一来源。`messages` 的键是稳定 ID，界面通过 `qsTrId`／`qtTrId` 引用，不能按文案修改重新生成既有 ID。语义键用于设置等新增入口；`ui.<hash>` 是从原有文案迁移而来的稳定标识，内容修改仍保留 ID。

值为字符串，或含 `one`／`other` 的复数对象。Qt 的 `%n` 接收数量，`%1` 等占位符接收运行数据，两种语言必须保持参数集合一致。具体语言使用自称，`language.auto` 随界面语言翻译，`format.*` 是用户可见时间的 Qt 格式，协议时间和数值不修改。首次默认“自动识别”（`auto`），按系统首选界面语言选择中文 (简体) 或 English (US)，其他语言及无可用系统语言均回退 English (US)。选择模式与实际生效语言分别保存、解析；每次启动重新解析自动模式，手动选择与切回自动均即时生效。

English: The default selection is Auto-detect (`auto`). The preferred system UI language selects Simplified Chinese or English (US); unsupported or missing system languages fall back to English (US). The saved selection remains distinct from the resolved locale. Automatic mode resolves again at startup; manual selection and returning to automatic mode take effect immediately.

新增文案同时填写两份目录，运行 `python3 scripts/check-ui-i18n.py`。HTML 设置设计所需目录由 `python3 scripts/generate-ui-messages.py` 生成，字体覆盖由 `design/assets/fonts/build-subset.py` 检查。工程文档、高保真审阅说明、测试断言、示例归档内的文件名、用户内容和 Rust 协议标识不属于产品翻译资源。

设置与保存边界见[设置和界面语言](../../docs/ux/settings-localization.md)。运行验证报告仅生成到 build/ 或 dist/。

简体中文产品用词遵循[中文术语规范](../../docs/ux/chinese-terminology.md)与 JSON 配置。`check-ui-i18n.py` 同时拒绝旧容器称呼和旧大小标签，保留 ID、占位符、英文 archive 及用户文件名称。
