# xdzip 仓库工作约定

开始前阅读 README.md、docs/README.md 和当前任务涉及的规范。当前基线为 Qt Quick + 薄 C++ + Rust；高保真原稿统一位于 design/hifi；视觉规范与 tokens 位于 docs/visual。框架选型变更须通过架构决策流程。

界面创作与修正必须先读 docs/visual/design-standard.md、docs/visual/design-workflow.md；密度与单位以 docs/visual/layout-scale.json 为准。主窗口和菜单必须同时测量比例，不得把 DPR、浏览器缩放或截图物理像素写入组件逻辑尺寸。

M1 产品骨架、M2 ZIP／ZIP64 只读、M3 Store／Deflate 安全解压已完成开发与三平台自动化验证。WP-10 ZIP 创建已实现输入树、Store／Deflate、写后校验、整体提交与真实创建窗口，实现契约见 docs/engineering/m4-creation.md，运行证据只保留为本地／CI 工件；下一工作包为 WP-11 TAR/GZIP。桌面人工验收与独立安全审查单独登记，当前开发包保持实验性。阶段状态以开发就绪评审和验收证据为准。

- 保持领域和格式逻辑独立于 Qt；QML 只表达界面状态和用户意图。
- 不导入旧实验工作区代码、第三方截图、凭据、用户归档或大型构建产物。
- 所有层级的 dist/ 及其内容均不提交到 Git；build/、target/ 也只存放本地或 CI 生成产物。
- macOS 保留 Apple Silicon 原生编译、测试与启动验证；经用户明确要求，可在本地部署 Qt 并生成开发 DMG，允许必需的 ad-hoc 完整性签名，不冒充 Developer ID 或公证。CI 仍不部署、签名、压缩或上传 Apple Silicon 应用包，仅保留日志与 JSON 验证记录。
- 修改领域、桥接、格式、文件写入或任务语义时，同时更新规范和对应验收条目。
- 不假装测试、签名、平台验证或独立安全审查已经通过；记录实际证据和限制。
- 所有 unsafe/FFI/文件提交解释前提和失败行为；避免重复代码的注释。
- 验证范围与变更匹配；文档改动检查链接和生成输出，产品改动按 CI 策略执行相关检查。
- 不在日志、提交或 issue 中输出凭据及完整用户路径。保留工作区已有改动。
- 默认中文设计说明，公共 API 标识使用英文；对外英文文档应与中文版本一致。
- 文本信息使用逗号、句号或换行分隔，不使用中点。测试截图、量测数据和验证报告只生成到 build/ 或 dist/，不提交到 Git；设计原稿、素材、规格和测试代码继续纳入版本管理。
- 操作窗口使用共享 OperationButton、OperationFooterButton、OperationFrame、OperationSelect 和 OperationSegmentedGroup；禁止在窗口中覆盖 footer 按钮的圆角、字号、颜色、描边和密度。

后续开发在 main 推进；本地相关检查通过后直接推送 main，不另建 PR。共享 main 不强制推送；遇到远端新提交先同步并复验。

按明确授权的完整里程碑连续推进；内部提交、测试和 CI 通过只是检查点，不以此要求再次输入“继续”。达到该里程碑或存在必须外部处理的真实阻塞时才交付/暂停。
