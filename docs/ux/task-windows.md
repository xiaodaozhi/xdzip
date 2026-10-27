# 任务窗口与完整性检查

应用任务入口聚合所有会话，以会话 UUID 和后端任务 ID 路由进度、密码、冲突和结果。关闭页面保留所属控制器，显式退出等待全部异步 join，见[应用会话契约](../engineering/application-sessions.md)。

解压确认和创建确认分别是 ExtractWindow、CreateWindow，由各自主窗口持有，原生窗口装饰交给系统。每个主窗口只有一个 TaskWindow；选择解压或创建任务时绑定所选任务快照，任务终止后在原窗口显示结果。关闭窗口只隐藏视图，取消需要明确点击取消动作。各会话共享重任务串行准入，不为每个任务或状态同时创建窗口。普通文件双击复用此窗口，系统接受临时文件打开后自动隐藏；拒绝时显示错误并允许重试。手动重新查看历史结果不会因无关进度更新再次关闭，见[临时打开契约](../engineering/archive-opening.md)。

创建格式为 ZIP、TAR、TAR.GZ 和 7z，RAR 仅用于读取。7z 保留既有四档级别，默认正常级别 LZMA2、非固实、不加密且不分卷；级别 0 仅存储。固实和文件名加密使用共享 OperationCheck，文件名加密仅在选择密码后可用；撤销密码或切换格式时清空密码并取消文件名加密，切换格式同时恢复非固实、不分卷。所有新选项进入滚动正文，不增加窗口宽度或覆盖 footer 外观。

完整性检查复用同一 TaskWindow 布局，真实工具栏“测试”提交整包只读后台任务，按 `kind=check` 展示读取、密码等待、取消和终态。检查对象不受当前选择影响，首错停止，已验证与未检查文件数分别显示，不把解压或创建的提交计数当作检查数量。格式与安全边界见[检查契约](../engineering/archive-checking.md)。组件开发预览仍提供独立的 IntegrityWindow，并明确标记“界面预览，不执行实际校验”。

HTML 的检查按钮运行明确标注的界面演示。关闭或返回浏览保留演示状态，重新打开显示当前进度；取消结束演示，终态不得继续推进。完成、取消使用同一结构和 footer。对象路径、读取字节、文件数、读取速度、剩余估算、阶段和当前条目是数据槽位，不在 UI 中扫描、读取或解压文件。真正后端接入前不能把示例数字当成校验结果。

窗口的宽度、密度、字号、圆角、边框、进度条、图标槽和说明对齐来自 [视觉规范](../visual/design-standard.md)与 [冻结原稿](../visual/integrity-reference.json)。未知总量不显示确定百分比；大字号允许正文滚动，footer 保持可达。关闭不会发出取消信号；取消动作显式发出取消请求。检查终态不得显示已保存文件或打开目标文件夹。

高保真原稿为 [完整性检查进度](../../design/hifi/screens/integrity-progress.png)，运行图标为统一素材库的 document-check。原稿裁剪只作参考，动态界面使用共享组件。


## 正常尺寸与任务发现

带图标、标题、副标题的创建、添加、解压、任务与结果、冲突、设置、关于窗口，共用 OperationHeader。48逻辑图标槽、20标题、13副标题、96正常高度基准；长文案与大字号按共享字体度量增高，正文继续滚动，footer不随之移动。文本型密码、预检与退出／重启确认不凭空添加装饰图标。参数、独立状态稿与登记见[视觉规范](../visual/design-standard.md)。

任务按钮右侧保留上下箭头。应用内任务菜单最高360逻辑像素，同时受当前窗口可用高度约束；历史列表内部滚动，底部清除按钮固定可达。清除仅隐藏已完成、失败或已取消的任务历史；运行、排队、等待密码／冲突、取消与清理中的任务保留。没有可清除历史时按钮禁用。动作不取消任务，不删除源归档或用户输出，不改变任务的原始控制器或退出屏障。清除后关闭已被隐藏的结果窗口，不显示空快照。

清除按钮位于菜单的可交互内容层，不放在只用于绘制的背景层；鼠标、键盘与原生辅助功能必须执行同一清除动作。分隔线上下各保留4逻辑像素，按钮底部仅使用共享菜单内距，不叠加另一份footer留白。任务菜单项的左右内距与上下内距相同，状态标记右对齐可用行内容，仅列表实际溢出时预留滚动条宽度。任务名称根据原快照的kind区分检查、解压、创建、添加、移动与导出，不用解压作为所有非创建任务的名称；点击仍按完整任务身份打开原控制器，不将检查任务改路由为解压。

English: The fixed clear action belongs to the interactive popup content, not its decorative background. Pointer, keyboard and native accessibility invoke the same clear intent. Leave 4 logical units on either side of the separator; only shared menu padding remains below the button. Task rows use the same horizontal and vertical padding. Right-align status badges within the row, reserving scrollbar space only for actual overflow. Labels reflect the snapshot kind (check, extract, create, add, move or export), while activation retains the original composite task identity and controller.

English: All existing icon/title/subtitle operation headers share a 48-unit icon slot, 20-unit title, 13-unit subtitle and 96-unit normal height baseline. Long text and larger fonts grow using the same content metrics. Text-only prompts retain their roles. The task button has paired chevrons. Task menus are capped at 360 logical units and the available viewport, with a scrolling history and fixed clear button. Clearing hides only completed, failed or cancelled history, preserves live tasks and prompts, closes cleared result views, and never cancels work or removes source archives or user outputs.

2026-10-15 修订：创建、解压与共享任务窗宽度统一为 491 逻辑像素。进度和结果按内容自适应高度，最多 600；明细折叠时不保留展开空间，展开列表最多 168。切换任务 ID 清空明细展开状态，同一任务的进度更新不重置用户选择。文件已存在窗口同宽，最高 560，沿用正常密度和共享 footer。尺寸修订不改变原生独立窗口及取消语义。

状态栏显示当前控制器保留的全部任务数（最多 64），右侧状态标记区分终态与运行状态。活动任务计数继续用于异步退出，不因历史展示而改变任务调度或关闭屏障。

任务发现列表以任务 ID 保持条目身份。进度与状态变化在原条目更新，不重建整张列表，不中断鼠标悬停、高亮或键盘焦点；新增任务、淘汰历史时只插入或移除对应条目，保留其余条目的身份和选择。排序仍为任务 ID 倒序，点击或键盘激活始终重开对应 ID 的任务。QML 使用的数据模型只是任务历史的界面投影，不改变任务快照、取消、提交、终态或退出屏障语义。

此更新不改变既有窗口、菜单的布局、逻辑单位或视觉状态。HTML 设计的任务菜单不随持续进度重建节点，沿用现有原稿；运行验证仍需覆盖静止悬停、键盘导航、状态原位更新、任务新增与历史淘汰。

## 原生选择器与拖放激活

打开压缩包、添加文件／目录、更改解压或创建目标位置，均将实际调用的 Quick 窗口传给共享 NativeDialogPolicy。选择器仍为独立原生对话框，使用应用模态和调用窗口的 transient ownership；退出嵌套事件循环后，仅当调用窗口仍存在且可见时，排队恢复该窗口的激活，不能固定恢复主窗口。关闭调用窗口不复活窗口。系统选择器文案继续遵循系统语言。

欢迎页只在有效本地单压缩包拖放被接受后，在下一轮事件循环提升并激活接收主窗口，使激活发生在 DropArea 接受操作之后。无效拖放不改变前台窗口，普通后台任务完成不请求激活。自动化使用自有窗口与夹具，真实 Finder 拖放仍须桌面人工复核。

## 退出与重启确认

QuitWindow 和 RestartWindow 继承 OperationConfirmationWindow，统一应用模态、正常密度、自适应高度、可滚动正文与固定 footer。按钮继续表达原有返回、等待、取消退出、稍后和重启意图，文件提交与任务清理仍由既有控制器处理。正在停止时禁用决策动作与 Escape 关闭，标题和说明变化不恢复旧尺寸。

此前仅验证任务退出成功而遗漏确认框视觉，现将功能与视觉证据分开记录。窗口完整性、基类和尺寸检查见[强制流程](../visual/design-workflow.md)，对应状态稿见[确认窗口](../../design/hifi/sources/confirmations.html)。

密码解压使用独立的 `waiting_password` 状态和请求身份，TaskWindow 显示等待输入密码并提供重开动作，应用模态 PasswordWindow 处理输入与取消。它不使用重名冲突的 `promptId`；具体生命周期见[密码解压交互](password-extraction.md)。
