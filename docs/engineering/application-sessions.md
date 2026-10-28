# 应用会话与退出契约

标签右键菜单提供“关闭、关闭其他、关闭右侧”，Shift+F10 或菜单键作用于焦点标签。右击不切换当前页，命令冻结标签稳定身份，仅按该标签所属窗口的当前视觉顺序操作，不能影响其他窗口。重排、窗口尺寸变化、目标消失或进入模态操作时关闭菜单；没有对应目标时禁用批量项。批量关闭一次提交，保留未被关闭的当前页，否则切换到锚点标签，避免逐页闪切。最后一页关闭后恢复起始页，任务和结果继续留在全局任务入口。

一个用户登录会话仅运行一个正常产品进程，共用一个托盘和应用菜单。版本查询在创建 GUI 前返回；烟测、桌面探针和测试使用独立路径，不连接真实用户实例。

`SingleInstance` 在创建主窗口前用 QLockFile 选举，持锁者才可移除崩溃残留的本地服务器。QLocalServer 使用 UserAccessOption，结构化 JSON 协议版本为 1，限制单条 256 KiB、128 个绝对路径和 32 个连接。后启动者等待主实例就绪，收到请求 ID 对应的确认后退出，15 秒内通信失败明确报错，不创建第二个正常 UI。Windows 根据服务器握手中的 PID 转交本次用户启动的前台激活权限，不设置永久置顶。IPC 不经过命令解释器，不记录用户路径。

`ApplicationSessions` 是 Qt 应用层协调器，不解释格式、授权目录或文件提交。每个会话拥有独立 SessionPage、BrowserController 和 Rust RuntimeConnection，保留查询、选择、目录与浏览状态。窗口只承载页面，标签页不会为每个归档创建隐藏的原生主窗口。文件关联、命令行、原生文件打开事件、打开按钮、最近记录、拖放和创建结果统一提交打开意图。优先复用无归档且无任务的欢迎页，重复路径激活已有会话，多文件请求按顺序接收并激活最后一个。路径规范化只用于 UI 去重，不授予写入权限。

`desktop/openMode=tabs|windows` 默认 tabs，schema 保持 1，未知字段保留。设置草稿保存后影响后续打开，取消不生效，不迁移已打开页面。标题栏加号按此设置新建页面或窗口。

标签排序通过稳定会话身份和窗口内目标索引提交，只改变同一窗口未关闭会话的展示顺序，不重排用于任务准入的控制器集合。拖动释放或键盘排序后保持当前活动页、归档、控制器、选择和任务身份，其他窗口与已关闭后台会话的相对顺序不变。关闭非活动标签不切换页面，关闭活动标签优先选择视觉顺序中的右邻，否则选择左邻。无效身份、已关闭会话、越界或退出期间的排序请求忽略；不创建或销毁 Rust 连接。当前仅支持窗口内重排，不支持跨窗口拖放、分离窗口或跨启动恢复顺序。标签栏溢出仅改变可视区域，不能丢弃会话；菜单与键盘激活后保证活动标签可见。

最多保留 32 个存活会话，达到限额明确提示。所有槽均由后台任务保留时，关闭最后浏览页复用原页显示欢迎与全局任务入口，不创建额外运行时。关闭标签或非最后主窗口不取消任务。仍有任务或完成结果的页面保留所属控制器，统一任务菜单使用“会话 UUID/后端任务 ID”，不能把另一个连接中相同数字 ID 当作同一任务。密码、冲突和结果窗口仍由原会话控制器处理。无任务的关闭页面先请求异步关闭，接收线程完成 join 后才从会话集合移除。

会话关闭后，延迟到达的开始、预检、密码与冲突事件不自动弹窗或抢焦点。任务与请求令牌继续保留，用户从全局任务入口主动打开时再显示原会话的进度和待处理请求，关闭页面不等于取消请求。

全局任务入口只按复合身份显示原控制器的任务窗口，先关闭调用菜单，不切换当前归档标签，不恢复已关闭的归档页面或主窗口。任务的密码、冲突和取消仍路由到原控制器；无效或已失效的任务身份忽略。关闭浏览页与显式查看任务是独立的用户意图。

任务菜单的内容变化后，能在视口内完整容纳的短列表从首项显示，不能沿用旧滚动位置而把任务行裁出视口；超出高度限制的列表仍可滚动，清除区保持固定。

清除任务历史是应用层展示意图。控制器保留后端快照与迟到回执，仅在有界历史中标记已终结项目不再展示，同一终态的重复更新不能使其重新出现。活动计数、准备占位、密码、冲突、取消及join不使用隐藏标记；没有完成历史时不发送变化。历史淘汰同时删除隐藏标记，已关闭且无可见任务的控制器按既有异步退出流程回收。跨会话清理按各控制器身份操作，不能用相同数字ID清除另一个会话的活动任务。

English: Clearing history changes only its bounded UI projection. Authoritative snapshots and late acknowledgements remain intact; repeated terminal updates stay hidden. Live counts, preparation placeholders, password/conflict tokens, cancellation and joins ignore visibility markers. Eviction removes the corresponding marker, and closed sessions with no visible tasks retire through the existing asynchronous path. Clearing is scoped per controller, never by a numeric task ID shared across sessions.

English: Global task entries close their invoking menu before showing the original controller's task dialog. They never select an archive tab, restore a closed page or reopen its main window. Password, conflict and cancellation actions remain bound to the task's original controller. Invalid or expired composite identities are ignored.

After task menu contents change, a short list that fits in its viewport starts at the first item; a stale scroll offset must not clip task rows. Longer lists remain scrollable above the fixed clear area.

创建、解压和完整性检查在应用级串行准入，包括准备、密码或冲突等待、提交与清理阶段。等待准入的控制器仅保留一个有界命令，可取消，不占用 GUI 线程。单个 Rust 连接的任务顺序和安全语义继续有效。明确退出时，先对所有会话作统一确认，选择等待则全部任务结束后退出，选择取消则等待全部连接完成取消、清理与 join；只检查关闭完成状态，不在状态检查中再次触发关闭。

重任务快照携带创建任务的原始请求 ID，即使先于 `TaskAccepted` 或晚于该请求的 `Finished` 到达也保持不变。控制器仅在请求、原始会话与任务身份匹配时，将准备占位绑定到实际任务；其他请求或会话的快照不能抵消占位。实际任务与占位只计一次，已终结任务不再计为活动任务。绑定不代替真实接纳回执，不提前触发任务窗口，不绕过串行准入、取消和全部连接 join 屏障。

Heavy-task snapshots retain their originating request ID before `TaskAccepted` and after the request's `Finished`. A controller binds its preparation placeholder only to a matching request, source session and task, counting it once. Unrelated snapshots cannot consume the placeholder. A terminal task is no longer active. Correlation does not acknowledge the request, present a task window early, or bypass admission, cancellation or the join barrier.

等待全部任务退出只由应用级任务总数与 join 屏障收敛。某一页面先完成任务时，不执行旧单页自动关闭，也不因该页结束而隐藏最后窗口或再次显示退出确认。

退出确认显示前即设置应用级 `quitDecisionPending`。所有会话与窗口的排队密码提示及显式重新显示在此期间暂停，不能覆盖退出确认；返回或选择等待解除暂停后，只重新显示仍有效的同一任务与提示令牌。进入取消、清理与 join 后保持暂停，不重新打开密码窗口。该状态不隐藏或取消已有请求，也不改变关闭会话后的显式任务入口。

系统 Quit、应用菜单与托盘的显式退出经过同一桌面路由。受管理会话自己的密码与冲突提示允许被全会话退出确认覆盖；选择返回保留原任务和请求令牌，取消并退出仍等待所有连接清理与 join。原生文件选择器、其他模态控件及不属于受管理窗口的提示继续阻挡退出请求并保持前台，不能用通用模态检查误吞密码等待期间的退出，也不能直接绕过确认取消任务。

The application sets `quitDecisionPending` before presenting its quit confirmation. Queued and explicitly reopened password prompts in every session and window defer to this decision. Returning or choosing to wait resumes only a still-valid task and prompt token. Cancellation, cleanup and joins keep presentation paused. This state does not hide or cancel existing requests or change explicit task access after closing a session.

System Quit, application-menu and tray quit share the desktop route. An owned password or conflict prompt in a managed session may be superseded by the global quit confirmation. Returning preserves the task and prompt token; cancelling and quitting still waits for every cleanup and Rust join. Native file selectors, other modal controls and prompts outside managed windows retain their blocking behavior.

主窗口可见时，“显示并激活”只提升和请求激活。隐藏或最小化时恢复先前普通、最大化或全屏状态，不先还原再最大化。阻塞的模态窗口优先激活，弹窗 transientParent 绑定实际调用窗口。关闭最后一个主窗口继续遵循托盘设置。设置与关于从统一入口进入，已有窗口优先激活，不创建另一份可见草稿。

`showMinimized()` 会替换原状态位。恢复时先在仍最小化的状态下补齐保存的最大化或全屏标志，再仅移除最小化标志，不先显示普通窗口。原生回归等待窗口实际暴露、激活及最终状态确认，避免把 Qt 请求值当作窗口管理器已完成的状态。

密码与退出顺序回归使用真实 AES 等待，覆盖任务通知已建立请求、延迟窗口尚未显示时的同步 Quit，同页、其他会话、其他窗口及选择等待，另以窗口已显示后的真实 posted Quit 作投递对照。必须记录实际通知与投递状态，确认排队密码窗口不覆盖退出决定，返回或等待保留同一令牌，取消后完成全部 join；受控同步通知顺序不能替代某个平台历史事件顺序的证据。

Password/quit ordering regressions use real AES waiting and synchronous Quit after the task prompt exists but before deferred window presentation. Coverage includes the same page, another session, another window, and choosing to wait, plus a genuinely posted Quit after presentation as a delivery control. Record actual notification and delivery states, preserve the prompt token on return or wait, and require every join after cancellation. A controlled synchronous notification does not establish an earlier platform event order.

验收包括实际原生窗口状态变化、并发启动与崩溃残留、Unicode 和批量路由、两种打开模式、保存与取消及重启、跨会话 ID 冲突、后台任务、密码与冲突、全局退出和接收线程回收。离屏测试只能证明状态契约，不能代替原生视觉、辅助功能和桌面人工验收。证据保存在 build/ 或 CI 工件。

English: One normal product process and tray serve each user login session. Each archive owns an independent controller and Rust connection. The saved open mode selects tabs or windows for future opens; duplicate archives activate their existing session. Tab reordering changes only the visual order within one window, preserving the active page, controller identity and task admission order. Closing an inactive tab preserves selection; closing the active tab selects its right neighbor, or its left neighbor when last. Overflow scrolling and the tab list preserve all sessions and reveal the selected tab. Cross-window dragging and order persistence are not supported. Closing a page keeps its tasks and results accessible through composite session/task identities. Heavy operations are admitted serially across the application. Explicit quit waits for cancellation, cleanup and asynchronous joins across every session. Activation preserves normal, maximized and fullscreen state and prioritizes blocking dialogs. Diagnostic runs remain isolated. Automated evidence does not claim manual desktop acceptance.

退出进入实际取消与 join 阶段后，IPC 明确拒绝新意图。关闭通道后才释放选举锁，全部连接完成后才启动重启进程，旧实例的重复清理不能移除新端点。Windows 重启同样转交本次用户操作的前台权限。

阻塞弹窗期间接收的外部打开保留最后一个激活意图，先激活阻塞窗口，待其关闭后显示对应新窗口，不使隐藏的新窗口失去入口。设置中的关于窗口保留实际设置调用者，不因另一主窗口获得焦点而改变所属关系。

原生回归入口为 `python3 scripts/check-native-sessions.py --tests <browser-tests> --output build/review/native-sessions.json`。Linux 使用 Xvfb 和 Openbox 的真实 X11 窗口管理路径，Windows、macOS 使用系统窗口管理器；脚本只管理自己启动的验证进程。普通 CI 与 Release 共用此入口，原生日志和 JSON 保存在 build/ 或 CI 工件，不能据此宣称完成人工视觉、GPU、输入法或屏幕阅读验收。

关闭主窗口与最小化不同。关闭有托盘的最后窗口会清空该窗口所有浏览标签，关闭包括设置、关于在内的全部所属弹窗并撤销未保存的设置预览，销毁其原生窗口资源后再隐藏主窗口；从托盘、Dock 或无文件参数的第二次启动唤出后仅显示一个起始页，不恢复旧标签、草稿或附属弹窗。非最后窗口继续关闭并清理自己的浏览标签与附属弹窗，其他窗口不受影响。后台任务及提示令牌仍由原控制器保留，用户明确打开任务时才重新创建对应原生窗口。存在阻塞模态窗口时先激活它，不接受主窗口关闭。

English: Closing a main window dismisses every owned dialog, including Settings and About, cancels unsaved appearance previews and releases native dialog surfaces. Tray restoration shows only the welcome page. Background task identities and prompt tokens remain owned by their original controllers, and explicit task activation can recreate their windows.

关闭窗口不取消真实后台任务，任务所属会话继续按组合身份保留。正常情况下使用新的空白会话承载起始页。达到 32 个存活会话时暂用已关闭页面的无归档外壳，隐藏旧标题、加载及错误反馈；异步 join 释放槽位后再创建空白会话，不越过上限。该替换不唤出隐藏窗口，不抢走另一窗口焦点。全局任务入口仅显示任务窗口，文件打开请求可以重新显示保留的归档页面。

English: Closing a tray-resident window clears its browsing tabs and operation drafts; a plain restore shows one welcome tab. Minimizing preserves the window. Background tasks retain their original session/controller identity and remain reachable through the global task entry. A fresh blank session replaces the closed-page shell when capacity permits, without exceeding 32 live sessions, revealing a hidden window or stealing focus. Global task requests show only their task dialog; file requests may reopen the retained archive page. Other open windows are unaffected, and blocking dialogs prevent closing their owner.

接受主窗口关闭时递增激活世代并断开模态等待连接，已经排队的旧唤醒回调也必须检查世代，不能在模态弹窗刚关闭后重新显示主窗口或恢复旧标签。此失效处理与正常打开入口共用，不取消后台任务。

更新交接使用独立 `prepareForUpdate` 屏障，不复用立即重启。存在任务、任务启动或模态交互时拒绝交接；下载可继续，等待可取消。接受后关闭新意图入口并清理全部 Rust 连接，所有 join 完成仅发 `readyForUpdate`，不提前退出。更新提供者在释放安装提交之前发出退出屏障信号，主程序此时才解除正常退出拦截。安装提交失败时保留关于页恢复入口，用户可正常退出，不启动旧版本自动重启。详细行为见[主动更新](software-update.md)。

English: Updates use a distinct prepareForUpdate barrier. Active/starting tasks and modal interaction reject handoff. Once accepted, new work is blocked and every Rust connection joins before readyForUpdate. The provider releases the application's normal quit interceptor only after this barrier and before committing installation. Failed handoff keeps a recovery entry and permits explicit exit without blindly restarting the old application.
