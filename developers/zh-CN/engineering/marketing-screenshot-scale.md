# 宣传截图缩放比例规范

[English](../../engineering/marketing-screenshot-scale.md) | 简体中文

用于商店或宣传素材的 Windows、macOS、Linux 原生应用截图统一采用 **150%**，即 **1 个 Qt 逻辑像素对应 1.5 个物理像素**。这是截图环境要求，不改变应用布局，也不要求三个系统使用名称相同的系统显示设置。

当前 [README 截图](../../../docs/images/main-window.png)为 1536 × 932 像素。其 Windows 原始记录为 144 DPI，即 150%，主窗口客户区为 1536 × 933 物理像素，对应 1024 × 622 逻辑像素。发布图因裁去顶部一行而少一像素。机器可读目标及参考尺寸见 [marketing-screenshot-scale.json](../../../specs/capture/marketing-screenshot-scale.json)。

截图前测量应用客户区的实际逻辑尺寸和原生物理像素，排除系统窗框与阴影。确认宽、高的物理像素与逻辑像素之比均为 1.5，边缘允许整数取整误差。默认主窗口客户区处理前应为 1536 × 933 物理像素；其他窗口按各自实际逻辑尺寸计算。记录测量结果和实际截图环境；仅凭图片尺寸无法证明截图主体或缩放比例。

Windows 的参考设置为 144 DPI。macOS 和 Linux 应选择实际达到 1.5 比例的截图环境，并核对最终客户区几何。Retina 底层倍率、系统显示设置或单独的 `QT_SCALE_FACTOR` 数值都不能单独证明最终比例；必须核对 Qt 实际设备像素比与原生截图。不得通过缩放图片将其他倍率伪装成合规截图，也不得将 DPR 或截图物理尺寸写入组件逻辑尺寸。无法达到或测量目标比例时，截图不得标为通过，并记录限制。

[macOS、Linux 截图记录器](platform-marketing.md)会检查 PNG 完整性及操作人员提供的身份，但不会独立测量 Qt 客户区几何或证明截图主体，操作人员必须提供并核对比例证据。Windows DPI 检查见 [Windows 截图流程](../market/windows/capture.md)，逻辑尺寸见[视觉设计规范](../../../docs/zh-CN/visual/design-standard.md)。
