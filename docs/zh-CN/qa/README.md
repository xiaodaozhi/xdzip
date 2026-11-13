# 验证方法与证据管理

本目录保存可重复使用的验收要求、测试方法、样本规则、审查流程和空白模板。它不保存具体版本的验收结果、阶段报告、截图、测量 JSON、运行日志或交接进度。

- [产品验收要求](acceptance.md)
- [样本与夹具规则](fixtures.md)
- [按需桌面验证方法](private-test-manual.md)
- [发行审查流程](release-review.md)
- [CI 策略](../engineering/ci-policy.md)
- [部署依赖与 SBOM 契约](../engineering/deployment-audit.md)

## 验收范围、证据和收口

开始时依据用户明确的交付要求、适用规范中的强制项和本次改动风险，确定能验证实际行为的最小充分检查、通过条件及必要的平台。复用已有且适用于当前提交的证据；源码或环境发生影响结果的变化时重验受影响部分。执行者可以补充排障检查，但不得把自己临时提出的录像、截图数量、人工矩阵、重复轮次或工作流，自动升级为合并、关闭 issue 或发布的条件。历史进度评论中的建议也不自动成为强制要求；用户明确要求和适用规范规定的条件仍须遵守，不能为收口而取消或降低。

启动构建、下载工件或重跑工作流前，先核对已有结果的提交或受影响内容、环境、覆盖范围和有效性；符合本次需要时直接复用，不因换分支或设备而重复。结果不适用或相关代码、环境变化时重验受影响范围。失败后根据原因优先重试失败作业或受影响检查；确需重跑完整矩阵时说明原因。复用不改变证据的适用范围，自动、原生和人工检查仍分别判断，必需的独立验证不得省略。

| 验收类型 | 必要检查与证据边界 |
| --- | --- |
| 文档与静态资源 | 检查受影响的链接、JSON 和生成输出；不因文档修改自动要求产品实机验收。 |
| 产品、领域与桥接 | 按 [CI 策略](../engineering/ci-policy.md)构建并验证受影响路径，以及相应失败、取消和保护行为；自动测试只证明其实际覆盖范围。 |
| 界面与原生桌面 | 执行适用的窗口、共享控件和双语多字号检查，并按用户要求或适用规范操作实际窗口、观察相关状态。功能测试不能代替要求的视觉检查；静态截图不能证明未观察到的瞬时行为。录像不是默认要求，只在用户明确要求或该问题无法以其他适当证据验证时使用。 |
| 系统交互、安装与更新 | 按实际改动和契约检查涉及的系统操作、身份、失败及恢复；隔离夹具、测试密钥和受控服务须与生产环境分别记录。 |
| 跨平台与发行 | 只对用户要求、支持承诺或受影响契约所需的平台作相应检查；发布仍按[发布协议](../engineering/release-protocol.md)验证版本、签名、资产与发行许可，不能以普通 CI 代替。 |

记录提交、环境、已执行检查、结果和与结论有关的限制。失败时定位、修复并重验受影响范围，保留历史失败；通过后不因可选补充检查反复扩大范围。满足必要条件后继续合并和清理；只有必要条件无法满足，或用户要求的外部条件确实不可用时才暂停，不要求用户为内部检查点反复确认。未执行的检查如实写明，既不记为通过，也不在不适用时制造新的阻塞。

## 界面对照回归

菜单首帧布局回归按对象名称定位序号和状态标签，分别核对真实文字度量及随字号变化的共享内距。覆盖中英文、浅深色、100%／125%／150% 字号，序号、状态、快捷键的八种组合和子菜单箭头；弹出与字号改变后的首个渲染帧直接断言，不通过额外等待或扩大容差消除布局失败。

HTML 正文对照保留冻结原稿及哈希。任务入口图标使用已批准的 16 逻辑像素修订，严格测量实际 SVG 的尺寸、垂直中心及在按钮和文字之间的位置；旧原稿墨迹保留为审阅记录，不改写目标。主窗口与菜单的比例由 Qt 逻辑尺寸和 DPR 矩阵检查；真实浏览器缩放可按需本地复核 HTML。

English: First-frame menu checks locate sequence and status tags by object name and verify actual text metrics with the shared, font-scaled insets. Both languages and themes, 100/125/150% font scales, all eight tag/shortcut combinations and submenu arrows remain covered without geometry retries or wider tolerances. HTML body checks retain the frozen artwork and hash, recording its superseded task icon ink while strictly measuring the approved 16-unit SVG size, vertical center and placement. Qt main-window/menu proportions retain logical-size and DPR checks; browser zoom is optional for local HTML review.

Windows 可在构建 Qt 浏览器测试后按需运行 `scripts/check-windows-update-uia.ps1`，传入 `-TestExecutable`、`-QtBin` 和尚不存在的 `-OutputDirectory`。脚本使用隔离测试进程与模拟更新状态，检查中英文更新文字是否可由 Windows UI Automation 读取；输出只放在 `build/`。它不连接真实更新服务，也不代替真实读屏或其他平台检查。

English: On Windows, the optional `check-windows-update-uia.ps1` takes a built browser test executable, a Qt bin directory and a new output directory. It observes bilingual synthetic update states through Windows UI Automation in an isolated test process; it does not exercise a real update service or replace a screen-reader or other-platform check.

Windows 设置侧栏的原生悬停回归先确认目标字号与侧栏几何已生效，并等待 Qt 完成待处理的布局整理，再检查目标位置是否由测试窗口接收；若被遮挡，只将测试自己的窗口临时置顶，不操作其他应用。检查仍核对系统指针位置、窗口命中、进入、切换和离开时的真实悬停状态；置顶是测试布置，不改变产品窗口策略。另用自有遮挡窗口验证该回退。失败时可设置 `XDZIP_SETTINGS_HOVER_FAILURE_AUDIT` 输出测试窗口自身的截图和诊断数据。

English: The Windows settings-sidebar hover test checks that the requested font and sidebar geometry have taken effect and waits for pending Qt layout polish, then checks that its own window receives the target position. If obscured, it temporarily raises only the test window to topmost; it does not manipulate other applications or change product window policy. It still checks the system cursor, window hit, hover entry, switching and exit, and exercises the fallback with an owned obstruction. `XDZIP_SETTINGS_HOVER_FAILURE_AUDIT` optionally retains the test window's own failure screenshot and diagnostic data.

## 状态与证据放在哪里

具体工作、验收结论、未覆盖项和阻塞原因记录在 [GitHub Issues](https://github.com/xiaodaozhi/xdzip/issues)。工作包、缺陷与发行许可复核分开跟踪；同一问题复用既有 issue，不用新文档累积“第几批完成”。WP-12 历史验收保留于 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)，`v0.1.0` 的历史发行许可复核见 [#4](https://github.com/xiaodaozhi/xdzip/issues/4) 和 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)，不因目录整理重复要求用户验收；未来候选须建立新的发行 issue。

截图、日志、逐次测量和机器报告输出到 `build/` 或 `dist/`，按 CI 保留政策上传 [Actions 工件](https://github.com/xiaodaozhi/xdzip/actions)。issue 引用实际运行、提交、工件摘要和限制，不粘贴整个生成报告，不将过期工件或配置成功冒充验证通过。长期发行材料由实际 Release 随包提供，发布仍需明确授权。

桌面检查按实际改动和可用环境选择，不要求维护者为每次提交填写固定人工验收表。本机暂时无法提供无遮挡前台时，可手动调度 [Windows 原生视觉诊断](../../../.github/workflows/windows-native-visual.yml)，从一次性 hosted runner 收集菜单与自有窗口图像。方法和限制见 [CI 策略](../engineering/ci-policy.md)。它使用独立的自动化来源记录，保留原生捕获护栏；不得将无人值守结果填成真人点击，也不得将虚拟屏幕当成物理多显示器验收。

`design/hifi/` 保存设计原稿，`design/assets/` 保存产品素材，`tests/fixtures/` 保存自有可再分发回归输入；它们是持久输入，与实际运行的截图或测量输出不同。内容边界和历史整理方法见[仓库内容管理](../engineering/repository-hygiene.md)。
