# 实验安装器与文件关联

本轮交付 Windows x64 当前用户安装器和 Ubuntu 24.04 amd64 X11 的 .deb。它们是未签名开发安装器，不代表 WP-12 全部准入、独立安全审查或通用 Linux 支持完成。后续用户明确要求接入[本地 macOS 开发 DMG](macos-deployment.md)，CI 不部署或上传 Mac 应用，Developer ID 与公证仍待发行环境。

## Windows

build-windows-installer.py 只接受干净构建的 deployed 审计，重新核对每个文件的 SHA-256。固定 Inno Setup 6.7.3、官方安装程序哈希、源码提交、简体中文资源与许可正文见 [工具链配置](../../scripts/installer-toolchain.json)。编译器不进入产品，许可正文随 notices 交付；向导支持简体中文与 English，遵循系统语言。

安装器使用固定 AppId、PrivilegesRequired=lowest，默认安装至当前用户 LocalAppData/Programs/xdzip，不需要管理员权限。主程序固定为安装根目录的 xdzip.exe，Qt 库位于同目录，插件、QML、翻译与通知保留各自必要子目录。快捷方式、文件关联与卸载显示图标均指向根目录，不再创建 versions/提交号-哈希 的产品层级。提交与二进制哈希继续保留在构建和安装器回执中，不作为安装路径。Inno Setup 的[累计卸载日志](https://jrsoftware.org/ishelp/topic_appendnotes.htm)登记各次安装拥有的文件，不递归删除额外内容。设置和最近记录位于既有用户配置目录，不由安装器删除。

新布局写入有界 install-manifest.txt，记录审计过的相对路径和 SHA-256。升级前验证旧清单的路径、重复项和大小，拒绝路径逃逸、设备别名及清理路径中的重解析点；待废弃文件内容被修改或无法核验时，在写入前停止安装。安装成功后只清理旧清单中不再交付且哈希仍匹配的文件，仅移除空目录，保留额外内容。清理期间文件变化或占用时保留文件和重试记录，返回非零退出码 20；写重试记录失败需人工修复，不声称事务性恢复或掉电安全。此当前用户安装流程假设目录不被外部进程并发修改，路径与哈希复检不是基于句柄的恶意竞态隔离。

现有版本目录布局可直接升级，启动命令和快捷方式迁移到根目录，设置及近期记录位置不变。旧安装器未交付独立所有权清单，因此历史 versions/ 目录保留至正常卸载，由 Inno 累计卸载日志清理原记录的文件，不按目录递归删除；用户额外文件继续保留。回退可重新安装指定旧版，旧版 QA 夹具仍使用历史布局，随后可再次升级到根目录。全新安装不产生 versions/。这不包含旧安装目录的全量自动擦除。

产品整个生命周期持有 Local\xdzip.desktop.running 命名互斥量。[AppMutex](https://jrsoftware.org/ishelp/topic_setup_appmutex.htm)阻止安装／卸载，CloseApplications=no，不强制结束应用。FFI 句柄创建失败时启动失败，进程关闭时释放。任务退出仍沿用现有选择与 Rust join 屏障。

关联只写 HKCU：自有 ProgID、Applications/xdzip.exe、Capabilities、RegisteredApplications 及 OpenWithProgids 中自有值。不写 HKLM、扩展默认值或受保护 UserChoice。卸载不删除其他应用候选。命令为带引号的可执行路径、-- 和带引号的 %1，不经过命令解释器。文档图标使用 xdzip-zip.ico。

ZIP、TAR、TGZ 和 GZ 注册为候选；GZ 仅用于 TAR.GZ，普通独立 GZIP 仍不支持，关联不扩大格式能力。默认应用由用户在系统设置选择，安装器不绕过确认。

## Ubuntu 24.04

build-linux-installer.py 从同一核验目录生成 .deb，包名 xdzip，版本绑定产品渠道和提交，架构 amd64。系统依赖来自实际部署清单，补充桌面数据库和锁工具，glibc 至少 2.39。Qt／QML／插件及通知位于 /opt/xdzip/versions/；/usr/bin/xdzip 是固定目标的引号化转发入口。dpkg 负责替换和清除旧包文件，回退需保留原 .deb。

桌面文件按 [MimeType 与 Exec 规范](https://specifications.freedesktop.org/desktop-entry/latest-single/)注册 ZIP、TAR、压缩 TAR，使用 %f 逐文件启动和 -- 参数边界。不执行 xdg-mime default，不修改 mimeapps.list 或全局 MIME 图标。卸载删除自有启动项、图标、运行文件并刷新缓存，保留用户设置及额外文件。

安装的产品持有 /run/xdzip/usage.lock 的共享 flock，维护脚本先取非阻塞独占锁，活动进程存在就失败。脚本在根用户独占的 /run/xdzip 下设置 installing 门，产品取得锁后再次检查，避免在 dpkg 钩子间启动。符号链接或错误所有者／权限被拒绝；关闭 QFile 释放锁。tmpfiles.d 在系统启动时准备运行锁。此约束不影响 SDK 开发应用。

成功及 dpkg abort 路径清理门；中断／掉电可能留下未完成事务，应执行 sudo dpkg --configure -a 并修复实际错误，不能把无法启动当作安装成功。脚本不递归清理用户目录，产物只生成至 build/ 或 CI。

## macOS 与文件打开

Info.plist 以 Viewer／Alternate 声明文档和 ICNS，[不声明默认所有权](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)。默认应用由用户和 Launch Services 决定，本轮不执行开发机全局注册。

FileOpenRouter 在 QML 前接收 [QFileOpenEvent](https://doc.qt.io/qt-6.8/qfileopenevent.html)，只接受本地文件，等控制器空闲后复用真实打开与激活。命令行 -- 后都作为文件路径，避免误解为诊断开关。待处理意图最多 128 个，不抢占活跃任务。现有产品仍为单个真实会话，多输入串行打开并显示最后一个；不宣称跨进程单实例或多会话标签已实现。

启动参数在加载 QML 前入队，主窗口先隐藏，附接控制器时立即提交首个可用请求，不再要求窗口已可见。主窗口在初始事件队列中显示，允许平台启动事件走同一处理路径；外部打开期间先隐藏欢迎入口再改变会话，避免首帧或替换会话闪现欢迎页。取消／失败恢复欢迎页，成功显示真实列表，详见[读取状态](../ux/welcome-opening.md)。

macOS 原生启动单独等待 AppKit 的 NSApplicationDidFinishLaunchingNotification，再在隐藏状态处理该批排队的 Qt 文件事件，随后显示窗口。通知与排队回调均限定在主窗口生命周期，不替换 Qt 的应用 delegate，不使用固定毫秒延迟；offscreen 等非 Cocoa 后端仍走普通事件队列。实际 Launch Services 回归曾检出 1 帧欢迎正文，修正后要求为 0。check-macos-open.py 在本地与 Mac CI 执行 argv 和原生 open 两种路径，仅保存 JSON／日志，不生成或上传应用包。

## 自动验证与限制

三平台 CTest 检查启动前文件事件、非本地 URL 拒绝及读取忙时排队。显式 XDZIP_ASSOCIATION_PROBE 加隔离设置仅供 CI：通过正常打开流程读取自有十万条目 ZIP，保存计数／构建身份并正常关闭，不记录来源路径或执行解压。

当前关联探针等待至少一次实际帧呈现，逐帧记录欢迎正文可见次数；Windows ShellExecuteEx 和 Linux GIO 的当前产品要求 presented_frames 大于 0 且 welcome_frames 为 0。固定旧产品仍按历史探针能力验证，不伪造旧二进制具有新的首帧计数。

Windows 原生 CI 安装 QA 变体 1、升级 2、回退 1，再安装真正交付包；ShellExecuteEx 按自有注册类打开中文、空格、& 路径，检查默认及 UserChoice 不变、忙时拒绝替换、卸载保留设置／额外文件／其他候选。Linux 在无 SDK／系统 Qt／编译器、无网络容器执行相同 dpkg 生命周期，UID 1000 的产品由 GIO 桌面启动项实际打开十万条目，核对 MIME、默认、锁、卸载和 purge。

QA 变体共用本次核验二进制，只改变测试包版本／目录，验证安装机制，不作为发行工件，也不冒充真实不同产品版本的数据迁移。设置 schema 当前为 1，两平台安装元数据记录它，旧安装器拒绝覆盖声明更新 schema 的安装；未来迁移须同步契约。

后续故障注入只允许 QA 变体 2，在 postinst configure 阶段失败一次；正式产品禁止携带该分支。容器实际验证 half-configured 状态、运行门阻止启动、设置保持，再运行 dpkg --configure -a 恢复并经 GIO 打开真实文件。这是可重复配置失败，不冒充设备掉电。设置兼容检查另编译固定 4cf0f8c 与当前源码两个实际组件，往返同 schema 配置；不冒充两个完整安装器的数据迁移。

完整产品兼容另由 prepare-previous-product.py 在一次性 Windows/Linux CI 主机构建固定 4cf0f8c 的完整历史源码：使用 detached worktree 保留真实干净身份，使用当前固定 SDK 生成历史产品 QA 安装器，不下载易过期的旧 Actions 二进制。先安装历史产品并实际打开，再安装当前产品并检查 revision 改变、配置字节保留，再回退历史产品重新打开并检查数据。它证明两个完整源码提交在同 schema 的兼容，不冒充历史已签名原包、尚不存在的 schema 2 迁移或设备重启。历史安装器只作 QA，不上传为产品；回执绑定两个二进制与安装器 SHA-256。

windows-install-lifecycle.json、linux-install-lifecycle.json 及包哈希为实际证据。真实 Explorer/Finder 双击、默认选择 UI、重启后的服务／缓存、不同产品版本的数据迁移、签名信任、GPU 与桌面专项仍需人工，INSTALL-01／ASSOC-01 不自动标记全部通过。

Windows 忙时替换保护的无任务探针使用独立配置关闭托盘，使 `CloseMainWindow` 按该配置执行正常退出；其他关联打开探针保留默认托盘配置并通过显式退出屏障结束。安装保护探针不得把“窗口隐藏”误判为“进程退出失败”，也不得通过强制终止正常路径跳过退出验证。

## 兼容基线的长期输入

真实升级／回退使用固定的旧版完整产品源码，不从已重写的 main 历史获取。`prepare-previous-product.py` 锁定独立 `codex/installer-compat-baseline` 分支中的单根源码快照；该分支只保存构建和回归所需的原始源码及资源，不携带旧 main 父历史、执行报告或工作流。快照内 `COMPAT-BASELINE.json` 记录原始来源提交与树身份，不能合并回 main，也不作为发行包或安全审查结论。

构建后 CLI 身份必须匹配快照自身的固定 SHA，回执另记原始来源身份，不能伪装成原发布二进制。设置组件另以三份原始小型源码作为持久测试输入，逐文件哈希固定，双向读写及未来 schema 拒写不依赖网络或历史 commit。不要随当前实现修改这些回归输入；需要新基线时另行评审并保留来源。
