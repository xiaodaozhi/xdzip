# 产品界面语言资源

`zh-CN.json` 与 `en-US.json` 是 Qt 产品可见静态文案的唯一来源。`messages` 的键是稳定 ID，界面通过 `qsTrId`／`qtTrId` 引用，不能按文案修改重新生成既有 ID。语义键用于设置等新增入口；`ui.<hash>` 是从原有文案迁移而来的稳定标识，内容修改仍保留 ID。

值为字符串，或含 `one`／`other` 的复数对象。Qt 的 `%n` 接收数量，`%1` 等占位符接收运行数据，两种语言必须保持参数集合一致。`language.*` 使用语言自称，`format.*` 是用户可见时间的 Qt 格式，协议时间和数值不修改。首次默认为简体中文，未知语言回退简体中文，保存的语言在下次启动时加载。

新增文案同时填写两份目录，运行 `python3 scripts/check-ui-i18n.py`。HTML 设置设计所需目录由 `python3 scripts/generate-ui-messages.py` 生成，字体覆盖由 `design/assets/fonts/build-subset.py` 检查。工程文档、高保真审阅说明、测试断言、示例归档内的文件名、用户内容和 Rust 协议标识不属于产品翻译资源。

设置与保存边界见[设置和界面语言](../../docs/ux/settings-localization.md)。运行验证报告仅生成到 build/ 或 dist/。
