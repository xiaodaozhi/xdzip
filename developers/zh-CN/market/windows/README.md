# Microsoft Store 商店材料

[English](../../../market/windows/README.md) | 简体中文

[构建和提交商店包](build.md)。

[可复制的商店文案](listing.md)包含说明、十项产品功能、补充字段、七个关键字、版权、开发者信息、链接和图片说明。`zh-CN` 使用本目录，`en-US` 使用[英文文案](../../../market/windows/listing.md)。

商店文案对应 MSIX **1.0.30.0**、应用版本 0.2.1、构建号 30。[支持政策](../../../../docs/zh-CN/engineering/support-policy.md)仍为 Windows 11 x64，清单中 Windows 10 的技术最低版本不代表扩大支持承诺。商店版本由 Microsoft Store 管理更新，商店构建声明现代和传统 Explorer 菜单。现代菜单要求已安装包具有微软 Store 签名。

使用[本机双语截图流程](capture.md)，以自有演示文件拍摄新版界面，自动生成商店图、宣传图和 README 图片。[通用捕获模块](../../engineering/windows-screenshots.md)还可用于其他 Windows 场景。

## 文本字段

| 合作伙伴中心字段 | 材料或填写决定 |
| --- | --- |
| 产品名称 | 两种语言均为 `xdzip` |
| 说明 | 使用 [listing.md](listing.md) 中的纯文本块，不超过 10,000 个字符 |
| 此版本的新增功能 | 使用 [listing.md](listing.md#此版本的新增功能) 中的本地化文本 |
| 产品功能 | 十项分别填写，每项不超过 200 个字符，页面自动添加项目符号 |
| 短标题 | `xdzip` |
| 排序标题 | 留空，无需另设字母排序名称 |
| 语音标题 | 留空，本次为桌面应用，不使用 Xbox 语音标题 |
| 简短描述 | 使用本地化文案，控制在建议的 270 个字符以内 |
| 关键字 | 七项分别添加，每项不超过 40 个字符，空格分词总数不超过 21 |
| 版权和商标信息 | 使用不超过 200 个字符的本地化版权说明 |
| 其他许可条款 | 留空，不额外制定商店专用补充条款 |
| 开发者 | `xiaodaozhi` |
| 网站、支持和隐私政策 | 使用 [listing.md](listing.md) 中的专用链接，不把 URL 写进说明正文 |

补充条款留空不会改变应用已有的 Apache-2.0 许可及第三方通知。本材料包不决定商店定价，也不代替法律条款的接受操作。

## 应用截图与融合宣传图

[本机截图流程](capture.md)能自动准备每种语言的四个场景，生成处理后的 PNG 预览供目视确认。

每种语言均提供四张真实应用界面，使用自有演示文件，截图采用默认窗口尺寸。[截图清单](../../../../specs/marketing/windows-store-captures.json)记录程序包摘要、原生尺寸和导出尺寸。生成过程裁掉顶部一行像素，清理四角旧桌面底色，并沿圆角内侧绘制一像素灰蓝色细边框。边框不改变图片尺寸或外侧透明度。圆角和边框区域以外的像素不变，不重绘文字或缩放界面。[原始素材](../../../../design/assets/marketing/windows-store-native)保留与原生捕获逐字节一致的 PNG。根目录中英文 README 分别使用对应语言带边框的 `01-browse` 图片。

提交 Microsoft Store 更新时，按下表顺序将 **1920 × 1080 融合宣传图**上传到桌面截图栏。它将实机界面嵌入蓝色版式，配以本地化文案、细边框和柔和阴影。较大的界面只缩小一次以适配版面。3840 × 2160 版保留界面原生尺寸，供其他宣传场景使用。

| 场景 | 默认窗口 | 纯界面图 | 1920 × 1080 商店上传图 | 3840 × 2160 宣传图 |
| --- | --- | --- | --- | --- |
| 浏览目录与选择文件 | [PNG](images/screenshots/01-browse.png) | [PNG](images/store-screenshots/01-browse.png) | [PNG](images/combined/01-browse-1920x1080.png) | [PNG](images/combined/01-browse-3840x2160.png) |
| 创建窗口与 ZIP 设置 | [PNG](images/screenshots/02-create.png) | [PNG](images/store-screenshots/02-create.png) | [PNG](images/combined/02-create-1920x1080.png) | [PNG](images/combined/02-create-3840x2160.png) |
| 按需图像预览 | [PNG](images/screenshots/03-preview.png) | [PNG](images/store-screenshots/03-preview.png) | [PNG](images/combined/03-preview-1920x1080.png) | [PNG](images/combined/03-preview-3840x2160.png) |
| 多标签与网格浏览 | [PNG](images/screenshots/04-workspace.png) | [PNG](images/store-screenshots/04-workspace.png) | [PNG](images/combined/04-workspace-1920x1080.png) | [PNG](images/combined/04-workspace-3840x2160.png) |

[listing.md](listing.md) 提供四张界面的说明文字。创建窗口展示三张演示图片和 ZIP 选项，没有启动压缩任务。本套截图采用浅色主题，文案中的外观选项仍以候选功能为准。

## 功能宣传图

原有矢量插画用于其他宣传场景，明确标注为**功能示意图**。

| 内容 | 1920 × 1080 | 3840 × 2160 |
| --- | --- | --- |
| 浏览与选择性解压 | [PNG](images/promotional/01-browse-1920x1080.png) | [PNG](images/promotional/01-browse-3840x2160.png) |
| 创建与受支持的密码保护 | [PNG](images/promotional/02-create-1920x1080.png) | [PNG](images/promotional/02-create-3840x2160.png) |
| 按需预览内容 | [PNG](images/promotional/03-preview-1920x1080.png) | [PNG](images/promotional/03-preview-3840x2160.png) |
| 多归档与外观选择 | [PNG](images/promotional/04-workspace-1920x1080.png) | [PNG](images/promotional/04-workspace-3840x2160.png) |

每张图均有不超过 200 个字符的本地化说明。请使用对应语言的图片，不把英文宣传图称为中文界面的截图。

## 徽标、头像和商店品牌图

这里提供的是 xdzip 的产品素材，不是 Microsoft 企业标识，也不表示获得认证。小尺寸图标直接复用已有矢量品牌，不重新设计标识。无语言差异的图标和无文字横幅由两种语言共用。

| 字段或用途 | 文件 | 说明 |
| --- | --- | --- |
| 应用磁贴图标 | [300 × 300](../../../market/windows/images/branding/app-icon-300.png) | Windows 10/11 商店布局建议使用的图标覆盖项 |
| 小尺寸图标覆盖项 | [150 × 150](../../../market/windows/images/branding/app-icon-150.png)、[71 × 71](../../../market/windows/images/branding/app-icon-71.png) | 与当前页面字段一致 |
| 可复用头像 | [512 × 512](../../../market/windows/images/branding/avatar-512.png)、[1024 × 1024](../../../market/windows/images/branding/avatar-1024.png) | 补充品牌导出，不是当前页面的独立字段 |
| 招贴画 | [720 × 1080](images/branding/poster-720x1080.png)、[1440 × 2160](images/branding/poster-1440x2160.png) | 实际像素比例为 **2:3**，尽管页面标注为“9:16” |
| 方形品牌图 | [1080 × 1080](images/branding/box-art-1080.png)、[2160 × 2160](images/branding/box-art-2160.png) | 使用中文标语，桌面应用的可选素材 |
| 无文字主角横幅 | [1920 × 1080](../../../market/windows/images/branding/hero-1920x1080.png)、[3840 × 2160](../../../market/windows/images/branding/hero-3840x2160.png) | 不含产品名称、文字、软件界面或设备外框 |

招贴画和方形图把名称及主要标识放在上方三分之二区域。横幅的主标识位于画面中部及底部叠加区域上方。所有图片均为 PNG，每张小于 5 MB，满足页面图标 5 MB、大图 50 MB 的限制。

本包只声明 `Windows.Desktop`，因此不提供或启用 Xbox 专用的 584 × 800 品牌图、带标题主角图和重点推广方图，不宣传 Xbox、全息或移动设备支持。预告片是可选项，本套图片材料不含视频。

## 编辑与重新生成

[设计画廊](../../../../design/hifi/sources/windows-store-artwork.html)是高保真入口。可编辑构图位于 [build-windows-store-assets.cjs](../../../../scripts/build-windows-store-assets.cjs)，本地化文案、配色与输出画布参数位于 [windows-store.json](../../../../specs/marketing/windows-store.json)，[素材清单](../../../../specs/marketing/windows-store-assets.json)保存 PNG 及源图、输出摘要，包含原生输入和两张 README 图片；[截图素材清单](../../../../specs/marketing/windows-store-captures.json)保存候选身份、原生截图与捕获回执摘要、客户区裁切、尺寸、DPI、图片说明和圆角处理参数。固定输出画布只用于商店图片，不改变 Qt 窗口逻辑尺寸，也不把屏幕缩放写入产品。

通过[本地刷新命令](capture.md)捕获两种语言的原生截图，生成隔离预览并按 100% 比例审阅 PNG，确认后再应用变更。刷新命令使用 [import-windows-store-captures.cjs](../../../../scripts/import-windows-store-captures.cjs) 校验两个完整捕获目录，核对全部八张无损 PNG 的候选身份、摘要及客户区与窗框像素的精确对应。JPEG、不完整或身份不符的截图会被拒绝。更换候选时，先更新营销规格中的基线。

安装 Node.js 后，首次使用时安装固定版本的图片依赖。脚本会自动从本地工具链查找，无需设置 `NODE_PATH`：

```sh
npm install --prefix build/toolchain/windows-store-tools --no-save --no-package-lock sharp@0.34.5
```

如需从两个已审阅的截图目录手动导入并生成素材，执行：

```sh
node scripts/import-windows-store-captures.cjs build/captures-en-US build/captures-zh-CN
node scripts/build-windows-store-assets.cjs
node scripts/build-windows-store-assets.cjs --check
node scripts/test-windows-store-images.cjs
python scripts/check-docs.py
python scripts/package-windows-store-materials.py --output dist/windows-store-materials
```

导出路径必须尚不存在。打包脚本生成目录和 ZIP，按 `en-US`、`zh-CN` 分开，包含 62 张上传或宣传 PNG、可复制文案、截图说明 CSV、补充字段、审核备注和 SHA-256 清单。原始捕获与透明抠图留在仓库中，不放入上传包。材料包不含产品二进制或源码压缩包。

检查覆盖字段长度、双语条目对应、PNG 尺寸、透明度、文件大小、完整素材清单及源图和输出摘要，同时核对不透明界面像素、透明圆角的背景、商店版与 4K 版的原生 1:1 放置、文案时效和无文字横幅。修改目标候选或文案后，同时重新生成两种语言。

## 审核字段

[审核说明](reviewer-notes.md)提供双语 `runFullTrust` 用途说明与认证测试说明，可复制到相应栏目。本材料包不表示受限功能已经获批或已提交认证。定价、年龄分级和账户级问卷仍属于合作伙伴中心的其他提交栏目。

## 官方参考

[商店一览字段说明](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info)规定说明、功能和补充字段，[MSIX 图片说明](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images)规定截图与品牌图的用途。当前页面可显示比文档更多的上传位置；本套每种语言提供四张界面截图及四组功能示意图，不因出现某类设备字段就推定产品支持该设备。
