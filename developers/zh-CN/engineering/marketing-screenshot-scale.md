# 宣传截图缩放比例规范

[English](../../engineering/marketing-screenshot-scale.md) | 简体中文

用于商店或宣传素材的 Windows、macOS、Linux 应用截图的导出图统一采用 **150%**，即 **1 个 Qt 逻辑像素对应 1.5 个物理像素**。保持应用的正常逻辑尺寸，按显示器原生密度采集。

每种语言的 README 使用同语言的 macOS 起始页截图。主窗口为 1024 × 622 逻辑像素，按 200% Retina 密度采集得到 2048 × 1244 的原图，再等比缩小到 150%，发布尺寸为 1536 × 933，保留完整窗口。 [README 参考图](../../../docs/images/main-window.png)和[机器可读尺寸](../../../specs/capture/marketing-screenshot-scale.json)。

截图前测量应用客户区的实际逻辑尺寸和原生物理像素，排除系统窗框与阴影。核对宽、高的物理像素与逻辑尺寸之比，边缘允许整数取整误差。默认主窗口客户区导出时应为 1536 × 933 物理像素；其他窗口按各自实际逻辑尺寸计算。记录测量结果和实际截图环境；仅凭图片尺寸无法证明截图主体或缩放比例。

Windows 的参考设置为 144 DPI。原生密度高于导出比例时，用正常窗口采集，再通过 `--capture-scale` 记录实际测量的比例。导出器保留原始 PNG，并将展示图等比缩小到 150%。例如，Retina 按 200% 采集的 2048 × 1244 原图导出为 1536 × 933。不得放大截图，原始比例和导出比例分别记录。

[macOS、Linux 截图记录器](platform-marketing.md)会检查 PNG 完整性及操作人员提供的身份，但不会独立测量 Qt 客户区几何或证明截图主体，操作人员必须提供并核对比例证据。Windows DPI 检查见 [Windows 截图流程](../market/windows/capture.md)，逻辑尺寸见[视觉设计规范](../../../docs/zh-CN/visual/design-standard.md)。
