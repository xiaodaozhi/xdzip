# 命令行运行契约

[English](../../engineering/command-line.md) | 简体中文 | [繁體中文](../../zh-TW/engineering/command-line.md) | [Français](../../fr-FR/engineering/command-line.md) | [Español](../../es-ES/engineering/command-line.md) | [Português (Brasil)](../../pt-BR/engineering/command-line.md) | [Русский](../../ru-RU/engineering/command-line.md)

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
xdzip create data.iso docs/
xdzip create portable.sfx.exe docs/ --level normal
xdzip create private.zip docs/ --encrypt --password-stdin
xdzip add backup.zip notes.txt --into docs
xdzip test backup.zip
xdzip update --check
xdzip update --download --output downloads
xdzip update --yes
xdzip update --status
xdzip gui archive.zip
```

不带参数或使用 `--help` 时显示帮助。`version`、`--version` 和 `--version-json` 根据 `VERSION.json` 及构建时的 Git 状态报告版本身份。`--version-json` 返回独立对象，包含 product、version、build、revision、dirty、component、channel、configuration、distribution、platform 和 architecture。可用 `--language zh-CN|en-US` 选择帮助和消息的语言。默认按 `LC_ALL`、`LC_MESSAGES`、`LANG` 的顺序取值，所用值以 `zh` 开头时使用中文。公开标识和错误码使用英文。`l`、`x`、`c`、`a`、`t` 分别是 `list`、`extract`、`create`、`add`、`test` 的别名。用 `--` 传递字面文件参数。

所有业务命令接受 `--json`。stdout 只输出一个 JSON 对象，字段固定为 schema_version（1）、command、status、error_code、data、summary、warnings。未知大小使用 null，包括共享压缩 TAR 流的子项；原始 DEB ar 成员保留精确压缩尺寸。列表 data 包含 format 和 entries；每条保留 id、path、display_name、size、packed_size、method、directory、link、payload_supported、encrypted、warning。path 是冻结的有效名称，异常或未解析名称为 null；display_name 只用于转义展示。任务 summary 包含实际读取、解码、校验、提交、跳过、目录与分卷计数，并保留清理警告及不确定结果。

进度、错误和交互提示写入 stderr。JSON、非终端或 `--non-interactive` 禁止询问，缺少密码或冲突决定时取消等待中的任务，等待清理后返回 interaction_required。退出码为 0（成功或明确交接）、1（失败、部分完成或清理警告）、2（参数错误）、3（缺少交互、授权或安装条件）、130（已确认取消）。显式 skip 是已执行的策略，跳过数量仍单独报告。取消未被接纳且提交已完成时保留真实完成结果。

## 归档行为

`list` 返回物理记录并保留重名条目。`extract` 接受完整、精确、区分大小写的有效包内路径，使用正斜杠。目录递归展开，保留根目录下的层级。精确重名返回 `ambiguous_member`。可用 `--entry-id` 传入 `list` 返回的数字 ID（从 1 开始），选择多个 ID 时重复该参数。无法解析指定路径或 ID 时，会在创建目标目录前失败。当前不支持通配符、从 stdin 读取整个归档，或将解压内容写到 stdout。

指定 `--limit` 或 `--offset` 均启用分页。偏移量从 0 开始，最大为 1,000,000。只指定 `--limit` 时从 0 开始，只指定 `--offset` 时每页最多返回 128 条。`--limit` 的取值为 1～4,096。JSON 的 `data` 包含 `offset`、`total` 和可为空的 `next_offset`，`summary.entries` 是实际返回的行数。普通单文件 ZIP／ZIP64 可索引一百万条，未分页的列举在总量超过十万条时失败。两种模式下，序列化后的 JSON 行与格式化文本合计均限 16 MiB。超限返回 `resource_limit`，不返回部分结果。分页不改变物理 ID，详见[大归档索引](large-index.md)。

extract 默认输出到当前目录，`-C` 指定缺失目录时，从最近已存在的安全父目录逐级获得授权，不使用无条件递归建目录。目标、源身份、链接、Unicode／大小写碰撞、暂存、校验及提交继续遵守[安全解压](zip-safe-extraction.md)和[文件名编码](filename-encoding.md)。默认询问重名；`--conflict ask|skip|rename|overwrite` 显式选择策略。`--allow-high-ratio` 只确认高压缩比，不取消硬资源限制。

`create` 按输出[后缀](format-registry.md)选择创建配置，要求目标父目录已经存在。`--level store|fast|normal|best` 按所选配置校验。普通 TAR 和 ISO 可省略级别，或显式使用 `store`。LHA（`.lzh` 或 `.lha`）的 `fast`、`normal`、`best` 对应 LH5 的 1、6、9 级。ZIP 和 7z 支持 `--encrypt`、`--volume-size`。7z 还可按配置使用 `--encrypt-names`、`--solid`。分卷大小接受正整数字节数，或带 `KiB`、`MiB`、`GiB` 后缀的值。不适用于所选配置的参数会在写入前被拒绝。

密码只通过终端隐藏输入或 `--password-stdin` 的一行 UTF-8 输入传递，移除行结束符但不裁剪密码本身。创建交互加密包时确认两次。拒绝明文密码参数，不读取密码环境变量，不写设置或日志。密码长度与归零规则沿用[密码契约](password-extraction.md)和[创建契约](zip-creation.md)。

`add` 提交宿主更新，默认写入包内根目录。`--into` 按包内相对名称选择已有的安全目录。符合条件的输入可添加或替换条目，包内移动是桌面操作。CLI 复用[归档修改](archive-transfers.md)的格式与记录限制，以及输入保护、临时重建和最终替换流程。`test` 检查整包，不写出归档条目。检查成功后，JSON 的 `data.check_kind` 报告实际检查强度，见[完整性检查](archive-checking.md)。

每次 CLI 操作拥有独立运行时，不加入桌面浏览会话。Ctrl+C／终止信号请求协作取消，等待任务终态、清理和运行时 join；先前已提交的文件保留。共享服务的 submit_wait 仅串行获得短状态锁，不等待队列容量，不改变 GUI 的非阻塞准入。Catalog 每页最多 128 个物理条目，SelectMembers 的数量和名称预算沿用既有上限，所有请求绑定不可变会话身份。

## 构建与命令安装

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
python3 scripts/cli.py package --output build/package/cli
build/cli/bin/xdzip --help
```

该入口使用仓库锁定的 Rust、CMake、平台 C++ 工具链及现有隔离 worker，无需安装 Qt。Windows 打包还需通过既有 prepare-windows-runtime.py 准备固定、附带原许可的 app-local VC++ 运行库。Qt 的统一构建入口也构建 CLI 并放入私有 cli 目录，GUI 可执行文件、应用图标与关联入口不改名。

macOS 的内置 CLI 可执行文件位于 `Contents/Helpers/cli/`，普通上下文 JSON 位于 `Contents/Resources/xdzip-cli-context.json`，由应用资源签名封装。CLI 只在固定的 Contents／Helpers／cli 布局查找该资源，继续核对 schema、desktop kind 与固定 GUI 相对路径；缺少标记按便携身份处理，非法标记拒绝。重复构建清除 Helpers 下的旧生成标记，不能通过放宽签名验证解决布局错误。

新的桌面包在上下文中增加 `worker_relative`，内置 CLI 与桌面共享一个格式工作进程。只接受固定平台相对路径：Windows 为桌面旁的工作进程，macOS 为 `Contents/MacOS/xdzip-native-codec`，Linux 为构建目录旁或部署目录 `bin/` 中的工作进程。目标缺失、为链接或路径不符时拒绝上下文，不搜索 PATH 或其他副本。旧上下文没有该字段时，以及独立或管理式 CLI 包，保留原有的同目录查找。显式开发环境覆盖仍优先。使用 `scripts/check-cli.py --default-worker` 验证部署后的实际查找，普通测试路径覆盖不能替代该检查。

独立包展开后运行 bin/xdzip，Windows 加 .exe。运行 `bin/xdzip setup` 显式安装用户级稳定入口及版本化运行目录。Unix 默认入口为 ~/.local/bin，Windows 默认位于用户 LocalAppData 下的 xdzip/cli/bin；管理目录使用平台用户数据位置。--bin-dir 和 --install-dir 可显式指定位置。用户自行将目录加入 PATH，工具不修改 Shell 启动文件。已有入口或登记文件一律拒绝覆盖。

管理式身份要求有效的 installation.json；目录恰好名为 versions 而没有登记文件时仍视为便携运行。

Windows 撤销正在运行的启动器时，将哈希仍匹配的自有副本移到同一命令目录的私有 .xdzip-retired 子目录，入口立即撤销，结果记录 retained_launcher。后续显式 setup 只按有界回执回收哈希仍匹配且已退出的副本，修改过的文件保留并拒绝覆盖；保留版本目录中的 CLI 可重新执行 setup 恢复入口。

版本身份的 distribution 区分 source 与 release。普通源码构建即使使用 Release 配置并保持干净，也只保留检查与下载；只有专用 Release 管线标记的发行身份、有效管理式安装和受信目标签名共同满足时才自动替换。

桌面安装器的 Windows PATH 选项默认不勾选，并拒绝与其他 xdzip 命令冲突。Mac/Linux 可运行桌面包私有 CLI 的 setup；它只登记桌面安装内的 CLI，不复制出独立可升级核心。`setup --remove` 只删除经哈希确认、属于本工具的入口和登记文件。版本目录及恢复资料保留，不删除用户输入或仍可能被进程使用的运行目录。

稳定启动器不经过 Shell 转义，在 Unix exec 实际 CLI，在 Windows 等待控制台子进程并转发取消及退出码。当前运行目录由原子 current.json 指针选择；初次安装、新版自检或文件提交失败不能产生已安装成功的结果。

## 更新、桌面交接与恢复

update 针对当前安装，独立 CLI 只升级 CLI，桌面包内的 CLI 更新整套桌面版。--check 查询，--download 下载并验证，默认确认后安装，--status 核对回执和实际身份。--yes 只确认本次更新，不替代系统授权。便携、dirty、源码或身份不明构建只允许检查和下载；独立自动替换要求管理式、干净 Release 安装。

CLI 使用固定公开 GitHub Releases、渠道与单调 build 规则。CLI 三平台资产使用单独的 cli-release.json 与 cli-release.json.sig，内置公钥验证原始字节，再检查大小与 SHA-256。缺签名或未受信密钥只提供人工获取信息，错误签名明确失败。桌面六包 release.json 不加入 CLI 条目，旧客户端继续使用原协议。

网络使用系统证书，默认读取标准代理环境变量，--proxy 选择无凭据 HTTP／SOCKS5，--no-proxy 直连，--mirror 选择无凭据、无查询和片段的 HTTPS 前缀。原始 GitHub 地址、重定向主机、响应大小、超时和下载预算继续有界。网络、TLS、离线、限流与无兼容新版本分别报告，不自动轮询或重试。

独立更新在用户拥有的私有目录暂存，以同一归档运行时安全解包，验证完整固定文件清单及每文件摘要，核对产品、组件、本机平台架构和签名清单渠道，运行新版 --version-json 自检后才切换版本指针。事务互斥，回执绑定目标身份；失败保持原指针或明确进入恢复状态，不伪造自动回滚。

桌面检查与下载只查询主程序的无 GUI 安装身份入口，不创建 QApplication 或消费安装回执。安装意图通过原有同用户单实例通道传递，只接受有界请求目录和候选版本；不接受任意命令、包文件或信任根。请求／结果文件复用 Rust 原生目录授权与安全提交。桌面控制器独立重新检查并核对候选，复用关于页、安装器、Sparkle 与全部任务 join 屏障。桌面 CLI 任务也持有安装占用保护，安装不能破坏它们。

终端收到 accepted 仅报告 handed_off。实际安装与恢复由桌面回执和实际运行身份判断，不能把下载或交接记为 installed。无交互模式拒绝需要图形界面或系统授权的桌面安装。安装安全及原生验收边界见[更新契约](software-update.md)。

`.iso` 配置以固定的 0 级创建无压缩 ISO9660/Joliet 数据映像，可省略 `--level` 或使用 `--level store`。Windows x64 的 `.sfx.exe` 配置使用随包部署的相邻 `xdzip-sfx.exe` 封装 Store 或 Deflate ZIP，拒绝加密、固实和分卷。两种配置均不支持修改现有归档。详见 [ISO 创建](iso-creation.md)与[自有 SFX 创建](sfx-creation.md)。
