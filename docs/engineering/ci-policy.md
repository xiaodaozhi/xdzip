# CI 与合入策略

窗口状态、单实例与会话原生回归共用 `scripts/check-native-sessions.py`，在打标签前执行。Linux 安装 Xvfb、Openbox 和 xprop，以窗口管理器的 X11 状态验证最大化、全屏及弹窗行为；Windows、macOS 使用系统插件和窗口管理器。此步骤不部署或上传普通 macOS 应用包。

Linux 等待 Openbox 的 `--startup` 回调后才创建 Qt 测试进程，启动回执使用新建私有目录，等待与进程清理均有界。`_NET_SUPPORTING_WM_CHECK` 仅声明窗口管理器身份，写入时主题和事件处理仍可能未初始化完成，不能作为原生首窗测试的就绪条件。该屏障不创建预热 Qt 窗口，不增加首窗断言的等待期限，也不在断言失败后重试。

`scripts/test-native-window-manager.py` 检查回执、失败、路径与子进程清理；Linux CI 另运行 `scripts/check-x11-startup.py`，在自有 Openbox 子进程中延迟身份声明后的初始化，分别验证旧身份轮询的预期失败、启动回调后的正常及延迟冷启动。每项使用独立 Xvfb 和新 Qt 进程，保留首窗实际暴露断言、就绪耗时与完整日志。故障注入库只用于这一隔离回归，不进入产品运行库或发布包。

专用 Release 工作流是经用户明确授权的例外，允许 Apple Silicon 原生部署、ad-hoc 签名、app.zip 和 DMG 打包及上传。上述 Mac 禁止条件仅约束普通 CI。专用流程必须通过同提交原生回归、包核验并齐备六类资产后才公开，详见[发布协议](release-protocol.md)。

已配置 Documentation、Foundation probes 和 Qt components 三组产品工作流和 HTML design review，分别检查文档与资源、无 Qt 核心及诊断桥接、三平台 Qt 产品。检查方法见[验证入口](../qa/README.md)，具体结果绑定提交并记录到 issue 与 Actions 工件。下表定义按变更类型执行的门槛，M3 已加入安全 I/O 和真实 ZIP 解压回归，WP-10 已加入真实 ZIP 创建互操作，正式发行还须按[发行审查流程](../qa/release-review.md)单独确认。

| 变更 | 必需检查 |
| --- | --- |
| 文档/视觉 | 本地链接、JSON、HTML 生成、图片存在；核对设计与范围 |
| Rust 核心 | fmt、Clippy、领域/性质测试、受影响集成测试 |
| Qt/C++/QML | format、选定 tidy、qmllint、构建和焦点/模型行为测试 |
| FFI/安全 I/O | 三平台原生测试、所有权/竞态/故障注入，额外审查 |
| 格式 | 双向互操作、坏输入、资源/取消、能力清单 |
| 发布 | 三平台安装验收、未正式签名包与必要 ad-hoc 完整性、SBOM、许可材料、基准和回退 |

普通 PR 默认最小权限、无签名凭据，外部贡献不运行带秘密的代码。Actions 依赖固定完整提交 SHA；缓存以工具链、依赖锁和架构分键，不从不可信 PR 恢复发布工件。fuzz 与性能长任务独立调度，性能判断使用稳定参考机而非共享 runner 偶然成绩。

Documentation、Foundation probes、Qt components 和 HTML design review 也支持 `workflow_dispatch` 手动核对指定分支或标签。手动执行复用相同权限、检查和平台限制；按实际 `head_sha` 绑定结果，不能用旧提交的成功记录补齐当前提交的缺失检查，也不因此获得发布授权。

[Windows native visual diagnostic](../../.github/workflows/windows-native-visual.yml)是仅支持手动调度的一次性 Windows hosted runner 诊断，使用同提交的 Release 测试程序和原生 windows/D3D11 路线。先执行一次菜单桌面可用性采样，成功后另启完整 15 类、各 3 次的冷进程采样，试跑不计入 45 次；真实前台、自有窗口遮挡、首帧代次和同帧阴影检查保持不变。无法捕获或记录不完整时保留失败和诊断工件，不能记为通过，也不冒充人工点击。

该工作流只在一次性 hosted Windows 虚拟机内，从显示驱动真实枚举且通过系统测试的模式中选择足以容纳完整测试窗口的分辨率。切换不写注册表、不改系统 DPI，记录实际切换前后及测试期间读数，并在 finally 中按同一运行身份恢复原模式；不支持时阻断，不缩小产品组件或放松捕获护栏。该入口禁止在普通本机调用，虚拟屏幕不能冒充物理显示器。

原生布局诊断另启三个独立进程，分别请求 DPR 1、1.25、2，执行双语、浅深色、三档字号和三种窗口宽度的 108 项量测，以及列宽、标签菜单和关于页状态回归。DPR 覆盖来自进程内 Qt 缩放，需与每张图像的实际 DPR 核对；窗口请求值与实际值分别记录，运行器屏幕导致的尺寸钳制必须列为未实现的请求，不能据完整记录数宣称所有请求尺寸已通过。

该工作流另以独立进程采集目录菜单原因、关于页初始视图、文件图标和 Windows 原生边框。截图、日志及实际运行身份只上传到临时 Actions 工件，不生成或安装发行包。共享虚拟机上的时间、系统装饰和单屏图像只用于诊断，不替代固定参考机性能、Windows 11 装饰、物理双屏切换或人工视觉验收。测试程序的实际来源和运行器环境必须与历史人工证据分开记录。

手动输入 `focused_failures=true` 仅运行图标悬停与 DPR 1 混合目录菜单的失败定位。目录组保留主布局、列宽、标签操作、关于页四个前序方法及全部六个双语目录输入数据行，避免省略进程内前序条件。保留原断言、失败判定时的标量状态，以及判定后捕获的自有窗口图像，不将后捕获图像标为失败首帧，不增加重试或放宽标准。该路线单独标记为 `focused_failure_diagnosis`，不执行完整 45 次菜单或三 DPR 的 108 项矩阵，不能作为完整视觉准入。

分支保护必须等真实检查存在且成功运行后指定检查名称；CODEOWNERS 等实际审查者加入后建立。仓库当前可见性和保护设置不由文档生成过程默默修改。

添加窗口使用 C01 独立原稿哈希和 `add-reference.json` 对照，三平台 Qt 量测 12 组语言、主题及字号组合，并执行独立草稿、真实元数据、辅助功能移除、取消和文件/空目录更新。HTML 独立验证 36 组语言、主题、字号、DPR 组合和 24 组真实浏览器 100%/125% 缩放，复用私有 Xvfb 输入隔离，不使用 CSS zoom 代替。检查入口为 `scripts/check-add-dialog.py`，结果只保存为 build/ 或 CI 工件。

元数据过滤回归在三平台核心测试中生成真实 ZIP、TAR、TAR.GZ 创建和更新夹具，再由 `scripts/check-macos-metadata-interop.py` 使用独立 Python 读取器核对元数据排除、原记录保留、空目录、内容和 UTF-8 名称。Qt 覆盖中英文空输入失败反馈及原包不变，沿用共享结果窗口与双语多字号布局门槛；离屏结果不等于桌面人工验收。

Foundation probes 在 Linux x86_64、Windows MSVC x64、macOS arm64 执行 Cargo workspace 测试、Clippy、桥接探针；另检查 Rust/C++ 格式。核心测试不依赖 Qt。

自研后端在 Foundation 通过 `scripts/check-rust-codec-policy.py` 检查普通和构建依赖的传递闭包，开发测试参照不计作生产依赖。TAR/GZIP 包含独立读写器双向测试、Huffman 与距离边界、CRC/ISIZE、多成员、截断、资源与取消；`scripts/check-tar-interop.py` 另外使用 Python zlib 的固定、动态、RLE、Huffman-only 与 filtered 策略输入，并通过实际工作进程核对旧编码名称。`gzip-probe` 生成合成零字节、文本及随机数据的四档编码大小、耗时和独立内容核对，JSON 保留在 build/review/independent-rust-codecs/ 并作为 CI 工件上传。有限固定种子变异回归不冒充长期 fuzz，性能数字来自实际工件，不用共享 runner 偶然耗时批准性能。

CAB/ISO/UDF 增量复用同一 Foundation 门槛，`scripts/check-cab-iso-interop.py` 通过实际原生工作进程验证固定来源夹具、已知明文字节、四种 CAB 算法、跨卷文件、ISO9660/Joliet/Rock Ridge/UDF 及只读拒绝。镜像由独立 pycdlib 测试工具预生成并以小型 gzip 输入保存，CI 只校验和展开，不安装该生成器。Rust 与 Qt 覆盖任意 CAB 卷入口、冻结来源、混合命名空间选择、普通文件的解压/临时打开/导出及不支持特殊记录的整包测试失败。结果保留在 build/review/native-codec-interop/，不会进入 docs 或源代码历史。

独立 Rust 光盘后端额外运行 `check-optical-interop.py`：同一已知内容先经直接 codec，再经实际产品 worker 和宿主提取，核对 ISO 多区段、El Torito、UDF 稀疏及 4 GiB 以上随机偏移源；虚拟源检查不冒充实际磁盘大文件提取。预算、取消、快照变化、输出失败、截断及结构损坏分别验证。`optical_backend_tests` 的显式忽略测试在 Foundation 中使用本次构建的 worker 执行，覆盖真实 service 检查、Running 时源变化、已有暂存字节后的取消及发布前来源复验。只有实际执行并核验的路径才记通过。可选保留旧官方 worker 作为独立字节参照，报告必须记录两者摘要且拒绝同一二进制充当参照。结果保留于 build/review/independent-rust-codecs/optical/，分区映射扩展与人工桌面验收仍单列。

Qt components 通过 [scripts/desktop.py](../../scripts/desktop.py)复用本地的 prepare、build、基础 test，以及 Windows/Linux package／verify。参数和本地顺序统一见[开发构建指南](developer-build.md)。共用阶段不依赖 RUNNER_TEMP 或 RUNNER_ARCH；工作流显式提供 CI 路径，普通本地使用仓库默认目录。系统编译器／SDK 和 Linux 图形开发依赖仍由宿主／工作流准备，SDK 与打包工具下载使用 prepare 的显式开关。

[ci-desktop.ps1](../../scripts/ci-desktop.ps1)只保留 CI 扩展编排和平台策略，不另实现基础构建链。Qt CI 使用 `test --skip-rust`，同提交的 Rust fmt、Clippy 和 workspace 测试由 Foundation probes 负责；qmllint 与 7 组 CTest 仍由共用 test 执行。测试仅对子进程设置 offscreen／software，之后的原生或部署检查使用各自平台环境。CI 另外重复并发生命周期套件 10 次，保留性能协议、资源与链接证据、双语和多 DPR 视觉、M3／M4 互操作、设置兼容、来源及 SBOM 检查，不因入口复用缩减门槛。

历史完整产品、安装升级／回退、忙时拒绝、卸载和关联生命周期是 CI 显式扩展，继续限定一次性 Windows/Linux 主机或隔离容器。普通本地 verify 不安装产品、不写 HKCU 或更改默认关联；不得为了复用命令移除历史兼容脚本的 CI 与平台限制。运行时核验通过也不替代安装生命周期及人工桌面验收。

macOS 使用已安装的 Qt SDK、sdk 模式部署审计和原生 cocoa 插件执行启动／关闭及文件打开检查。CI 禁止调用 Mac package 或 DMG 核验，不执行 macdeployqt、签名或应用归档，只上传 xdzip-checks-macOS-ARM64 日志与 JSON。这证明原生开发构建可启动，不代表独立发行包验证。Windows 部署后清除 SDK 搜索路径，Linux 使用 SDK-free 运行容器；离屏或软件渲染不证明 GPU、输入法和辅助功能人工验收。

Windows/Linux 继续上传实验运行包、安装器、同提交源码、许可及证据；CI 的 output-dir 保持 build/review，构建目录为 build/desktop，普通本地打包默认 build/package。macOS 不上传 Apple Silicon 应用包。所有记录或工件保存 14 天，均不作为正式签名发行。

Qt 组件的原始逐项 `qml-tests.log` 在成功与失败路径都保存到诊断工件，不以可能被后续重复测试覆盖的 `LastTest.log` 摘要替代。构建检查成功时，该原始日志必须存在且非空，缺失或为空使证据收集明确失败；构建本身失败时仍保留可获得的原始诊断。macOS 继续只上传日志和 JSON。

浏览器首轮与十次整套重复各自保留独立的 Qt Test XML 格式 `.log` 和运行身份、耗时 JSON，位于 `build/review/browser-timings/`。各测试方法的耗时用于定位慢项，不改变 240 秒整套时限、断言或重复次数。被终止的进程可能只有 `running` 记录或不完整日志，不能记为完成；runner 的 post-job 和最终 job 结论仍须从 Actions 原始步骤与状态独立核对。

所有层级的 dist/ 及其全部内容都在 .gitignore 中排除，build/、target/ 也不进入 Git 源码树。Documentation 的 scripts/check-docs.py 检查 Git 索引，即使强制暂存 dist/ 文件也会失败。HTML 离线审阅工具仍可在本地生成 dist/，但生成结果只能作为本地文件或 Actions 工件，不能提交到仓库。

三平台 Qt 构建会生成十万条平面／分层 ZIP64 样本，执行实际 Qt 查询与源变化回归，并上传核心乱序元数据探针和 Qt 测量。Windows/Linux 开发工件按平台与架构区分，实际名称和上传范围以 [Qt 工作流](../../.github/workflows/qt-components.yml)为准。源码树不再保留历史截图、量测记录或开发包回执；历史整理必须遵循[内容与历史管理](repository-hygiene.md)，经所有者明确授权并备份后才可执行。

检查直接 main 推送时使用 `/actions/runs?head_sha=...`，核对 push 事件；连接器 `fetch_commit_workflow_runs` 仅筛选 PR 事件，空结果不表示没有自动运行。

M3 额外执行真实解压、冲突、取消等待、关闭源会话后继续任务和退出决定的 Qt 流程；测量 64 个 Deflate 文件／32 MiB 实际输出并独立复核全部字节。核心覆盖 CRC、流终止、描述符、源／目标变化、链接、ACL、提交竞争、清理及磁盘写失败。磁盘满为可重复的部分写入故障注入，不冒充真实填满设备。Windows 权限回归直接修改新建自有夹具的 ACL，只读、枚举与执行权限允许，写入、删除、所有权与 ACL 修改继续拒绝，覆盖有效与仅继承授权。Windows 另检查产品 PE 子系统为 GUI，诊断程序保持 console，并验证内建窗口按钮的最大化、还原、最小化与异步关闭。

WP-10 增加三平台真实创建窗口→生成 ZIP→重新打开→重名取消的 Qt 流程，DPR 1／1.25／2 检查逻辑尺寸、字号、字段和 footer。独立 Python zipfile 校验 Store、Deflate 1／6／9、Unicode、空项与 66,001 条记录的 ZIP64；记录实际时间与可获得的 RSS，缺失值为 null。高 DPR 离屏屏幕变小可能触发窗口高度上限，报告记录可用屏幕高度并核验约束后的逻辑尺寸，不将裁切误记为正常全窗口体验。

测试截图、量测 JSON、运行日志和生成报告均不得提交到 Git。设计源图、透明运行素材、设计参数与字体材料清单保留；它们不是实际运行的验收证据。Documentation 对 Git 索引执行检查，禁止通过强制暂存恢复旧工件。所有生成器输出到 build/review、其他 build 子目录或 dist，当前工件只作为 Actions artifact 上传。

ZIP 高级创建增加真实 AES-256 与编号分卷输出，核心覆盖错误密码、边界预算、来源／目标变化、整组冲突、身份校验及晚取消。Foundation 三平台使用独立 7-Zip 双向验证，包含 8 种 Store／Deflate、单文件／分卷和明文／AES 创建组合，以及 4 种独立 AES 输入组合，复核空文件、空目录、Unicode 和全部有效载荷。另用独立 Python 写入的两种 ZIP64 尾部占位样本验证真实宿主扫描与解压，不只检查内存记录层。Linux/macOS 另以 Info-ZIP 创建两种 PKZIP 分卷，Windows 因缺少该独立工具显式登记未执行，不用编号分卷替代这两项结论。Linux Qt CI 继续复用该入口。合成密码仅通过终端或探针的有界 stdin 输入，不进入命令行、环境和报告。Windows 7-Zip 的重定向密码输入存在 Unicode 编码限制，独立工具矩阵使用 ASCII 密码并明确登记该限制；Unicode 密码另由独立 zip-rs 密文比对及读取测试覆盖。Linux/macOS 保留 UTF-8 密码终端矩阵。Qt 控件校验覆盖默认级别、自定义空值／零值、单位、密码非法值与清空。结果明细检查使用真实任务记录，截图与量测遵守本地／CI 工件政策。

密码解压增加三平台格式与任务回归：AE-1／AE-2、严格结构及认证 EOF、错误重试、不同条目密码、空密码、满队列控制、取消／退出、来源变化、旧目标与链接保护。Qt 通过真实 AES 创建、自动打开、密码请求、重试、解压与关闭链路，核对密码输入角色及辅助功能动作。HTML 与 Qt 分别检查中英文、浅深色、多字号、小屏与 DPR，生成记录不含输入密码；原生装饰与系统读屏人工结果继续独立登记。互操作按[密码解压契约](password-extraction.md)及实际 CI 工件记录。

WP-11 复用三平台核心和 Qt 门槛，增加 TAR/TAR.GZ 真实创建、浏览及解压，以及 scripts/check-tar-interop.py 的 Python tarfile 与可用系统 tar 双向字节核对。GNU/PAX 长名称、空目录、多 member GZIP、逃逸及特殊项、CRC／截断、配额和取消均由实际测试登记。桌面人工验收不随自动化结果标记通过。

WP-12 对实际工件递归核验原生导入，Windows/Linux 使用 deployed 模式，macOS 使用 sdk 模式并保留外部 SDK 限制。源码与实际文件分别生成 CycloneDX 1.6，绑定实际 CLI 身份并校验锁定官方 schema；不把文件清单称为完整运行时组件合规证明。所有平台生成八项全 pending 的人工表作为新候选模板，不覆盖 issue 中既有验收结论，具体边界见[部署核验](deployment-audit.md)。macOS 新增上传仍仅限 JSON，禁止应用包自动部署、签名或上传。

Linux 增加[独立部署与运行容器](linux-deployment.md)：固定 SDK 的 QML 扫描、Qt 库递归收集和相对 RPATH，清除开发搜索路径后核验。独立 Ubuntu 24.04 镜像只安装实际 OS 运行依赖与 Xvfb，确认没有系统 Qt／编译器；只读挂载带空格路径下的运行目录，关闭网络，检查版本、X11 启动及十万条目打开／取消／退出。该测试使用软件渲染，不冒充真实 GPU 或完整桌面安装。Linux 下载使用 tar.gz 保留执行权限，原始 linux/ 目录不再重复上传。

安装器 gate：Windows 固定官方 Inno Setup 编译器／中文资源哈希并验证当前用户安装、原生 ShellExecuteEx 打开、升级／回退、忙时阻止和自有项卸载；Ubuntu 在 SDK-free 容器用非 root 产品验证 dpkg／GIO 同类生命周期。QA 包只生成到 build/installer-tests/，不上传为产品；正式实验安装器及 JSON／日志随已有 Windows/Linux 工件保存。Mac 只补充文档类型与文件事件并原生编译／启动，不生成安装包。

本地部署与 CI 分开：用户明确要求时可制作开发 DMG，允许本地 Qt 部署与必需的 ad-hoc 完整性签名，见[Mac 部署](macos-deployment.md)。此例外不改变上述 CI 禁止生成／上传 Mac 应用的范围，不授权 Developer ID、公证或发布。三平台 CI 另编译固定历史／当前设置适配器并验证同 schema 往返和较新 schema 拒写；Linux 安装测试增加一次真实 postinst 配置失败、运行阻止与 dpkg 恢复。

Mac 包体积规则在 Documentation 的跨平台夹具测试 `scripts/test-macos-bundle.py` 中检查：覆盖嵌套 Mach-O 架构识别及 `arm64` 裁剪、Basic 样式与共享依赖保留、未知或缺失依赖拒绝，以及输入应用／SDK 不变。CI 不为这些测试调用 macdeployqt、lipo 或 codesign，不因此引入 Mac 包部署。真实开发 DMG 在本地部署后递归复验全部原生分片和样式集合，记录优化前后字节数，并完成重定位 Cocoa／Metal 与兼容 QtCore 修改副本启动核验；实际执行结果独立登记，不以夹具测试代替。

独立源码验证使用[Qt 源码专用工作流](../../.github/workflows/qt-source.yml)，由可信 main 显式分派，单独运行三平台所需模块的源码编译与真实产品测试。Mac 仍只上传有界日志和 JSON，不缓存或上传 SDK／应用；Windows/Linux 依赖缓存以 runner 镜像、源码配置和脚本指纹区分，源码恢复后重新校验。普通 Qt CI 同时校验原生指标协议、实际资源与链接证据；不在共享 runner 用单次性能数字批准发行预算。

源码库内容边界由 `scripts/repository_contents.py` 对 Git 索引核验，禁止强制暂存生成目录、文档阅读版、已退役阶段报告、QA 执行输出及超过审查阈值的大 blob。具体执行状态使用 issue，不把 CI 或验收报告重新提交到 docs。

旧文件名编码回归使用自有原始字节和 Python 独立编码，Foundation 三平台验证 Rust 添加/移动后的 26 个归档及 UTF-8 声明，Qt 三平台运行实际服务探针解压 26 个 ZIP/TAR/TAR.GZ 输入，并执行双语 Qt 模型、导航、搜索、打开、导出与解压。Unicode 扩展、PAX/GNU、AES、分卷、碰撞、预算与声明损坏由格式/核心回归覆盖。入口为 scripts/check-filename-encoding.py，报告只进入 build/ 与 CI 工件；统计检测不承诺所有短名称或混合编码均能唯一恢复。
