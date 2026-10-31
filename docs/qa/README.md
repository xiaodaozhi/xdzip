# 验证方法与证据管理

本目录保存可重复使用的验收要求、测试方法、样本规则、审查流程和空白模板。它不保存具体版本的验收结果、阶段报告、截图、测量 JSON、运行日志或交接进度。

- [产品验收要求](acceptance.md)
- [样本与夹具规则](fixtures.md)
- [原生桌面与安装人工验证方法](private-test-manual.md)
- [发行审查流程](release-review.md)
- [CI 策略](../engineering/ci-policy.md)
- [部署依赖与 SBOM 契约](../engineering/deployment-audit.md)

## 界面对照回归

菜单首帧布局回归按对象名称定位序号和状态标签，分别核对真实文字度量及随字号变化的共享内距。覆盖中英文、浅深色、100%／125%／150% 字号，序号、状态、快捷键的八种组合和子菜单箭头；弹出与字号改变后的首个渲染帧直接断言，不通过额外等待或扩大容差消除布局失败。

HTML 正文对照保留冻结原稿及哈希。任务入口图标使用已批准的 16 逻辑像素修订，严格测量实际 SVG 的尺寸、垂直中心及在按钮和文字之间的位置；旧原稿墨迹保留为审阅记录，不改写目标。主窗口与菜单的比例由 Qt 逻辑尺寸和 DPR 矩阵检查；真实浏览器缩放可按需本地复核 HTML。

English: First-frame menu checks locate sequence and status tags by object name and verify actual text metrics with the shared, font-scaled insets. Both languages and themes, 100/125/150% font scales, all eight tag/shortcut combinations and submenu arrows remain covered without geometry retries or wider tolerances. HTML body checks retain the frozen artwork and hash, recording its superseded task icon ink while strictly measuring the approved 16-unit SVG size, vertical center and placement. Qt main-window/menu proportions retain logical-size and DPR checks; browser zoom is optional for local HTML review.

## 状态与证据放在哪里

具体工作、验收结论、未覆盖项和阻塞原因记录在 [GitHub Issues](https://github.com/xiaodaozhi/xdzip/issues)。工作包、缺陷与发行许可复核分开跟踪；同一问题复用既有 issue，不用新文档累积“第几批完成”。WP-12 历史验收保留于 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)，发行许可复核见 [#4](https://github.com/xiaodaozhi/xdzip/issues/4)，不因目录整理重复要求用户验收。

截图、日志、逐次测量和机器报告输出到 `build/` 或 `dist/`，按 CI 保留政策上传 [Actions 工件](https://github.com/xiaodaozhi/xdzip/actions)。issue 引用实际运行、提交、工件摘要和限制，不粘贴整个生成报告，不将过期工件或配置成功冒充验证通过。长期发行材料由实际 Release 随包提供，发布仍需明确授权。

本机暂时无法提供无遮挡前台时，可手动调度 [Windows 原生视觉诊断](../../.github/workflows/windows-native-visual.yml)，从一次性 hosted runner 收集菜单与自有窗口图像。方法和限制见 [CI 策略](../engineering/ci-policy.md)。它使用独立的自动化来源记录，保留原生捕获护栏；不得将无人值守结果填成真人点击，也不得将虚拟屏幕当成物理多显示器验收。

`design/hifi/` 保存设计原稿，`design/assets/` 保存产品素材，`tests/fixtures/` 保存自有可再分发回归输入；它们是持久输入，与实际运行的截图或测量输出不同。内容边界和历史整理方法见[仓库内容管理](../engineering/repository-hygiene.md)。
