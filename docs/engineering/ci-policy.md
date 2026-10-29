# CI 与合入策略

Documentation、Foundation probes、Qt components 和 HTML design review 的普通自动入口为 main push 与 pull_request，继续保留全部 tag push 及 workflow_dispatch。无 PR 的临时分支按需显式分派普通工作流，不再为分支 push 与随后 main 集成重复生成全套证据。标签及手动入口不得因本次收敛被禁用。专用 Release、更新、Qt 源码与原生诊断入口独立管理。

普通并发组按工作流和事件隔离。同一个 PR 的更新可以取消该 PR 的旧运行；main、tag 和 dispatch 使用各自 run ID，不相互取消，不共享排队槽。取消结果继续保留为取消，不作为通过。仓库保护规则、权限和发行环境不随此策略自动修改。

重型作业的分类失败回退使用 `always() && !cancelled()`，保留依赖失败时的全量验证，同时尊重工作流取消。[GitHub 取消机制](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-cancellation)会重新判定运行中作业的条件，不能仅用 `always()` 保住重型作业。取消后的汇总仍要求真实结论与证据，不把取消记为成功。

普通工作流按命令记录阶段耗时、原始退出状态和源码提交，记录不包含命令参数或宿主路径。Foundation、Qt 的 `ci-timings.json` 与 HTML 的 `html-ci-timings.json` 随现有证据保留；命令失败或源码后置检查失败仍使作业失败。阶段计时不改变测试环境、测试顺序、十次整套重复及其时限。

依赖下载缓存由 `scripts/ci-cache-policy.json` 声明，只包含 Cargo 下载归档和 Git 数据、pip 下载、HTML 的 Playwright 工具，以及固定 Qt 对应源码归档。缓存不包含已编译产品、安装的 Qt SDK、历史产品夹具、测试结论或凭据。键绑定输入文件、平台、架构和 runner 镜像，恢复时核对可信 main push 来源与逐文件哈希；不接受部分键或内容差异。只有成功的 main push 保存输入，PR、标签及手动运行仅恢复并重新执行检查。具体冷暖耗时和命中情况以实际运行记录为准。

HTML 输入集合在 `scripts/ci-plan-policy.json` 中集中声明，覆盖 `docs/visual/*.json` 及共享 CI 工具，不再匹配 Git 排除的 `build/review` 生成物。普通工作流始终触发计划与汇总，避免整个工作流因 paths 不匹配而没有检查上下文。shadow 阶段仍执行完整 HTML 检查，输入命中情况只记入计划。Qt 扩展入口中只保留一次完全相同的共享操作控件源码检查；其他领域、桥接、视觉、部署与安装生命周期门槛继续独立执行。

变更分类由 `scripts/ci-plan.py` 使用完整 Git 树差异，不依赖 API 的文件分页或平台 paths 截断。当前为 shadow 模式，全量验证继续运行。普通文档候选仅限 `docs/README.md` 导航页的修改；README 产品说明、工程／接口／CI／发行及验收契约、字体、QML、视觉 JSON、运行资源和未知路径均走全量。新增、删除、改名、模式变化、大差异、缺少基线、fork PR、不能确认的 PR merge tree、显式运行及读取失败也走全量。启用跳过必须另有充分的 shadow 正反例证据；切换配置的提交本身仍走全量，不能把夹具测试当实际平台结果。

四个普通工作流各保留独立的 `docs-gate`、`foundation-gate`、`qt-gate` 和 `html-gate`，使用同一运行中的计划、作业结论及有界证据记录汇总，不新增跨工作流读取权限。重型作业在分类作业失败时继续全量，汇总仍明确失败。适用作业必须成功，三平台必须齐备；失败、取消、超时、重复或缺失平台、跨提交／运行证据及缺少原始日志均不能通过。Qt 额外复核首轮加十次完整浏览器套件、各分片原始 XML、三份 QML 原始日志、原生会话及产品／资源身份；既有平台专用测试的明确 skip 单独计数，不冒充已执行。自动汇总不替代人工桌面验收或发行许可，服务端 required checks 如需调整仍由所有者单独决定。

新增的计划与汇总输入按 run attempt 分别保留。只重跑失败作业时，允许使用同一 run、同一 SHA 中先前成功平台的完整证据，汇总注明各平台的实际 attempt；未来 attempt、跨 run 或跨提交记录不能使用。该规则不跨工作流共享测试结论，也不覆盖既有工件。当前作业结论为失败或取消时，历史成功记录不能使汇总通过。

浏览器分片清单记录驱动实际启动的进程 ID，逐项 JSON/XML 的文件名前缀和身份字段共同绑定该 worker。设置重启等辅助子进程继承计时目录，仍保留各自原始记录，其退出结果继续由原有父测试断言检查；汇总不把这些辅助记录误作分片主记录，也不能用成功的子进程替代缺失、失败或歧义的 worker。

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

[生产更新验收](../../.github/workflows/production-update-acceptance.yml)是明确授权后的独立手动入口，只接受 main，固定源码 SHA，绑定 release-signing 审批且保持 contents:read。构建、生产签名和真实安装分别执行，只有签名步骤接收生产秘密；候选、应用和签名附件不上传。普通 CI 与现有测试密钥更新回归不因此获得生产秘密或 Mac 应用部署权限。English: The separately authorized production-update acceptance workflow runs only from main under the protected signing environment with read-only repository permissions. Only its isolated signing steps receive production secrets. Signed candidates and applications stay on disposable runners; ordinary CI remains unchanged.

Documentation、Foundation probes、Qt components 和 HTML design review 也支持 `workflow_dispatch` 手动核对指定分支或标签。手动执行复用相同权限、检查和平台限制；按实际 `head_sha` 绑定结果，不能用旧提交的成功记录补齐当前提交的缺失检查，也不因此获得发布授权。

HTML 验收和设计导出使用 `scripts/html_review_server.py` 的共享本地 HTTP 服务，保持回环绑定与有界接收队列。浏览器会并发加载脚本、样式、图片和字体，不能用默认的小队列造成资源连接被拒绝，再把缺失全局对象误判为页面状态错误。缩放检查同时保留失败资源请求的路径、类型与错误，以及 JavaScript 堆栈和场景；修正驱动时仍保留原交互断言与等待期限。

English: HTML review and export drivers share the bounded loopback server. Asset-load failures remain distinct from page-state failures, with request and JavaScript diagnostics retained. Server fixes do not relax interaction assertions or deadlines.

[Windows native visual diagnostic](../../.github/workflows/windows-native-visual.yml)是仅支持手动调度的一次性 Windows hosted runner 诊断，使用同提交的 Release 测试程序和原生 windows/D3D11 路线。先执行一次菜单桌面可用性采样，成功后另启完整 15 类、各 3 次的冷进程采样，试跑不计入 45 次；真实前台、自有窗口遮挡、首帧代次和同帧阴影检查保持不变。无法捕获或记录不完整时保留失败和诊断工件，不能记为通过，也不冒充人工点击。

该工作流只在一次性 hosted Windows 虚拟机内，从显示驱动真实枚举且通过系统测试的模式中选择足以容纳完整测试窗口的分辨率。切换不写注册表、不改系统 DPI，记录实际切换前后及测试期间读数，并在 finally 中按同一运行身份恢复原模式；不支持时阻断，不缩小产品组件或放松捕获护栏。该入口禁止在普通本机调用，虚拟屏幕不能冒充物理显示器。

原生布局诊断另启三个独立进程，分别请求 DPR 1、1.25、2，执行双语、浅深色、三档字号和三种窗口宽度的 108 项量测，以及列宽、标签菜单和关于页状态回归。DPR 覆盖来自进程内 Qt 缩放，需与每张图像的实际 DPR 核对；窗口请求值与实际值分别记录，运行器屏幕导致的尺寸钳制必须列为未实现的请求，不能据完整记录数宣称所有请求尺寸已通过。

该工作流另以独立进程采集目录菜单原因、关于页初始视图、文件图标和 Windows 原生边框。截图、日志及实际运行身份只上传到临时 Actions 工件，不生成或安装发行包。共享虚拟机上的时间、系统装饰和单屏图像只用于诊断，不替代固定参考机性能、Windows 11 装饰、物理双屏切换或人工视觉验收。测试程序的实际来源和运行器环境必须与历史人工证据分开记录。

手动输入 `focused_failures=true` 仅运行图标悬停与 DPR 1 混合目录菜单的失败定位。目录组保留主布局、列宽、标签操作、关于页四个前序方法及全部六个双语目录输入数据行，避免省略进程内前序条件。保留原断言、失败判定时的标量状态，以及判定后捕获的自有窗口图像，不将后捕获图像标为失败首帧，不增加重试或放宽标准。该路线单独标记为 `focused_failure_diagnosis`，不执行完整 45 次菜单或三 DPR 的 108 项矩阵，不能作为完整视觉准入。

分支保护必须等真实检查存在且成功运行后指定检查名称；CODEOWNERS 等实际审查者加入后建立。仓库当前可见性和保护设置不由文档生成过程默默修改。

添加窗口使用 C01 独立原稿哈希和 `add-reference.json` 对照，三平台 Qt 量测 12 组语言、主题及字号组合，并执行独立草稿、真实元数据、辅助功能移除、取消和文件/空目录更新。HTML 独立验证 36 组语言、主题、字号、DPR 组合和 24 组真实浏览器 100%/125% 缩放，复用私有 Xvfb 输入隔离，不使用 CSS zoom 代替。检查入口为 `scripts/check-add-dialog.py`，结果只保存为 build/ 或 CI 工件。

元数据过滤回归在三平台核心测试中生成真实 ZIP、TAR、TAR.GZ 创建和更新夹具，再由 `scripts/check-macos-metadata-interop.py` 使用独立 Python 读取器核对元数据排除、原记录保留、空目录、内容和 UTF-8 名称。Qt 覆盖中英文空输入失败反馈及原包不变，沿用共享结果窗口与双语多字号布局门槛；离屏结果不等于桌面人工验收。

Foundation probes 在 Linux x86_64、Windows MSVC x64、macOS arm64 执行 Cargo workspace 测试、Clippy、桥接探针；另检查 Rust/C++ 格式。核心测试不依赖 Qt。

真实 RAR7 algorithm 1 的大逻辑字典检查由 `scripts/check-rar7-genuine-fixture.py` 串行执行组件完整 SHA/CRC、Native 逐字节已知原文对照与 runtime 整包检查，使用固定 commit、长度和双摘要核验的 build-only 小归档及来源 notice，不保存 4.25 GiB 明文。实际 worker 保持 1 GiB 限制，分页容量、WORK、AUX、信用、64 位计数及零目标写入分别记录；只准备夹具或只运行组件均不能标成整链成功。该完整流验证纳入三平台 Foundation，工作流时间预算为 35 分钟，超时仍为失败，不放宽产品内存或文件配额。English: Foundation serially verifies the pinned genuine RAR7 stream at component, Native and runtime layers without materializing plaintext. The 35-minute job allowance changes no product resource limit; preparation or component-only results do not establish full-chain success.

Foundation 的独立 Linux 参考 job 使用固定官方 UnRAR 7.23，仅从固定小安装材料读取 unrar 与原始许可，不执行安装器或作者程序。参考子进程单独受 8 GiB 地址空间、600 秒 CPU/墙钟及完整输出字节上限约束，stdout 以最多 64 KiB 流式核对全零配方、SHA、CRC、精确尺寸和成功退出，不落盘明文。该额度仅属于 CI 外部 oracle，不进入产品请求或 worker 配额；公开作者过去的成功陈述不替代本次实际参考结果。English: A separate Linux oracle checks the complete stream using pinned official UnRAR under its own 8 GiB/600-second guard. This reference-only allowance never changes the product worker budget, and historical creator records do not count as this run's observed output.

自研后端在 Foundation 通过 `scripts/check-rust-codec-policy.py` 检查普通和构建依赖的传递闭包，开发测试参照不计作生产依赖。TAR/GZIP 包含独立读写器双向测试、Huffman 与距离边界、CRC/ISIZE、多成员、截断、资源与取消；`scripts/check-tar-interop.py` 另外使用 Python zlib 的固定、动态、RLE、Huffman-only 与 filtered 策略输入，并通过实际工作进程核对旧编码名称。`gzip-probe` 生成合成零字节、文本及随机数据的四档编码大小、耗时和独立内容核对，JSON 保留在 build/review/independent-rust-codecs/ 并作为 CI 工件上传。有限固定种子变异回归不冒充长期 fuzz，性能数字来自实际工件，不用共享 runner 偶然耗时批准性能。

CAB/ISO/UDF 增量复用同一 Foundation 门槛，`scripts/check-cab-iso-interop.py` 通过实际原生工作进程验证固定来源夹具、已知明文字节、四种 CAB 算法、跨卷文件、ISO9660/Joliet/Rock Ridge/UDF 及只读拒绝。镜像由独立 pycdlib 测试工具预生成并以小型 gzip 输入保存，CI 只校验和展开，不安装该生成器。Rust 与 Qt 覆盖任意 CAB 卷入口、冻结来源、混合命名空间选择、普通文件的解压/临时打开/导出及不支持特殊记录的整包测试失败。结果保留在 build/review/native-codec-interop/，不会进入 docs 或源代码历史。

Rust CAB 固定为产品后端，Foundation 和 Qt 普通构建均执行同一 Rust 路由，没有测试开关或失败回退。构建身份记录固定 Rust 标记，互操作工具同时检查编译清单仅含官方 7z，官方 Cab/Iso/Udf 及 CAB 专用算法不回流。`check-cab-rust-interop.py` 分开记录直接 codec、原始工作进程、真实宿主解压、可选官方参照和负例，绑定实际 worker、探针与夹具摘要；CI 未提供旧参考 worker 时参照数为零，不借用本地结果。makecab 固定输入覆盖 MSZIP 历史及 LZX 多窗口和 aligned 块，Quantum 数学编码向量标明自有生成与独立读取范围，不冒充第三方压缩器样本。`cab_backend_tests` 在本次工作进程中检查帧计量、选中文件、整包检查、源变化、真实部分暂存后的取消和旧目标保护。`test-native-cab-credit.py` 检查有界协议、预计费消费、重放及会话隔离，宿主单测另核对文件、累计输出和部分写入失败的额度处理。有限变异和小预算故障注入不宣称长期 fuzz 或物理耗尽任务总配额。

独立 Rust 光盘后端额外运行 `check-optical-interop.py`：同一已知内容先经直接 codec，再经实际产品 worker 和宿主提取，核对 ISO 多区段、El Torito、UDF 稀疏及 4 GiB 以上随机偏移源；虚拟源检查不冒充实际磁盘大文件提取。预算、取消、快照变化、输出失败、截断及结构损坏分别验证。`optical_backend_tests` 的显式忽略测试在 Foundation 中使用本次构建的 worker 执行，覆盖真实 service 检查、Running 时源变化、已有暂存字节后的取消及发布前来源复验。只有实际执行并核验的路径才记通过。可选保留旧官方 worker 作为独立字节参照，报告必须记录两者摘要且拒绝同一二进制充当参照。结果保留于 build/review/independent-rust-codecs/optical/，分区映射扩展与人工桌面验收仍单列。

IMAPI 和 mkudffs/udfclient 固定光盘输入只含项目自有内容，生成版本、命令及输入/输出摘要保存在 fixture 来源清单。CI 不安装或调用这些生成器，先核验压缩输入，再按有界流展开到 build/，核验逻辑大小与摘要后才交给 codec 和实际宿主服务；不能因 gzip 很小而取消展开上限。sparable 预留空间像具有较大逻辑长度，probe 使用真实文件随机读取，源不变检查采用流式摘要。参考工具拒绝的样本明确单列，不因此减少对 Rust 产品链的已知字节核验。

Qt components 通过 [scripts/desktop.py](../../scripts/desktop.py)复用本地的 prepare、build、基础 test，以及 Windows/Linux package／verify。参数和本地顺序统一见[开发构建指南](developer-build.md)。共用阶段不依赖 RUNNER_TEMP 或 RUNNER_ARCH；工作流显式提供 CI 路径，普通本地使用仓库默认目录。系统编译器／SDK 和 Linux 图形开发依赖仍由宿主／工作流准备，SDK 与打包工具下载使用 prepare 的显式开关。

[ci-desktop.ps1](../../scripts/ci-desktop.ps1)只保留 CI 扩展编排和平台策略，不另实现基础构建链。Qt CI 使用 `test --skip-rust`，同提交的 Rust fmt、Clippy 和 workspace 测试由 Foundation probes 负责；qmllint 与 7 组 CTest 仍由共用 test 执行。测试仅对子进程设置 offscreen／software，之后的原生或部署检查使用各自平台环境。CI 另外重复并发生命周期套件 10 次，保留性能协议、资源与链接证据、双语和多 DPR 视觉、M3／M4 互操作、设置兼容、来源及 SBOM 检查，不因入口复用缩减门槛。

历史完整产品、安装升级／回退、忙时拒绝、卸载和关联生命周期是 CI 显式扩展，继续限定一次性 Windows/Linux 主机或隔离容器。普通本地 verify 不安装产品、不写 HKCU 或更改默认关联；不得为了复用命令移除历史兼容脚本的 CI 与平台限制。运行时核验通过也不替代安装生命周期及人工桌面验收。

macOS 使用已安装的 Qt SDK、sdk 模式部署审计和原生 cocoa 插件执行启动／关闭及文件打开检查。CI 禁止调用 Mac package 或 DMG 核验，不执行 macdeployqt、签名或应用归档，只上传 xdzip-checks-macOS-ARM64 日志与 JSON。这证明原生开发构建可启动，不代表独立发行包验证。Windows 部署后清除 SDK 搜索路径，Linux 使用 SDK-free 运行容器；离屏或软件渲染不证明 GPU、输入法和辅助功能人工验收。

Windows/Linux 继续上传实验运行包、安装器、同提交源码、许可及证据；CI 的 output-dir 保持 build/review，构建目录为 build/desktop，普通本地打包默认 build/package。macOS 不上传 Apple Silicon 应用包。所有记录或工件保存 14 天，均不作为正式签名发行。

Windows 部署验证拒绝 `opengl32sw.dll`，在隔离副本中分别观察默认 Direct3D 11 初始化和显式请求后选中的 Microsoft WARP 软件设备。它不模拟显卡或驱动故障，不将显式 WARP 检查称为自动回退或人工视觉验收。

Qt 组件的原始逐项 `qml-tests.log` 在成功与失败路径都保存到诊断工件，不以可能被后续重复测试覆盖的 `LastTest.log` 摘要替代。构建检查成功时，该原始日志必须存在且非空，缺失或为空使证据收集明确失败；构建本身失败时仍保留可获得的原始诊断。macOS 继续只上传日志和 JSON。

浏览器首轮与十次整套重复各自保留运行清单，以及每个工作进程独立的 Qt Test XML 格式 `.log` 和运行身份、耗时 JSON，位于 `build/review/browser-timings/`。离屏测试自动发现并分配全部测试方法，每个方法只执行一次，进程使用各自的临时设置；原生平台或调用方指定共享设置、视觉审计时保持串行。整个 CTest 仍受同一个 240 秒时限约束，断言、逐项等待时限和重复次数不变。被终止的进程可能只有 `running` 记录或不完整日志，不能记为完成；runner 的 post-job 和最终 job 结论仍须从 Actions 原始步骤与状态独立核对。English: Each offscreen browser run keeps a suite manifest and raw Qt Test timing logs per worker. Offscreen workers receive isolated settings, while native, explicitly shared settings and visual-audit runs stay serial. The same 240-second CTest deadline covers all discovered methods, with unchanged assertions, per-check deadlines and repetition count.

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
