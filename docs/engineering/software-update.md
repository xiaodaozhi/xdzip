# 用户主动更新契约

关于页通过应用级 `UpdateController` 检查公开版本。关闭关于页不取消检查或下载，再次打开仍显示同一状态。检查只由用户主动触发，不在启动时轮询，不自动安装。产品窗口继续复用关于页和共享操作控件。

## 选择、网络与隐私

客户端只读取固定仓库 `xiaodaozhi/xdzip` 的公开 [Releases API](https://docs.github.com/en/rest/releases/releases)，不使用凭据，不上传文件名、归档内容、密码、最近记录或设置。GitHub 与其下载 CDN 会收到网络连接信息及包含应用版本的 User-Agent。点击发布页或下载目录使用系统打开接口。

每页最多 100 条，最多 10 页，每次响应最多 8 MiB，清单最多 256 KiB，签名最多 4 KiB。请求只允许 HTTPS 和明确登记的 GitHub/CDN 主机，重定向最多 5 次；元数据有 15 秒无数据超时和 30 秒总上限，下载有 30 秒无数据超时及一小时总上限。内存 ETag 缓存最多 24 项，304 只复用先前成功响应。离线、TLS、超时、限流、截断、缺包和清单错误均为失败，不等同于“已是最新”。重复点击不发起并行更新。

无 Qt Rust 核心按语义版本、渠道和单调 build 选择，验证固定仓库、完整六类包、架构、包型、最低系统契约和同候选身份。稳定版只接收稳定版；alpha 可接收 alpha、beta、rc、stable；beta 不接收 alpha；rc 只接收 rc、stable；dev 可检查公开版本。语义版本和 build 都必须递增，不按发布时间推断。客户端再检查当前系统支持范围和 Releases 资产字节数。

## 信任与下载

`release.json.sig` 验证清单原始字节，信任根编入客户端和安装助手，不能从下载内容或请求中增加。JSON 元数据进入字符串 FFI 前先验证 UTF-8，非法编码明确失败，不能通过替换字符改变待验签原字节。签名格式、独立 Sparkle 密钥、密钥交付顺序和不可变资产要求见[发布协议](release-protocol.md)。缺签名或未知公钥只提供官方发布页手动下载；错误签名明确失败。生产公钥尚未配置时不允许自动覆盖。

Windows/Linux 将最多 2 GiB 的包流式写入私有更新缓存，同时核对长度和 SHA-256，成功提交后才允许交给助手。取消或校验失败不产生可安装状态。便携包、dirty、开发/源码或身份不明的构建只提供检查与下载。自动覆盖还要求 Release 配置和有效提交身份，不把无 Git 身份的源码归档构建当作安装版。安装身份须同时匹配系统安装记录、运行路径及随包标记。

macOS 使用固定 Sparkle 2.10.0，通过薄 Objective-C++ 用户驱动把状态送入统一关于页。feed 和归档由 Sparkle 原生验证，feed 只包含已选定的同一个候选；不声称 Sparkle 执行了额外 SHA-256。只读 DMG、App Translocation、未登记的 .app 或缺少更新公钥均提供手动安装入口。普通 CI 不因引入 Sparkle 而部署、签名或上传 Mac 应用包。

## 安装交接与恢复

下载期间原有任务继续运行。选择下载安装后有活动任务则等待，用户可取消等待并暂缓更新，不隐式取消任务或密码请求。助手准备完成后 `prepareForUpdate` 拒绝活动任务、任务启动与模态交互；进入交接后禁止新会话和新任务，所有 Rust 连接完成清理及 join 才发送安装提交。交接不使用“立即重启旧程序”路径。

Windows 仅从登记的当前用户安装根复制独立无 Qt 助手到私有缓存，助手独立复验签名、大小和哈希，等待已捕获主进程退出后交给现有 Inno 安装器，保留交互许可确认。安装门阻止替换期间再次启动。

Ubuntu 安装版使用固定 `/usr/libexec/xdzip-update-helper`，普通用户协调器仅通过固定参数调用 polkit。root 助手将包复制到 root 私有暂存后重新验签及核对 xdzip 包名、架构、版本，只允许固定 APT 操作，不接受命令、任意包名或目标目录，不使用 sudo shell。授权取消与缺少认证代理返回独立状态。dpkg 未配置不报告成功，不自动回滚或启动旧程序。新版以原用户重启时保留有界白名单中的 XDG 设置、数据、缓存与状态目录及桌面语言，不继承动态加载器、Qt SDK 或测试环境覆盖；这些变量不传入 root 的安装命令。

macOS 在全部 join 完成后才释放 Sparkle 安装回调；Sparkle 负责归档替换和重启。取消不会留下下次退出时安装的计划。Framework、XPC 与 helper 单独保留所需签名参数和 entitlements，不能套用通用重签规则。

Sparkle 的取消回复在异步更新周期中处理，周期结束不等同于独立安装进程已经退出。未提交安装时，普通退出路径先请求取消，等待周期结束，再通过系统 `/bin/ps` 只读观察与本应用框架路径精确匹配的 Autoupdate 进程，确认其结束后才释放退出拦截。准备完成时先记录匹配的安装进程，观察失败不当作退出许可，不终止安装助手，也不记录无关进程信息。准备失败后的重试同样等待旧安装助手结束，不能替换尚未完成退出观察的更新驱动。已经提交的安装由 Sparkle 按原生流程退出和重启。

Windows/Linux 的事务使用随机 ID 和固定名称的 ready/proceed/cancel/result 文件。客户端在退出前把事务、目标身份和缓存目录保存到设置旁独立 `update-state.ini`，不混入设置草稿。助手先原子写成功结果，再释放门和重启。下次启动限制回执路径与大小，核对事务及当前版本/build/安装身份；失败、缺回执或身份不符显示恢复入口，不自动重试安装。恢复记录缺少有效候选地址时仍可打开固定的官方发布列表。缓存材料保留供用户修复，公开证据不得含私人完整路径。

macOS 在释放 Sparkle 安装前保存目标版本/build 的本地尝试记录。重开时以实际运行身份和可靠安装标记判断完成或需要恢复；此记录不赋予安装权限，也不代替 Sparkle 验签。

## 验证边界

核心夹具覆盖渠道、dev、乱序分页、缺包、错误签名、最低系统和身份冲突；网络夹具覆盖取消、重复点击、限流、TLS、响应上限、ETag 与损坏下载；会话回归核对任务保留和所有 join 后交接。真实三平台旧版到新版、授权取消、再次启动、安装失败/恢复、设置/关联保留以及无 SDK HTTPS 需要隔离平台实证，不能由单元测试或 CI 编译替代。Mac 完整更新必须在 Apple Silicon 原生环境执行。具体完成状态及证据由 issue 跟踪。

[专用原生更新回归](../../.github/workflows/update-regression.yml)仅手动调度，不携带生产秘密，不上传应用包。默认验证全部平台，也可选择单个平台定位故障，各选择使用独立队列；最终验收仍要求同提交的全平台结果。Windows 在一次性 GitHub runner、Linux 在专用容器先使用拥有的 C 协议产品和真实 Inno/APT 验证助手取消、失败及恢复。另一条私有 Qt 升级链路编译两个真实 `browser_main`/QML 版本，通过实际关于页鼠标输入，执行清单与包验证、退出 join、Inno/APT、普通用户单实例重启和回执恢复，并核对设置、未知设置键、现有默认关联及安装目录的无关文件保留。Linux 将构建 SDK 和运行容器分开，运行容器没有编译器或 Qt SDK，并使用自定义 XDG 目录；Windows Server CI 仅在私有宏下替换最低系统判断，不替代 Windows 11 原生验收。该链路替换公开 GitHub URL 的网络 transport，生产 HTTPS/TLS 则单独通过无 SDK 运行验证。

增强场景延迟包下载完成事件，启动真实 AES Check 直到等待密码，再验证更新进入等待任务。密码窗保持原生应用模态，此时夹具通过 controller 调用取消更新等待，并在证据中明确标记该步骤未使用关于页鼠标；取消后同任务及密码令牌保持，再通过真实输入法事件及密码确认按钮完成 Check，最后从关于页再次安装。普通退出取消、实体桌面和人工许可／授权仍需独立验收，不因空闲或增强升级通过而自动视为完成。

另有真实双会话组件回归：AES Check 等待密码时，实际退出窗口的“返回”保留原任务及密码令牌，“取消并退出”使同任务取消、密码请求清空，全部会话完成 Rust join 后只发出一次退出就绪信号，不触发更新交接。此回归不代替完整主程序在更新等待期间的普通退出及原生安装验收。

Mac 单独命名的私有 Qt 应用使用真实关于页、双会话 join、Sparkle、签名 app.zip 和新 build 重启，覆盖 feed/归档篡改和取消；仅候选发现使用 loopback 替身，公开 GitHub 清单与渠道另外由更新核心和网络回归验证。测试密钥只在私有副本中注入，生产资源不改。只有实际运行产出的证据能登记为通过。

Mac 另有完整产品链回归，以两份干净 Release 源码身份构建真实主程序，保留正式 bundle ID、安装标记和 Alternate 文档类型，安装到一次性 runner 的独有 Applications 路径。旧版通过真实关于页和更新控制器完成清单认证、Sparkle 安装及 join 交接，新版普通启动核对安装身份、更新回执、默认配置目录、设置和未知键、单实例，以及 LaunchServices 的 ZIP 默认处理器。私有网络接点仅映射固定元数据 URL、已认证 feed URL 和精确归档 URL；签名仍真实验证，无 SDK HTTPS 单独执行。该回归使用原用户 HOME 和默认配置路径，不以测试路径注入代替重启恢复。

The dedicated native workflow defaults to all platforms and also supports a separate queue for a selected diagnostic platform; final acceptance still requires all platforms on the same commit. The Windows/Linux regression additionally builds two private real Qt products, clicks About, runs native installers and verifies ordinary single-instance restart, receipt recovery and preserved settings/associations. Its enhanced scenario starts a real AES Check waiting for a password, cancels update waiting while preserving that task/token, enters the password through an input-method event and clicks confirmation before upgrading. Cancellation is explicitly a controller call because the real application-modal password window blocks About pointer input. A separate two-session component regression clicks the actual quit window: returning preserves the waiting AES Check and password token, while cancelling and quitting cancels that same task, clears its prompt and emits readiness exactly once after every Rust join, without update handoff. This does not replace full-main-program quit during update waiting or native installation acceptance. Fixture transport retains production signature/hash policies; SDK-free HTTPS is tested separately. Linux uses custom XDG directories in a runtime without build tools or the Qt SDK. The private Windows Server minimum-system override does not validate Windows 11 acceptance. Manual desktop/license/authorization acceptance remains separate; only executed native results count as passed.

A separate macOS product regression builds two clean Release identities with the real main program, formal bundle ID, installation markers and Alternate document types in an owned Applications path on a disposable runner. Actual About input and UpdateController authentication precede native Sparkle installation and the join barrier. Ordinary new startup verifies installed identity, update receipt restoration, the default profile, persistent and unknown settings, single-instance behavior and LaunchServices ZIP defaults. Private transport maps fixed metadata URLs, the authenticated feed URL and the exact archive URL while retaining real signature validation; SDK-free HTTPS remains separate. The original user HOME and default settings path are used without a test profile override.

Sparkle cancellation is asynchronous, and finishing its update cycle does not prove that its separate installer has exited. Before an uncommitted normal quit, the application requests cancellation, waits for the cycle, then uses the system `/bin/ps` read-only interface to confirm that the Autoupdate process matching its own exact framework path has ended. The matching installer is observed at readiness. Inspection errors never grant exit permission, no installer is killed, and unrelated process information is not logged. Retries after preparation errors also wait for the old installer to exit before replacing its update driver. Committed installation keeps Sparkle's native quit and relaunch flow.

English: Updates are user-initiated from About, with app-wide state and explicit cancellation. Fixed public GitHub endpoints receive connection metadata and the application version, never archive data, paths, passwords or settings. The Qt-independent core validates immutable six-target manifests, semantic version/channel progression, monotonic builds and exact-byte Ed25519 signatures. Invalid UTF-8 metadata fails before string FFI without rewriting authenticated bytes. Portable, dirty, development or unknown installations remain manual; automatic replacement also requires Release configuration and a valid source revision. Windows/Linux stream-check size/SHA-256 and independently reverify in standalone helpers. Sparkle 2.10.0 validates the signed single-candidate feed and archive on Apple Silicon. Installation waits for tasks and every Rust join; no new work enters after handoff. Fixed privilege boundaries, installation gates and transaction receipts prevent false success or blind restarts. Linux relaunch preserves bounded XDG profile directories and desktop locale only for the ordinary user, excluding loader, SDK and test overrides. Manual recovery falls back to the fixed official releases list when candidate metadata is unavailable. Production trust roots must be provisioned before release. Native isolated upgrade and SDK-free HTTPS evidence remain distinct from automated component checks.
