# 验证方法与证据管理

[English](../../qa/README.md) | 简体中文

这些指南帮助开发者根据改动选择验证方法、准备测试样本，并判断检查结果能够说明什么。

- [产品验收要求](acceptance.md)
- [样本与夹具规则](fixtures.md)
- [通过真实工作进程验证 runtime](runtime-worker-tests.md)
- [按需桌面验证方法](private-test-manual.md)
- [发行审查流程](release-review.md)
- [CI 策略](../engineering/ci-policy.md)
- [部署依赖与 SBOM 契约](../../../docs/zh-CN/engineering/deployment-audit.md)

## 验收范围、证据和收口

开始时依据用户明确的交付要求、适用规范中的强制项和本次改动风险，确定能验证实际行为的最小充分检查、通过条件及必要的平台。复用已有且适用于当前提交的证据；源码或环境发生影响结果的变化时重验受影响部分。执行者可以补充排障检查，但不得把自己临时提出的录像、截图数量、人工矩阵、重复轮次或工作流，自动升级为合并、关闭 issue 或发布的条件。历史进度评论中的建议也不自动成为强制要求；用户明确要求和适用规范规定的条件仍须遵守，不能为收口而取消或降低。

启动构建、下载工件或重跑工作流前，先核对提交或受影响文件树、工件身份、环境、覆盖范围及已有证据是否可用。等待构建或下载完成后，再核对工件大小和摘要，随后才能运行或打包。切换分支或设备时复用仍适用的结果。源码、环境或候选包字节变化后重验受影响范围，优先重试失败项。自动、原生和人工证据分别记录。

小型界面改动未改变业务逻辑、文件写入或任务语义，且用户明确采用截图确认时，收到确认即可按该方式完成视觉验收。记录获确认的区域、平台及状态，复用适用构建并保留必要定向检查。确认只覆盖展示的结果，其他平台、未观察到的交互和发行许可仍按各自的证据判断。

| 验收类型 | 必要检查与证据边界 |
| --- | --- |
| 文档与静态资源 | 检查受影响的链接、JSON 和生成输出；Bandizip 评估还需运行 `python scripts/test-bandizip-format-inventory.py`，保证已核对的双语 44 行清单完整且顺序稳定。不因文档修改自动要求产品实机验收。 |
| 产品、领域与桥接 | 按 [CI 策略](../engineering/ci-policy.md)构建并验证受影响路径，以及相应失败、取消和保护行为；自动测试只证明其实际覆盖范围。 |
| 界面与原生桌面 | 执行适用的窗口、共享控件和双语多字号检查，并按用户要求或适用规范操作实际窗口、观察相关状态。功能测试不能代替要求的视觉检查；静态截图不能证明未观察到的瞬时行为。录像不是默认要求，只在用户明确要求或该问题无法以其他适当证据验证时使用。 |
| 系统交互、安装与更新 | 按实际改动和契约检查涉及的系统操作、身份、失败及恢复；隔离夹具、测试密钥和受控服务须与生产环境分别记录。 |
| 跨平台与发行 | 只对用户要求、支持承诺或受影响契约所需的平台作相应检查；发布仍按[发布协议](../../../docs/zh-CN/engineering/release-protocol.md)验证版本、签名、资产与发行许可，不能以普通 CI 代替。 |

桌面语言清单维护在 [ui-languages.json](../../../resources/ui-languages.json)，目前包含十种语言。修改界面译文时运行 `python3 scripts/check-ui-i18n.py`，检查文案目录完整性、占位符和源码覆盖。标为双语的检查只覆盖所列的中英文用例，不能据此认定其他语言的布局、语义或原生集成通过；应根据受影响译文选择额外的语言检查，并记录实际覆盖范围。

记录提交、环境、已执行检查、结果和与结论有关的限制。失败时定位、修复并重验受影响范围，保留历史失败；通过后不因可选补充检查反复扩大范围。满足必要条件后继续合并和清理；只有必要条件无法满足，或用户要求的外部条件确实不可用时才暂停，不要求用户为内部检查点反复确认。未执行的检查如实写明，既不记为通过，也不在不适用时制造新的阻塞。

## Snap 候选包的运行库检查

在 Ubuntu 24.04 amd64 上使用 `scripts/desktop.py` 准备共享 Qt SDK，并从干净源码构建。使用 `scripts/collect-notices.py` 和 `scripts/prepare-runtime-notices.py` 收集许可通知，再用 `scripts/deploy_linux.py --experimental-wayland` 生成运行目录。执行 `scripts/build-linux-snap.py --runtime <运行目录> --qt-prefix <SDK> --core24 <基础包.snap> --package-cache <缓存目录> --output <输出目录>`。配方使用锁定的 Ubuntu 软件包，核对应用提交、已修补的 Qt 模块、桌面入口，以及所有 ELF 是否能从包内和固定的 core24 基础包加载。生成的 Snap 随附清单和报告。商店候选传入 `--grade stable`，并提供 Qt 图像库修复回执。

在 Ubuntu 24.04 amd64 准备 C++ 编译器、`pkg-config`、`libnautilus-extension-dev` 和 `dpkg-dev` 后，运行 `python3 scripts/build-linux-snap-nautilus.py --output <目录>` 生成可选的 Files 配套包。确认缺少 Snap 菜单协议标记时隐藏菜单，触发右键创建 ZIP 和解压并核对输出字节，同时独立检查原生 DEB 扩展。受控启动器测试覆盖请求交接和操作结果，Ubuntu 安装验收另外检查 Files 加载配套扩展以及 `snap run` 跨隔离环境的交接。

Snap 的关于页面由商店负责更新。`snapLeavesUpdatesToTheStore` 验证 Snap 更新请求会被拦截且不访问更新服务，同时核对普通 Linux 构建仍能发出请求。

上游 Qt SDK 启用了 ICU、fontconfig、XCB、Wayland 和 AT-SPI 辅助功能桥接。在 Ubuntu 上使用锁定的图像库配方时，安装相应的开发头文件，包括 `libicu-dev`、`libfontconfig-dev`、`libdbus-1-dev`、`libatspi2.0-dev`、`libdrm-dev`、`libgbm-dev`、`libwayland-dev` 和 `libxcb-glx0-dev` 等 XCB 开发包。缺少功能时检查 QtGui 配置日志，重建模块的功能和符号必须与 SDK 匹配。

准备实验性的 strict Snap 时，应将最终包的运行依赖与包内库及所声明的基础镜像（例如 `core24`）中实际提供的库逐一对照。独立 Linux 运行目录的 `share/deployment.json` 列出的是系统依赖，并不能证明 Snap 基础镜像已经包含它们。检查应用、工作进程、Qt 平台插件及其他随包原生插件，包括传递依赖。例如，应用可能需要 `libXext.so.6`，随包的 Nautilus 插件可能需要 `libnautilus-extension.so.4`；应以实际产物为准，不能把示例当作固定的完整依赖清单。对随包附带的系统库，记录软件包版本、哈希和许可证通知。

宿主机上的 `ldd` 和 `--version` 可能借助 Snap 无法访问的宿主库而成功；`--version` 还会在初始化图形界面之前退出。应在不能回退到宿主库的环境中，使用声明的基础镜像检查最终包，并运行应用的离屏 `--smoke-test`，覆盖图形界面初始化。依赖检查不能证明所有运行时动态加载的库都可用。这些检查应与真实 strict Snap 桌面会话中的安装和交互分开记录；隔离加载检查和离屏启动均不能代替桌面验收。

对于实验性 Wayland 候选包，`scripts/deploy_linux.py --experimental-wayland` 在 X11 插件之外加入 Qt Wayland 平台、窗口协议和装饰插件。Snap 还须声明 `wayland` 接口，并按桌面会话选择相应的 Qt 平台。检查最终 Snap 中新增插件的依赖，单独记录实际 Wayland 启动和拖放结果；包含插件不代表 Wayland 支持已通过验收。默认独立部署仍使用 X11。

主窗口在 X11 和原生 Wayland 下共用集成标题栏、边框和阴影。在 Wayland 合成器中设置 `QT_QPA_PLATFORM=wayland` 运行浏览器测试 `linuxMainFrameOutline`，检查标题栏按钮是否可见、还原时的客户区尺寸、阴影区域是否透过鼠标输入，以及最大化和还原状态。在同一会话内使用已安装的 Snap 验证标题栏移动、边缘缩放和归档条目拖到 Files。

在同一后端运行 `waylandOperationWindowsUseFixedCloseOnlyFrame`，检查共用的操作弹窗标题栏、关闭图标及动作、阴影输入区域和固定尺寸约束。分别在浅色和深色主题下，比对四个圆角外侧的阴影像素与对应素材，并在有对比度的桌面背景上检查轮廓和阴影。测量窗框以内的实际客户区，复查内容驱动的高度变化、正文滚动和底部按钮位置。在安装后的 Snap 中尝试拖动弹窗边框和双击标题栏。

检查包内启动脚本是否将 libxkbcommon 的 `XKB_CONFIG_ROOT` 指向随包的 XKB 数据。如果启动脚本回退到 `/usr/share/X11/xkb`，带有系统 XKB 数据的宿主机可能会掩盖这一问题。在已安装的 Snap 上执行 `XKB_LOG_LEVEL=debug snap run xdzip`，可查看初始化键盘时实际使用的数据搜索路径。

Snap 应用使用私有的 `XDG_RUNTIME_DIR`，而合成器套接字可能位于其父目录。`apps/desktop/packaging/linux/snap-launch.sh` 将已有套接字衔接到私有目录，不替换已有路径；套接字不可用时回退到 X11。在 Linux 上运行 `python3 scripts/test-snap-launch.py`，检查套接字连接、回退和显式指定平台。离屏冒烟测试无法检查合成器连接，套接字测试也不能代替 strict Snap 的桌面验收。

## 界面对照回归

菜单首帧布局回归按对象名称定位序号和状态标签，分别核对真实文字度量及随字号变化的共享内距。覆盖中英文、浅深色、100%／125%／150% 字号，序号、状态、快捷键的八种组合和子菜单箭头；弹出与字号改变后的首个渲染帧直接断言，不通过额外等待或扩大容差消除布局失败。

任务入口图标使用已批准的 16 逻辑像素修订，严格测量实际 SVG 的尺寸、垂直中心及在按钮和文字之间的位置；旧原稿墨迹保留为审阅记录，不改写目标。主窗口与菜单的比例由 Qt 逻辑尺寸和 DPR 矩阵检查。


Windows 可在构建 Qt 浏览器测试后按需运行 `scripts/check-windows-update-uia.ps1`，传入 `-TestExecutable`、`-QtBin` 和尚不存在的 `-OutputDirectory`。脚本使用隔离测试进程与模拟更新状态，检查中英文更新文字是否可由 Windows UI Automation 读取；输出只放在 `build/`。它不连接真实更新服务，也不代替真实读屏或其他平台检查。


Windows 设置侧栏的原生悬停回归先确认目标字号与侧栏几何已生效，并等待 Qt 完成待处理的布局整理，再检查目标位置是否由测试窗口接收；若被遮挡，只将测试自己的窗口临时置顶，不操作其他应用。检查仍核对系统指针位置、窗口命中、进入、切换和离开时的真实悬停状态；置顶是测试布置，不改变产品窗口策略。另用自有遮挡窗口验证该回退。失败时可设置 `XDZIP_SETTINGS_HOVER_FAILURE_AUDIT` 输出测试窗口自身的截图和诊断数据。


## 状态与证据放在哪里

具体工作、验收结论、未覆盖项和阻塞原因记录在 [GitHub Issues](https://github.com/xiaodaozhi/xdzip/issues)。工作包、缺陷与发行许可复核分开跟踪；同一问题复用既有 issue，不用新文档累积“第几批完成”。WP-12 历史验收保留于 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)，`v0.1.0` 的历史发行许可复核见 [#4](https://github.com/xiaodaozhi/xdzip/issues/4) 和 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)，不因目录整理重复要求用户验收；未来候选须建立新的发行 issue。

截图、日志、逐次测量和机器报告输出到 `build/` 或 `dist/`，按 CI 保留政策上传 [Actions 工件](https://github.com/xiaodaozhi/xdzip/actions)。issue 引用实际运行、提交、工件摘要和限制，不粘贴整个生成报告，不将过期工件或配置成功冒充验证通过。长期发行材料由实际 Release 随包提供，发布仍需明确授权。

桌面检查按实际改动和可用环境选择，不要求维护者为每次提交填写固定人工验收表。本机暂时无法提供无遮挡前台时，可手动调度 [Windows 原生视觉诊断](../../../.github/workflows/windows-native-visual.yml)，从一次性 hosted runner 收集菜单与自有窗口图像。方法和限制见 [CI 策略](../engineering/ci-policy.md)。它使用独立的自动化来源记录，保留原生捕获护栏；不得将无人值守结果填成真人点击，也不得将虚拟屏幕当成物理多显示器验收。

`design/hifi/` 保存设计原稿，`design/assets/` 保存产品素材，`tests/fixtures/` 保存自有可再分发回归输入；它们是持久输入，与实际运行的截图或测量输出不同。内容边界和历史整理方法见[仓库内容管理](../engineering/repository-hygiene.md)。
