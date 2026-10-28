# 系统应用图标

应用品牌源和平台图标位于 [design/assets/brand](../../../design/assets/brand/)；各格式文档源和产品使用的 ICO／ICNS 统一位于 [file-associations](../../../design/assets/file-associations/)。应用和文档图标各司其职。

| 平台 | 当前接入 |
| --- | --- |
| Qt | 共用图标与实际文件打开路由，忙时排队 |
| macOS | 应用及文档 ICNS，Viewer/Alternate 文档声明和 QFileOpenEvent；只原生构建，不自动制作安装包或全局注册 |
| Windows | EXE 图标、当前用户 Inno Setup 安装器、OpenWith 候选和文档图标，保留原默认 |
| Ubuntu 24.04 | 独立 Qt 运行目录及 amd64 .deb，桌面 MIME 候选，保留原默认 |

安装、升级、运行保护、卸载与平台边界见[安装与文件关联](../../../docs/engineering/installers-associations.md)，运行目录见[Linux 部署](../../../docs/engineering/linux-deployment.md)。普通 CMake install 仍是 SDK 安装规则，不代替独立运行包。格式边界沿用项目 README，GZ 候选仅用于 TAR.GZ，不支持普通独立 GZIP。未签名安装器不作为正式发行或人工桌面验收通过。

图标留白：macOS ICNS／Dock PNG 主体约 81.2%，通用 PNG／Windows ICO 约 90%，见[素材说明](../../../design/assets/README.md)。
