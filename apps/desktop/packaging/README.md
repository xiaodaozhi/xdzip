# 系统应用图标

品牌源文件和派生格式统一位于 [design/assets/brand](../../../design/assets/brand/)。`xdzip.png` 为 512×512 透明应用图标；`xdzip.ico` 和 `xdzip.icns` 提供对应平台的多尺寸表示。`xdzip-zip.png`、`xdzip-zip.ico` 和 `xdzip-zip.icns` 是 ZIP 文件图标，供后续安装器与文件关联工作包使用。

| 平台 | 当前接入 |
| --- | --- |
| Qt 窗口 | 正式浏览应用和组件预览通过 `QGuiApplication::setWindowIcon` 读取共用的应用图标；标题栏中的品牌和压缩文件图标使用各自的 24 单位共享素材 |
| macOS | `xdzip.app/Contents/Resources/xdzip.icns` 与 `Info.plist` 的 `CFBundleIconFile` 配套，供 Finder 和 Dock 显示 |
| Windows | 应用 ICO 编入可执行文件的资源 1，供 Explorer 和任务栏使用；版本信息与 CMake 项目版本一致 |
| Linux | `cmake --install` 安装可执行文件、桌面启动项和 hicolor 512×512 应用图标；启动项不声明 MIME 文件关联 |

Linux 安装规则不打包 Qt 依赖，仍需匹配的 Qt 运行环境；不是可直接分发的独立安装包。macOS/Windows 依赖部署沿用开发构建脚本。

当前支持 ZIP、TAR／TAR.GZ 浏览、创建与安全解压，具体边界见项目 README。系统图标采用独立的平台留白，macOS ICNS 与 Qt Dock PNG 主体约 81.2%，通用 PNG 与 Windows ICO 约 90%，相关规则见[素材库说明](../../../design/assets/README.md)。此次不注册文件关联，不修改默认打开应用，也不向 macOS 发布文档类型声明。系统双击打开、已运行应用接收文件、安装与卸载关联均留待独立实现和验证；交付了 ZIP 图标文件不表示这些行为已经完成。标题栏和文件列表实际使用的归档图标可在 [统一素材库](../../../design/assets/index.html) 核对。
