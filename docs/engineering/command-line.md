# 命令行运行契约

CLI 是独立的 Rust 宿主，通过同一个 `xdzip-runtime` 服务调用格式和安全 I/O，不依赖 Qt、QML 或显示会话。7z、RAR、CAB、ISO、UDF 继续使用随包提供的隔离工作进程，不搜索 PATH 中的替代解包工具。平台和架构沿用[支持政策](support-policy.md)，独立 Linux CLI 不要求 X11；运行需要系统 OpenSSL 3 和证书，构建另需 libssl-dev、pkg-config。

自 `v0.2.0` 起，正式发行提供三平台独立 CLI 包，桌面包也包含私有 CLI；更早版本的附件不会因此改变。下载入口见[发布页](https://github.com/xiaodaozhi/xdzip/releases)，源码构建、安装与更新边界见下文及[发布协议](release-protocol.md)。具体实现与验收进度登记在 [#254](https://github.com/xiaodaozhi/xdzip/issues/254)。

## 使用与输出

```sh
xdzip --help
xdzip version
xdzip list archive.zip --json
xdzip extract archive.zip docs/ -C output
xdzip extract archive.zip --entry-id 3 -C output
xdzip create backup.7z docs/ --level normal
xdzip create private.zip docs/ --encrypt --password-stdin
xdzip add backup.zip notes.txt --into docs
xdzip test backup.zip
xdzip update --check
xdzip update --download --output downloads
xdzip update --yes
xdzip update --status
xdzip gui archive.zip
```

不带参数显示帮助。`version`、`--version` 和 `--version-json` 使用 VERSION.json 与构建时 Git 身份；旧版版本字符串及 JSON 的 product、version、build、revision、dirty 字段保留，CLI 额外说明 component、channel、configuration、distribution、platform、architecture。`--language zh-CN|en-US` 选择帮助和结果语言，默认依据桌面语言环境变量；公共标识与错误码为英文。`l`、`x`、`c`、`a`、`t` 分别是 list、extract、create、add、test 的别名。`--` 之后作为字面文件参数。

所有业务命令接受 `--json`。stdout 只输出一个 JSON 对象，字段固定为 schema_version（1）、command、status、error_code、data、summary、warnings。未知大小使用 null。列表 data 包含 format 和 entries；每条保留 id、path、display_name、size、packed_size、method、directory、link、payload_supported、encrypted、warning。path 是冻结的有效名称，异常或未解析名称为 null；display_name 只用于转义展示。任务 summary 包含实际读取、解码、校验、提交、跳过、目录与分卷计数，并保留清理警告及不确定结果。

进度、错误和交互提示写入 stderr。JSON、非终端或 `--non-interactive` 禁止询问，缺少密码或冲突决定时取消等待中的任务，等待清理后返回 interaction_required。退出码为 0（成功或明确交接）、1（失败、部分完成或清理警告）、2（参数错误）、3（缺少交互、授权或安装条件）、130（已确认取消）。显式 skip 是已执行的策略，跳过数量仍单独报告。取消未被接纳且提交已完成时保留真实完成结果。

## 归档行为

list 返回物理记录，重名不合并。extract 的成员参数是包内完整、精确、区分大小写的有效路径，使用正斜杠，目录递归展开并保留根目录以下层级。精确重名返回 ambiguous_member，使用 --entry-id 选择物理记录。任何未匹配、外来 ID 或歧义先于目标目录创建失败。首版不解释通配符，不支持将归档从 stdin 读取或将解压内容发送到 stdout。

大包列举使用 `list archive.zip --offset 999872 --limit 128 --json`，offset 从 0 开始，limit 为 1～4096；只指定 offset 时默认 128。JSON data 同时返回 offset、total 和可为空的 next_offset，summary.entries 是本次返回数。普通单文件 ZIP/ZIP64 可索引一百万条；未指定分页时整体列举最多十万条，JSON 行与文本累计最多 16MiB，超限返回 resource_limit，不输出被误认为完整清单的前缀。分页不改变物理 ID，详见[大索引](large-index.md)。English: Explicit list pages use zero-based `--offset` and `--limit 1..4096` (128 by default when offset is supplied), returning total/next_offset. Unpaged responses are bounded to 100,000 records and 16MiB of serialized rows plus text; oversized results fail atomically. Million-record ZIP indexing and stable physical IDs remain available through bounded pages.

extract 默认输出到当前目录，`-C` 指定缺失目录时，从最近已存在的安全父目录逐级获得授权，不使用无条件递归建目录。目标、源身份、链接、Unicode／大小写碰撞、暂存、校验及提交继续遵守[安全解压](m3-extraction.md)和[文件名编码](filename-encoding.md)。默认询问重名；`--conflict ask|skip|rename|overwrite` 显式选择策略。`--allow-high-ratio` 只确认高压缩比，不取消硬资源限制。

create 依据 .zip、.tar、.tar.gz、.tgz、.7z 扩展名选择现有后端，目标父目录须已存在。`--level store|fast|normal|best` 对应现有四档级别，默认为 normal；TAR 仅打包。`--encrypt` 仅用于 ZIP／7z，7z 另可使用 --encrypt-names、--solid；--volume-size 接受正整数字节及 KiB、MiB、GiB 后缀，分卷只用于 ZIP／7z。格式不适用的参数明确拒绝。

密码只通过终端隐藏输入或 `--password-stdin` 的一行 UTF-8 输入传递，移除行结束符但不裁剪密码本身。创建交互加密包时确认两次。拒绝明文密码参数，不读取密码环境变量，不写设置或日志。密码长度与归零规则沿用[密码契约](password-extraction.md)和[创建契约](m4-creation.md)。

add 默认加入包内根目录，--into 只选择已有安全目录。加密、分卷、RAR、CAB、ISO、UDF 及其他不满足编辑条件的包继续拒绝。普通包重建和替换、7z 工作集限制、macOS 元数据过滤与输入保护复用[归档修改](archive-transfers.md)。test 检查整包而不输出文件，TAR、ISO、UDF 只保证结构与可读性，其他格式只保证其已有校验能力，不将未支持能力称为通过。

每次 CLI 操作拥有独立运行时，不加入桌面浏览会话。Ctrl+C／终止信号请求协作取消，等待任务终态、清理和运行时 join；先前已提交的文件保留。共享服务的 submit_wait 仅串行获得短状态锁，不等待队列容量，不改变 GUI 的非阻塞准入。Catalog 每页最多 128 个物理条目，SelectMembers 的数量和名称预算沿用既有上限，所有请求绑定不可变会话身份。

## 构建与命令安装

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
python3 scripts/cli.py package --output build/package/cli
build/cli/bin/xdzip --help
```

该入口使用仓库锁定的 Rust、CMake、平台 C++ 工具链及现有隔离 worker，无需安装 Qt。Windows 打包还需通过既有 prepare-windows-runtime.py 准备固定、附带原许可的 app-local VC++ 运行库。Qt 的统一构建入口也构建 CLI 并放入私有 cli 目录，GUI 可执行文件、应用图标与关联入口不改名。

macOS 的内置 CLI 可执行文件位于 `Contents/Helpers/cli/`，普通上下文 JSON 位于 `Contents/Resources/xdzip-cli-context.json`，由应用资源签名封装。CLI 只在固定的 Contents／Helpers／cli 布局查找该资源，继续核对 schema、desktop kind 与固定 GUI 相对路径；缺少标记按便携身份处理，非法标记拒绝。重复构建清除 Helpers 下的旧生成标记，不能通过放宽签名验证解决布局错误。English: Bundled macOS CLI executables stay under Helpers/cli, while their context JSON is sealed as a Resources file. Lookup requires the fixed bundle layout and existing identity/target checks; missing context remains portable and invalid context fails. Rebuilding removes the obsolete generated marker from Helpers without relaxing signature checks.

独立包展开后运行 bin/xdzip，Windows 加 .exe。运行 `bin/xdzip setup` 显式安装用户级稳定入口及版本化运行目录。Unix 默认入口为 ~/.local/bin，Windows 默认位于用户 LocalAppData 下的 xdzip/cli/bin；管理目录使用平台用户数据位置。--bin-dir 和 --install-dir 可显式指定位置。用户自行将目录加入 PATH，工具不修改 Shell 启动文件。已有入口或登记文件一律拒绝覆盖。

管理式身份要求有效的 installation.json；目录恰好名为 versions 而没有登记文件时仍视为便携运行。English: Managed identity requires its installation marker; a directory named versions alone remains portable.

Windows 撤销正在运行的启动器时，将哈希仍匹配的自有副本移到同一命令目录的私有 .xdzip-retired 子目录，入口立即撤销，结果记录 retained_launcher。后续显式 setup 只按有界回执回收哈希仍匹配且已退出的副本，修改过的文件保留并拒绝覆盖；保留版本目录中的 CLI 可重新执行 setup 恢复入口。English: Windows may retain an active, hash-verified launcher in a private sibling directory. A later explicit setup reaps only recorded, unchanged and unlocked copies; the retained versioned CLI can restore registration.

版本身份的 distribution 区分 source 与 release。普通源码构建即使使用 Release 配置并保持干净，也只保留检查与下载；只有专用 Release 管线标记的发行身份、有效管理式安装和受信目标签名共同满足时才自动替换。English: Clean Release-mode source builds remain check/download-only; automatic replacement additionally requires release distribution identity, a managed installation and an authenticated target.

桌面安装器的 Windows PATH 选项默认不勾选，并拒绝与其他 xdzip 命令冲突。Mac/Linux 可运行桌面包私有 CLI 的 setup；它只登记桌面安装内的 CLI，不复制出独立可升级核心。`setup --remove` 只删除经哈希确认、属于本工具的入口和登记文件。版本目录及恢复资料保留，不删除用户输入或仍可能被进程使用的运行目录。

稳定启动器不经过 Shell 转义，在 Unix exec 实际 CLI，在 Windows 等待控制台子进程并转发取消及退出码。当前运行目录由原子 current.json 指针选择；初次安装、新版自检或文件提交失败不能产生已安装成功的结果。

## 更新、桌面交接与恢复

update 针对当前安装，独立 CLI 只升级 CLI，桌面包内的 CLI 更新整套桌面版。--check 查询，--download 下载并验证，默认确认后安装，--status 核对回执和实际身份。--yes 只确认本次更新，不替代系统授权。便携、dirty、源码或身份不明构建只允许检查和下载；独立自动替换要求管理式、干净 Release 安装。

CLI 使用固定公开 GitHub Releases、渠道与单调 build 规则。CLI 三平台资产使用单独的 cli-release.json 与 cli-release.json.sig，内置公钥验证原始字节，再检查大小与 SHA-256。缺签名或未受信密钥只提供人工获取信息，错误签名明确失败。桌面六包 release.json 不加入 CLI 条目，旧客户端继续使用原协议。

网络使用系统证书，默认读取标准代理环境变量，--proxy 选择无凭据 HTTP／SOCKS5，--no-proxy 直连，--mirror 选择无凭据、无查询和片段的 HTTPS 前缀。原始 GitHub 地址、重定向主机、响应大小、超时和下载预算继续有界。网络、TLS、离线、限流与无兼容新版本分别报告，不自动轮询或重试。

独立更新在用户拥有的私有目录暂存，以同一归档运行时安全解包，验证完整固定文件清单及每文件摘要，核对产品、组件、本机平台架构和签名清单渠道，运行新版 --version-json 自检后才切换版本指针。事务互斥，回执绑定目标身份；失败保持原指针或明确进入恢复状态，不伪造自动回滚。English: Payload and self-check identities must match the CLI product/component, native host target and signed channel before activation.

桌面检查与下载只查询主程序的无 GUI 安装身份入口，不创建 QApplication 或消费安装回执。安装意图通过原有同用户单实例通道传递，只接受有界请求目录和候选版本；不接受任意命令、包文件或信任根。请求／结果文件复用 Rust 原生目录授权与安全提交。桌面控制器独立重新检查并核对候选，复用关于页、安装器、Sparkle 与全部任务 join 屏障。桌面 CLI 任务也持有安装占用保护，安装不能破坏它们。

终端收到 accepted 仅报告 handed_off。实际安装与恢复由桌面回执和实际运行身份判断，不能把下载或交接记为 installed。无交互模式拒绝需要图形界面或系统授权的桌面安装。安装安全及原生验收边界见[更新契约](software-update.md)。

## English contract

The Rust CLI runs without Qt or a display session and reuses the archive runtime, safe I/O and bundled isolated worker. Supported OS/architecture and format boundaries remain unchanged; standalone Linux requires system OpenSSL 3 and CA certificates, not X11. Current published releases do not retroactively acquire CLI assets.

No arguments prints help. Commands are version, list/l, extract/x, create/c, add/a, test/t, update, gui and setup. --language selects zh-CN/en-US, while public identifiers stay English. --version and --version-json preserve the existing identity fields. --json emits one schema-v1 object containing command, status, error_code, data, summary and warnings. Progress and prompts use stderr. JSON, nonterminal and --non-interactive calls never prompt. Exit codes are 0 success/explicit handoff, 1 failure/partial result/cleanup warning, 2 usage, 3 missing interaction/authority/installation conditions, and 130 confirmed cancellation.

Member selectors use exact, case-sensitive effective archive paths. Duplicate names require --entry-id; missing or ambiguous selections fail before destination creation. Extraction preserves hierarchy, defaults to the current directory and safely authorizes missing -C components. Conflict policy is ask/skip/rename/overwrite; high-ratio confirmation never removes hard limits. No archive stdin/stdout streams or wildcard expansion are added. Creation infers the existing format from the suffix, defaults to normal compression and requires an existing parent. Existing encryption, solid, split, edit restrictions, input filtering and source protection apply. Passwords use hidden terminal input or one UTF-8 stdin line, never argv, environment, logs or settings. Interactive creation confirms twice. Tests report only the format's supported verification guarantees. Cancellation joins the runtime after cleanup and preserves prior commits.

scripts/cli.py builds, tests and packages without Qt. Desktop builds include a private CLI while retaining GUI filenames and associations. setup explicitly registers a user-owned command; --bin-dir/--install-dir override defaults. Existing entries are preserved, Shell startup files are not edited, and Windows installer PATH integration is opt-in. Removing registration verifies its ownership/hash and retains payload/recovery directories. The stable launcher selects a version through an atomic pointer, preserving cancellation and exit status.

Updates target the current installation. Standalone CLI releases use a separate, exact-byte signed three-target manifest, without changing legacy desktop six-target metadata. System TLS trust, credential-free proxy/mirror options, bounded responses/redirects/downloads, channel progression, monotonic builds and hash verification apply. Managed clean Release installations validate and self-check a complete staged payload before pointer replacement. Other builds remain check/download-only. Network failures never become up-to-date results.

Desktop metadata queries require no GUI and do not consume receipts. Installation uses bounded same-user IPC, independently revalidates the candidate and retains the existing task, installer and Sparkle barriers, including CLI occupancy. accepted means handed_off, not installed. Actual receipts and running identities determine success/recovery. --yes cannot supply system authority, and noninteractive desktop installation refuses required UI. Native, automated and manual acceptance remain distinct.
