# WP-10 · 真实 ZIP 创建验收

2026-10-14：开发与三平台自动化验收完成。实现提交为 `a69d2fd2e76ad07789aa5eeb3c5b61acb4ddc39f`。[Qt components](https://github.com/xiaodaozhi/xdzip/actions/runs/38535053993)、[Foundation probes](https://github.com/xiaodaozhi/xdzip/actions/runs/38535053997)、[HTML design review](https://github.com/xiaodaozhi/xdzip/actions/runs/38535053981)和 [Documentation](https://github.com/xiaodaozhi/xdzip/actions/runs/38535054002)全部通过。[原生 CI 回执](m4/native-ci.json)记录九项任务与三平台工件身份。macOS 仅上传日志／JSON，没有 Apple Silicon 应用包；build/、target/ 和 dist/ 均不提交。桌面人工验收和独立安全审查仍单独登记。

## 实际能力和边界

从选择的文件／目录创建 ZIP，支持 Store、Deflate 1／6／9，保留 Unicode 名称、空文件、空目录与所选目录根名称。同路径和嵌套选择去重；输出位于输入树内时，仅排除本次输出与身份匹配的私有暂存。写入后重新扫描并逐条解码检查头、大小、CRC 和流终止，最终整体保存一次。重名可取消创建、自动改名或明确替换；关闭任务窗口后任务继续，可以重开、取消或异步退出。

输入不授权链接、重解析点、设备或其他特殊对象。后台发现／读取／验证使用有限根句柄和固定块，格式后端没有系统路径与提交能力。来源变动或暂存损坏不能发布，原目标变动不能替换。输入身份／长度／时间戳检测不是原子快照；同账户刻意恢复元数据的编辑不保证识别。完整预算、目录授权与最终提交竞态前提见 [实现契约](../engineering/m4-creation.md)，不宣称抗同账户的文件系统 CAS。

TAR／TAR.GZ、加密、分卷、就地编辑、源权限完整保留、实际内容预览与第三方工作进程仍未实现。M4 阶段还包含 WP-11，因此 WP-10 完成不等同于整个 M4 或正式发行完成。

## 实际验证

- 核心：Store／Deflate 真实编码与逐项读回；13 项创建回归覆盖选择去重、名称碰撞、来源变化、父目录链接替换、目标版本变化、输出自包含排除、暂存损坏、冲突取消、写入中取消及提交后的迟到取消。
- 独立互操作：Python zipfile 检查 Store 和 Deflate 1／6／9 的全部内容与 CRC；Unicode、空文件和空目录；66,001 条记录触发 ZIP64 条目数。计数夹具不证明大字节偏移已独立验证。
- Qt：从真实“新建”按钮打开独立窗口，选择输入、目标检查、Store 禁用级别、创建 ZIP、重新打开、进入真实子目录，以及重名窗口取消后逐字节保持原归档。整个 CTest 6 组通过；创建与进度窗口在 DPR 1／1.25／2 分别采集几何、字号及未加工截图。
- HTML：三平台浅深色与 DPR 的操作窗口、统一下拉、按钮与示例创建流程；真实浏览器 100%／125% 缩放。HTML 只使用示例数据，不访问本机文件。

实际参数、耗时与检查输出见 [本地记录](m4/local-validation.json)和 [核心互操作测量](m4/core-performance.json)。约 8 MiB 有规律夹具在本机的单次测试不代表稳定性能预算；66,001 项的峰值 RSS 包含输入树与 ZIP 中央目录。短任务的 Verifying 快照可能被有界通道合并，无法单独计时则记录 null。

## 视觉和桌面状态

[Qt／HTML／原稿对照](m4/index.html)展示未加工的正常逻辑尺寸截图。C01 的三段卡片、白色内层、图标、字号与字段密度已同步；TAR／TAR.GZ 禁用，目录选择项的字节总量在后台扫描后确定。正文可滚动，footer 固定；高 DPR 离屏虚拟屏幕变小会按可用高度约束窗口，检查不把该裁切当成原生桌面尺寸放大。

[进度窗口整改](progress-density/README.md)将旧的 125% 密度调整为真实 0.8 几何：宽 677、客户区高 490、标题 20、正文 13，footer 保持 13／12。旧原稿和历史回执未改写。当前 Qt 捕获来自 Linux Qt 6.8.2 offscreen/software，不证明最新版 macOS 的原生装饰、Retina GPU、输入法、辅助功能或桌面人工体验。独立安全审查与正式签名发行继续单独登记。
