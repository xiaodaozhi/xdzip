# 验收执行计划

## macOS 候选来源证据

| 场景 | 操作与判定 |
| --- | --- |
| QT-NETWORK-SECURITY | 固定源码、五份官方补丁和构建器身份，保留原 Network 功能与导出、对原 QtCore 的导入。实际加载候选库，超限状态行、未终止响应头、分片和 CONNECT 在夹具取消前拒绝，正常响应、合法长头、各行终止形式、受信测试 CA 与 CONNECT TLS 可用，不受信证书仍拒绝；执行上游认证坏输入与正常向量。失败恢复原 SDK；缺失、陈旧回执、替换库或 SDK 工具、来源补丁缺失及仅文件名归属均不得进入产品/发行。记录实际平台，macOS 14 无环境时继续未验证，不由其他系统代填 |

English: QT-NETWORK-SECURITY binds source, patches, features and ABI to the actual library. Bounded HTTP/CONNECT/TLS and upstream authentication controls verify early rejection and legitimate behavior. Failed preparation restores the SDK, and changed/missing receipts, tools, patches or filename-only provenance prevent delivery. Platform coverage remains explicit; other macOS versions do not prove macOS 14 execution.

| 场景 | 操作与判定 |
| --- | --- |
| RELEASE-MAC-EVIDENCE | 专用 Release 在部署应用仍存在时实际运行 Cocoa／Metal 资源探针，并核对审计、版本和源码提交；链接输入绑定对应编译应用，复用既有 Mach-O 比较器核对节字节、地址、标志、零填充大小、UUID及归一化导入版本。签名／加载命令布局和LINKEDIT保留独立范围。负向检查拒绝变化节／UUID／导入、截断／越界、错误架构及混合资源／链接／SDK 身份；检查 SDK 对应、差异分析、来源／文件 SBOM 和总回执确实进入同包材料。原始 gaps／incomplete、参考机预算和真人视觉分别保留，自动结果不代填人工通过 |

English: RELEASE-MAC-EVIDENCE runs a real Cocoa/Metal resource probe against the deployed candidate, reuses exact section/address/flag, zero-fill, UUID and normalized import comparison to bind compiled link inputs, rejects changed or mixed/boundedness-invalid evidence, and checks the same-package SDK, difference and SBOM records. Signing/load-command layout and LINKEDIT, original gaps, reference budgets and human visual acceptance remain separate.

| 场景 | 操作与判定 |
| --- | --- |
| RELEASE-VALIDATION-UNSIGNED | 本地候选汇总的 `--validation-only` 模式不接收更新签名密钥或 key ID；即使调用环境注入自有测试密钥，也不生成清单签名或 appcast，并拒绝残留签名附件及与强制签名冲突的参数。该模式不属于 Release 工作流，正式发行路径只使用单次手动运行并强制验证完整更新签名 |

English: RELEASE-VALIDATION-UNSIGNED applies only to local `--validation-only` candidate assembly. It prevents that mode from receiving signing inputs or producing update signatures, including with injected fixture keys, and rejects stale signing attachments and contradictory flags. The Release workflow has no validation-only dispatch; production signing and upgrade acceptance remain separate.

## 标题区、任务历史与拖动提示

| 场景 | 操作与判定 |
| --- | --- |
| USER-DATA-FONT | 中英文、100/125/150% 字号及支持的平台，使用普通中文、子集未覆盖的汉字、多语言文字、表情和长路径；核对列表／网格、目录树、面包屑、标签与提示、任务及操作窗口优先使用启动时系统字体，固定文案保持内置子集。缺字由系统回退，按实际平台记录可用性；检查长文字省略、换行、宽度计算、焦点和可访问名称，字体变化不改变实际名称或文件操作。添加窗口的静态根目录说明保持界面字体，真实包内目录使用系统字体。原生视觉和系统字形覆盖单独记录 |
| HEADER-SHARED | 创建、添加、解压、任务、结果、冲突和设置，检查中英文、浅深色、100/125/150%字号。图标48、标题20、副标题13、左右24、正常高度基准96；长文字同规则增高，标题区至首个正文分组的额外间距为0；任务运行、成功、失败、取消均量测该边界，正文内部段距保留，正文与适用的 footer 可达，设置无 footer，主窗口／菜单比例另验 |
| CLOSED-BROWSER-STATE | 目录、选择、搜索、排序和网格视图变化后关闭标签或窗口，分别在无任务、运行中及完成记录下重新打开；新页面从根目录、无选择和默认列表开始，旧任务身份及提示保留。归档修改中的重新打开可等待或取消等待，不取消原任务；覆盖迟到回执、其他窗口和 32 会话上限 |
| DESTINATION-CHECK | 创建与解压分别在初始、选目录和编辑结束时检查目标；交错及快速编辑只接受当前窗口、路径和解压会话的最新结果，未通过前不能提交。创建准备期间添加／移除按钮及起始页不闪烁；两窗状态图标与提示一致，保留解压路径布局、同名文件夹与暂存说明，覆盖双语、浅深色和多字号 |
| APP-ZIP-DESTINATION | 对 `*.app.zip` 保留默认同名文件夹，核对预览、任务实际目标和外层普通目录名称一致且不以 `.app` 结尾；真实应用包位于其中。覆盖大小写、Unicode、空名称回退与其他归档后缀，取消同名文件夹时路径不变；本机对实际应用 ZIP 与独立解包结果比较内容、链接和普通文件安全权限，并检查签名完整性及启动 |
| TASK-HISTORY-CLEAR | 混合已完成、失败、取消、运行、等待、取消清理与跨会话同数字ID；只清除终态历史，活动计数、密码／冲突令牌和稳定行不变，重复终态不复活，结果窗口关闭，源与输出不被删除 |
| TASK-MENU-VIEWPORT | 短列表不预留滚动条宽度，状态标记靠右；长列表内部滚动，Home/End和键盘可达，最大高度360且受窗口约束，清除区固定。分隔线上下各4，菜单项左右内距与上下相等，按钮底部只保留共享菜单内距。鼠标点击、键盘Enter/Space、原生辅助功能press均真实清除终态历史，重开不复活；没有完成历史时禁用 |
| TASK-MENU-SHORT-REOPEN | 完成任务、清除历史，再产生新任务并打开菜单；短列表的每行无需滚动即可完整显示，点击新行立即打开对应任务。中英文、浅深色和三档字号均覆盖；长列表仍能滚动至首尾 |
| TASK-MENU-KIND | 中英文、浅深色、100/125/150%字号下，检查、解压、创建、添加、移动与导出的菜单图标、无障碍操作名及实际打开窗口的kind一致；跨会话同数字ID仍打开原任务，不切换归档标签 |
| TASK-MENU-ICON-TAGS | 任务行按操作图标、文件名、序号标签、状态标签排列，序号与状态共用样式，长文件名省略且不覆盖标签；辅助功能名称保留操作和序号，跨会话身份与唤起行为正确 |
| EXPORT-NO-AUTO-OPEN | 先执行明确选择完成后打开目录的普通解压，再拖出及复制文件，覆盖任务先完成和登记先完成两种顺序；导出完成不自动打开系统目录，原文件内容和明确的解压打开选择保持各自行为 |
| EXPORT-BROWSER-STABLE | 列表／网格通过越出窗口或窗口失活交接为系统拖出，逐次检查清单准备的状态通知与加载层可见性，不闪现“正在读取”，列表、选择及目录保持；释放取消和真实导出行为不变，后续目录查询仍显示真实等待反馈 |
| DRAG-SOURCE-TOOLTIP | 列表与网格先显示源名称，再按下拖到其他位置并保持超过1200ms，检查无源提示；原生交接同样抑制，结束后缓存hover不能复活，需新指针位置及完整延时；Escape取消不改包 |
| MAIN-FOOTER-PARITY | 起始页与压缩文件页在正常和大字号下等高，最低32逻辑像素，切换页面不改变底栏高度；两页任务按钮、图标和箭头的右侧位置一致，距底栏右边缘14逻辑像素，菜单随按钮对齐。中英文、浅深色、三档字号及常规／窄窗口均检查 |
| MAIN-FOOTER-SELECTION | 中英文下检查未选择、已知大小、零字节、虚拟目录和未知大小文件及其混合选择；未知总大小只显示选中数量，已知大小继续显示，欢迎页与预检行为不变 |
| OWNER-DIALOG-CLOSE | 带设置、关于和操作窗关闭主窗口，附属原生窗口资源释放，未保存预览撤销，托盘恢复不重显；保留任务仍可由全局任务入口明确重新打开 |
| ADD-CONFLICT-PAGE-CANCEL | 拖入同名文件并开始添加，关闭重名提示和任务窗后从主页面取消；任务终结，原包字节不变，归档重开后恢复选择与浏览 |
| ARCHIVE-OPERATION-OVERLAY | 添加、包内移动期间原文件页可见且不可重复操作，等待层背景透明；中英文及大字号下，标题、说明、进度辅助名称和取消按钮对应操作，清理和刷新期间不误称加载 |
| TASK-PROMPT-LIFECYCLE | 页面、任务窗或全局入口取消后，重名和密码提示关闭，进度显示最新取消与终态；关闭同一提示后其他任务更新不使其复活，显式重开、任务切换和新提示令牌按各自身份处理，旧决定被拒绝 |

这些是验收要求，实际结果及原生平台限制另记issue与build/或CI工件。

## 用户主动更新

- 检查核心夹具：稳定/alpha/beta/rc/dev、同版本和降级、build 不递增、分页乱序、缺包、错误平台/架构/最低系统、签名或候选身份不一致。无签名或未知信任根只提供手动入口。
- 网络检查离线、TLS、超时、限流、受控重定向、截断、空间不足、取消、重复点击及 ETag；任何失败不显示已是最新或启动安装。部署后清除 SDK 搜索路径，验证真实 HTTPS。
- 在隔离三平台用独立测试密钥从旧版升级到新版，核对任务/密码等待、退出取消、安装中再启动、授权取消、失败、恢复和单实例重启；设置和默认关联须保留。Windows 交互安装保留许可确认，Linux 未配置 dpkg 不报成功或回滚，Mac 必须 Apple Silicon 原生执行完整 Sparkle 归档验证、替换和重启。
- 密码与更新等待并存时，通过真实主程序的系统 Quit 路由进入退出确认，点击返回后原任务与请求令牌不变，取消并退出后所有 Rust join 完成且没有安装交接。另核验文件选择器和不相关模态窗口仍阻挡退出，不能以直接调用会话接口代替桌面入口回归。While password and update waiting coexist, exercise the real main program's system Quit route and Return/Cancel buttons. Preserve the task/token on Return; require all joins and no install handoff on cancellation. File selectors and unrelated modals remain blocking; a direct session call does not cover the desktop route.
- macOS 密码和冲突任务提示期间核对应用菜单的退出项及 Command-Q，通过退出确认返回后任务和提示身份不变；不相关模态窗口及文件选择器仍阻挡退出，期间触发的退出不能在模态关闭后延迟执行。On macOS, check the application-menu Quit item and Command-Q during password and conflict task prompts. Return preserves the task and prompt identity; unrelated modals and file selectors still block Quit, including after they close.
- 关于页覆盖全部状态，关闭重开保持任务，取消显式执行；检查中英文、浅深色、三档字号和窄屏滚动，复用操作控件。生产密钥和真实平台未验证项独立列出，普通 CI 不部署/签名/上传 Mac 应用包。
- macOS 手动回退分别覆盖安装身份不合格、无 appcast、无效网络配置及无可交接会话，显示实际首先未满足的条件并保留发布页和下载位置；双语、浅深色及三档字号下说明完整可读。下载完成不得显示为安装完成。

T01～T18 的权威映射见 design/requirements-traceability.md。以下是可复用的验收规格，不表示本次已执行或均未执行。实际结果在对应 issue 和工件记录，既有人工结论见 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)；新候选按受影响项复验，不创建假的 pass 报告。

| 用例 | 前置与操作 | 通过标准 |
| --- | --- | --- |
| AC-01 / T01 | 记录输出/临时目录快照，打开普通 ZIP 并浏览 | 没有文件内容被解压；只允许受控元数据和明确文档规定的缓存 |
| AC-02 / T02 | 十万条归档，排序/搜索/范围选择，快速切换查询 | 选择绑定 EntryId；旧页不污染新查询；内存有界 |
| AC-03 / T03 | 搜索后清空选择，点击全部解压；再选择部分解压 | 范围明确，全部不偷偷等同搜索结果，结果字节相符 |
| AC-04 / T04 | 创建 ZIP/TAR/TAR.GZ，独立工具读取；反向生成再读取 | 路径与内容一致，能力外特性明确拒绝 |
| AC-05 / T05 | 创建 AES-256／编号分卷并用独立工具校验；读取支持的 AES-256、编号与 PKZIP 分卷，再尝试其他加密 | 密码创建及受支持解压双向互操作；不支持的读取明确拒绝，不发布未认证文件 |
| AC-06 / T06/T16 | 队列满、取消、重复取消、关闭应用与提交竞争 | 无死锁，终态一次，取消前已提交项可追踪 |
| AC-07 / T07 | 原生 OS 上注入 ../、绝对路径、链接替换、ADS 等 | 授权目录外无变化，已有目标不被非授权修改 |
| AC-08 / T08 | 三平台中文 IME、键盘、读屏、125/150/200% 缩放 | 无关键操作不可达、无裁字/焦点丢失，状态可听读 |
| AC-09 / T09 | 固定参考机冷/热启动、空闲、多任务、真实 I/O | 输出全进程树及完整包数据，与预算比较并说明误差 |
| AC-10 / T10/T13 | 无 Qt 核心测试；后续两个独立外部扩展互通 | 依赖单向、能力驱动；越权/异常帧被拒绝且宿主继续运行 |
| AC-11 / T11 | 默认使用、错误、诊断导出 | 无默认网络传输，不包含秘密/完整路径 |
| AC-12 / T12/T18 | 干净机安装、更新、卸载及 Qt 依赖验证 | 无开发机隐式依赖，必要 ad-hoc 完整性和未正式签名说明、来源与许可材料齐全，活跃任务不被破坏 |
| AC-13 / T14 | 超大图像/文本、HTML/可执行内容触发预览 | 资源受限、不自动执行、关闭后清理；仅 M6+ |
| AC-14 / T15 | 索引后改变源、磁盘满、发布前后终止进程 | 不报假成功，不删除已提交/用户后改文件，恢复明确 |
| AC-15 / T17 | 元数据/名称/帧/恢复日志 fuzz 与最小回归样本 | 无越界/不受控资源增长，失败可诊断，修复有回归 |

结果记录字段：提交、用例、平台/文件系统/架构、工具版本、样本哈希、步骤、期望、实际、状态、日志位置、执行者和日期。状态仅为未运行/通过/失败/阻塞；不把跳过映射为通过。

首版只执行适用的能力；第三方扩展/预览后期用例保留不适用原因。每次变更选择受影响用例；发行前执行所声明能力的完整集合。

## 创建输入与输出重叠，AC-04 / AC-06 / AC-07 / AC-14

| 用例 | 输入与操作 | 通过标准 |
| --- | --- | --- |
| CREATE-OUT-01 | ZIP、TAR、TAR.GZ 显式添加已有目标普通文件及其他输入，与选择包含目标的目录对照 | 全部输入字节完整入包；最终显示正常重名决策，目标参与输入时禁用覆盖，改名与取消保留原文件，伪造覆盖回答也不能绕过保护 |
| CREATE-OUT-02 | 只添加已有目标文件，运行真实 Qt 创建 | 正常进入重名守卫，新名称的压缩包包含旧目标原字节，不报空输入或特殊对象；双语保护提示清楚，大字号下正文和 footer 可达 |
| CREATE-OUT-03 | 父路径系统别名、大小写／Unicode 等价目录项，异目录同名文件、大小写敏感卷的独立文件，以及硬链接、链接、目录和特殊对象 | 不静默过滤普通输入；输入身份保护由最终守卫判断，链接／特殊对象与硬链接目标限制继续有效，私有工作目录及别名不得成为输入 |
| CREATE-OUT-04 | 分卷输入包含已有 `.001`、`.1024`、基名 `.zip`、`.1025` 及其他 `.xdzip-*` 普通文件 | 仅排除身份匹配的私有暂存目录；所有旧卷及其他普通文件原字节保留于新卷组，整组改名或取消不覆盖旧卷 |

规则与失败前提见[创建契约](../engineering/m4-creation.md)。只使用自行生成的临时输入，核对实际输出内容、取消后的旧字节和暂存清理；运行结果记录在对应 issue 与工件。

## 密码解压，AC-05 / AC-06 / AC-07 / AC-08 / AC-11

| 用例 | 输入与操作 | 通过标准 |
| --- | --- | --- |
| AES-01 | 本应用及独立 7-Zip 创建 Store／Deflate AES-256，AE-1／AE-2、ZIP64 描述符、空文件与 Unicode 名称 | 物理条目、内容及目录一致；AE-1 CRC 和 AE-2 零 CRC、完整认证均按契约处理 |
| AES-02 | 无密码开始、错误密码、正确重试、不同文件使用不同密码、明密混合与同名物理记录 | 新 token 只消费一次，缓存仅当前任务；重试正确时逐文件提交，不将其他物理项内容混入当前项 |
| AES-03 | 空密码、空格／控制字符、1,024 字节边界、过长或过大容量输入 | 解压独立规则有效，原样传递；超限拒绝且不保留秘密，不套用创建密码限制 |
| AES-04 | 认证码、CRC、声明、局部／中央字段、Deflate 终止或尾部损坏 | 明确失败，无未认证目标发布，原有目标不变；已有较早提交如实计数，清理失败有警告 |
| AES-05 | 等待密码时取消、关闭密码窗、退出、填满普通队列；重复／过期／跨类型响应 | 控制通道仍可用，无死锁；输入清空，旧响应不能复活任务或成为重名决定 |
| AES-06 | 连续错密至上限、认证前输出、重复解码触及字节上限、等待期间修改来源 | 次数与累计预算不重置；来源复核和认证失败停止，不能绕过配额或提交旧来源内容 |
| AES-07 | POSIX 加密链接图、安全执行权限、非法链接、占用叶子、Windows／TAR 链接 | 链接图认证在输出授权前完成，既有 no-follow、闭合图、权限过滤和平台拒绝策略保持有效 |
| AES-08 | 真实 Qt 创建→自动打开→解压→错密→正确输入，关闭／退出；双语、浅深色、100／125／150%、DPR 1／1.25／2、小屏与长路径 | 密码窗 491 宽、最高 600、footer 固定可达；输入密码角色与可读名称正确，显示／隐藏及提交辅助功能动作路由真实请求，不记录密码 |
| AES-ACCESSIBLE-NAME | 中英文的创建与解压密码输入，辅助功能在创建前及显示后激活，检查名称、密码标记、文本与值遮蔽以及显示／隐藏切换 | 名称可读，遮蔽状态不泄露输入；普通输入框保持原行为。自动接口检查、Windows UI Automation 和真人读屏分别记录，不记录输入值 |

格式、秘密、输出及交互前提见[密码解压契约](../engineering/password-extraction.md)。实际执行结果在 issue #11 与本地／CI 工件记录，不把自动化接口检查记为系统读屏人工验收。

## 密码提示与全会话退出顺序，AC-06 / AC-08

以下是可重复的验收要求，执行结果另记 issue 与工件，不预记为通过。使用真实 AES 密码等待与正常应用 Quit 路由，不通过手动隐藏窗口、停用自动显示或直接调用会话退出接口制造时序。

| 用例 | 输入与操作 | 通过标准 |
| --- | --- | --- |
| QUIT-PROMPT-01 | 密码窗已实际显示，系统 Quit、返回，再 Quit、取消 | 原生文件选择器仍阻挡退出，受管理密码窗允许退出确认覆盖；返回保留任务与令牌，取消后全部 Rust join 且无更新交接 |
| QUIT-PROMPT-02 | 真实任务通知已建立 token、QML 已同步 shown token、密码窗仍隐藏时，在同一通知栈同步发送 Quit | 退出确认先显示并在排队密码回调执行后仍为当前模态；返回后同一密码请求可用，取消仍完整 join |
| QUIT-PROMPT-03 | 同样的真实早期请求，在另一会话或另一主窗口活动时 Quit | 应用级决定阻挡所有会话的延迟密码显示，不能由非当前页或非当前窗口抢走退出确认 |
| QUIT-PROMPT-04 | 自然显示密码窗后，在已运行且有时限的事件循环中正常 post Quit | 真实事件经过应用退出过滤器，保存实际投递窗口状态；这是已显示后的投递对照，不计为早期窗口覆盖 |
| QUIT-PROMPT-05 | 早期 Quit 后选择等待，再 Quit、返回，再 Quit、取消 | 等待解除密码显示暂停且保留同 token；再次询问清除旧等待决定，返回后不保留退出暂停，最终取消仍等待全部连接 |

记录六个数据行的自然 hook、Quit 实际处理前后、延迟回调后、当前模态、任务与提示 token、活动任务数及 join 结果。受控同步通知是顺序回归，不能证明历史 macOS 普通 posted Quit 的唯一先后关系。Qt 显示新窗口可能移除已排队的 Quit，不能把未送达事件改报为产品已处理或降低投递断言。窗口尺寸与双语多字号按窗口清单检查；原生视觉按需检查并记录实际结果。

English: These are repeatable acceptance requirements, not execution results. Use real AES password waiting and the normal application Quit route. Cover a ready prompt, synchronous Quit while a naturally created prompt has not yet been presented, another active session and window, a genuinely posted Quit after presentation, and Wait → Quit → Return → Quit → Cancel. The quit decision must remain modal after queued callbacks; Return/Wait preserve the same token, and cancellation joins every runtime without update handoff. Preserve actual hook/delivery/modal/join records. A controlled notification order does not establish an earlier macOS event order. A dropped posted event is not a handled Quit. Native visuals and accessibility remain separate checks.

## 文件输入与 macOS ACL 回归，AC-07 / AC-14 / AC-15

| 用例 | 输入与操作 | 通过标准 |
| --- | --- | --- |
| IO-01 | 核对运行时首次打开的真实源句柄使用非阻塞分类；以 FIFO 及指向 FIFO 的别名替换路径，另检查目录等特殊对象 | 普通压缩包可浏览，特殊对象返回 `Io`，不会等待 FIFO 对端；使用有时限的独立测试进程，不能因回归挂死整套测试 |
| IO-02 | 通过指向普通文件的源符号链接打开，再对来源复核及任务来源副本执行路径置换 | 合法源别名仍可用；特殊对象和来源身份／版本变化返回 `SourceChanged`，不发布失效查询或启动错误来源的任务 |
| MAC-ACL-01 | 自有临时目标配置 `everyone deny delete`、外部只读／执行 ACL 及其可继承形式，执行解压或创建的实际保存 | 目录授权、暂存、新建父目录、发布及清理可完成，输出字节正确，用户 ACL 未被改写 |
| MAC-ACL-02 | 分别给外部用户和组授予写入、创建、追加、删除、写属性、写扩展属性、改 ACL、改所有者及仅继承修改权限 | 输出前拒绝，旧内容与目标外文件不变，无半文件发布；前置 deny 不豁免危险 allow |
| MAC-ACL-03 | 当前有效用户的修改 ACL，与外部用户／组规则进行对照 | 系统解析和合成用户 UUID 均可识别当前用户；组主体不能误认作当前用户，未知权限、未知规则及查询失败保持拒绝 |
| MAC-ACL-04 | 授权后给目标父链或已有输出增加危险 ACL，再继续暂存或提交 | 复核拒绝不再安全的对象；POSIX mode、所有权、单硬链接及 no-follow 限制保持有效 |

ACL 夹具只修改测试自行创建的对象，不清除或更改真实用户目录权限。非阻塞分类打开只解决特殊对象打开边界，普通文件的缓慢 I/O、原子内容快照和同账户恶意并发不由这些用例证明。原生平台结果和剩余项记录于对应 issue 与本地／CI 工件，不能以跨平台编译、mock 或独立进程超时替代实际安全结果。


## 操作与选择回归清单

本表为验收要求，运行结果、截图和测量仅保存到 build/ 或 dist/。

| 编号 | 输入与操作 | 必须观察的结果 |
| --- | --- | --- |
| UI-REV-01 | 运行创建／解压，切换终态，展开再收起明细 | 窗口同宽 491，内容决定高度，最多 600，明细列表最多 168，收起恢复紧凑高度 |
| UI-REV-02 | 在三个格式选项中依次选择、聚焦和按下 | 只有最左的左角与最右的右角为圆角，内侧角始终平直 |
| UI-REV-03 | 浅深色下悬停、鼠标按下、空格按下、禁用蓝色按钮 | normal、hover、pressed、disabled 清晰区分，使用共享色值 |
| UI-REV-04 | 编辑解压目标，外部点击或按 Escape／Return／Enter | 字段失焦且不启动解压，第一次 Escape 不关闭窗口 |
| UI-REV-05 | 显示所选与全部两种范围 | 标题在左，数量在最右，数量为次要色，整行可选 |
| UI-REV-06 | 创建与解压遇到重名，100%／125%／150% 字号及历史 200% 压力场景 | 重名窗宽 491，标题 20、正文 13，footer 13，最高 560，正文滚动且动作可达 |
| UI-REV-07 | 对同一查询全选、取消、逐项选到末项 | 全选与逐项末项属性一致，按钮可用；无选择时四项属性和动作隐藏 |
| UI-REV-08 | 悬停和按下预览区解压动作 | hover／pressed／focus 均可见，未授权状态禁用 |
| UI-REV-09 | 完成多个任务，从底栏菜单重开 | 历史数准确，活动计数仍为 0，状态标记在右，菜单正常密度且可键盘导航 |
| UI-REV-10 | 清空选择，打开全部解压下拉 | 所选及预检禁用，不出现多余选择提示行 |
| UI-REV-11 | HTML 100%／125%，Qt DPR 1／1.25／2，长路径及辅助字号 | 逻辑尺寸不随 DPR 改变，正文与菜单比例正确，footer 可达 |
| UI-REV-12 | 目录树根入口、普通父目录与叶子目录，右键、Menu 键、Shift+F10，双语、主题与三档字号 | 根入口的“收起子目录”禁用，所有不可用菜单项均无可见 hint、原因副行或 tooltip；辅助功能仍能读取准确的根入口及叶子原因。父目录可展开和收起，禁用操作不改变选择或路径。主标签在菜单项局部坐标中的中心偏差不超过 0.5 逻辑像素，祖先平移不得改变测量结果；不能忽略局部链上的缩放、旋转或自定义变换。覆盖关闭时切换字号后重开及已打开时切换字号的首个观测渲染，核对当前文字高度和各条件槽位。Disabled menu actions show no hint, reason subline or tooltip; accessible descriptions retain accurate root and leaf reasons, while parent toggle and selection/navigation behavior remain intact. Measure label centering within 0.5 logical pixels in the command's local coordinates, independently of ancestor translation; reject unhandled scale, rotation or custom transforms. Check current text height and conditional slots at the first observed rendering after closed and open font changes. |
| UI-REV-13 | HTML 共享菜单的延迟滚动事件 | 通过真实右键输入在菜单打开前改变非祖先容器位置，延迟送达但位置未再变化的真实滚动事件不关闭菜单；打开后再次改变位置仍关闭。原双语三档字号及三种输入的 18 组目录组合保持通过。A trusted scroll queued by the preceding real context-menu input preserves the menu when the non-ancestor position is unchanged since opening; a subsequent position change closes it. Retain all 18 directory input, language and text-scale combinations. |
| UI-REV-14 | 中英文、浅深色和三档字号下启动并打开创建窗口 | 选项标签可换行，四个格式文字不截断，窗口与 footer 可达；初始化和展示均不产生 Qt Quick 布局递归重排警告。In both languages, themes and all three text scales, labels wrap, format captions fit and the footer remains reachable, without a recursive layout warning during creation or presentation. |
| UI-REV-15 | 在主窗口搜索、普通和密码输入框及可编辑下拉框中用鼠标、键盘选择与替换文字，切换浅深色 | 选区文字与背景均可读，密码模式和原有编辑行为保持有效。Selection foreground and background remain readable in both themes; password mode and editing behavior remain intact. |
| UI-REV-16 | 单个空闲起始页、加载或错误状态、单归档及多个标签，浅深色和三档字号 | 唯一普通起始页不显示关闭按钮且按钮不可聚焦；其他情况按原规则显示，鼠标、键盘和辅助功能仍可关闭。关闭按钮悬停及按下背景为圆形，主窗口和菜单比例不变。The sole ordinary idle welcome tab hides its close button and removes it from focus; loading, errors, archives and multiple tabs preserve existing behavior. Hover and pressed backgrounds are circular. |

实现回归位于 tst_operationrevisions.qml、browser_test.cpp 和共享控件检查脚本；高保真与 HTML 修订入口见 design/hifi/operation-revisions.html。

## WP-11 TAR/GZIP

| ID | 输入／操作 | 预期 |
| --- | --- | --- |
| TAR-01 | 系统 tar／Python USTAR、GNU、PAX，TAR/GZIP 和分段多 member GZIP | 内容探测，完整扫描，名称及全部字节一致，空目录保留 |
| TAR-02 | 从 Qt 选择 TAR／TAR.GZ，创建后重开并解压 | 格式与扩展名一致，任务成功，输出字节一致，不显示虚构的逐条压缩大小 |
| TAR-03 | 密码、分卷启用后切换格式 | 不适用参数清空且禁用，TAR 级别禁用，TAR.GZ 0／1／6／9 可写 |
| TAR-04 | 路径逃逸、链接、设备、FIFO、稀疏和全局 PAX | 不创建不安全输出，明确拒绝，不通过库 unpack 写入 |
| TAR-05 | 截断、头校验和、CRC／ISIZE、非零尾部、扩展头和字节预算 | 完整扫描失败，不发布部分索引；超额、取消可解释 |
| TAR-06 | 复合扩展名冲突改名，覆盖任务在校验阶段取消 | 保留 .tar.gz，旧归档不变，本次暂存清理 |
| TAR-07 | 高压缩比 GZIP，未允许及明确允许两种意图 | 默认在写入前拒绝，允许后仍执行来源、字节配额和安全提交检查 |

测试入口为 codec-tar、runtime creation 测试、Qt tarCreationBrowseAndSafeExtraction 及 [独立互操作脚本](../../scripts/check-tar-interop.py)。实际日志与截图仅在 build/review/wp11 或 CI 工件。

桌面权限或设备曾导致的历史阻塞不作为当前状态。人工验收结论以对应 issue 为准，不把离屏或 Cocoa 自动化记为人工操作。

## W01 欢迎页接入，AC-01 / AC-04 / AC-08

| 用例 | 操作 | 通过标准 |
| --- | --- | --- |
| W01-01 | 启动、打开普通归档、关闭归档 | 欢迎页切换真实浏览并恢复，空归档仍进入浏览 |
| W01-02 | 欢迎页新建、关闭草稿 | 打开真实创建窗口，不生成任务，焦点恢复 |
| W01-03 | 拖入一个普通 ZIP/TAR/TAR.GZ/TGZ | 通过既有核心探测打开，不自动解压 |
| W01-04 | 拖入多个、目录、网络 URL、不支持扩展名或损坏 ZIP | 输入形态拒绝或显示真实核心错误，无假成功 |
| W01-05 | 浅深色，DPR 1 / 1.25 / 2，最小窗口，100%／125%／150% 字号 | 卡片与共享菜单比例正确，入口可键盘访问，正文可滚动 |
| W01-06 | 运行任务后关闭归档，再从欢迎页任务菜单重开 | 历史数量与真实状态保留，欢迎页不取消后台任务 |
| W01-09 | 打开失败或取消后重试，关闭含后台任务的窗口并再次打开同一归档 | 可重用空闲起始页；仅回收同窗内未使用的自动欢迎页，手动页面、其他窗口、对话框、密码提示和任务身份保留。Reuse an idle welcome page after failure or cancellation. Reopening a task-retained archive retires only an untouched automatic welcome in its window, preserving user pages, other windows, dialogs, prompts and task identity. |

自动化记录只写入 build/review/welcome/ 或 CI 工件；原生拖放、系统文件选择器取消、读屏、输入法和平台窗口外观按变更需要选择桌面检查，未执行时记录限制，不以离屏截图标记通过。

W01-07，欢迎页与打开归档后的标题栏、标签及外观按钮的位置、尺寸、字体和图标沿用同一组件，不允许欢迎页专属重排。自动化比较 Qt 实际几何，并在 HTML 检查共享标题栏的源坐标。

### 全窗口焦点与标题栏修订

- 共享控件获得鼠标或键盘焦点时，无额外 outline、光晕或加粗边框；正常边框与按钮状态不变。
- 外部点击、Escape、Return、数字键盘 Enter 能退出控件焦点；Enter 原有动作执行一次，搜索文字保留，第一次 Escape 不误关闭窗口。菜单方向键与逐级关闭可用，关闭后不残留触发控件焦点。
- 所有应用窗口标题无“ — xdzip”后缀。macOS 主窗口启动、调整大小及重新激活后，红绿灯中心与标题栏中心误差不超过 0.5 逻辑像素，且不被祖先容器裁剪。
- 自动化结果保存于 build/ 或 dist/；执行真实桌面外观检查时单独记录结果。

### S01 设置与国际化

- 保留原稿图标标题、主题插画卡片、列表密度、最近记录和隐私区域；新增语言沿用原有行布局。
- 标准／紧凑影响真实表格，取消回滚；最近记录关闭后停止新增，清空先标记，完成后仅删除记录文件。

- 标题栏设置、更多菜单和平台设置快捷键进入同一独立非模态设置窗口。
- 主题跟随系统／浅色／深色，文字 100%／125%／150% 均为真实预览；取消及系统关闭回滚，恢复默认只改草稿，保存成功才持久化，失败保留草稿与错误说明。
- 中文 (简体) 与 English (US) 保存后下次启动生效；跨进程恢复已保存选择，不翻译归档内容、协议码、路径或用户文件名。
- 两份语言目录 ID、复数及占位符完整，QML/C++ 产品文案无散落静态中文。英文欢迎页、操作窗标题、文件类型、错误提示、数字／时间／字节显示与 100%／125%／150% 文字布局需验证。
- 原生系统主题、键盘、读屏及真实桌面外观按需检查；自动化与设计稿不能冒充未执行的桌面检查。

- 设置下拉框在 100%、125%、150% 下右对齐，宽度覆盖完整选项文字与勾选列，至少等于按钮宽，超长项受窗口与最大宽度约束。
- 双语、浅深色和三档字号下，密度、文字大小、语言与打开方式四个下拉框等高，文字及指示器不裁切。All four settings selectors have equal height with unclipped text and indicators in both languages and themes at every supported text scale.
- footer 在 150% 及历史 200% 布局验证中按钮保持同一行，提示可独立换行；旧 200% 设置迁移为 150%。
- macOS 本地化声明和实际构建包均检查，中文系统下原生选择器使用系统语言，应用语言切换不写入系统偏好。按需检查真实原生对话框外观并登记实际结果。

### 解压任务提交的视觉稳定性

- 实际点击“开始解压”时，主窗口工具栏与文件列表保持既有状态，不短暂显示目录加载遮罩；解压窗口“更改”按钮不切换禁用样式。
- 目录、搜索或排序查询期间，已提交的非空列表／网格与空结果均保持可见，直到新结果提交；核对遮罩可见性、最终目录、查询结果以及 busy／取消行为。
- 开始动作立即防重入，重复调用只创建一个任务；受理后切换任务窗口，失败保留确认内容。
- 真实打开／切换目录、搜索与预检仍遵守原有请求准入和加载反馈，任务提交不重置模型或清空选择。

### 预检、行边界与窗口归属回归

- 预检 491 宽、正文 13、标题 20；少量内容收紧，64 条问题与 150% 字号正文可滚动，客户区最高 600，footer 返回按钮始终可达。
- 主列表色块与分隔线两侧可见空隙相等；预览显示与关闭按钮的窗口坐标相同，主窗口与菜单字号比例不变。
- 所有原生文件／目录选择入口归属于实际调用窗口，取消返回该窗口，不能把主窗口抬到操作窗口上方；调用窗口关闭时不能重新显示它。自有非原生选择器可验证共用 ownership 策略，不冒充已执行 Cocoa 原生选择器检查。
- Finder 拖放有效归档后接收主窗口留在前台；无效拖放不激活。原生桌面行为按需验证，未执行时记录限制。
- POSIX ZIP Framework 多级符号链接和普通文件安全执行位可恢复；合成应用包校验链接字节、解引用内容、0755 执行位及特权位剥离，并覆盖 macOS 应用包常用的父相对目标归一化。绝对／越过输出根的父级／悬空／循环／设备／控制字节／链接父目录写入／损坏 CRC 在输出前拒绝；部分选择缺失依赖时同样拒绝。带链接任务遇到占用叶子不得改名、跳过或覆盖。Windows 和 TAR 链接仍明确不支持，不降级为普通文件。原生发布检查父链置换、已有符号链接、外部目标不变；第三方真实归档只保留本地汇总证据，不能替代针对实际变更的边界与回归测试。

预检 footer 回归：100%／125%／150% 字号下与同字号共享纯操作 footer 的高度一致；说明属于正文，长报告或长说明只使正文滚动，返回按钮始终可达。

### WP-12 首批桌面自动化

- M5-01，实际窗口产生呈现帧并记录渲染 API；硬件验证模式拒绝软件与 Null 后端。
- M5-02，中文预编辑不提前提交，中文与非 BMP 文件名提交正确，Escape 退出焦点后文本保留。
- M5-03，真实 Qt 无障碍接口提供输入框角色、名称、值和文本接口，按钮名称与禁用状态准确。
- M5-04，压缩格式两端的选中面具有平滑外侧圆角，内部接缝为直角；Qt 软件／GPU、DPR 1／2 与 HTML 不重复内缩。

历史结果见 [私测准入](https://github.com/xiaodaozhi/xdzip/issues/5)，系统输入法或读屏按需验证；合成事件与接口检查不等同于已执行的真人检查。

### 2026-10-16 用户反馈修订

- W01-08，欢迎页读取期间正文遮罩、活动进度和取消可见，禁止重复正文操作；取消后不延迟打开，错误可恢复，标题栏不变。
- C01-08，深色密码框启用及未输入时 placeholder 清晰；创建结果为“已保存”。创建完成不调用系统文件管理器，即使旧调用传入 openAfter=true；成功结果（含受支持编号分卷）仍在应用内打开。

参数与语义见 [读取及创建完成契约](../ux/welcome-opening.md)。

## Windows 原生回归

- PE 子系统检查要求产品 `xdzip.exe` 为 GUI，诊断与测试程序保留 console；部署后的 windows 平台启动检查仍执行。
- Windows 主窗口没有额外系统标题栏，设置右侧的最小化、最大化／还原与关闭按钮可执行；双击标题空白区域切换最大化，关闭仍等待运行时退出。边角使用原生命中测试，窗口保留可缩放边框、DWM 阴影和受支持系统的圆角；检查 WS_CAPTION／WS_THICKFRAME、非客户区渲染及客户区 1024×622，外部装饰由桌面合成截图单独核对。贴靠、原生拖动和多显示器行为按需验证。
- 权限回归在自有临时目录上加入 Everyone 的只读／执行权限，完成阶段文件、父目录、发布及清理，再分别加入写入、删除、改所有者、改 ACL、泛型写入和仅继承写入，确认输出前拒绝且旧内容不变。外部所有者、链接与特殊对象限制不变。现有用户目录 ACL 不被改写。
- 本地／CI 日志、量测及截图保存在 build/ 下或短期 CI 工件，实际平台结论以对应执行记录为准；不把本轮开发验证记为未执行的平台验收。

## 平台图标留白修订

检查应用和 ZIP 文档两组 PNG、ICO、ICNS，覆盖 16～1024 像素。通用 PNG／ICO 主体高度约占画布 90%，macOS PNG／ICNS 约占 81.2%，小尺寸允许像素取整；四角透明，圆角连续，纸张不变形，阴影不作为主体量测边界。ICO 七种表示与 ICNS 十一种表示均需解码，嵌入图像与对应平台 PNG 来源一致，macOS Qt 运行时图标必须选择 macOS PNG。检查浅深背景的实际尺寸联系表；主窗口与菜单的 24 单位界面图标几何和密度不变。`build-brand-assets.py --check` 自动校验边界、透明度、来源及容器，原生 Finder／Dock／Explorer 缓存显示另外人工核对，不能以容器解码代替该项验收。

## 版本与设置的关于页

产品构建文件名统一为 Linux 的 `xdzip`、Windows 的 `xdzip.exe` 和 macOS 的 `xdzip.app/Contents/MacOS/xdzip`，普通构建与部署包均核对实际入口。测试和诊断程序各自保留名称，不作为用户运行入口。

验证 `VERSION.json`、Cargo workspace、CLI、Windows 版本资源、macOS bundle 与设置的关于页一致。覆盖开发／稳定渠道、非法版本与构建号边界、无 Git 的 unknown、dirty 提示、两种语言、浅深色、100/125/150% 字号和高 DPR；复制 JSON 不含个人文件路径，所有关于入口进入同一设置窗口的关于页。HTML 不伪造原生 Qt 或提交。完整进程资源观测及其限制见 [WP-12](https://github.com/xiaodaozhi/xdzip/issues/5)。

### 设置、关联与网络

- 六个标签完整可达，侧栏与正文相接，没有 footer、保存或取消按钮；变更成功后即时生效并在重开及跨进程后保留，写入失败保留旧值并提示。
- 中英文及 100/125/150% 字号下，侧栏按实际文字尺寸保留完整单词，英文只在词间换行；正文与侧栏内容均可达且不横向溢出。
- 六个页面正文不显示重复的页名及其下方分隔线；检查更新时，原按钮位置显示“取消检查”，鼠标、键盘均能实际取消，其他更新操作保持可用。
- 发现更新、下载、下载完成和恢复时，主要操作在状态右侧，文案及行为符合信任和安装能力；下载中进度与“取消下载”可用，保留可用的打开目录入口，状态变化不自动滚动或改变滑块高度。
- 浅深色设计稿、HTML 与 Qt 的标题下分隔线、侧栏悬停色和图标左边距一致；在两种语言及 100/125/150% 字号下，以鼠标点击检查更新并切换更新状态时，正文滚动位置和滚动条滑块高度均保持不变；键盘导航仍能显示视口外的控件，底部许可内容仍可到达。
- 双向修改语言立即更新界面、原生菜单与托盘菜单，包括应用菜单中的关于／设置／退出、任务子菜单和提示，不中断运行任务，不弹出重启确认；写入失败保持先前语言，系统自带菜单项仍遵循系统语言。清空最近记录立即生效且不删除原文件。
- 各支持类型显示系统实际默认应用，系统拒绝或取消后状态不伪造；验证 Windows 系统确认、macOS 内容类型、Linux MIME，以及共享的复合扩展。
- 验证直连、系统、自定义代理和 HTTPS 镜像的输入校验、持久化、更新流量、失败提示与签名验证；活动更新保持原配置。macOS 系统、直连及自定义 HTTP CONNECT／SOCKS5 均支持原生安装。一次性 hosted runner 的静态系统代理和 PAC 对照须先证明系统模式的 feed／归档真实命中代理，再证明直连零命中，并核对 HTTPS 连接去向、不可信 TLS 拒绝及原设置恢复。核对自定义代理失败无直连回退、真实产品直连升级和活动配置快照，以及签名、任务退出和恢复；单纯下载成功不证明代理生效。
- 创建压缩包的空添加列表没有引导占位文字，保留添加入口、列表边框及摘要。

## 原生菜单与托盘

自定义单选控件须使用 QAccessibleActionInterface 实际执行 press 和 toggle，覆盖主题、创建格式、重复激活和禁用项；检查选择模型、真实主题、代理、格式扩展名及所有选中状态一致，设置变更成功后持久化。只测试鼠标点击或只检查 Accessible.checked 不算该项通过。

复选控件须实际执行 QAccessible press/toggle，核对设置实际值与失败回滚、列表单选与全选三态、重复切换和禁用项；原生树中设置开关须有完整名称。

按[桌面集成契约](../ux/desktop-integration.md)验证 macOS 顶部菜单角色与标准快捷键，文本焦点编辑与文件选择命令分离，最近记录隐私，托盘默认开关与即时生效，关闭后驻留及任务详情恢复，退出取消／等待／清理，文件关联与 Dock 恢复，无托盘及托盘消失回退。自动测试不代替真实系统栏、系统缩放、读屏与 Linux 桌面验证，结果只记录在 build/ 或 CI 工件。

## 只读压缩包检查

- 从真实工具栏“测试”启动整包任务，选择、目录与搜索不缩小范围。覆盖 ZIP／ZIP64、Store／Deflate、受支持分卷和 AES-256、TAR／TAR.GZ，包含重复名称、目录、空文件和 ZIP 链接负载；检查期间不创建文件、暂存目录或链接。
- 破坏本地头、描述符、CRC、压缩流终止或 AES 认证，验证首错终止、失败条目、已验证与未检查文件数；不支持条目不能标成整包成功。TAR 只声明结构与可读性，TAR.GZ 必须消费尾部并验证 GZIP CRC／ISIZE。
- 核对实际读取与验证计数，不出现已保存／提交数量或输出目录动作。扫描期间未知进度不得由计时器伪造；错误密码重试不重置资源预算，旧提示和终态响应被拒绝。
- 覆盖密码等待取消、排队取消、关闭原浏览页后重开任务、来源／分卷变化、跨会话串行准入及退出 join。分别核对双语多字号、浅深色、密码与成功／失败／取消窗口，原生视觉与平台限制另行登记。

## 确认窗口完整性与视觉门槛

- 窗口清单包含全部 QML 顶层与内联窗口，系统选择器例外明确。未登记的新窗口和历史 612 宽退出确认必须被自动检查拒绝。
- 退出、停止、重启确认在中英文、浅深色、100/125/150% 字号与 DPR 1/1.25/2 下均保持 491 宽，20 标题、13 正文。短内容不保留大片空白，长正文与小屏幕可滚动，footer 在客户区内固定可达。
- 三按钮保持一行，英文大字号可在按钮内换行，不裁切、不超出边界、不自定义 footer 样式。
- 验证返回应用、等待、取消退出、稍后、立即重启的原有意图，停止状态按钮禁用；功能结果和视觉检查分别登记，不能互相替代。
- 检查命令：`python3 scripts/check-window-contracts.py --self-test`、`python3 scripts/check-operation-components.py`、Qt `ConfirmationWindows` 测试和 `scripts/check-html-confirmations.py`。截图与量测只写入 build/ 或 dist/。

## Windows 固定安装目录

- 全新安装的主程序、快捷方式、文档图标和打开命令指向根目录，Qt 必需子目录保留，不创建 versions/。
- 使用完整历史产品的旧布局夹具执行升级、回退和再次升级，验证实际提交、设置、用户额外文件及系统默认关联保留。
- 清单只允许审计过的有界相对路径，遍历、设备别名、大小写重复、重解析点及已修改的废弃文件须拒绝；成功后只删除清单内内容未变的废弃文件。
- 运行中拒绝升级和卸载；卸载清理根目录及累计记录的历史程序，同时保留根目录和历史目录中的额外文件。
- Windows 在完整 Qt 编译前用明确标为 QA、从不执行的文件夹具编译并安装真实 Inno 安装器，先检查根目录、快捷方式、实际清单清理与卸载。快捷方式使用 IShellLinkW 读取 Unicode 目标，再按现存文件句柄解析的路径比较；兼容路径别名，避免 WSH 的 ANSI 路径把中文转成问号，更不能用通配符把问号误判为有效目标。
- scripts/test-installers.py 检查生成与路径契约，Windows 原生 scripts/check-windows-install.ps1 检查实际安装/关联/清理结果。其他平台目录布局不随本次调整改变。
- Windows 检查全部受支持扩展的候选、独立图标、命令和真实归档打开，验证旧共享 ProgID、外部候选和 UserChoice 原样保留；历史产品回退后通过新格式入口打开回退版本，卸载只移除自身登记。`.tar.gz` 的最终扩展名及已有用户选择的图标限制单独记录。
- Windows 的九类格式在升级与全新安装后，原生应用枚举均只有一个 xdzip，关联启动器有真实嵌入的品牌图标和产品身份。升级移除自有旧推荐项并保留兼容命令，REG_NONE 标记阻止自动选作默认；核对外部候选、UserChoice／UserChoiceLatest 不变，ZIP 与其他格式的新入口在历史回退后仍可打开。人工另查默认应用设置和 Explorer“打开方式”：只出现一个带品牌应用图标的 xdzip，不能将通用应用方框与文档图标混为一项，也不能以原生 API 枚举代替实际显示。
- Windows 九个逐格式 ProgID 的类型描述在全新安装、已有中英文设置的升级及语言重启后分别为 `XXX 压缩包`／`XXX archive`。验证便携和隔离设置进程不改注册表，安装身份或任一打开命令不匹配时不改任何描述，保留默认应用与其他候选；写入失败的部分状态须如实登记并检查下次启动重试。人工另核对 Explorer 的实际显示及缓存影响。
- macOS 逐类型核对 ICNS 已进入 bundle、Viewer 和 None／Alternate 声明，以及原生候选、真实文件打开和默认应用不变；同 UTI 的压缩 TAR 共用图标。Linux 逐类型验证 MIME 候选、真实 GIO 打开和原默认不变，保持系统文件图标，不新增全局 MIME 图标。
- macOS 在无现成第三方类型声明的隔离环境核对 RAR、CAB、UDF 的文件类型解析与候选；CAB 同时覆盖系统及兼容类型，设置查询使用实际识别的受支持类型。已有默认应用在注册期间保持，原先无处理器的类型允许系统临时解析到新候选，所有测试类型注销后恢复原状态。
- macOS 在保留旧版无效 UDF 类型声明时，核对真实 `.udf` 文件的打开方式、None／Alternate 候选边界及设置状态；另核对既有第三方默认处理器和用户关联偏好保持不变，注销后恢复原有解析。
- 浏览器／离屏结果与 Windows Explorer、Finder 的原生显示和人工操作分别登记，不因关联命令或 CI 通过宣称视觉验收通过。

English: Verify each Windows format handler, document icon, real archive dispatch and historical rollback while preserving protected choices and foreign candidates. Check one branded xdzip application candidate, its embedded launcher icon/identity, owned legacy recommendation cleanup and no automatic default takeover; separately inspect actual Settings and Explorer UI. Verify bundled macOS ICNS, Viewer ranks and native handler candidates; compressed TAR may share one UTI. On an isolated Mac, verify RAR, CAB and UDF type resolution and candidates, both supported CAB identifiers, preservation of existing defaults, and restoration after cleanup. With a legacy invalid UDF declaration present, check actual `.udf` files, None and Alternate candidates, and the Settings state; preserve any third-party default and user preferences. If no handler existed, the system may temporarily resolve the new candidate. Linux keeps system MIME icons. Native shell visuals and human acceptance remain separate from automated checks.

## 会话与预发布验收

- 原生普通、最大化、全屏、最小化及托盘隐藏状态下打开设置、关于、新建、解压、任务、密码、冲突和文件选择器，不发生短暂还原；弹窗绑定调用窗口。
- macOS 全屏进入期间及进入回调内最小化时，验证原生窗口最终实际最小化；期间较新的恢复、普通或最大化请求优先；补发期间恢复保持全屏且不再执行旧请求，窗口销毁时不得重新创建。
- 并发启动、未就绪转发、崩溃残留恢复、通信超时、Unicode 与批量文件请求，确认一个正常进程及托盘，确认后退出转发者。
- 两种打开模式的保存、取消与重启、欢迎页复用、重复激活、目录与选择隔离、关闭后后台继续、同数字任务 ID 的密码和冲突路由、串行重任务与全局异步退出。
- 双语、100%／125%／150% 字号、DPR、胶囊标签与共享操作控件、辅助功能 press/toggle 和旧配置未知字段保留。
- 同标签六类包独立启动与安装生命周期、Mac 签名及重定位、发布清单渠道与语义版本选择、缺包及哈希错误拒绝、公开后重新下载核对。未执行的原生人工视觉验收保持明确限制。

## 分卷浏览与临时打开

| 编号 | 操作 | 必须满足 |
| --- | --- | --- |
| OPEN-01 | 从编号卷、PKZIP 中间卷与最终卷打开 Store／Deflate、ZIP64、AES-256 | 完整列表，跨卷本地头与负载可解压，密码错误可重试，缺卷及损坏不返回部分列表 |
| OPEN-02 | 缺少中间卷、重复卷号、卷为链接／特殊对象，扫描后替换或修改任意卷 | 明确失败，任何源卷不得被输出覆盖，取消及资源上限继续有效 |
| OPEN-03 | 列表与网格双击目录、普通文件与链接 | 目录继续导航，普通文件仅解压对应物理记录到私有临时目录，链接和特殊对象不调用系统打开 |
| OPEN-04 | 临时解压成功，系统接受或拒绝打开 | 接受后自动隐藏对应进度窗，拒绝保留错误及重试；失败、取消、密码等待不自动打开或隐藏 |
| OPEN-05 | 中文、空格、百分号、命令字符、重复文件名、关闭标签与退出 | 本地文件 URL 原样传递，不经过 shell；任务继续受统一生命周期管理，临时副本在异步打开后仍存在 |
| OPEN-06 | 过期、活动锁定、外来标记与链接缓存项 | 仅清理过期且未锁定的本应用目录，保留活动和外来对象；双语、三档字号及 DPR 下按钮和错误正文完整可达 |

## 文件名旧编码自适应

- ENC-01，自有 GBK/GB18030、Big5、Shift-JIS、韩文、俄文、CP437 ZIP，以及传统 TAR/TAR.GZ 名称，核对物理身份、目录树、列表、搜索和目录选择使用相同有效拼写，不依赖界面语言或操作系统代码页。
- ENC-02，UTF-8 标志、Unicode Path 的版本/CRC/重复/损坏、未标记 UTF-8、PAX UTF-8/BINARY 和 GNU 长名称，按编码契约核对优先级与保守失败，不插入替代字符修复声明。
- ENC-03，预检、实际解压、临时打开、外部导出、普通包添加与移动，独立读取输出名称和文件内容；损坏声明、路径异常、Unicode/大小写碰撞、取消、预算及源保护仍有效。
- ENC-04，AES 与编号分卷使用同一名称并正常验证内容；加密和分卷添加/移动继续拒绝。普通包改写后独立验证 UTF-8 标志或 PAX path。
- 原生桌面人工核对不同系统下的文字显示、系统打开和拖放；自动化与统计检测不能替代这些实际结果，也不能宣称任何短名称或混合编码均可唯一识别。结果与边界记录于对应 issue。

## 隔离 7z 与 RAR 后端

以下为能力验收要求，实际结果和未覆盖项记录于对应 issue，不预记通过。构建与权限前提见[隔离后端契约](../engineering/native-codecs.md)。

| ID | 操作 | 必须结果 |
| --- | --- | --- |
| NATIVE-01 | 检查固定源码、校验和、编译清单、运行时格式枚举和部署目录 | 7z／RAR／CAB／ISO／UDF 固定使用 Rust，生成编译清单中没有官方 7-Zip 或 UnRAR 源码；ZIP／TAR／TAR.GZ 使用进程内 Rust，没有第二条容器路由 |
| NATIVE-02 | 7z 创建与独立官方工具双向读取，普通、空文件、Unicode、目录、四档级别、固实、内容及文件名加密 | 元数据和逐文件字节一致，完整校验后才提交，文件名加密仅在设置密码后可用 |
| NATIVE-03 | 打开旧版 RAR、RAR5 和 RAR 7 新算法样本，与独立 UnRAR 输出比较 | 三代分别核验已知输出，密码、头部加密、固实与分卷组合单独记录，不以魔数替代解码证明 |
| NATIVE-04 | 从 7z 编号卷和 RAR 新旧命名的任意卷打开，缺卷、错卷、大小写重复、源卷替换 | 授权完整卷组，RAR 保留边界，所有卷受身份与覆盖保护，异常明确失败 |
| NATIVE-05 | 加密目录无密码、错误输入、正确重试、取消、换页、退出 | 未成功扫描不发布索引，旧打开请求不污染新会话，秘密不进入事件和日志，原生密码窗与退出顺序单独验收 |
| NATIVE-06 | 普通及固实 7z 添加、替换和移动；加密／分卷 7z、RAR 尝试修改 | 普通 7z 完整重建校验后原子替换，能力外包拒绝，取消／损坏／512 MiB 重建缓存超限保留原包字节 |
| NATIVE-07 | 截断、校验损坏、未知算法、危险路径、特殊对象、超大字典、工作进程崩溃与取消 | 不发布失败条目或未验证文件，现有目标保持，真实读取、重试及固实跳过消耗预算 |
| NATIVE-08 | 浏览、临时打开、导出、测试、解压、创建和修改完整产品链路 | 后台任务、错误、密码和来源身份贯通，检查不写出，RAR 无创建和修改入口；实际执行的平台自动化和桌面检查分别登记 |
| NATIVE-09 | 创建超过 512 MiB 的 Store 7z，再重新打开和整包测试；同时创建小编号分卷包 | 整包输出不在原生内存累积，宿主随机回写仅作用于私有暂存，累计写入与长度高水位分开，头部 patch 不多算分卷尾部或产生截断 |
| NATIVE-10 | 固定自有明文的24组独立官方作者样本，经普通 Rust 测试、真实工作进程与宿主服务读取 | Copy、LZMA/LZMA2、Deflate/Deflate64、BZip2、PPMd、过滤器、BCJ2、固实、压缩头及 AES 组合逐文件完整字节一致，头加密经实际打开密码重试；不以这些小样本代替边界、资源和桌面验收 |
| NATIVE-11 | 宿主来源在初始 seek、随机 seek/read、创建输入和 EOF 探测返回 typed 限额、取消、不支持或来源变更错误，包括 Interrupted 包装 | 保留稳定分类，仅无 typed 原因的 Interrupted 可重试，取消立即生效；已准入来源计数不退款，错误不发布文件或替换原归档 |
| NATIVE-12 | RAR 未知尺寸与已知空文件，经真实 worker、运行时分页、大小双向排序、Qt 选择、预检及输出任务 | 未知标记与 0 分开，双语状态及大小汇总不误导，未知项排序靠后，虚拟目录不能因汇总提前返回而获得预检；未知项在解码、密码请求和暂存之前拒绝 |
| NATIVE-13 | 旧密码的 OS 字符投影、现代时间本地日历及辅助工作计量，经实际平台回调和真实 worker | Rust 保留格式决策与预算，薄 C++ 仅提供 OS 原语；秘密缓冲清除，类型化错误不丢失，辅助工作不计作解码产量；各平台实际证据独立登记 |
| NATIVE-14 | 从跨至少三卷文件的各 CAB 卷打开，其中逻辑前卷指向文件起始卷；核对独立已知原文，另试缺卷、错误来源、截断及取消 | 正向卷序和所有文件片段完整核验，非紧邻前卷必须属于实际持续文件的起始卷；异常不发布部分索引或文件，既有紧邻前卷样例继续支持 |


## CLI，AC-04 / AC-05 / AC-06 / AC-07 / AC-11 / AC-12 / AC-14

| 场景 | 操作与判定 |
| --- | --- |
| CLI-INTERFACE | 无 Qt／显示会话执行裸命令、双语帮助、版本、别名、字面路径、JSON、管道与错误参数，核对唯一结果对象、稳定退出码、未知大小及重名物理身份 |
| CLI-ARCHIVE | ZIP、TAR、TAR.GZ／TGZ、7z 创建／添加／选择解压，独立读取输出字节；覆盖密码、文件名加密、分卷、只读拒绝、过滤、源保护、路径歧义与失败前零写入 |
| CLI-CANCEL | 真实终止信号和冲突／密码等待，核对协作取消、终态、join、实际提交统计及暂存清理，不用强杀替代正常取消 |
| CLI-INSTALL | 自有临时位置安装、稳定启动器转发、已有入口拒绝、已改入口保护及登记移除；实际用户 PATH、系统安装授权和真人终端操作分别记录 |
| CLI-UPDATE | 私有签名和原生可执行夹具验证清单分离、篡改、离线／限流／超时错误、摘要和长度、暂存、自检、原子指针、并发互斥及恢复；模拟传输不写成实际 HTTPS 验收 |
| CLI-DESKTOP | 同用户有界 IPC 不变为文件打开，关闭时拒绝；原有桌面更新复验候选和全部任务屏障，CLI 占用不被安装破坏，交接不冒称已安装；无 GUI 身份查询不消费回执；Mac 普通上下文仅位于 Resources，重复构建无旧标记残留，实际原生签名及固定 GUI 上下文解析通过 |

English: CLI acceptance covers real headless interfaces and archive bytes, cancellation/cleanup, owned installation and launch forwarding, private signed/native update transactions and failure recovery, plus bounded desktop IPC and task barriers. Mock transport is not real HTTPS evidence, handoff is not installation, and automated/native/manual outcomes remain distinct.


## 大索引与主动预览

| 条目 | 必要检查与通过条件 |
| --- | --- |
| INDEX-MILLION | 真实百万条 ZIP64，含中文、长名、重复名与嵌套目录；物理数量和展示数量分别核对，读取全部结果批次，记录扫描／排序／检索及峰值内存和磁盘字节，不将单机结果推广为全平台保证 |
| INDEX-PAGES | Qt 保留完整逻辑行数，远距离跳转、搜索和排序可用，驻留最多 1024 行；会话、查询、代际、修订或偏移失配时拒绝旧页，切换查询清空旧选择 |
| INDEX-SELECTION | 跨页选择和全选覆盖完整当前查询，缓存回收后选择保持；解压确认前冻结为稳定身份，查询改变后使用冻结集合；超出任务限制时整体拒绝，不提交部分选择 |
| INDEX-OUTPUT | 从大索引选择实际普通文件，经负载完整性及认证和既有安全授权输出；源变化、重复偏移、超限、取消、链接及不支持的整包操作明确失败，原有文件保持 |
| INDEX-CLEANUP | 正常关闭、替换查询、失败和取消清理已知私有文件；七天回收保留活动租约、未知对象、链接与身份失配对象，不清理用户内容 |
| CONTENT-PREVIEW | 点击共享动作后完整校验真实文本、PNG 和 JPEG，保持纯文本语义；不支持类型、编码、尺寸、像素、输入及算法工作超限拒绝，不显示部分内容，不启动外部应用 |
| PREVIEW-LIFETIME | 密码缺失、错误、正确及认证损坏，取消和关闭密码窗口、更换条目、隐藏页面和关闭会话清空内容；迟到结果不能恢复内容，事件及 Qt mailbox 实际计入内容字节 |
| PREVIEW-VISUAL | 主窗口与独立属性查看器的中英文、浅深色、100%／125%／150% 字号，文本、图片、错误和取消动作可读可达；保留主窗口和菜单比例，实际原生观察与自动功能检查分开记录 |

English: Large-index acceptance covers real million-record ZIP64 data, complete logical results with a 1,024-row Qt cache, full query identities, frozen cross-page selection, actual safe selected-file output and private leased cleanup. Explicit preview acceptance covers authenticated plain text and bounded PNG/JPEG, resource failures, password/cancel/close/source-change lifecycles, real mailbox accounting and bilingual multi-size native visual checks. Logs and measurements remain artifacts rather than repository reports.

## 外部 SDK 只读试验

按 [SDK 试验契约](../engineering/sdk-trial.md)执行 `scripts/sdk-trial.py test`，独立于产品第三方启用与发行。

| 条目 | 必要检查与通过条件 |
| --- | --- |
| SDK-WIRE | Rust 与独立 Python 互通实际元数据、多块内容及宿主摘要；非最短整数、重复键、不定长、无效 UTF-8、过深／过长、未知必需字段与不兼容版本明确拒绝 |
| SDK-BROKER | 来源／操作／sink 身份、序号、越界与溢出、重复 token、不安全路径、旧信用、错误摘要均失败；不向 peer 交付宿主文件路径或输出句柄 |
| SDK-COMMIT | 源变化、目标冲突、缺失或重复终态、崩溃、迟到数据和取消不发布文件；暂存与目录清理，已有目标保持；最终源复核、正常关闭及 EOF 后才提交 |
| SDK-PACKAGE | 未知权限、路径逃逸与摘要失配拒绝；实际执行副本被验证并私有保存，环境与继承句柄受限，不能绕过沙箱或改用 shell |
| SDK-KERNEL | 对自有读／写文件、回环监听器和 self-exec 先作正向对照，再验证真实内核拒绝；绑定实际平台、策略与二进制身份，资源与响应监督有效，未实现平台执行失败且产品第三方保持禁用 |

English: The standalone SDK trial requires independent wire/content interoperability, bounded adversarial decoding and I/O, source/target protection and cleanup across abnormal completion/cancellation, strict verified packages, and real kernel read/write/network/descendant denial with positive controls. Unimplemented platforms must refuse execution. Evidence never implies product plugin enablement or release approval.
