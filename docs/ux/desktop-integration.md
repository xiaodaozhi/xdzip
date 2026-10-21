# 系统菜单与托盘

多会话、单实例、窗口状态及全局退出以[应用会话契约](../engineering/application-sessions.md)为准。关闭非最后主窗口保留任务，最后窗口遵循托盘设置；退出确认覆盖所有会话。

macOS 顶部菜单和三平台托盘使用操作系统原生菜单，保留已有主窗口标题栏。参数见 [桌面集成参数](../visual/desktop-integration.json)，[交互稿](../../design/html/desktop-integration.html)，[高保真入口](../../design/hifi/sources/desktop-integration.html)。原生字体、行高、菜单宽度与系统高亮不套用应用内 CommandMenu 的密度参数；HTML 为交互说明，不作为原生像素验收。

macOS 菜单按 xdzip、文件、编辑、显示、操作、窗口、帮助排列。“帮助”提供“使用说明”和“反馈问题”，通过默认浏览器打开项目 README 与 Issues 页面，不自动提交反馈；分隔线后提供“关于 xdzip”，打开与应用菜单、托盘相同的关于窗口。帮助菜单的关于项使用 NoRole，保留在原位置；应用菜单使用 AboutRole、PreferencesRole、QuitRole，系统提供服务、隐藏及显示全部。文件含打开、新建、最近打开、关闭压缩包、关闭窗口；编辑按当前文本焦点提供撤销、重做、剪切、复制、粘贴、全选与查找；显示含列表、网格与预览；操作含全部解压、解压所选与元数据预检；窗口含最小化、缩放、主窗口、任务及可见产品窗口。

菜单和既有界面通过统一命令入口执行，打开时及触发前重新检查能力。⌘W 关闭当前窗口，⇧⌘W 关闭压缩包。文本编辑不误操作文件选择，模态窗口阻止主窗口命令，禁用项不附加解释行。最近文件仅在允许记录时展示，不额外复制或上传路径。文案使用 zh-CN/en-US 资源，系统自带项目遵循系统语言。

托盘默认启用，设置开关保存后立即生效，取消不修改。schema 1 增加 desktop/trayEnabled，旧配置默认 true，保留未知字段与原子保存策略。菜单提供显示主窗口、打开、新建、任务、设置、关于、退出；任务由真实控制器数据驱动，状态与任务详情一致。macOS 点击展开菜单；Windows/Linux 普通激活恢复窗口、右键展开菜单。任务完成不自动弹出通知或抢焦点。

托盘可用时关闭最后主窗口会清除该窗口的浏览标签与操作窗草稿，再隐藏窗口；再次显示仅保留一个起始页。后台任务及其会话身份继续保留，从全局任务入口主动打开。最小化不清除标签。模态窗口或文件选择器存在时先激活阻塞窗口。明确退出、系统退出和重启绕过隐藏，继续现有任务确认与异步 join；取消退出恢复驻留策略。关闭托盘先恢复隐藏主窗口。无托盘时恢复正常关闭，运行中托盘消失且窗口隐藏时恢复主窗口。macOS Dock 保留，点击 Dock、文件关联重新打开均可恢复。测试探针以明确退出语义完成，不依赖驻留设置。

托盘图标从现有品牌标志提取单色字形，macOS 为模板图标，Windows 按任务栏主题切换黑白图标，Linux 使用黑白双层字形兼容独立面板主题，并提供多尺寸位图。图标不做闪烁或持续动画。

“显示”菜单在“文件预览与属性”之后保留不可折叠的分隔线，将 AppKit 自动追加的全屏命令独立分组，避免系统全屏图标使预览项额外预留图标列。预览项与列表、网格的文字起点对齐，勾选标记仍由系统绘制；不靠空格、固定宽度或自绘菜单补偿。HTML／高保真交互稿只表达应用命令，系统追加项由原生桌面验证。

## 验证

应用自己的 Qt Quick 窗口隐藏或最小化时，同步结束原窗口的指针悬停，清除按钮、MouseArea、HoverHandler 及 tooltip 的旧位置记录。恢复首帧同步前按系统实际指针位置重新核对，不能沿用关闭前的 hover 底色或旧坐标；晚到的原生进入事件也按实际位置处理，直至真实鼠标移动或按下同步坐标。此规则适用于主窗口与操作窗口，不改变焦点、按钮绑定或原生文件选择器行为。

Windows 托盘隐藏后保留原生窗口、状态、QML 对象、控制器与后台任务，在隐藏状态下暂时遮蔽窗口透明度，恢复后首个新帧完成提交才恢复原透明度。仅清理 hovered 属性不能防止 DWM 在新帧提交前短暂显示窗口的最后一帧。新一轮隐藏使旧帧回调失效；此策略只适用于托盘隐藏，不用于最小化或仍可见的窗口，不通过截图读回、固定延时或禁用系统动画实现。

English: Hiding or minimizing an application-owned Qt Quick window ends pointer hover and its cached position before restoration. The first restored frame reconciles against the current system pointer position before scene synchronization. Late native enter events remain normalized until real mouse movement or a press supplies current coordinates. Controls, MouseAreas, HoverHandlers and tooltips resume from fresh pointer input instead of the old hover color or coordinates. Focus, control bindings and native file choosers retain their existing behavior.

没有系统全局指针的后端以注入事件为准，不查询物理鼠标覆盖其状态。offscreen／minimal 的组件检查与原生桌面恢复检查分开，不能用离屏检查替代 DWM、Cocoa 或 X11 的实际窗口验收。

Backends without a system-wide pointer treat injected events as authoritative instead of overriding them with physical cursor queries. Offscreen/minimal component checks remain separate from native DWM, Cocoa and X11 window acceptance.

On Windows, tray hiding retains the native window, state, QML objects, controllers and background tasks. While hidden, its opacity is masked until the first newly submitted frame after restoration, preventing DWM from revealing the previous hover-colored image. Hiding again invalidates old callbacks. This does not affect minimization or visible windows, and uses no screenshot readback, fixed delay or animation override.

自动化覆盖菜单结构和角色、状态同步、命令与编辑路由、配置兼容与取消、驻留及退出策略、无托盘回退、任务菜单、模态阻塞与重启。原生桌面另核验 macOS 顶部菜单、Dock 恢复，Windows 通知区与 Linux StatusNotifier/XEmbed，深浅外观、高 DPR、键盘与屏幕阅读。离屏验证不等于原生桌面验收；截图与报告仅输出 build/ 或 dist/。

原生编辑角色仅在 `QCocoaMenuItem` 的 Cut、Copy、Paste、Select All 翻译上下文中使用应用语言目录，`QCocoaMenuBar/Edit` 也使用同源标题，使 Qt 的文本角色检测与编辑菜单识别不依赖系统语言。系统文件选择器及其他 Qt 控件继续使用系统 locale 的 qtbase 翻译。须同时验证中文应用与英文系统，以及英文应用与中文系统，不通过修改用户全局 AppleLanguages 来测试。

托盘 tooltip 使用 `xdzip (活动中的任务: %1)`，任务菜单使用 `任务 (活动中的任务: %1)`，英文分别为 `xdzip (Active tasks: %1)` 与 `Tasks (Active tasks: %1)`。计数来自应用级真实活动任务总数，包含被关闭浏览页面仍在运行的任务；完成结果不计为活动任务。
