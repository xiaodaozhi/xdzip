# 验证工件与验收入口

仓库保存测试代码、验收规范、设计原稿和设计参数，不保存测试截图、量测记录、运行日志或生成的对照报告。已提交的旧工件从当前文件树移除，既有提交历史保持不变。

运行界面检查时，截图和 JSON 写入 `build/review/`、其他 `build/` 子目录或 `dist/`。这些目录全部忽略。GitHub Actions 把实际运行工件作为短期保留的 Actions artifact 提供，禁止把工件再次提交到源代码仓库。

- [WP-12 私测准入](private-test-readiness.md)
- [WP-12 人工专项操作清单](private-test-manual.md)
- [部署依赖核验与 SBOM](../engineering/deployment-audit.md)
- [CI 策略](../engineering/ci-policy.md)
- [设计流程](../visual/design-workflow.md)
- [统一尺寸规范](../visual/layout-scale.json)
- [操作窗口测试](../../scripts/check-m3-operation-windows.py)
- [新建窗口检查](../../scripts/check-html-create-controls.py)

`design/hifi/` 是设计原稿，`design/assets/` 是产品使用的素材库；两者与测试捕获的用途不同，继续保留。

主界面操作准备防闪烁由 `BrowserTest::operationPreparationKeepsBrowserPresentationStable` 验证：真实点击“全部解压”、选择项解压、目标重检、创建目标检查、创建输入预览期间，逐次监听控件 enabled、列表遮罩 visible 和控制器状态，要求无瞬时加载／禁用切换、无模型重置且保留选中项。任务提交与重复准入继续由相邻的解压提交测试覆盖。日志保留于 `build/` 或 CI 工件，不代替桌面人工验收。
