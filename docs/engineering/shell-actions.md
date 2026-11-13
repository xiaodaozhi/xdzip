# 文件管理器操作与独立任务宿主

系统入口将选择和动作交给同一登录会话的桌面进程。Windows 使用经典 `IContextMenu`，位于 Windows 11 的“显示更多选项”。Ubuntu 24.04 Files 使用 Nautilus 4.0 菜单扩展。macOS 使用 Finder Sync 的“用 xdzip 打开”动态子菜单，保留“服务 → 用 xdzip 打开”及独立操作面板作为回退。

当前工作包的实际平台验证与剩余项见 [#270](https://github.com/xiaodaozhi/xdzip/issues/270)。源码能力不追溯加入既有发布附件。

Finder Sync 的零付费实现与本机验证见 [#273](https://github.com/xiaodaozhi/xdzip/issues/273)。Apple 将 Finder Sync 定位于同步应用，本项目接受其用于通用压缩菜单的适用性限制，不据此宣称 App Store 审核可接受。本机 ad-hoc 完整性签名不要求付费账号、Developer ID、App Groups 或公证，也不提供 Gatekeeper 发行者信任。依据为 [Finder Sync 扩展文档](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/Finder.html)。

## macOS Finder Sync

`Contents/PlugIns/xdzip-finder-sync.appex` 是不链接 Qt、Rust 或格式库的原生扩展。菜单覆盖全部本地 Finder 目录，观察根为 `/`，仅处理 Finder 提供的选择，不遍历目录，不添加工具栏、徽标或同步任务。单选普通项提供三项创建操作，多选增加逐项 ZIP，全部为归档候选时再增加四项解压操作。空选择、非本地 URL 或超过 128 项时不生成菜单。原生菜单使用系统尺寸，独立操作继续使用共享正常密度窗口。

扩展只读独立 JSON 配置，位于用户 Application Support 下的 `xdzip/finder-sync.json`，包含当前应用位置、开关、语言、登记修订与本机随机交接令牌。文件限 8 KiB、当前用户所有、0600 普通文件，写入使用原子替换，拒绝链接和外来配置。它不是 CFPreferences 文件。沙箱仅增加该精确文件的只读临时例外，不共享整个用户目录、不使用 App Groups 或网络监听。路径 entitlement 必须以 `/` 起始，见 [Apple 临时例外规范](https://developer.apple.com/library/archive/documentation/Miscellaneous/Reference/EntitlementKeyReference/Chapters/AppSandboxTemporaryExceptionEntitlements.html)。

Finder 不保留菜单项的 `representedObject`，因此扩展通过单调递增 tag 保留一个有界选择快照，旧菜单 tag 失效后不能对应新选择。扩展通过显式 `NSWorkspace` 目标向 `Contents/Helpers/xdzip-shell-relay.app` 发送 `xdzip-shell://v1/` 的 base64url JSON 事件，不能依赖沙箱调用者的启动参数。relay 为不链接 Qt 的独立原生应用，先校验当前用户私有配置中的随机令牌，再验证 URL、UUID、动作、路径与 JSON 边界，保持 UUID，调用现有私有请求文件与单实例入口；接纳失败显示原生错误。令牌不进入请求文件、桌面协议或产品日志，外部构造的无令牌或错误令牌 URL 不执行文件操作。URL 仅用于本机交接。

扩展按固定 bundle 布局定位所属应用，不在沙箱中读取外层应用元数据。relay 每次重新读取外层 `Info.plist`，同时要求产品身份、`XdzipShellProtocol=1`、`XdzipFinderSyncProtocol=1` 和合法可执行文件名，回退或不兼容目标拒绝执行。仍加载的旧扩展可能暂时保留菜单，点击必须被 relay 拒绝；服务入口独立核对其原有协议。

安装版首次启动登记自有 appex 与服务，系统设置中的 Finder 扩展仍需用户启用。设置开关同步管理两种入口，语言只切换扩展菜单，系统服务标题遵循系统语言。重复保存设置不重复添加未变化的 appex，避免断开 Finder 连接；内容摘要变化或登记缺失才重新登记。关闭时仅撤销本应用精确路径的登记，保留禁用配置；旧副本卸载不禁用较新应用的配置。显式登记的开发副本关闭后仍可通过设置重新启用，未经登记的开发副本不自动启用。

更新可能受 Finder 持有旧扩展实例影响，必要时在系统设置中关闭再启用自有扩展，不强制结束 Finder。回退和卸载只操作自有登记及文件，不改变默认文件关联。Services 的可见位置由系统管理，可能提升到右键菜单顶层。

English: macOS offers a native Finder Sync submenu plus the existing Service/panel fallback. The Qt-free sandboxed extension retains a bounded selection snapshot by menu tag and sends a local URL event to its explicit native relay. The relay authenticates the private same-user random token, validates the current desktop protocols and preserves the UUID through the existing private manifest/single-instance handoff. Only one dedicated same-user configuration file receives a read-only sandbox exception. Registration is idempotent for unchanged bundles; settings, language, upgrades, rollback and exact-path removal preserve foreign registrations and default handlers. System enablement remains user-controlled. Local ad-hoc signing needs no paid account, App Groups or notarization and confers no Gatekeeper trust; Finder Sync's documented synchronization-oriented scope remains a distribution limitation.

## Windows 应用包边界

实验性 MSIX 构建不登记右键菜单，既不把经典 `IContextMenu` 伪装成现代菜单，也不让侧载包启用现代菜单。Windows 11 现代菜单需要包身份、清单中的 COM 与文件管理器扩展，以及新的 `IExplorerCommand` 实现；现有实现不能直接复用为现代入口。未来若实现，除要求扩展所在包的系统签名类型为 Store，还须有可靠的安装来源判断以排除侧载；查询失败及 Developer、Enterprise、None 均不启用，并分别测试 Explorer 实际显示、设置开关和现有请求协议。Store 签名不单独证明安装途径，商店签名包也可能被复制侧载；无法可靠区分时不能启用现代菜单。Windows 10 没有 Windows 11 的现代菜单。具体实现与真实商店包验证不属于当前评估结论。

English: The experimental MSIX registers no context menu. A future Windows 11 modern menu requires packaged COM registration, IExplorerCommand, a Store signature and a reliable installation-channel check that excludes sideloading. Store signature kind alone does not prove installation through the Store; fail closed if the channel cannot be determined. Actual Explorer and package validation remain separate.

候选来源接口为 [GetStagedPackageOrigin](https://learn.microsoft.com/en-us/windows/win32/api/appmodel/nf-appmodel-getstagedpackageorigin) 的 `PackageOrigin_Store`。它报告暂存包来源，而不是本文已验证的逐用户安装记录；同包多用户、Store 签名包复制侧载和升级等情形必须实测后才能决定是否足以满足侧载禁用要求。Explorer 内的扩展还须识别自己的包，不能把 Explorer 进程当作 xdzip 包查询。查询失败或结果不确定时隐藏入口。

English: GetStagedPackageOrigin and PackageOrigin_Store are candidates for further investigation, not yet proof of a per-user installation channel. Test copied Store-signed packages, multiple users and upgrades, and identify the extension's own package rather than Explorer's process.

需要采集来源对照时，在当前用户已注册目标包的 Windows 环境运行以下只读诊断；包名称仅为示例，输出必须是 `build/` 下尚不存在的 JSON 文件。诊断同时记录系统签名类型、暂存来源和匿名化包完整名称摘要，拒绝没有唯一登记版本的结果。场景参数是执行者标签，不证明安装途径；系统查询失败、未知值或相同返回值均不能单独批准启用菜单。无需修改证书、安装或系统设置。

```powershell
python scripts/msix-origin-probe.py --name xdzip.Example --scenario store-install --output build/store-origin-observation.json
```

English: The read-only probe records the current user's unique package registration, signature kind, staged origin and a hashed full name in a new JSON file under `build/`. The scenario is only a caller-supplied label. No value or API failure establishes the per-user installation channel or enables the menu.

## 选择与输出

右键创建确认直接检查预填的源目录或指定目标，不先检查默认下载目录，避免不相关的 macOS 隐私访问等待阻塞操作。正常主窗口创建仍使用默认目标。

English: Shell creation checks its prefilled target directly, without touching the default Downloads directory first. Normal main-window creation retains its default target.

压缩提供快捷 ZIP、7z、创建确认及多选逐项 ZIP。单个普通文件保留完整名称，例如 `a.txt.zip`；文件夹使用 `foo.zip`；同目录多选合包使用所在目录名。跨目录合包进入独立创建窗口确认目标。快捷创建采用正常级别，不加密、不分卷，目标重名使用现有询问机制。

全部所选项是受支持后缀或分卷命名的归档候选时，显示解压操作。菜单不扫描文件内容，真正打开时仍由 Rust 发现完整卷组、识别内容、校验来源和格式能力。相同完整来源组在一批解压中只执行一次；来源组标记是持有文件身份的进程内不透明标记，不是写入授权。

- 当前目录：各归档内容直接输出到各自所在目录。
- 智能目录：完整已验证输出图仅含一个顶层目录时不增加外层目录，隐式顶层目录也参与判断；根目录文件或多个顶层项需要归档同名外层目录。不剥除或重命名包内目录。
- 逐包目录：始终增加归档同名外层目录。新增目录通过持有的目标能力排他创建，已有文件、目录或链接均不复用，按 `name (2)` 等编号选择新名称，最多尝试一万次。
- 指定目录：独立解压窗口选择共同目标，默认逐包目录，可改为智能或直接合并。

目录派生继续遵循[安全解压](m3-extraction.md)的 portable 名称与 `.app` 后缀规则。智能判断不能授权输出；每个任务仍需重新验证目标、来源、CRC／认证、冲突和提交权限。某个包内部的失败仍按既有整包停止策略处理，批次协调器随后处理下一个包。

## 进程、请求与窗口

私有入口为桌面程序 `--shell-request <request.json>`，不属于独立 CLI 的脚本接口。原生扩展只生成选择清单并启动桌面入口，不加载 Qt 或格式库。清单位于系统临时根下排他创建的 `xdzip-shell-<32位随机十六进制>` 私有目录，固定叶名称为 `request.json`。Windows 在写入前设置当前用户与 SYSTEM 的受保护 DACL，POSIX 使用 0700 目录与 0600 文件。桌面使用共享 Rust no-follow 与目录能力核对根、所有权和权限，限读 256 KiB，拒绝链接、相对路径、无效 Unicode、未知动作及超过 128 个选择。

请求包含 schema 1、UUID、动作和绝对路径。可选布尔值 `archive_candidates` 仅传递文件管理器已有的 MIME 或 UTI 类型提示，与共享后缀、分卷规则共同决定菜单可见性，不授权读写或绕过 Rust 内容验证。Windows 读取扩展名已登记的 Content Type，Nautilus 读取 FileInfo MIME，macOS helper 读取 URL Content Type，不在文件管理器中解析归档。单实例协议保留原打开与 CLI 操作入口，新增结构化 `shell_action`；接纳与执行成功分别判断。应用层保留至多 128 个已接纳请求身份，相同身份与内容只接纳一次，同身份不同内容拒绝。清单接纳后只删除固定文件和空目录，不递归清理外部输入。

冷启动操作只创建 `ShellHost`，不创建隐藏主窗口。正常打开文件与显式“显示主窗口”按需创建浏览窗口。已有实例接收右键操作不切换浏览标签、不恢复隐藏主窗口。操作会话占一个既有有界会话槽，批次内部串行复用控制器，不按归档数量创建浏览页面。

`OperationTasks` 为浏览入口和独立入口共享进度、结果、密码及冲突窗口。顶层 `OperationWindow` 允许没有 owner；有 owner 时跟随调用窗口，无 owner 时约束在所选屏幕。模态提示绑定实际操作窗口，退出确认复用 `OperationConfirmationWindow`。布局、共享控件、双语和辅助功能仍遵循[视觉流程](../visual/design-workflow.md)。

macOS 单实例身份使用当前用户与 Security 框架的实际登录会话 ID，查询失败拒绝选举，不依赖 helper 可能缺失的环境变量。普通启动、Finder relay 和服务入口因此进入同一登录会话的实例，见 [SessionGetInfo](https://developer.apple.com/documentation/security/sessions)。

English: On macOS, single-instance election uses the current user and the actual Security session ID, failing closed on query errors. LaunchServices helpers and the desktop must not derive different identities from missing environment variables.

## 批次、完成与退出

批次逐项记录实际成功、失败、取消、未执行与卷组去重。单项失败继续处理其余项目；取消停止后续项目，并向正在准备或运行的任务发送既有取消意图。已经提交的输出保留，不声称整批回滚。准备、等待密码、冲突、取消和清理仍计入全局活动数及退出屏障。

独立创建窗口尚未取得任务回执时，提交被拒绝应保留输入、参数与错误，允许修改后重试，不生成任务终态或消费批次索引。快捷操作没有配置窗口，提交失败则按实际失败继续批次。

右键创建成功不提交应用内打开意图，解压的“完成后打开文件夹”默认关闭。成功保留独立结果，用户可主动打开目标目录。任务完成不抢焦点；窗口被主动关闭至托盘时，从任务入口可重开批次结果。托盘消失时恢复独立进度或保留的结果，避免丢失唯一操作入口。有可用托盘则关闭进度后继续，无托盘时先显示等待或取消决定。无主窗口、无托盘、无活动任务及可见操作窗口后，等待全部 Rust 连接完成 join 再退出。

## 安装与兼容

安装版默认启用，在通用设置中可关闭。注册只管理自有入口，不更改默认文件关联。便携与开发副本不自动注册。显式桌面入口 `--register-shell-integration`、`--remove-shell-integration` 用于受控注册与移除，均不显示主窗口。

安装器使用精确的单参数同步或移除命令，只更新自有集成元数据，在安装事务启动锁释放前执行；普通文件、右键任务及主窗口启动仍须取得运行时准入。Windows 安装身份同时核对当前用户登记的产品身份、安装位置和文件清单，便携目录即使带有部署清单也不自动注册。

交接前核对桌面程序支持协议 1，防止回退后仍加载的扩展把请求文件交给旧程序。Windows 读取 GUI 可执行文件的版本资源 `XdzipShellProtocol`，并核对自有 `Applications/xdzip.exe` 登记命令指向同一个实际文件；关联启动器不声明协议能力。回退到旧的分版本布局后，即使新版根目录程序被保留，也不再作为右键目标，不读取或改变用户默认文件关联。macOS helper 每次重新读取目标 Info.plist 的产品身份、协议和单个可执行文件名，不使用 NSBundle 的缓存决定回退状态；Ubuntu 安装包拥有 `/usr/share/xdzip/shell-protocol`，回退到旧包及卸载均移除标记。缺失、未知、不兼容或已经不属于当前登记入口的目标拒绝启动。能力标记仅管理交接兼容性，不替代请求和 Rust 内容、目标验证。

Windows 的自有 CLSID 与菜单处理器写入 HKCU。运行目录包含按内容摘要命名的扩展 DLL 与校验清单，避免升级覆盖 Explorer 已加载的同名 DLL。清理旧文件仍按安装器所有权与占用策略执行，不强制结束 Explorer。macOS 安装应用首次正常启动时将自有服务安装到用户 Library/Services，服务 helper 只收集选择并交接；关闭集成只移除核对过的自有文件，额外内容、链接或所有权不符时保留并报错。服务显示和启用状态仍由系统控制。

Ubuntu 的原生扩展随安装包登记到 Nautilus 4.0 扩展目录，读取同一用户设置。运行中的 Files 可能继续持有旧模块，重开 Files 后更新，不自动结束文件管理器。独立运行目录不自动写入系统扩展目录，需显式安装对应集成组件。

English: File-manager actions use the elected desktop process and standalone operation windows. Windows provides a classic context submenu, Ubuntu Files a Nautilus 4.0 submenu, and macOS a Finder Sync submenu with a Service/panel fallback. Smart extraction avoids a redundant wrapper only for a single top-level directory; forced wrappers are reserved exclusively. Batches run sequentially, continue after individual failures and retain committed outputs when cancelled. Shell creation never opens the resulting archive in the browser, and folder opening defaults off. Request validation, task admission, passwords, conflicts, cancellation and asynchronous joins reuse the existing safety contracts. Helpers check the installed protocol capability before launching, including after a rollback. Windows also verifies that its registered application command identifies the target file, rejecting a retained newer executable after rollback to the legacy layout. Registration-only commands run inside the installation transaction without admitting normal application launches; a manifest alone never confers installed identity on a portable copy. Installation owns only its registrations and does not change default file associations. Actual platform verification remains recorded in the linked issue.
