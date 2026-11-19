# 安装器与文件关联

[English](../../engineering/installers-associations.md) | 简体中文

Windows 11 x64 用户可以在 Microsoft Store 搜索 **XDZip**，安装或更新商店版。Windows EXE 安装器、便携 ZIP 及 macOS、Linux 安装包可从 [GitHub 最新发布页](https://github.com/xiaodaozhi/xdzip/releases/latest)下载。两条渠道独立发布，安装前可在所选渠道核对版本。

Windows MSIX 使用包清单声明同一组扩展作为打开方式候选，不覆盖用户默认选择。文件关联页完整显示可关联类型，开关为禁用状态。当前默认应用以 Windows 设置为准。“打开默认应用设置”使用当前包的 `registeredAUMID` 进入对应页面，身份查询失败时打开通用默认应用选择页。应用不读取普通安装版注册作为包状态，不写 UserChoice。实际文件激活、图标、共存和卸载行为须在目标系统验收，见[MSIX 构建](developer-build.md#windows-msix-实验构建)。


逐格式图标素材及平台映射见[文件关联图标规范](../visual/file-association-icons.md)。Linux 保留系统 MIME 图标，安装不改变默认选择或现有注册范围。

MSIX 文件关联引用逐格式 PNG，直接提取自普通 Windows 安装版使用的同一套 ICO 帧。ZIP、7z、RAR 等已映射格式保留各自的文档图案、尺寸配置和透明边距；应用标志另有独立的无底板目标尺寸资源。两组资源均编入包内的 `resources.pri`，修改商店一览图片不会替换这些已安装图标。图标缓存和默认处理程序由 Windows 管理，安装不会自动更改默认选择。

自 `v0.2.0` 起的桌面包及当前源码构建将命令行程序放在独立的 `cli/` 目录，macOS 位于 `Contents/Helpers/cli/`，保留原 GUI 与文件关联入口。更早版本的附件不追溯加入该功能。Windows 安装器提供默认关闭的当前用户 PATH 选项，检测同名命令冲突；卸载只撤销本安装目录。Mac/Linux 显式运行私有 CLI 的 `setup`，独立 CLI 包也通过 `setup` 安装用户级版本目录和稳定启动入口，不改 Shell 配置。撤销用 `setup --remove`，保留版本目录供核查。详见 [CLI 安装契约](command-line.md#构建与命令安装)。

下方打包命令生成实验性开发包。GitHub 正式发行遵循[发布协议](release-protocol.md)，商店包使用独立的 [MSIX 构建流程](developer-build.md#microsoft-store-msix)。macOS 直接下载包采用 ad-hoc 完整性签名，未使用 Developer ID 签名或公证。适用系统见[支持政策](support-policy.md)。

当前源码的 Windows 安装器发行方、Linux 包维护者及更新策略 vendor 均为 `xiaodaozhi`。这些元数据不代替发行者证书签名，已有发行包以其原始元数据为准。

macOS 手动更新时优先使用对应 Release 的 DMG；使用 app.zip 时，先解压并将 xdzip.app 移出下载目录，退出正在运行的旧版，再替换“应用程序”中的副本。下载不代表安装完成，未公证包可能触发系统安全提示。具体回退条件见[软件更新契约](software-update.md)。

系统右键菜单、独立操作宿主和自有注册生命周期见[文件管理器操作契约](shell-actions.md)。右键入口不打开浏览主窗口，完成后默认不打开目标文件夹。

## 共用构建入口

环境准备与操作顺序统一见[开发构建指南](developer-build.md)。已有同提交的干净构建与测试后，使用 `python3 scripts/desktop.py prepare --packaging` 准备锁定材料，再执行 package 和 verify，不直接把 CI 全量脚本用于日常构建。

Windows/Linux 的 `package --kind runtime|installer|all` 共用同一部署与审计链，默认 `all`。runtime 先生成部署目录，installer 从同一 output-dir 已核验的运行目录生成安装器，Linux 同时生成 tar.gz；macOS 只支持本地 `all` 开发 DMG。产物及回执位于指定 output-dir，默认 build/package。准备阶段可下载固定打包工具与材料，但不安装 xdzip，不自动安装系统编译器／SDK。Windows 准备 CRT／Inno Setup，Linux 预先安装 patchelf 与 Debian 包工具，Mac 使用系统部署与 ad-hoc 完整性工具。

普通 `verify` 检查已有运行时／包，Windows 在独立副本清除 SDK 搜索路径，Linux 使用隔离运行容器，Mac 使用 DMG 重定位与 Cocoa／Metal。它不默认执行安装、升级／回退或关联生命周期，不写用户默认打开方式。下面的安装机制与一次性 CI 专项仍是独立门槛，不能用运行时通过替代。

## Windows

安装器展示随包的原始 Microsoft VC++ 运行库协议，许可页明确适用组件范围并要求接受，随后展示双语 Windows-runtime-NOTICE.txt。便携包附带同一原文和说明；项目原创代码仍采用 Apache-2.0，Qt 等组件的原始授权和替换权不受此页影响。编译前核对原始协议哈希和完整说明，材料附件保留同一文本，详见 [Windows 许可](windows-licensing.md)。

便携包与安装器使用同一部署目录，由 [deploy_windows.py](../../../scripts/deploy_windows.py) 调用 windeployqt，再按已固定的 Qt Quick Basic 策略移除 FluentWinUI3、Fusion、Imagine、Material、Universal、Windows、仅样式引用的 NativeStyle／Effects、qmltooling 和 .qmltypes。Basic、Controls impl、Templates、运行时 QML 与 qmldir 保留；Qt Widgets 的 qmodernwindowsstyle 继续服务原生控件。新部署目录不携带 DXC、DXIL、D3DCompiler，任意位置存在这些名称时部署失败。`opengl32sw.dll` 不进入新部署目录；已存在的根目录副本会在受控暂存中移除，其他位置的同名残留使部署失败。增加新样式或 Effects 功能时必须同时调整部署策略并复验。

Windows 上 Qt Quick 默认使用 Direct3D 11。Qt 在部分硬件或驱动功能不足的情况下可回退到 Direct3D WARP；这与软件 OpenGL 是不同路径。新包不承诺通过 `QT_OPENGL=software` 或 OpenGL 后端提供 Mesa 软件回退。历史包及其许可材料保持原样。[Qt 图形后端说明](https://doc.qt.io/qt-6.8/windows-graphics.html)列出默认和显式软件设备行为。


VC++ 采用应用目录内部署，使用 [固定来源与文件哈希](../../../scripts/windows-runtime.json) 的微软官方 x64 14.44.35211 运行库。[准备脚本](../../../scripts/prepare-windows-runtime.py) 用固定 WiX 工具解包，再校验每个 DLL 和许可正文，不执行 EXE 或 MSI 安装、不从 System32 复制文件。交付所需 MSVCP／VCRUNTIME DLL 与许可、来源记录，工具、CAB、MSI 和未执行的 vc_redist 安装程序不进入产品；Windows 10／11 自带 UCRT。应用目录内运行库须随产品更新维护，不能依赖系统集中运行库更新替换它们，参见 [Microsoft 部署方式](https://learn.microsoft.com/en-us/cpp/windows/choosing-a-deployment-method)。

[部署回归](../../../scripts/test-windows-deployment.py) 检查裁剪与运行库缺失边界；[实际运行检查](../../../scripts/verify_windows_runtime.py) 拒绝软件 OpenGL DLL，在独立副本中加入测试 DLL，清除 SDK 搜索路径，执行系统翻译、设置重启、原生标题栏和双语启动。它另在清除渲染覆盖后核对默认 D3D11 场景图初始化，并显式请求 WARP、核对实际选中的 Microsoft 软件设备。显式请求不证明故障下的自动回退，也不替代人工视觉验收。测试文件不写入产品目录，此检查不替代干净 Windows 上的安装／升级／卸载验收。

build-windows-installer.py 只接受干净构建的 deployed 审计，重新核对每个文件的 SHA-256。固定 Inno Setup 6.7.3、官方安装程序哈希、源码提交、简体中文资源与许可正文见 [工具链配置](../../../scripts/installer-toolchain.json)。编译器不进入产品，许可正文随 notices 交付；向导支持简体中文与 English，遵循系统语言。

安装器使用固定 AppId、PrivilegesRequired=lowest，默认安装至当前用户 LocalAppData/Programs/xdzip，不需要管理员权限。主程序固定为安装根目录的 xdzip.exe，Qt 库位于同目录，插件、QML、翻译与通知保留各自必要子目录。快捷方式、文件关联与卸载显示图标均指向根目录，不再创建 versions/提交号-哈希 的产品层级。提交与二进制哈希继续保留在构建和安装器回执中，不作为安装路径。Inno Setup 的[累计卸载日志](https://jrsoftware.org/ishelp/topic_appendnotes.htm)登记各次安装拥有的文件，不递归删除额外内容。设置和最近记录位于既有用户配置目录，不由安装器删除。

新布局写入有界 install-manifest.txt，记录审计过的相对路径和 SHA-256。升级前验证旧清单的路径、重复项和大小，拒绝路径逃逸、设备别名及清理路径中的重解析点；待废弃文件内容被修改或无法核验时，在写入前停止安装。安装成功后只清理旧清单中不再交付且哈希仍匹配的文件，仅移除空目录，保留额外内容。同一次安装即使多次收到完成事件也只执行一次清理。清理期间文件变化或占用时保留文件和重试记录，返回非零退出码 20；写重试记录失败需人工修复，不声称事务性恢复或掉电安全。此当前用户安装流程假设目录不被外部进程并发修改，路径与哈希复检不是基于句柄的恶意竞态隔离。

现有版本目录布局可直接升级，启动命令和快捷方式迁移到根目录，设置及近期记录位置不变。旧安装器未交付独立所有权清单，因此历史 versions/ 目录保留至正常卸载，由 Inno 累计卸载日志清理原记录的文件，不按目录递归删除；用户额外文件继续保留。回退可重新安装指定旧版，旧版 QA 夹具仍使用历史布局，随后可再次升级到根目录。全新安装不产生 versions/。这不包含旧安装目录的全量自动擦除。

产品整个生命周期持有 Local\xdzip.desktop.running 命名互斥量。[AppMutex](https://jrsoftware.org/ishelp/topic_setup_appmutex.htm)阻止安装／卸载，CloseApplications=no，不强制结束应用。FFI 句柄创建失败时启动失败，进程关闭时释放。任务退出仍沿用现有选择与 Rust join 屏障。

普通 Windows 安装包在 HKCU 注册打开方式候选，写入自有 ProgID、两个自有 Applications 项、Capabilities、RegisteredApplications 和 OpenWithProgids 中的自有值。HKLM、扩展名默认值、受保护的 UserChoice／UserChoiceLatest 和其他应用的候选均保留。每个登记后缀有单独的 ProgID 和对应 ICO，例如 `xdzip.Archive.ZIP`。旧共享 `xdzip.Archive` 保留直接打开命令，与 `Applications/xdzip.exe` 一起从打开方式中隐藏。升级移除旧推荐项，保留已有默认选择的兼容性。Windows 为 `.tar.gz` 等复合后缀文件选择图标时使用最后一段扩展名。

普通安装版在启动及应用语言切换成功后同步九个核心 ProgID 的类型描述，包括 ZIP、TAR、TGZ、GZ、7Z、RAR、CAB、ISO、UDF。安装包按应用保存的语言设置描述，自动模式下使用系统显示语言。向导语言独立设置。同步前核对安装标识、程序位置、安装清单、关联启动器及九个已有打开命令。登记缺失或不符时不写入。写入部分成功时，下次正常启动可重试。便携、开发和使用隔离设置的测试程序不会运行此同步。其他已登记后缀保留安装时写入的描述。Explorer 缓存和默认应用选择可能影响实际显示。


一次性 Windows CI 安装检查使用真实产品验证中英文启动同步，并以便携版及临时设置进程验证不写入；同时核对九类图标、命令、其他应用候选和默认选择未变。此自动检查不模拟运行中切换语言，也不替代 Explorer 或系统设置的人工显示验收。


全部格式 ProgID 的带引号命令启动 `xdzip-association.exe -- "%1"`，其 Applications 注册承载完整 SupportedTypes，使系统“打开方式”只呈现一个 xdzip 应用入口。启动器嵌入同一品牌应用 ICO 和独立 OriginalFilename、产品版本资源；应用图标与逐格式文档图标分别核验，不能用文档 DefaultIcon 代替可执行文件的应用资源。该小型 GUI 启动器只接受单个文件参数，先在自己的线程初始化 COM 单线程单元，再通过 ShellExecuteEx 的明确 `xdzip.Archive` 类委托既有打开命令，不经过命令解释器；调用返回后释放 COM。旧版回退会更新共享入口，逐格式候选因此仍指向回退后的程序；COM 初始化失败、入口不存在、参数无效或系统拒绝调用时启动器返回非零，不尝试其他应用或修改默认值。该机制依赖回退安装保留当前安装目录内的启动器，安装／回退和累计卸载须实测。

安装器在共享 ProgID 和各个已登记的格式 ProgID 上设置 REG_NONE 类型的 `AllowSilentDefaultTakeOver`。Windows 仍可在“打开方式”中提供这些候选。安装器用有界原生注册表调用写入已经创建的自有键，失败时终止安装并释放句柄。详见 [Windows ProgID 约定](https://learn.microsoft.com/en-us/windows/win32/shell/fa-progids)。


普通安装版和 MSIX 从[格式登记表](format-registry.md)生成 Windows“打开方式”候选，覆盖核心的 ZIP、TAR、TGZ、GZ、7z、RAR、CAB、ISO、UDF 以及其他已登记后缀。`.gz` 按内容区分独立 GZIP 与 TAR.GZ。通用 `.img`、`.001` 和 `.rNN` 不注册为候选，可通过显式打开或适用入口提交。关联提供系统启动 xdzip 的入口，读取器仍探测文件，并检查操作所需的格式与来源条件。

## Ubuntu 24.04

build-linux-installer.py 从同一核验目录生成 .deb，私有暂存目录按输出路径区分，允许相同包名在不同候选目录重新打包，不覆盖既有暂存。包名 xdzip，版本绑定产品渠道和提交，架构 amd64。系统依赖来自实际部署清单，补充桌面数据库和锁工具，glibc 至少 2.39。Qt／QML／插件及通知位于 /opt/xdzip/versions/；/usr/bin/xdzip 是固定目标的引号化转发入口。dpkg 负责替换和清除旧包文件，回退需保留原 .deb。

安装后的 [xdzip.desktop](../../../apps/desktop/packaging/linux/xdzip.desktop) 声明软件包的 MIME 候选，包括 ZIP 及部分别名、TAR 与压缩 TAR、独立压缩流、7z、RAR 和 CAB 等归档，以及 ISO／UDF 映像。`%f` 用于单文件启动，`--` 标记参数边界。软件包提供 `*.udf` 的 MIME 声明，不接管通用映像或二进制类型。安装和卸载刷新 MIME 及桌面数据库，不改写 `mimeapps.list`、用户默认选择或系统 MIME 图标。卸载清除自有启动项、图标及运行文件，保留用户设置。

安装的产品持有 /run/xdzip/usage.lock 的共享 flock，维护脚本先取非阻塞独占锁，活动进程存在就失败。脚本在根用户独占的 /run/xdzip 下设置 installing 门，产品取得锁后再次检查，避免在 dpkg 钩子间启动。符号链接或错误所有者／权限被拒绝；关闭 QFile 释放锁。tmpfiles.d 在系统启动时准备运行锁。此约束不影响 SDK 开发应用。

成功及 dpkg abort 路径清理门；中断／掉电可能留下未完成事务，应执行 sudo dpkg --configure -a 并修复实际错误，不能把无法启动当作安装成功。脚本不递归清理用户目录，产物只生成至 build/ 或 CI。

## macOS 与文件打开

主应用和交付包的 Info.plist 以 Viewer／Alternate 声明文档和 ICNS，[不声明默认所有权](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)。开发和测试构建默认使用 Viewer／None，不进入系统打开方式候选；主开发包或交付构建须显式设置 `-DXDZIP_MACOS_HANDLER_RANK=Alternate`。默认应用由用户和 Launch Services 决定，不主动修改默认应用。开发验证副本的临时注册按[Mac 维护方法](macos-deployment.md)收尾，保留已有应用与用户默认选择。

CAB、ISO 和 UDF 使用同一个 Viewer 角色，ISO 使用 `public.iso-image`。应用不注册 `.img` 或通用 `public.disk-image`。Viewer 是 Launch Services 中的关联角色，实际可执行的操作由[格式后端](native-codecs.md)决定。[ISO 创建配置](iso-creation.md)可单独写入新映像。

文档类型使用应用包内对应的 ICNS。压缩 TAR 类型可覆盖 `.tar.gz` 与 `.tgz`，独立 GZIP 也有文档类型。Finder 缓存、默认应用选择和系统类型识别影响实际显示。

macOS 的 RAR、CAB 和 UDF 导入类型声明均须符合 `public.data`，使系统能将实际文件识别为相应类型。CAB 同时声明系统使用的 `com.microsoft.cab` 和兼容标识 `com.microsoft.cab-archive`；设置页在系统首选类型是这两者之一时查询并更改该实际类型，否则沿用兼容标识。UDF 的 Viewer 声明使用 `.udf` 扩展名而不指定 `LSItemContentTypes`，使旧安装的无效 UTI 声明仍被系统采用时，实际文件也能发现候选；此[扩展名声明键](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)已被 Apple 标记为弃用，必须持续在受支持的 macOS 版本验证。设置页按系统实际识别的 `.udf` 类型查询和更改，类型不可用时回退至既有标识。已有默认选择不由安装或注册主动覆盖。此前没有处理器时，Launch Services 可能按[隐式规则](https://developer.apple.com/library/archive/documentation/Carbon/Conceptual/LaunchServicesConcepts/LSCGlossary/LSCGlossary.html)将新候选解析为当前处理器；撤销临时注册后应恢复原状，此解析不等于应用主动写入用户偏好。


FileOpenRouter 在 QML 前接收 [QFileOpenEvent](https://doc.qt.io/qt-6.8/qfileopenevent.html)，只接受本地文件并复用应用的打开与激活路由。GUI 命令行 `--` 后作为文件路径，避免误解为诊断开关。待处理意图最多 128 个，不抢占活跃任务。同一登录会话的正常产品进程共用一个实例；多文件请求按顺序打开并激活最后一个，标签和窗口行为见[应用会话契约](application-sessions.md)。

启动参数在加载 QML 前入队，主窗口先隐藏，附接控制器时立即提交首个可用请求，不再要求窗口已可见。主窗口在初始事件队列中显示，允许平台启动事件走同一处理路径；外部打开期间先隐藏欢迎入口再改变会话，避免首帧或替换会话闪现欢迎页。取消／失败恢复欢迎页，成功显示真实列表，详见[读取状态](../ux/welcome-opening.md)。

macOS 原生启动单独等待 AppKit 的 NSApplicationDidFinishLaunchingNotification，再在隐藏状态处理该批排队的 Qt 文件事件，随后显示窗口。通知与排队回调均限定在主窗口生命周期，不替换 Qt 的应用 delegate，不使用固定毫秒延迟；offscreen 等非 Cocoa 后端仍走普通事件队列。实际 Launch Services 回归曾检出 1 帧欢迎正文，修正后要求为 0。check-macos-open.py 在本地与 Mac CI 执行 argv 和原生 open 两种路径，仅保存 JSON／日志，不生成或上传应用包。

## 自动验证与限制

三平台 CTest 检查启动前文件事件、非本地 URL 拒绝及读取忙时排队。显式 XDZIP_ASSOCIATION_PROBE 加隔离设置仅供 CI：通过正常打开流程读取自有十万条目 ZIP，保存计数／构建身份并正常关闭，不记录来源路径或执行解压。

当前关联探针等待至少一次实际帧呈现，逐帧记录欢迎正文可见次数；Windows ShellExecuteEx 和 Linux GIO 的当前产品要求 presented_frames 大于 0 且 welcome_frames 为 0。固定旧产品仍按历史探针能力验证，不伪造旧二进制具有新的首帧计数。

Windows 原生 CI 安装 QA 变体 1、升级 2、回退 1，再安装真正交付包；ShellExecuteEx 按自有注册类打开中文、空格、& 路径，检查默认及 UserChoice 不变、忙时拒绝替换、卸载保留设置／额外文件／其他候选。Linux 在无 SDK／系统 Qt／编译器、无网络容器执行相同 dpkg 生命周期，UID 1000 的产品由 GIO 桌面启动项实际打开十万条目，核对 MIME、默认、锁、卸载和 purge。

Linux 发行验证还在容器的独立用户配置中切换 MIME 默认项。GIO 将样本的内容类型解析为由 xdzip 打开时，用 `gio open` 验证启动。检查覆盖所有声明的 MIME 默认项，另用独立处理程序验证切换生效，结束时恢复原默认配置。Linux 和 macOS 的自动打开检查使用无需交互的样本。AES Crypt 的浏览器测试验证打开时提示密码及认证后的读取。系统设置界面和其他桌面环境需单独验证。


QA 变体共用本次核验二进制，只改变测试包版本／目录，验证安装机制，不作为发行工件，也不冒充真实不同产品版本的数据迁移。设置 schema 当前为 1，两平台安装元数据记录它，旧安装器拒绝覆盖声明更新 schema 的安装；未来迁移须同步契约。

后续故障注入只允许 QA 变体 2，在 postinst configure 阶段失败一次；正式产品禁止携带该分支。容器实际验证 half-configured 状态、运行门阻止启动、设置保持，再运行 dpkg --configure -a 恢复并经 GIO 打开真实文件。这是可重复配置失败，不冒充设备掉电。设置兼容检查另编译保留的原始设置组件与当前组件，往返同 schema 配置；原始输入逐文件固定哈希，不依赖旧 main 历史，不冒充完整安装器的数据迁移。

完整产品兼容另由 prepare-previous-product.py 在一次性 Windows/Linux CI 主机构建固定兼容快照（来源为原 4cf0f8c）的完整产品源码：使用 detached worktree 保留真实干净身份，使用当前固定 SDK 生成历史产品 QA 安装器，不下载易过期的旧 Actions 二进制。先安装历史产品并实际打开，再安装当前产品并检查 revision 改变、配置字节保留，再回退历史产品重新打开并检查数据。它证明两个完整源码提交在同 schema 的兼容，不冒充历史已签名原包、尚不存在的 schema 2 迁移或设备重启。历史安装器只作 QA，不上传为产品；回执绑定两个二进制与安装器 SHA-256。

windows-install-lifecycle.json、linux-install-lifecycle.json 及包哈希为实际证据。真实 Explorer/Finder 双击、默认选择 UI、重启后的服务／缓存、未正式签名提示、GPU 与桌面专项按人工清单核验。自动脚本不得自行把 INSTALL-01／ASSOC-01 全部标记通过；既有人工结论见 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)，不因重新生成 pending 模板而重开验收。未来 schema 迁移须另增实际兼容用例。

Windows 忙时替换保护的无任务探针使用独立配置关闭托盘，使 `CloseMainWindow` 按该配置执行正常退出；其他关联打开探针保留默认托盘配置并通过显式退出屏障结束。安装保护探针不得把“窗口隐藏”误判为“进程退出失败”，也不得通过强制终止正常路径跳过退出验证。

Windows 更新协议夹具与真实 Qt 更新夹具同样提供关联启动器和全部九份文档 ICO，不能通过缺少必需载荷的安装器检查。协议夹具的小型 C 主程序与关联命令只验证更新交接；嵌入应用资源、原生候选和真实窗口由完整 Qt 产品链分别检查，不混用结论。


## 兼容基线的长期输入

真实升级／回退使用固定的旧版完整产品源码，不从已重写的 main 历史获取。`prepare-previous-product.py` 锁定独立 `codex/installer-compat-baseline` 分支中的单根源码快照；该分支只保存构建和回归所需的原始源码及资源，不携带旧 main 父历史、执行报告或工作流。快照内 `COMPAT-BASELINE.json` 记录原始来源提交与树身份，不能合并回 main，也不作为发行包或当前产品的验收结论。

构建后 CLI 身份必须匹配快照自身的固定 SHA，回执另记原始来源身份，不能伪装成原发布二进制。设置组件另以三份原始小型源码作为持久测试输入，逐文件哈希固定，双向读写及未来 schema 拒写不依赖网络或历史 commit。不要随当前实现修改这些回归输入；需要新基线时另行评审并保留来源。

该冻结快照早于独立更新助手，不含助手源码或二进制。只有 `prepare-previous-product.py` 为它传入 `--qa-historical-revision`：部署／打包脚本要求一次性 CI、完整固定 SHA 和实际干净产品身份同时匹配；Linux 还要求历史版本模式，禁止与当前 QA 变体混用。Windows 直接执行旧产品的版本命令核对身份，Linux 使用独立部署审计的身份与完整文件哈希。回执明确记录历史源码不含助手，不注入当前助手，也不为不存在的 Linux 助手安装 polkit 授权。普通产品和当前 QA 变体仍必须交付并核验独立助手；已有 `--qa-legacy-version` 本身不允许缺少助手。

## Debian 版本迁移

首个公开预发布开始使用 Debian epoch 1，包版本为 `1:语义版本映射+git提交`，例如 `1:0.1.0~alpha.1+git...`。历史 `0.1.0~dev.9` 按 dpkg 规则会排在无 epoch 的 alpha.1 之后，不能直接沿用旧映射。epoch 只用于系统包排序，产品界面、版本标签和 release.json 仍使用原语义版本，后续 Debian 包保持 epoch 1，alpha、beta、rc、stable 的顺序继续由上游版本表达，规则见 [Debian Policy](https://www.debian.org/doc/debian-policy/ch-controlfields.html#version)。

真实历史产品夹具使用仅 CI 允许的 `--qa-legacy-version` 保留原无 epoch 身份，不能把旧包改成新排序来掩盖升级问题。当前包必须在实际 dpkg 升级、回退和安装生命周期验收中通过；用户配置与外部默认关联保持原契约。

## 设置中的默认应用

设置页按生成的格式数据对关联后缀分组，开关能否操作取决于平台服务和安装身份。Windows 交由系统“默认应用”处理，不写 UserChoice。MSIX 禁用开关，改用当前包的系统设置入口。macOS 通过 NSWorkspace 查询内容类型，可提示用户选择替代应用，Viewer／None 构建禁用关联变更。Linux 把支持的后缀映射到 MIME 类型，用 `xdg-mime` 管理用户默认设置，移除默认时可选择已安装的替代程序。多个后缀可能共用一个系统类型，设置页在操作后以系统结果为准。


自动识别语言模式在 Windows 安装器中按当前用户的系统界面语言解析：中文各区域使用中文描述，其余以英文兜底；显式中文或英文选择优先。安装器不将自动识别改写为固定语言，应用启动后按自身系统语言解析并同步自有描述。
