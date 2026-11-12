# 文件管理器操作与独立任务宿主

系统入口将选择和动作交给同一登录会话的桌面进程。Windows 使用经典 `IContextMenu`，位于 Windows 11 的“显示更多选项”。Ubuntu 24.04 Files 使用 Nautilus 4.0 菜单扩展。macOS 使用 Finder 的“服务 → 用 xdzip 打开”，进入独立操作面板；Services 不支持动态子菜单，不使用 Finder Sync 模拟通用文件操作。

当前工作包的实际平台验证与剩余项见 [#270](https://github.com/xiaodaozhi/xdzip/issues/270)。源码能力不追溯加入既有发布附件。

## 选择与输出

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

## 批次、完成与退出

批次逐项记录实际成功、失败、取消、未执行与卷组去重。单项失败继续处理其余项目；取消停止后续项目，并向正在准备或运行的任务发送既有取消意图。已经提交的输出保留，不声称整批回滚。准备、等待密码、冲突、取消和清理仍计入全局活动数及退出屏障。

独立创建窗口尚未取得任务回执时，提交被拒绝应保留输入、参数与错误，允许修改后重试，不生成任务终态或消费批次索引。快捷操作没有配置窗口，提交失败则按实际失败继续批次。

右键创建成功不提交应用内打开意图，解压的“完成后打开文件夹”默认关闭。成功保留独立结果，用户可主动打开目标目录。任务完成不抢焦点；窗口被主动关闭至托盘时，从任务入口可重开批次结果。托盘消失时恢复独立进度或保留的结果，避免丢失唯一操作入口。有可用托盘则关闭进度后继续，无托盘时先显示等待或取消决定。无主窗口、无托盘、无活动任务及可见操作窗口后，等待全部 Rust 连接完成 join 再退出。

## 安装与兼容

安装版默认启用，在通用设置中可关闭。注册只管理自有入口，不更改默认文件关联。便携与开发副本不自动注册。显式桌面入口 `--register-shell-integration`、`--remove-shell-integration` 用于受控注册与移除，均不显示主窗口。

安装器使用精确的单参数同步或移除命令，只更新自有集成元数据，在安装事务启动锁释放前执行；普通文件、右键任务及主窗口启动仍须取得运行时准入。Windows 安装身份同时核对当前用户登记的产品身份、安装位置和文件清单，便携目录即使带有部署清单也不自动注册。

交接前核对桌面程序支持协议 1，防止回退后仍加载的扩展把请求文件交给旧程序。Windows 读取 GUI 可执行文件的版本资源 `XdzipShellProtocol`，关联启动器不声明该能力；macOS helper 每次重新读取目标 Info.plist 的产品身份、协议和单个可执行文件名，不使用 NSBundle 的缓存决定回退状态；Ubuntu 安装包拥有 `/usr/share/xdzip/shell-protocol`，回退到旧包及卸载均移除标记。缺失、未知或不兼容能力拒绝启动。能力标记仅管理交接兼容性，不替代请求和 Rust 内容、目标验证。

Windows 的自有 CLSID 与菜单处理器写入 HKCU。运行目录包含按内容摘要命名的扩展 DLL 与校验清单，避免升级覆盖 Explorer 已加载的同名 DLL。清理旧文件仍按安装器所有权与占用策略执行，不强制结束 Explorer。macOS 安装应用首次正常启动时将自有服务安装到用户 Library/Services，服务 helper 只收集选择并交接；关闭集成只移除核对过的自有文件，额外内容、链接或所有权不符时保留并报错。服务显示和启用状态仍由系统控制。

Ubuntu 的原生扩展随安装包登记到 Nautilus 4.0 扩展目录，读取同一用户设置。运行中的 Files 可能继续持有旧模块，重开 Files 后更新，不自动结束文件管理器。独立运行目录不自动写入系统扩展目录，需显式安装对应集成组件。

English: File-manager actions use the elected desktop process and standalone operation windows. Windows provides a classic context submenu, Ubuntu Files a Nautilus 4.0 submenu, and macOS a Finder Service followed by an operation panel. Smart extraction avoids a redundant wrapper only for a single top-level directory; forced wrappers are reserved exclusively. Batches run sequentially, continue after individual failures and retain committed outputs when cancelled. Shell creation never opens the resulting archive in the browser, and folder opening defaults off. Request validation, task admission, passwords, conflicts, cancellation and asynchronous joins reuse the existing safety contracts. Helpers check the installed protocol capability before launching, including after a rollback. Registration-only commands run inside the installation transaction without admitting normal application launches; a manifest alone never confers installed identity on a portable copy. Installation owns only its registrations and does not change default file associations. Actual platform verification remains recorded in the linked issue.
