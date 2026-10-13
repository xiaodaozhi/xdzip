# 根目录与文件项交互验收

日期：2026-10-13。Qt 与 HTML 同步采用“根目录：直属文件和文件夹，不递归显示后代”。规范来源为 [设计标准](../visual/design-standard.md)、[目录树参数](../visual/directory-tree.json)、[密度与布局](../visual/layout-scale.json)及[共享控件](../visual/controls.json)。

[图片审阅页](root-directory-interactions/index.html)提供实际截图；[主窗口上下对照](root-directory-interactions/comparison.png)上方是 HTML，下方是 Qt，保持各自原始采样尺寸。HTML 使用示例 JPEG 与演示属性，Qt 使用仓库自有 ZIP 的真实 PNG 元数据；Qt 尚未解码内容预览，因此照片和大小不能作为两者视觉差异的评分依据。原高保真图片保持原始字节，新交互以逻辑规范为准。

## 实现与检查

| 整改项 | Qt / HTML 行为与证据 |
| --- | --- |
| 档案标题 | “档案文件”与面包屑共用垂直中心；Qt 100% 与 200% 字号均检查，误差小于 0.5 逻辑像素 |
| 树引导线 | 父层竖线中心 X 对齐父节点展开箭头中心 X，Qt 在两种字号下检查误差小于 0.01；HTML 同样测量实际坐标 |
| 列表悬停 | 当前悬停行的全部列使用同一填充，选中填充优先；Qt 逐列检查，HTML 检查全部单元格颜色 |
| 网格 | 左右内边距 16；卡片内部左上 8 的复选框悬停显示、选中常驻；鼠标移出后的状态也验证 |
| 预览开关 | 关闭预览后重新打开按钮可见，固定 28×28；重新打开保留当前列表状态 |
| 菜单 | 与调用按钮留 4 间距；排序、任务、更多及面包屑菜单采用同一规范，菜单四周有柔和阴影；Qt 对阴影外侧像素作前后比较 |
| 深色分裂按钮 | 分隔线使用深色 border `#363D4B`；Qt 与 HTML 检查实际颜色 |
| 树提示 | 目录悬停不弹出浮动 tooltip；可访问描述保留完整路径；HTML 等待超过提示延时检查 |
| 根目录查询 | 只显示直属文件和目录；真实 ZIP 验证隐式父目录、显式空目录、重复记录、取消及递归搜索 |
| 名称与路径 | 普通视图显示末级名称，搜索结果显示完整归档内路径；复制与导航仍使用完整路径 |

## 查询与身份边界

`ListingRow` 区分实际物理记录和缺少显式记录的导航目录。物理重复记录保持独立 ID，不按名称合并。导航行传递 `navigation_id`，Qt 使用 `dN` 行键，物理 ID 为 0；它可以选中、复制与进入目录，但不会提交到物理记录预检。含导航行或超过 128 项的选择禁用预检，未知大小、日期、方法显示 `—`。

Page 的 `archive_total` 是归档物理记录数，`total` 是当前列表行数；搜索计数不覆盖归档摘要。Qt 收集全部内部批次后原子替换当前目录模型，界面没有分页。导航索引超限时不推测补齐目录，界面明确降级，搜索仍覆盖已准入的物理快照。详细契约见[只读浏览](../engineering/readonly-browser.md)和[M1 运行时](../engineering/m1-runtime.md)。

## 验证范围

- Rust 工作区测试、Clippy `-D warnings`、C++ 格式与 QML 静态检查通过。
- 本地 Qt 6.8.2，Linux x86_64，offscreen / software：5 组 CTest 全部通过；浏览器测试包含新增 `rootListingAndReviewedInteractions`。DPR 1.25、2 的交互和实际几何复验通过。
- [Qt 几何报告](root-directory-interactions/qt-fidelity.json)：729 项通过；[DPR 1.25](root-directory-interactions/qt-fidelity-dpr1.25.json)、[DPR 2](root-directory-interactions/qt-fidelity-dpr2.json)各 729 项通过。[布局与 200% 字号报告](root-directory-interactions/qt-layout.json)：75 项通过。
- [HTML 真实缩放报告](root-directory-interactions/html-validation.json)：594 项通过；私有 Xvfb / XTest 执行真实浏览器 100%／125% 页面缩放，覆盖 DPR 1／1.25／2，没有 CSS zoom 或 pinch 模拟，没有外部请求及页面错误。
- HTML 内容区域独立基线检查：67 项通过，保留原稿和明确的交互修订，19 处字形轮廓仅供目视复核。标题区、预览、原稿加载、共享交互及示例流程另有各自报告。

[采集来源](root-directory-interactions/evidence.json)记录采集时父提交和修改后源码 SHA-256，避免把尚未提交的工作区截图误归属于旧提交。三平台 CI 脚本已加入本次交互及高 DPR 检查；本地离屏证据不能代表最新 macOS Cocoa、Windows 或 Linux 桌面窗口的原生验收。推送后的工作流状态另行核实。
