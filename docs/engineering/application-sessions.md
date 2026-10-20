# 应用会话与退出契约

一个用户登录会话仅运行一个正常产品进程，共用一个托盘和应用菜单。版本查询在创建 GUI 前返回；烟测、桌面探针和测试使用独立路径，不连接真实用户实例。

`SingleInstance` 在创建主窗口前用 QLockFile 选举，持锁者才可移除崩溃残留的本地服务器。QLocalServer 使用 UserAccessOption，结构化 JSON 协议版本为 1，限制单条 256 KiB、128 个绝对路径和 32 个连接。后启动者等待主实例就绪，收到请求 ID 对应的确认后退出，15 秒内通信失败明确报错，不创建第二个正常 UI。Windows 根据服务器握手中的 PID 转交本次用户启动的前台激活权限，不设置永久置顶。IPC 不经过命令解释器，不记录用户路径。

`ApplicationSessions` 是 Qt 应用层协调器，不解释格式、授权目录或文件提交。每个会话拥有独立 SessionPage、BrowserController 和 Rust RuntimeConnection，保留查询、选择、目录与浏览状态。窗口只承载页面，标签页不会为每个归档创建隐藏的原生主窗口。文件关联、命令行、原生文件打开事件、打开按钮、最近记录、拖放和创建结果统一提交打开意图。优先复用无归档且无任务的欢迎页，重复路径激活已有会话，多文件请求按顺序接收并激活最后一个。路径规范化只用于 UI 去重，不授予写入权限。

`desktop/openMode=tabs|windows` 默认 tabs，schema 保持 1，未知字段保留。设置草稿保存后影响后续打开，取消不生效，不迁移已打开页面。标题栏加号按此设置新建页面或窗口。

最多保留 32 个存活会话，达到限额明确提示。所有槽均由后台任务保留时，关闭最后浏览页复用原页显示欢迎与全局任务入口，不创建额外运行时。关闭标签或非最后主窗口不取消任务。仍有任务或完成结果的页面保留所属控制器，统一任务菜单使用“会话 UUID/后端任务 ID”，不能把另一个连接中相同数字 ID 当作同一任务。密码、冲突和结果窗口仍由原会话控制器处理。无任务的关闭页面先请求异步关闭，接收线程完成 join 后才从会话集合移除。

会话关闭后，延迟到达的开始、预检、密码与冲突事件不自动弹窗或抢焦点。任务与请求令牌继续保留，用户从全局任务入口主动打开时再显示原会话的进度和待处理请求，关闭页面不等于取消请求。

创建和解压在应用级串行准入，包括准备、密码或冲突等待、提交与清理阶段。等待准入的控制器仅保留一个有界命令，可取消，不占用 GUI 线程。单个 Rust 连接的任务顺序和安全语义继续有效。明确退出时，先对所有会话作统一确认，选择等待则全部任务结束后退出，选择取消则等待全部连接完成取消、清理与 join；只检查关闭完成状态，不在状态检查中再次触发关闭。

主窗口可见时，“显示并激活”只提升和请求激活。隐藏或最小化时恢复先前普通、最大化或全屏状态，不先还原再最大化。阻塞的模态窗口优先激活，弹窗 transientParent 绑定实际调用窗口。关闭最后一个主窗口继续遵循托盘设置。设置与关于从统一入口进入，已有窗口优先激活，不创建另一份可见草稿。

`showMinimized()` 会替换原状态位。恢复时先在仍最小化的状态下补齐保存的最大化或全屏标志，再仅移除最小化标志，不先显示普通窗口。原生回归等待窗口实际暴露、激活及最终状态确认，避免把 Qt 请求值当作窗口管理器已完成的状态。

验收包括实际原生窗口状态变化、并发启动与崩溃残留、Unicode 和批量路由、两种打开模式、保存与取消及重启、跨会话 ID 冲突、后台任务、密码与冲突、全局退出和接收线程回收。离屏测试只能证明状态契约，不能代替原生视觉、辅助功能和桌面人工验收。证据保存在 build/ 或 CI 工件。

English: One normal product process and tray serve each user login session. Each archive owns an independent controller and Rust connection. The saved open mode selects tabs or windows for future opens; duplicate archives activate their existing session. Closing a page keeps its tasks and results accessible through composite session/task identities. Heavy operations are admitted serially across the application. Explicit quit waits for cancellation, cleanup and asynchronous joins across every session. Activation preserves normal, maximized and fullscreen state and prioritizes blocking dialogs. Diagnostic runs remain isolated. Automated evidence does not claim manual desktop acceptance.

退出进入实际取消与 join 阶段后，IPC 明确拒绝新意图。关闭通道后才释放选举锁，全部连接完成后才启动重启进程，旧实例的重复清理不能移除新端点。Windows 重启同样转交本次用户操作的前台权限。

阻塞弹窗期间接收的外部打开保留最后一个激活意图，先激活阻塞窗口，待其关闭后显示对应新窗口，不使隐藏的新窗口失去入口。设置中的关于窗口保留实际设置调用者，不因另一主窗口获得焦点而改变所属关系。

原生回归入口为 `python3 scripts/check-native-sessions.py --tests <browser-tests> --output build/review/native-sessions.json`。Linux 使用 Xvfb 和 Openbox 的真实 X11 窗口管理路径，Windows、macOS 使用系统窗口管理器；脚本只管理自己启动的验证进程。普通 CI 与 Release 共用此入口，原生日志和 JSON 保存在 build/ 或 CI 工件，不能据此宣称完成人工视觉、GPU、输入法或屏幕阅读验收。
