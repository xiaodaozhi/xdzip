# 主动内容预览

预览由用户在属性卡点击“预览内容”触发，选择条目或打开归档不会自动解码内容。主窗口和独立属性查看器使用同一组件，密码复用现有密码窗口；详情仍可独立显示元数据。

## 能力与预算

仅支持可读取、尺寸已知、没有危险名称的普通文件。文本扩展白名单为 txt、md、log、json、csv、tsv、toml、yaml、yml、ini、xml、rs、c、cpp、h、py；仅接受严格 UTF-8 或有 BOM 的 UTF-16。输入及转换后的 UTF-8 各最多 256 KiB，不允许 NUL 等二进制控制字节。内容通过 PlainText 控件显示，标签、网址、程序文本和宏不会执行。

图片仅接受与扩展一致的 PNG、JPEG，编码内容最多 4 MiB，长宽分别最多 4096 像素，总像素最多 4194304。Rust 先检查格式头及尺寸，Qt 异步提供器再次核对实际解码尺寸，只选择 PNG／JPEG 解码器，显示采样最多 512×512，不加载 SVG 或 HTML，不播放动画。提供器只接受随机令牌 URL，不把原始内容、系统路径或任意网络 URL 交给 Image；待加载编码数据全局最多 8 MiB，最多两个并行图片解码。

每次预览累计来源读取最多 256 MiB，解码与跳过的工作最多 64 MiB，辅助算法工作最多 256 MiB。格式字典、工作进程及方法图仍受[格式后端契约](native-codecs.md)的独立内存预算约束，不以条目尺寸作为解码额度。固实包或顺序压缩流可能因前置解码工作超过预算而拒绝小文件预览。

## 完整性、密码与生命周期

预览复用已冻结的物理身份、源版本、负载定位及现有解码器。必须完整解码到 EOF，并完成格式提供的完整性及认证检查后，才能发布文本或编码图片。TAR 等没有逐文件内容校验和的格式仍只具有其原有的结构与可读性保证。磁盘大索引使用受限的所选负载读取，不下载整份元数据到 Qt。

密码只绑定当前请求，沿用零化密码容器；错误、认证歧义与取消按原有类别处理，最多十次尝试。关闭密码窗口只取消预览，保留可用目录。关闭预览、更换条目、隐藏归档页面或关闭会话清空内容，失效请求的晚到结果不得恢复旧内容。失败和取消丢弃未验证的部分缓冲，不写出条目副本，也不启动系统应用；双击的临时打开继续使用[原有独立流程](archive-opening.md)。

图像异步加载和显示采样依据 [Qt 6.8 Image](https://doc.qt.io/qt-6.8/qml-qtquick-image.html)及 [QQuickImageProvider](https://doc.qt.io/qt-6.8/qquickimageprovider.html)，实际缩放、错误、取消和释放仍须产品回归。界面参数与独立状态稿见[视觉规范](../visual/design-standard.md#主动内容预览)。

## 验收

验证真实 ZIP/TAR 路径的文本和 PNG／JPEG，包含密码缺失、错误与正确密码、损坏认证、源替换、取消、关闭、类型伪装、巨大尺寸声明和实际算法工作超限。检查原生窗口中英文、浅深色及多个字号下的文字、图片、错误和操作可达性。自动测试不代替未执行的原生视觉观察，结果按[验证方法](../qa/README.md)登记。

English: Previews are explicit and never run on selection or opening. Strict plain text is limited to 256 KiB, PNG/JPEG input to 4 MiB, dimensions to 4,096 each and 4,194,304 pixels total. The host verifies complete archive payloads before publication, meters source/decode/auxiliary work and retains each format's independent dictionary limits. Text is never markup. Images use an asynchronous token-only provider, fixed raster decoders, a 512-square display sample, an 8 MiB pending-input budget and two decoder slots. Passwords are request-scoped and zeroizing. Cancellation, selection/page changes and closure clear content and reject late results. Preview writes no extracted copy and launches no external application.
