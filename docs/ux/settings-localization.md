# 设置窗口与界面语言

设置使用非模态独立系统窗口，标题栏、应用菜单和托盘复用同一个可见窗口。左侧依次为外观、通用、隐私、文件关联、网络和关于；所有关于入口打开同一设置窗口的关于页。窗口逻辑宽度为 612，客户区高度为 532，屏幕空间不足时限制高度，侧栏和正文分别滚动。侧栏与正文相接，顶部不留额外空隙。窗口没有 footer、保存或取消按钮。设计源和各状态图见[设置设计](../../design/hifi/sources/settings.html)，尺寸令牌见[布局规范](../visual/layout-scale.json)。

主题、字号、列表密度、语言、打开方式、托盘和最近记录开关在操作时写入并生效；关闭窗口不会撤销已成功的更改。语言即时刷新产品文字和应用菜单，运行中的任务不受影响。打开方式影响后续所有打开入口和标题栏加号，不迁移已打开的会话。清空最近记录立即执行，不删除原文件。清空失败与设置写入失败均在页面显示错误，保留既有数据和实际设置。多选框基于 IntentCheckBox，主题和代理单选基于 IntentRadioButton；鼠标、键盘及原生辅助功能 press/toggle 必须提交同一个选择意图，并核对模型，不得只更改图形。下拉框继续使用 OperationSelect，选择确定后提交一次。

设置持久化使用 schema 1，保留未知键，包含网络代理和镜像地址，不存放密码。保存持有独立配置锁，先在同目录暂存完整 INI，原子更新备份及主文件。读取失败、超过 1 MiB、非法或更高 schema、符号链接、备份或提交失败均拒绝覆盖原文件。设置应用失败时恢复运行中状态。最近路径独立保存在同目录 recent.json，最多 20 条、去重并限制读取为 64 KiB，文件仅用户可读写；关闭记录开关停止新增，清空入口删除已有记录。

文件关联显示系统当前默认应用。macOS 通过系统默认应用接口请求更改，取消或失败后重新查询，不假定用户已同意；关闭关联时由用户选择其他应用。开发副本不注册可选处理器，故不提供修改。Windows 打开系统的默认应用设置，由用户确认选择，不写入受保护的 UserChoice。Linux 使用桌面环境的 MIME 设置，关闭时由用户选择已安装的其他桌面应用。`.tar.gz` 与 `.tgz` 可共享同一系统类型，关联变化可能同时影响两项；Windows 按最终 `.gz` 扩展名关联。系统拒绝、未安装或用户取消时，界面以实际查询结果为准。安装和卸载本身仍不擅自修改默认选择，详见[安装与关联](../engineering/installers-associations.md)。

网络提供不使用代理、系统代理、自定义 HTTP 或 SOCKS5 代理，及可选 HTTPS GitHub 镜像地址。自定义代理必须包含主机和端口，不接收凭据；地址输入完成并验证后保存。镜像为空时直接访问官方地址；非空时仅改变更新流量的传输地址，校验和签名边界不变。进行中的更新保持原网络配置，下一次更新操作使用新配置。macOS Sparkle 只能使用系统代理，选择其他代理时提供经过验证的手动下载，不声称自动安装使用了该代理。完整信任和重定向策略见[软件更新](../engineering/software-update.md)。

所有产品静态文字通过 `resources/i18n/zh-CN.json` 和 `en-US.json` 的稳定 ID 引用；QML 使用 `qsTrId`，C++ 使用 `qtTrId`。协议码、文件名、路径和用户内容不翻译。语言默认简体中文，支持 English (US)；字号支持 100%、125%、150%，历史 200% 读取为 150%。系统文件选择器遵循系统语言，独立于应用语言。macOS 包声明 zh-Hans、en 并允许混合本地化，Qt 标准控件按系统 locale 加载可用翻译，不修改用户的 AppleLanguages。

验收覆盖六个标签、无 footer、各入口及跨会话关于页、即时保存与重开、保存失败回滚、语言即时切换、主题和字号、九类关联的系统确认与刷新、代理与镜像输入、更新各状态、两种语言及 150% 字号。必须通过 QAccessibleActionInterface 实际操作单选和开关，检查模型、互斥及事件；HTML 与组件测试不替代各平台真实桌面和系统默认应用的人工验收。测试截图和证据仅写 build/ 或 CI 工件。

English: The modeless 612 by 532 settings window contains Appearance, General, Privacy, File Associations, Network, and About tabs. Changes are applied and persisted on selection, with errors preserving the previous state. About and updates share this window. File associations reflect the operating system and require its confirmation where applicable. HTTP/SOCKS5 proxy and optional HTTPS GitHub mirror affect update transport only; signatures and verification remain mandatory. Existing transfers keep their configuration until completion. macOS non-system proxies use a verified manual download. Automated checks do not replace native desktop and accessibility acceptance.
