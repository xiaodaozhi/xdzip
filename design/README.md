# xdzip 设计资源

[高保真设计图](hifi/README.md)和[素材库](assets/README.md)提供界面视觉参考。界面尺寸、密度和控件行为以 [视觉规格](../docs/visual/README.md)及其机器可读参数为准。

| 目录 | 内容与用途 |
| --- | --- |
| `hifi/screens/` | 已保存的页面和组件图板，用于独立视觉测量与对照 |
| `hifi/catalog.json` | 设计图用途、尺寸及 SHA-256 清单 |
| `assets/svg/`、`assets/png/` | 统一矢量图标与不同分辨率的透明导出，Qt 使用共享资源 |
| `assets/reference/` | 保留背景和原始裁剪尺寸的视觉参考，不作为运行图标 |
| `assets/brand/`、`assets/file-associations/` | 品牌母版、平台图标及文件关联图标 |
| `assets/fonts/` | 界面字体来源、许可与生成说明 |
| `assets/manifest.json` | 素材用途、来源、画布、倍率及 SHA-256 |

窗口、菜单及其比例应以逻辑尺寸分别测量。截图、测试结果与量测报告保存在 `build/`、`dist/` 或 CI 工件，实际原生视觉验收单独记录，见[验证方法](../developers/qa/README.md)。
