# 新建窗口与操作控件复核

采用 C01 原稿、HTML 设计稿、Qt 实现的顺序。原图与旧捕获不改写；原图 SHA 与裁剪坐标见 [create-reference.json](../../visual/create-reference.json)。[对照页面](index.html) 独立展示原稿、HTML 与 Qt，另外展示深色、悬停、焦点与下拉展开的实际客户区捕获。

之前 AppButton 已共用，但 Create、Extract／Task、Conflict／Quit 的窗口分别覆盖圆角为 8、4.8、6；角色样式没有统一约束。创建移除还是文字按钮，格式选项分别画圆角，摘要没有普通文件大小。本次删除 footer 外观覆盖，并提取 OperationButton、OperationFooterButton、OperationFrame、OperationRemoveButton、OperationSegmentedGroup；所有操作下拉继续复用 OperationSelect。

正文动作高 28、footer 高 32，统一圆角 5；浅色描边 #C9D1E0，深色 #485469。下拉槽 18、右留白 6。圆形减号命中 28、图标槽 24，使用统一 remove 素材，环线颜色与控件描边一致。压缩格式共用连续分段组，等宽无间距，TAR／TAR.GZ 保持禁用。危险动作和焦点仍保留语义色。

Qt 夹具在临时目录真实建立三个普通文件，大小分别为 8,400,000、6,200,000、12,800,000 字节。后台元数据完成后显示“3 个文件，27.4 MB”；移除第一项后显示“2 个文件，19.0 MB”，三个源文件保持存在。目录不显示伪造总量，开始创建仍重新冻结输入树。

检查包括实际 QML 形状、颜色、字号、尺寸、图标中心、箭头边距、分段组接缝、footer 可达性；实际点击移除，键盘跳过禁用格式、选择下拉末项，验证 Store 禁用压缩级别。HTML 同步采用共用角色、连续分段组与圆形减号。检查拒绝六类窗口的 footer 样式覆盖，并检查当前源码没有中点分隔。历史位图和原始捕获 JSON 仍保持原证据身份。

本地环境为 Linux x86_64，Qt 6.8.2，offscreen／software，Chromium 离线画布；DPR 与逻辑尺寸见各捕获 JSON，页面缩放证据单独登记。实际客户区截图不含操作系统装饰，不能替代 macOS 原生标题栏或桌面人工验收。旧原图文件图标、插图及文字抗锯齿与当前统一素材／字体存在差异；当前 ZIP 能力禁用项、滚动正文和真实目录状态是明确功能修订。

本地 Qt 创建控件量测在 DPR 1／1.25／2 各 325 项通过；其他操作窗口量测 81 项通过，ZIP 创建往返 28 项通过。HTML 操作控件 1278 项通过，创建窗口的真实浏览器 100%／125% 页面缩放、DPR 1／1.25／2 检查 54 项通过；主界面比例回归 594 项通过。Rust fmt／Clippy／workspace 测试、Qt 六组 CTest 通过，浏览器后台生命周期整组连续十次通过。qmllint 成功返回，保留已有 columnWidthProvider 类型提示，未把它记作零警告。

[Qt 量测 DPR 1](qt-controls-dpr1.json)、[DPR 1.25](qt-controls-dpr1.25.json)、[DPR 2](qt-controls-dpr2.json)、[HTML 真实页面缩放](html-native-zoom.json)、[全部操作窗口 HTML](html-operation-controls.json)、[其他操作窗口 Qt](qt-other-operations.json)、[真实 ZIP 往返](qt-zip-roundtrip.json)。main 提交及远端 CI 待发布后记录；检查数不表示视觉还原百分比。
