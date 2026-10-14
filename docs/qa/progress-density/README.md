# 解压进度窗口，正常密度复核

2026-10-14：进度窗口沿用的 125% 逻辑几何改为 0.8。Qt 实际宽度 677、正常客户区高度 490、标题 20、正文 13；HTML 采用 819.2／1548 源坐标换算。DPR 仅改变截图像素，不再写入组件尺寸，Qt 没有 Item.scale。

原稿位于 [progress.png](../../../design/hifi/screens/progress.png)。HTML 的新捕获见 [progress.png](html/progress.png)与 [回执](html/receipt.json)；Qt 新捕获见 [task-waiting.png](qt/task-waiting.png)。[量测报告](validation.json)核对标题、footer、图标、按钮和提示文字共线；[新 HTML 量测基线](html-operation-metrics.json)明确记录 0.8 换算，历史 extract-fidelity 基线不覆盖。

两行提示的文字起点相同，字号均为 12；按钮高 32，footer 不重复缩小。正文在小屏幕独立滚动，footer 保持可达。DPR 1／1.25／2 的结果另登记在本页目录；离屏屏幕在高 DPR 下变小，正文提示可能需滚动，不能误记为完整桌面验收。捕获源为 Linux Qt6.8.2 offscreen/software，原生窗口装饰未纳入 grabWindow。

完整创建工作包与运行检查见 [WP-10 验收](../m4-acceptance.md)。
