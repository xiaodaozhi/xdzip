# 实验安装器与文件关联

提供 Windows x64 当前用户安装器、Ubuntu 24.04 amd64 X11 的 .deb，以及按明确需求制作的[本地 macOS 开发 DMG](macos-deployment.md)。它们仍是实验性开发交付；WP-12 开发与验收完成不代表发行许可复核获批。个人开源发行不要求付费证书、Developer ID 或公证，macOS 保留必要 ad-hoc 完整性签名；CI 不部署或上传 Mac 应用。支持范围见[支持政策](support-policy.md)。

## 共用构建入口

环境准备与操作顺序统一见[开发构建指南](developer-build.md)。已有同提交的干净构建与测试后，使用 `python3 scripts/desktop.py prepare --packaging` 准备锁定材料，再执行 package 和 verify，不直接把 CI 全量脚本用于日常构建。

Windows/Linux 的 `package --kind runtime|installer|all` 共用同一部署与审计链，默认 `all`。runtime 先生成部署目录，installer 从同一 output-dir 已核验的运行目录生成安装器，Linux 同时生成 tar.gz；macOS 只支持本地 `all` 开发 DMG。产物及回执位于指定 output-dir，默认 build/package。准备阶段可下载固定打包工具与材料，但不安装 xdzip，不自动安装系统编译器／SDK。Windows 准备 CRT／Inno Setup，Linux 预先安装 patchelf 与 Debian 包工具，Mac 使用系统部署与 ad-hoc 完整性工具。

普通 `verify` 检查已有运行时／包，Windows 在独立副本清除 SDK 搜索路径，Linux 使用隔离运行容器，Mac 使用 DMG 重定位与 Cocoa／Metal。它不默认执行安装、升级／回退或关联生命周期，不写用户默认打开方式。下面的安装机制与一次性 CI 专项仍是独立门槛，不能用运行时通过替代。

## Windows

安装器展示随包的原始 Microsoft VC++ 运行库协议，许可页明确适用组件范围并要求接受，随后展示双语 Windows-runtime-NOTICE.txt。便携包附带同一原文和说明；项目原创代码仍采用 Apache-2.0，Qt 等组件的原始授权和替换权不受此页影响。编译前核对原始协议哈希和完整说明，材料附件保留同一文本，详见 [Windows 许可](windows-licensing.md)。

便携包与安装器使用同一部署目录，由 [deploy_windows.py](../../scripts/deploy_windows.py) 调用 windeployqt，再按已固定的 Qt Quick Basic 策略移除 FluentWinUI3、Fusion、Imagine、Material、Universal、Windows、仅样式引用的 NativeStyle／Effects、qmltooling 和 .qmltypes。Basic、Controls impl、Templates、运行时 QML 与 qmldir 保留；Qt Widgets 的 qmodernwindowsstyle 继续服务原生控件。opengl32sw、DXC、DXIL、D3DCompiler 图形回退库保留。增加新样式或 Effects 功能时必须同时调整部署策略并复验。

VC++ 采用应用目录内部署，使用 [固定来源与文件哈希](../../scripts/windows-runtime.json) 的微软官方 x64 14.44.35211 运行库。[准备脚本](../../scripts/prepare-windows-runtime.py) 用固定 WiX 工具解包，再校验每个 DLL 和许可正文，不执行 EXE 或 MSI 安装、不从 System32 复制文件。交付所需 MSVCP／VCRUNTIME DLL 与许可、来源记录，工具、CAB、MSI 和未执行的 vc_redist 安装程序不进入产品；Windows 10／11 自带 UCRT。应用目录内运行库须随产品更新维护，不能依赖系统集中运行库更新替换它们，参见 [Microsoft 部署方式](https://learn.microsoft.com/en-us/cpp/windows/choosing-a-deployment-method)。

[部署回归](../../scripts/test-windows-deployment.py) 检查裁剪与运行库缺失边界；[实际运行检查](../../scripts/verify_windows_runtime.py) 在独立副本中加入测试 DLL，清除 SDK 搜索路径，执行系统翻译、设置重启、原生标题栏和双语启动。测试文件不写入产品目录，此检查不替代干净 Windows 上的安装／升级／卸载验收。

build-windows-installer.py 只接受干净构建的 deployed 审计，重新核对每个文件的 SHA-256。固定 Inno Setup 6.7.3、官方安装程序哈希、源码提交、简体中文资源与许可正文见 [工具链配置](../../scripts/installer-toolchain.json)。编译器不进入产品，许可正文随 notices 交付；向导支持简体中文与 English，遵循系统语言。

安装器使用固定 AppId、PrivilegesRequired=lowest，默认安装至当前用户 LocalAppData/Programs/xdzip，不需要管理员权限。主程序固定为安装根目录的 xdzip.exe，Qt 库位于同目录，插件、QML、翻译与通知保留各自必要子目录。快捷方式、文件关联与卸载显示图标均指向根目录，不再创建 versions/提交号-哈希 的产品层级。提交与二进制哈希继续保留在构建和安装器回执中，不作为安装路径。Inno Setup 的[累计卸载日志](https://jrsoftware.org/ishelp/topic_appendnotes.htm)登记各次安装拥有的文件，不递归删除额外内容。设置和最近记录位于既有用户配置目录，不由安装器删除。

新布局写入有界 install-manifest.txt，记录审计过的相对路径和 SHA-256。升级前验证旧清单的路径、重复项和大小，拒绝路径逃逸、设备别名及清理路径中的重解析点；待废弃文件内容被修改或无法核验时，在写入前停止安装。安装成功后只清理旧清单中不再交付且哈希仍匹配的文件，仅移除空目录，保留额外内容。清理期间文件变化或占用时保留文件和重试记录，返回非零退出码 20；写重试记录失败需人工修复，不声称事务性恢复或掉电安全。此当前用户安装流程假设目录不被外部进程并发修改，路径与哈希复检不是基于句柄的恶意竞态隔离。

现有版本目录布局可直接升级，启动命令和快捷方式迁移到根目录，设置及近期记录位置不变。旧安装器未交付独立所有权清单，因此历史 versions/ 目录保留至正常卸载，由 Inno 累计卸载日志清理原记录的文件，不按目录递归删除；用户额外文件继续保留。回退可重新安装指定旧版，旧版 QA 夹具仍使用历史布局，随后可再次升级到根目录。全新安装不产生 versions/。这不包含旧安装目录的全量自动擦除。

产品整个生命周期持有 Local\xdzip.desktop.running 命名互斥量。[AppMutex](https://jrsoftware.org/ishelp/topic_setup_appmutex.htm)阻止安装／卸载，CloseApplications=no，不强制结束应用。FFI 句柄创建失败时启动失败，进程关闭时释放。任务退出仍沿用现有选择与 Rust join 屏障。

关联只写 HKCU：自有 ProgID、Applications/xdzip.exe、Capabilities、RegisteredApplications 及 OpenWithProgids 中自有值。不写 HKLM、扩展默认值或受保护 UserChoice。卸载不删除其他应用候选。命令为带引号的可执行路径、-- 和带引号的 %1，不经过命令解释器。文档图标使用 xdzip-zip.ico。

ZIP、TAR、TGZ 和 GZ 注册为候选；GZ 仅用于 TAR.GZ，普通独立 GZIP 仍不支持，关联不扩大格式能力。默认应用由用户在系统设置选择，安装器不绕过确认。

## Ubuntu 24.04

build-linux-installer.py 从同一核验目录生成 .deb，私有暂存目录按输出路径区分，允许相同包名在不同候选目录重新打包，不覆盖既有暂存。包名 xdzip，版本绑定产品渠道和提交，架构 amd64。系统依赖来自实际部署清单，补充桌面数据库和锁工具，glibc 至少 2.39。Qt／QML／插件及通知位于 /opt/xdzip/versions/；/usr/bin/xdzip 是固定目标的引号化转发入口。dpkg 负责替换和清除旧包文件，回退需保留原 .deb。

桌面文件按 [MimeType 与 Exec 规范](https://specifications.freedesktop.org/desktop-entry/latest-single/)注册 ZIP、TAR、压缩 TAR，使用 %f 逐文件启动和 -- 参数边界。不执行 xdg-mime default，不修改 mimeapps.list 或全局 MIME 图标。卸载删除自有启动项、图标、运行文件并刷新缓存，保留用户设置及额外文件。

安装的产品持有 /run/xdzip/usage.lock 的共享 flock，维护脚本先取非阻塞独占锁，活动进程存在就失败。脚本在根用户独占的 /run/xdzip 下设置 installing 门，产品取得锁后再次检查，避免在 dpkg 钩子间启动。符号链接或错误所有者／权限被拒绝；关闭 QFile 释放锁。tmpfiles.d 在系统启动时准备运行锁。此约束不影响 SDK 开发应用。

成功及 dpkg abort 路径清理门；中断／掉电可能留下未完成事务，应执行 sudo dpkg --configure -a 并修复实际错误，不能把无法启动当作安装成功。脚本不递归清理用户目录，产物只生成至 build/ 或 CI。

## macOS 与文件打开

主应用和交付包的 Info.plist 以 Viewer／Alternate 声明文档和 ICNS，[不声明默认所有权](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)。开发和测试构建默认使用 Viewer／None，不进入系统打开方式候选；主开发包或交付构建须显式设置 `-DXDZIP_MACOS_HANDLER_RANK=Alternate`。默认应用由用户和 Launch Services 决定，不主动修改默认应用。开发验证副本的临时注册按[Mac 维护方法](macos-deployment.md)收尾，保留已有应用与用户默认选择。

FileOpenRouter 在 QML 前接收 [QFileOpenEvent](https://doc.qt.io/qt-6.8/qfileopenevent.html)，只接受本地文件，等控制器空闲后复用真实打开与激活。命令行 -- 后都作为文件路径，避免误解为诊断开关。待处理意图最多 128 个，不抢占活跃任务。现有产品仍为单个真实会话，多输入串行打开并显示最后一个；不宣称跨进程单实例或多会话标签已实现。

启动参数在加载 QML 前入队，主窗口先隐藏，附接控制器时立即提交首个可用请求，不再要求窗口已可见。主窗口在初始事件队列中显示，允许平台启动事件走同一处理路径；外部打开期间先隐藏欢迎入口再改变会话，避免首帧或替换会话闪现欢迎页。取消／失败恢复欢迎页，成功显示真实列表，详见[读取状态](../ux/welcome-opening.md)。

macOS 原生启动单独等待 AppKit 的 NSApplicationDidFinishLaunchingNotification，再在隐藏状态处理该批排队的 Qt 文件事件，随后显示窗口。通知与排队回调均限定在主窗口生命周期，不替换 Qt 的应用 delegate，不使用固定毫秒延迟；offscreen 等非 Cocoa 后端仍走普通事件队列。实际 Launch Services 回归曾检出 1 帧欢迎正文，修正后要求为 0。check-macos-open.py 在本地与 Mac CI 执行 argv 和原生 open 两种路径，仅保存 JSON／日志，不生成或上传应用包。

## 自动验证与限制

三平台 CTest 检查启动前文件事件、非本地 URL 拒绝及读取忙时排队。显式 XDZIP_ASSOCIATION_PROBE 加隔离设置仅供 CI：通过正常打开流程读取自有十万条目 ZIP，保存计数／构建身份并正常关闭，不记录来源路径或执行解压。

当前关联探针等待至少一次实际帧呈现，逐帧记录欢迎正文可见次数；Windows ShellExecuteEx 和 Linux GIO 的当前产品要求 presented_frames 大于 0 且 welcome_frames 为 0。固定旧产品仍按历史探针能力验证，不伪造旧二进制具有新的首帧计数。

Windows 原生 CI 安装 QA 变体 1、升级 2、回退 1，再安装真正交付包；ShellExecuteEx 按自有注册类打开中文、空格、& 路径，检查默认及 UserChoice 不变、忙时拒绝替换、卸载保留设置／额外文件／其他候选。Linux 在无 SDK／系统 Qt／编译器、无网络容器执行相同 dpkg 生命周期，UID 1000 的产品由 GIO 桌面启动项实际打开十万条目，核对 MIME、默认、锁、卸载和 purge。

QA 变体共用本次核验二进制，只改变测试包版本／目录，验证安装机制，不作为发行工件，也不冒充真实不同产品版本的数据迁移。设置 schema 当前为 1，两平台安装元数据记录它，旧安装器拒绝覆盖声明更新 schema 的安装；未来迁移须同步契约。

后续故障注入只允许 QA 变体 2，在 postinst configure 阶段失败一次；正式产品禁止携带该分支。容器实际验证 half-configured 状态、运行门阻止启动、设置保持，再运行 dpkg --configure -a 恢复并经 GIO 打开真实文件。这是可重复配置失败，不冒充设备掉电。设置兼容检查另编译保留的原始设置组件与当前组件，往返同 schema 配置；原始输入逐文件固定哈希，不依赖旧 main 历史，不冒充完整安装器的数据迁移。

完整产品兼容另由 prepare-previous-product.py 在一次性 Windows/Linux CI 主机构建固定兼容快照（来源为原 4cf0f8c）的完整产品源码：使用 detached worktree 保留真实干净身份，使用当前固定 SDK 生成历史产品 QA 安装器，不下载易过期的旧 Actions 二进制。先安装历史产品并实际打开，再安装当前产品并检查 revision 改变、配置字节保留，再回退历史产品重新打开并检查数据。它证明两个完整源码提交在同 schema 的兼容，不冒充历史已签名原包、尚不存在的 schema 2 迁移或设备重启。历史安装器只作 QA，不上传为产品；回执绑定两个二进制与安装器 SHA-256。

windows-install-lifecycle.json、linux-install-lifecycle.json 及包哈希为实际证据。真实 Explorer/Finder 双击、默认选择 UI、重启后的服务／缓存、未正式签名提示、GPU 与桌面专项按人工清单核验。自动脚本不得自行把 INSTALL-01／ASSOC-01 全部标记通过；既有人工结论见 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)，不因重新生成 pending 模板而重开验收。未来 schema 迁移须另增实际兼容用例。

Windows 忙时替换保护的无任务探针使用独立配置关闭托盘，使 `CloseMainWindow` 按该配置执行正常退出；其他关联打开探针保留默认托盘配置并通过显式退出屏障结束。安装保护探针不得把“窗口隐藏”误判为“进程退出失败”，也不得通过强制终止正常路径跳过退出验证。

## 兼容基线的长期输入

真实升级／回退使用固定的旧版完整产品源码，不从已重写的 main 历史获取。`prepare-previous-product.py` 锁定独立 `codex/installer-compat-baseline` 分支中的单根源码快照；该分支只保存构建和回归所需的原始源码及资源，不携带旧 main 父历史、执行报告或工作流。快照内 `COMPAT-BASELINE.json` 记录原始来源提交与树身份，不能合并回 main，也不作为发行包或当前产品的验收结论。

构建后 CLI 身份必须匹配快照自身的固定 SHA，回执另记原始来源身份，不能伪装成原发布二进制。设置组件另以三份原始小型源码作为持久测试输入，逐文件哈希固定，双向读写及未来 schema 拒写不依赖网络或历史 commit。不要随当前实现修改这些回归输入；需要新基线时另行评审并保留来源。

## Debian 版本迁移

首个公开预发布开始使用 Debian epoch 1，包版本为 `1:语义版本映射+git提交`，例如 `1:0.1.0~alpha.1+git...`。历史 `0.1.0~dev.9` 按 dpkg 规则会排在无 epoch 的 alpha.1 之后，不能直接沿用旧映射。epoch 只用于系统包排序，产品界面、版本标签和 release.json 仍使用原语义版本，后续 Debian 包保持 epoch 1，alpha、beta、rc、stable 的顺序继续由上游版本表达，规则见 [Debian Policy](https://www.debian.org/doc/debian-policy/ch-controlfields.html#version)。

真实历史产品夹具使用仅 CI 允许的 `--qa-legacy-version` 保留原无 epoch 身份，不能把旧包改成新排序来掩盖升级问题。当前包必须在实际 dpkg 升级、回退和安装生命周期验收中通过；用户配置与外部默认关联保持原契约。
