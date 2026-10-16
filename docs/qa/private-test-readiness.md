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
| 性能与资源 | 固定参考机、完整进程冷/热启动、空闲 CPU、全进程树内存、真实 I/O 与误差 | 完整新进程、十万条目与进程树观测已接入，发行基准待验证 |
| 许可与来源 | 锁定依赖通知、Qt 实际模块、字体/图标来源、构建清单和 SBOM | 已有通知、源码依赖图和工件文件级 CycloneDX SBOM，完整部署组件归属与许可仍待核验 |
| 干净机器 | 安装、首开、依赖完整性、系统选择器语言、文件关联、退出与卸载 | 待验证 |
| 发行身份与安全 | 支持系统、签名身份、漏洞入口、维护支持政策和独立安全审查 | 待外部确认与独立审查 |

人工记录应包含提交、系统/设备、渲染后端、缩放、操作步骤、实际结果和限制，保存到 build/ 或 CI 工件；不能只填写“通过”。现有自动化与当前用户反馈继续用于修正问题，不把任何未执行项自动勾选。

## 许可材料与发行边界

复用 `scripts/collect-notices.py build/review/private-test/notices` 收集宿主解析的 Cargo 依赖（含构建工具）、项目、Qt 和字体通知。它不是完整产品 SBOM，不证明所有部署文件的许可材料或可替换链接义务均已满足。Qt 模块和素材按 [许可说明](../engineering/licensing.md) 继续核验。

遵循 [CI 策略](../engineering/ci-policy.md)：macOS 只做 Apple Silicon 原生构建、测试和启动验证，不自动部署 Qt、签名、压缩或上传开发应用包。签名、发布身份、干净机器人工验收及独立安全审查需要相应人员与环境；本阶段不虚构这些结果。发行步骤按 [发布手册](../engineering/release-runbook.md) 执行。

## 2026-10-16 用户人工验收反馈

用户在本地 macOS 完成前述操作检查，并明确反馈“其他项已验收完毕，没有发现问题”。按用户结论登记其余清单项通过；本次剩余修订为深色密码 placeholder、创建后系统目录自动打开、“已保存”文案和欢迎页读取遮罩。修正基线为 1bd6d5e，人工反馈未单独附构建哈希、设备、后端或读屏工具记录，不把本机结论外推为 Windows/Linux、干净安装或独立安全审查通过。验收说明来源为用户本轮反馈，不要求重做已经确认的检查。

## 第二批，构建身份与完整进程观测

已引入[统一版本机制](../engineering/versioning.md)和“关于”窗口，让后续桌面反馈能携带实际版本、构建号、提交与 dirty 状态。既有用户确认的 macOS 操作清单继续有效，不要求因新增版本号重复验收。

`scripts/measure-desktop.py` 启动真实产品进程，使用隔离设置和自有十万条目 ZIP，按首帧、空闲、打开、取消与退出阶段记录事件。运行时显式指定 `--desktop-probe` 与 fixture；该入口只读取目录并调用既有取消／退出流程，不创建或解压文件。测量入口要求提供隔离设置路径，普通启动不启用探针。资源按进程及后代统计，输出不记录文件路径或用户名。

```sh
env -u QT_QUICK_BACKEND QT_QPA_PLATFORM=cocoa QSG_RHI_BACKEND=metal \
  python3 scripts/measure-desktop.py \
  --app build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip \
  --fixture build/ui/apps/desktop/m2-flat.zip \
  --output build/review/private-test/desktop-performance.json --runs 5
```

这是运行中宿主上的新进程观测，包含进程创建到首帧事件送达，不是清空磁盘缓存后的冷启动，也不是正式发行性能承诺。Debug／Release、平台、实际后端、构建身份与运行次数必须随记录保存。200 ms 加系统计数器命令耗时的采样可能漏峰值；后代 RSS 求和可能重复计算共享页；退出耗时含采样与轮询延迟，为上界。`ps` 的 CPU 时间分辨率因平台而异。五次样本的 p95 是 nearest-rank 最大值，只用于发现异常，不能替代稳定参考机的长期统计。真实磁盘 I/O 字节、发行资源预算和干净机器仍未验证。

许可收集器同时生成 `build-inventory.json`，记录版本源、提交、宿主解析的 Rust 包、声明的 Qt 产品模块、字体来源及许可证文件哈希。它是源码构建清单，包含构建工具，不能冒充安装后工件的完整 SBOM；Qt 传递依赖、实际部署文件与许可证义务仍属发行核验。

本批本地与 CI 证据保存到 `build/review/about/`、`build/review/private-test/` 及 CI 工件。当前完成的是可自动化的开发验证与观测；Windows／Linux 原生桌面人工专项、真实系统输入法与读屏、干净安装、签名身份和独立安全审查继续单独登记，WP-12 不自动标记完成。

## 第三批，部署核验与人工专项表

已接入[部署核验与 SBOM](../engineering/deployment-audit.md)：对实际工件逐文件哈希、递归解析原生导入，区分系统、工件内与外部 SDK；未解析依赖失败，Windows deployed 模式拒绝非系统外部依赖。源码图和实际工件分别生成 CycloneDX 1.6，使用锁定上游提交和哈希的官方 schema 检查，校验引用与实际二进制版本／提交／dirty 身份。两个 SBOM 均标记覆盖不完整，不能冒充全部运行时组件和许可已审查。

macOS 仍只读取开发 .app 与 SDK，不部署／签名／压缩／上传应用。第三批的 Linux 开发二进制仍保留 SDK 依赖，后续由第四批运行目录替代；Windows 的导入闭合和脱离 Qt 开发搜索路径启动也不能替代干净安装。正式部署阶段需补全按名称动态加载的插件、Qt 内嵌第三方、静态 Rust 归属及许可材料。

[人工专项操作清单](private-test-manual.md)提供八项实际步骤和记录方法。CI 为本平台生成全部 pending 的记录表，不自动填写通过；缺设备、安装渠道、系统关联实现或发布身份时填写阻塞。已确认的 macOS 常规操作验收继续沿用。当前尚未注册系统文件关联，须实现或取得发布负责人明确范围决策，不能以图标文件存在代替。

## 第四批，Linux 独立运行目录

已接入[Linux 部署流程](../engineering/linux-deployment.md)，将固定 Qt 6.8.2 的库、QML、X11 平台与输入相关插件、标准翻译及通知收集到相对目录，重写 ELF RPATH 并生成保留执行位的 tar.gz。系统图形／字体／glibc 等仍作为明确的 OS 运行依赖，不复制宿主系统库冒充通用兼容。

CI 在无 Qt SDK、无系统 Qt 和编译器的 Ubuntu 24.04 运行容器中，将应用目录只读挂载到带空格的新路径，关闭网络，执行版本、X11 软件渲染启动、十万条目打开、活动取消与退出；随后 deployed 模式核验和文件级 SBOM 必须通过。实际结果保存 linux-runtime.json、linux-deployment.json 和 linux-package.json，未执行时不标记通过。容器验证不替代完整桌面、物理 GPU、输入法／读屏、安装升级卸载、Wayland 或其他发行版验收；macOS 部署限制不变。
