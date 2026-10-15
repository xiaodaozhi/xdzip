# WP-12 私测准入

状态：已开始，尚未达到可发行门槛。WP-10/11 的格式能力和自动化结果继续沿用各自契约，不据此宣称桌面人工验收或独立安全审查通过。当前支持边界见 [README](../../README.md)，完整退出条件见 [工作清单](../planning/backlog.md)。

## 首批自动化

`BrowserTest::privateTestDesktopInputAndAccessibility` 使用实际创建窗口，检查：

- 首个 `frameSwapped`、非空抓图及实际渲染 API，记录窗口构建到帧呈现的耗时。
- 中文预编辑不会提前写入字段，提交包含中文和非 BMP 字符的文件名后退出组合状态，Escape 退出焦点且保留文本。
- Qt 无障碍接口实际暴露可编辑角色、名称、字段值和文本接口；创建按钮暴露按钮角色、名称及真实禁用状态。
- `XDZIP_REQUIRE_GPU=1` 时要求 Metal、OpenGL、Vulkan 或 Direct3D 后端，不允许 Qt 软件或 Null 后端冒充 GPU 通过。此标记仅区分渲染 API，不能据此认定 OpenGL/Vulkan 驱动一定运行在物理 GPU 上，驱动与设备仍须核验。

三平台 CI 保存 `build/review/desktop-input-accessibility.json`。默认离屏软件测试只证明合成输入事件与 Qt 接口语义；真实候选列表、系统输入法、读屏播报和 GPU 驱动兼容性仍须单独验证。`firstPresentedMs` 是同一测试进程内的温启动观测，不包含完整进程启动，不能当作发行冷启动性能。

本地 macOS 可在原生开发构建执行：

```sh
env -u QT_QUICK_BACKEND QT_QPA_PLATFORM=cocoa QSG_RHI_BACKEND=metal \
  XDZIP_REQUIRE_GPU=1 XDZIP_PRIVATE_TEST_AUDIT=build/review/private-test/metal \
  build/ui/apps/desktop/xdzip-browser-tests privateTestDesktopInputAndAccessibility \
  -o build/review/private-test-metal.log,txt
```

运行环境与构建目录应按真实平台调整。Qt 输入事件的自动化不是系统拼音输入法操作，Qt 无障碍接口自动化不是 VoiceOver/Narrator/Orca 的人工验收。

## 待完成门槛

| 检查 | 需要的证据 | 当前状态 |
| --- | --- | --- |
| 中文输入法 | 中文拼音候选、切换中英、组合中按 Escape/Enter、长文件名，检查搜索与操作窗口 | 待三平台桌面人工验证 |
| 键盘和读屏 | Tab 顺序、操作名称、禁用/选中状态、列表选中和任务变化播报 | Qt 接口有自动化，真实读屏待验证 |
| GPU 与缩放 | 原生后端的浅深色、DPR、100/125/150% 字号、菜单和对话框，无黑块或裁剪 | 本地可运行硬件探针，三平台桌面矩阵待补齐 |
| 性能与资源 | 固定参考机、完整进程冷/热启动、空闲 CPU、全进程树内存、真实 I/O 与误差 | 仅首帧开发观测已接入，发行基准待验证 |
| 许可与来源 | 锁定依赖通知、Qt 实际模块、字体/图标来源、构建清单和 SBOM | 有通知收集器，完整发行材料待核验 |
| 干净机器 | 安装、首开、依赖完整性、系统选择器语言、文件关联、退出与卸载 | 待验证 |
| 发行身份与安全 | 支持系统、签名身份、漏洞入口、维护支持政策和独立安全审查 | 待外部确认与独立审查 |

人工记录应包含提交、系统/设备、渲染后端、缩放、操作步骤、实际结果和限制，保存到 build/ 或 CI 工件；不能只填写“通过”。现有自动化与当前用户反馈继续用于修正问题，不把任何未执行项自动勾选。

## 许可材料与发行边界

复用 `scripts/collect-notices.py build/review/private-test/notices` 收集宿主解析的 Cargo 依赖（含构建工具）、项目、Qt 和字体通知。它不是完整产品 SBOM，不证明所有部署文件的许可材料或可替换链接义务均已满足。Qt 模块和素材按 [许可说明](../engineering/licensing.md) 继续核验。

遵循 [CI 策略](../engineering/ci-policy.md)：macOS 只做 Apple Silicon 原生构建、测试和启动验证，不自动部署 Qt、签名、压缩或上传开发应用包。签名、发布身份、干净机器人工验收及独立安全审查需要相应人员与环境；本阶段不虚构这些结果。发行步骤按 [发布手册](../engineering/release-runbook.md) 执行。
