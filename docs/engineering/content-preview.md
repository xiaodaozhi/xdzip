# 主动内容预览

预览由用户在属性卡点击“预览内容”触发，选择条目或打开归档不会自动解码内容。主窗口和独立属性查看器使用同一组件，密码复用现有密码窗口；详情仍可独立显示元数据。

## 能力与预算

仅支持可读取、尺寸已知、没有危险名称的普通文件。文本扩展白名单为 txt、md、log、json、csv、tsv、toml、yaml、yml、ini、xml、rs、c、cpp、h、py；仅接受严格 UTF-8 或有 BOM 的 UTF-16。输入及转换后的 UTF-8 各最多 256 KiB，不允许 NUL 等二进制控制字节。内容通过 PlainText 控件显示，标签、网址、程序文本和宏不会执行。

| 图片类型 | 三平台范围与实际语义 |
| --- | --- |
| PNG，JPEG（jpg、jpeg、jfif、jpe） | 完整读取一张静态图片，按可用的变换元数据处理朝向 |
| GIF，WebP | 静态图片或动画首帧，不播放动画 |
| BMP，ICO／CUR | 普通图片或解码器的默认图像，CUR 不激活系统光标 |
| ICNS | PNG 或可用系统解码器支持的 JPEG2000 内嵌图像，最多 256 块；未知内嵌类型、旧式裸位图／RLE 图标明确拒绝 |
| TGA | 无色表／无图像 ID、左侧起始的未压缩真彩 16／24／32 位；补齐同布局旧版空页脚，不支持 RLE／索引色 |
| TIFF（tif、tiff） | 第一页，不自动遍历页面 |
| SVG | UTF-8 静态矢量，在受限解析后栅格化，具体子集见下节 |
| HEIC／HEIF | 仅 macOS，Qt 官方插件调用系统 ImageIO；其他平台明确拒绝 |

不接入 AVIF、RAW、PSD、EXR、PDF、Office 或独立第三方解码库。文件扩展、Rust 签名检查及固定 Qt 解码器共同决定准入，不把系统安装了额外插件当作产品能力，也不自动探测其他解码器。

图片编码内容最多 16 MiB，Rust 事件队列与 Qt mailbox 各限 32 MiB，命令预算仍为 8 MiB，原图长宽分别最多 16384，总像素最多 33554432。原图准入与输出缩略图独立计量：最终输出宽高均不超过 512，保持比例，RGBA8 最多 1 MiB。12MP 等照片可以在预算内缩略读取；部分解码器先读取原图再缩放，不能据缩略图尺寸声称内部只使用 1 MiB。可用色彩描述转换为 sRGB，未知色彩描述保留解码器结果，不承诺专业色彩校样。

Qt 异步图片提供器只接收随机令牌，不接收系统路径或网络 URL。全局保留的编码内容最多 32 MiB，最多两个并行解码任务；已取消但尚在回收的任务各自最多持有一份 16 MiB 输入。每次任务启动固定的相邻私有工作进程 `xdzip-image-preview`，不向它传递归档路径、密码、目标句柄或应用凭据。进程输出只允许定长头和严格宽高／长度一致的 RGBA8，不允许额外输出。失败、崩溃、非法帧、超时和晚到结果均不能显示。

宿主从启动开始施加 5 秒总时限，每 50ms 检查请求是否仍有效；关闭、取消或更换条目会终止并回收该进程。工作进程对 QImage 分配设置 128 MiB 上限。Windows 使用 512 MiB 进程提交内存和 3 秒用户 CPU 的 Job；Unix 使用 3 秒 CPU、禁用 core／文件输出及有界描述符，Linux 地址空间限 512 MiB。macOS 在已有 dyld 虚拟保留之外增加 512 MiB 地址空间额度，不将此数值当作 RSS 硬上限。这是私有解码器的崩溃与资源隔离，不是任意第三方程序的内核权限沙箱。

每次归档内容读取累计来源最多 256 MiB，解码与跳过工作最多 64 MiB，辅助算法工作最多 256 MiB。格式字典、工作进程及方法图仍受[格式后端契约](native-codecs.md)的独立预算约束。固实包或顺序压缩流可能因前置解码工作超过预算而拒绝小文件预览。

## SVG 静态子集

SVG 在三个桌面平台使用同一 Qt Svg 渲染路径。进入渲染器前，以 XML 流式解析器重建受支持的静态图形、文本、样式、内部定义、渐变、裁剪、蒙版和限定滤镜；最多 8192 元素、64 层、每元素 64 属性，单属性最多 1 MiB，总输出仍限 16 MiB。metadata 内容不参与渲染。样式元素只接受文本，合并 XML 文本分段及 CDATA 后按完整 CSS 检查，不能通过分段绕过外部引用限制。

禁止 DTD、实体、处理指令、脚本、事件、动画、foreignObject、image 元素、字体文件、外部样式和外部资源引用。href 及 CSS url 只接受当前文档的简单片段身份，禁止 xml:base、src、CSS import 和转义形式。内嵌位图与 SVGZ 不在本子集内。未知绘制元素或被禁止内容使整次预览失败，不静默丢弃后冒充完整显示。此范围支持普通静态 SVG，不声称覆盖全部 SVG／浏览器能力。

## 完整性、密码与生命周期

预览复用已冻结的物理身份、源版本、负载定位及现有解码器。必须完整解码到 EOF，并完成格式提供的完整性及认证检查后，才能将文本或编码图片交付到预览阶段。TAR 等没有逐文件内容校验和的格式仍只具有其原有的结构与可读性保证。磁盘大索引使用受限的所选负载读取，不下载整份元数据到 Qt。

密码只绑定当前请求，沿用零化密码容器；错误、认证歧义与取消按原有类别处理，最多十次尝试。关闭密码窗口只取消预览，保留可用目录。关闭预览、更换条目、隐藏归档页面或关闭会话清空内容，失效请求和解码结果不得恢复旧内容。失败和取消丢弃未验证部分，不写出条目副本，也不启动系统应用；双击的临时打开沿用[独立流程](archive-opening.md)。

## 工具链与验收

`desktop.py prepare --install-qt` 为新安装或既有 SDK 补齐固定 Qt 6.8.2 的 qtimageformats；doctor 在构建前检查 Svg 与必需插件。官方源码、摘要、通知和 SDK 对应关系纳入统一配置，Qt 自带 libwebp／libtiff 与独立新增格式库分别管理。`check-image-preview.py` 核对实际工作进程、插件和最小解码，在默认测试、Windows／Linux 部署验证及获授权的 Mac 打包中执行；Mac 显式补齐 offscreen 并在自检前清除 SDK 搜索覆盖，不能以开发 SDK 的插件目录代替实际运行包验证。

回归使用自有图片覆盖各类型、动画首帧、TIFF 首页、透明度、12MP 照片、macOS HEIC 及其他平台拒绝、SVG 内部定义和外部资源拒绝、损坏输入、像素超限、异常帧、超时和取消。真实归档链路继续覆盖密码、完整性、来源身份及 UI 失效清理。布局沿用[视觉规范](../visual/design-standard.md#主动内容预览)，按[验证方法](../qa/README.md)复用未改变的布局证据，并观察新增图片的实际显示。

English: Explicit previews support bounded UTF-8/BOM UTF-16 text and PNG/JPEG/JFIF, GIF/WebP first frames, BMP, ICO/CUR, ICNS, TGA, TIFF first pages and a static, resource-free SVG subset on all desktop platforms. HEIC/HEIF uses Apple's system decoder only on macOS. No AVIF, independent format libraries or document engines are added. Encoded images are capped at 16 MiB; source dimensions at 16384 each and 33554432 pixels. A private process returns at most a 512-square, 1 MiB RGBA8 thumbnail under a five-second wall deadline, native resource limits and cancellation checks. Two jobs and a 32 MiB pending-input budget bound concurrency. Native limits have the platform distinctions stated above; macOS additional virtual address space is not an RSS cap. SVG rejects external/embedded resources, active content, entities and excessive complexity. Style elements accept only text and validate the complete CSS after joining XML/CDATA chunks. Complete archive verification precedes content delivery. Passwords, session identities, cancellation and teardown retain their existing guarantees. Actual format and platform results belong in issues and generated evidence, not this contract.
