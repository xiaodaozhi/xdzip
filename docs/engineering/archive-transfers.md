<!-- docs/engineering/archive-transfers.md -->

# 文件列表打开、双向拖放与归档修改

右键“打开”作用于右击条目，保留多选，复用双击和 Enter 的入口。目录进入，普通文件使用[临时打开](archive-opening.md)的私有缓存、密码和关联程序规则。会话、查询变化关闭菜单，执行时仍核对稳定条目身份。

## 外部拖出

列表和网格达到系统拖动阈值后冻结选择与包内目录。拖动未选项先单选，拖动已选项保留选择。目录展开后代，父子重复选择去重，保留当前目录以下的相对层级及空目录，不删除归档内容。链接不经拖出入口发布。清单最多十万项，名称及结构预算为 4 MiB，仍受安全解压的字节、路径、密码及配额约束。

Windows 通过 OLE `IDataObject` 的 FILEDESCRIPTOR/FILECONTENTS 和异步能力提供内容。目标接受后启动后台准备，私有副本完整校验后才交给消费者读取；同步消费者不能使 GUI 等待任务，未支持的目标明确失败。Explorer 决定最终位置和重名。FILEDESCRIPTORW 的路径须短于 260 个 UTF-16 单元。内容流只读，取消后停止提供后续数据。异步消费者结束前，退出仍等待本机传输结束。

macOS 通过 NSFilePromiseProvider 向 Finder 提供顶层文件或目录承诺。私有副本准备完成后，Finder 提供的本地文件 URL 经 Rust 的独立 PublishExport 任务授权，逐文件暂存并校验来源后发布，不覆盖既有普通文件。POSIX 普通文件沿用安全解压的 0600 | (mode & 0155) 权限策略，保留可执行位并剥离特权和外部写位。目录合并及部分完成遵循安全解压的输出边界，不承诺传输整体回滚。文件承诺回调在后台等待，文件写入始终进入有界任务队列与退出屏障。

Linux 的已支持基线为 Ubuntu 24.04 x86_64、X11。先准备私有副本，准备完成且鼠标仍按下时开始文件 URL 拖放；提前释放时提示再次拖动并复用副本。文件管理器决定最终复制与重名，应用不推断最终目标和保存成功。Wayland 不在本次支持承诺中。

导出缓存与临时打开缓存分别存放。使用随机目录、身份标记及活动锁，准备失败不发布 URL。保持七天保留与有界后续清理，不在拖动返回时删除消费者可能仍需读取的文件。页面关闭保留任务所属控制器，明确退出取消准备并等待 Rust 工作线程与原生消费者结束。

## 外部拖入和确认

已打开归档的文件列表正文接收真实本地文件与目录，仅接受复制。拖入其他压缩包文件时按普通输入添加，欢迎页仍执行拖入打开。远程 URL、虚拟附件、链接和特殊对象拒绝接纳。拖入时冻结会话、当前目录及输入，搜索页要求先进入目录。

UpdateWindow 显示输入清单、移除动作、目标归档、包内目录及询问、跳过、改名、替换策略。默认询问。同名目录合并并沿用原有拼写，文件/目录冲突拒绝。替换作用于包内普通文件，不授予覆盖主机输入文件的权限。确认前不修改归档。视觉使用创建窗口的 491 逻辑宽度和共享控件，见[独立稿](../../design/hifi/sources/archive-add.html)。

## 包内拖放移动

文件列表和网格达到系统拖动阈值后，先以包内移动处理，冻结当前会话、查询代次、选择版本、包内源目录与稳定条目 ID。拖动未选项先单选，拖动已选项保留多选。源项保持原位绘制，目标文件夹在列表、网格及目录树使用既有选择色高亮。操作说明复用提示的字号、最大宽度和内距，最多四行，边界受当前客户区约束；这是拖放中的即时状态，不沿用普通悬停提示计时。

同目录的文件夹及左侧目录树中的任意已索引目录可接收，包括从子目录移到根目录。只接受本会话的私有拖动身份，不把外部 URL 或其他会话的条目解释为移动。目标不能是选中的文件夹、选中文件夹的任一后代或源目录。搜索页、过期选择和当前不可修改的归档同样拒绝。拒绝目标保留即时说明和禁止投放状态，松手不启动任务；加密、分卷及两者组合各有明确说明。外部添加的拒绝说明也在悬停时展示，实际 drop 仍拒绝。

有效目标松手后直接进入独立 move 任务，复用既有进度、重名与结果窗口。普通文件重名默认询问，支持跳过、使用新名称及替换，并可应用到后续冲突。文件夹合并，文件与文件夹的类型冲突失败。跳过的源文件仍在原位置，保留所需的显式源目录；完全移动的源条目消失，空目录与隐式目录均可移动。移空当前隐式源目录时补充空目录记录，保持浏览位置。成功或失败后重新打开原归档，保留源目录及排序，清除旧物理 ID 的选择。编辑接纳、执行和重开期间保留列表快照并冻结目录查询，不允许旧身份再发起浏览。

MoveOptions 只传递有界的源目录、目标目录与选择身份，不接纳主机路径。核心再次解析直属源条目并拒绝循环、原位置及过期身份。移动与添加共用命名空间、重名决策、完整内容校验及同卷原子替换。ZIP 原样复制压缩载荷并改写名称；TAR 保留普通记录头和除 path 外的受支持 PAX 属性，路径以 UTF-8 PAX 改写，GNU 长名称转为 PAX，TAR.GZ 重建外层 GZIP。无法安全保留的扩展明确失败，提交前失败和取消保持原包字节。

鼠标离开源窗口客户区或源窗口失活后，冻结选择交给原有平台外部复制流程，包内拖动结束。晋升为系统拖出后，不将返回应用的导出数据当作包内移动或新增输入。该边界防止私有暂存内容被再次添加，平台外部拖出规则保持不变。

桥接新增 Move 命令，沿用有界重任务队列、任务退出屏障和稳定会话校验。Page 的 edit_restriction 位 0 表示加密，位 1 表示分卷，位 2 表示其他不支持记录；update_allowed 仅在全部位为零时成立。此原因来自整个只读源快照，不从当前页面推断。拒绝说明仅表达能力，不授予写入权限。

## 普通归档修改事务

Update 支持普通单文件 ZIP、TAR、TAR.GZ，不改写加密、分卷、链接、特殊记录或无法安全保留的名称。能力由整个源快照决定，不能仅检查当前页面。任务接纳后释放元数据会话，旧条目 ID 不再接受新操作；任务保留源句柄与身份。

源版本和 no-follow InputTree 冻结后，在原归档同卷的私有工作区生成完整新归档。ZIP 复制原压缩载荷及库支持的元数据，新增项使用 Deflate 6；TAR 保留原始记录组，TAR.GZ 重新编码外层 GZIP 流。原顺序保留，新项追加。重复或歧义记录、ZIP 自解压前缀、无法改写的扩展明确失败，不能静默丢弃。TAR 全局 PAX 记录暂不改写。

包内命名空间包含隐式父目录，上限十万节点，名称及规范化索引共用 16 MiB 预算，避免通过深层路径生成无界索引。新归档重新扫描并逐文件解码到有界校验接收端，再复核输入、原归档及目标身份。只有绑定原文件版本的 ArchiveUpdate 能力可以整体替换它，普通创建和解压的源保护不变。原归档作为添加输入明确拒绝。文件系统缺少所需原子原语时失败，不先删除、截断或以复制/删除模拟替换。版本复核是尽力检测，不承诺同账户恶意并发写入的内容 CAS 或断电持久化。

提交前取消或失败保留原归档字节。提交许可获得后成功发布胜出，清理失败单独记录。成功或失败后重新打开同一源，清除失效的物理条目选择。任务使用独立 update、move、export 和 export-copy 类型，不把私有准备完成当作系统复制成功。

## 验收边界

核心覆盖三格式、当前目录、原内容和空目录、重名策略、来源变化、受保护输入及取消；Qt 覆盖真实右键打开与添加确认，中英文、浅深色、多字号和冻结会话。Windows 原生协议测试验证异步数据对象和实际内容流。移动回归覆盖列表/网格、目录树、拒绝目标、混选、目录合并、元数据保留及边界交接，三格式由独立 Python 读取器复核。真实 Explorer、Finder、Ubuntu Files 及桌面投放须分别登记，离屏与协议接收器不能代替桌面人工验收。阶段证据记录于对应 issue 和本地/CI 工件。

English: Context Open acts on the clicked item through the same stable-identity entry point as double-click and Enter. External copies use Windows asynchronous virtual files, macOS file promises backed by Rust publication tasks, and prepared file URLs on the supported Linux X11 baseline. Incoming local files require confirmation and target the current archive directory. Ordinary single-file ZIP, TAR and TAR.GZ edits use a frozen source, private same-volume rewrite, full content verification and version-bound atomic replacement. Internal moves freeze source identities and accept sibling folders or the current session's directory tree. Selected folders, descendants and the original location are refused with visible reasons. File conflicts reuse Ask/Skip/Rename/Replace; skipped inputs stay at their source. ZIP payloads and supported TAR metadata are retained through the same verified rewrite transaction. Encrypted and split edits are unavailable, with explicit hover feedback. Failed or cancelled pre-commit edits preserve the original archive. Native desktop acceptance is recorded separately from automated tests.
