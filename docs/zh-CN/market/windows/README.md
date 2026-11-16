# Microsoft Store 商店材料

[English](../../../market/windows/README.md) | 简体中文

[可复制的商店文案](listing.md)包含说明、十项产品功能、补充字段、七个关键字、版权、开发者信息、链接和图片说明。`zh-CN` 使用本目录，`en-US` 使用[英文文案](../../../market/windows/listing.md)。

本套材料对应 MSIX **1.0.27.0**，包标识为 `xiaodaozhi.xdzip`，来源提交为 `90916a1992188e6e7e5c6b3d91695861ee1bd2d2`，不宣传当前开发分支中后续增加的格式。[支持政策](../../engineering/support-policy.md)仍为 Windows 11 x64，清单中 Windows 10 的技术最低版本不代表扩大支持承诺。商店版本由 Microsoft Store 管理更新，本候选不注册现代 Explorer 右键菜单。

## 文本字段

| 合作伙伴中心字段 | 材料或填写决定 |
| --- | --- |
| 产品名称 | 两种语言均为 `xdzip` |
| 说明 | 使用 [listing.md](listing.md) 中的纯文本块，不超过 10,000 个字符 |
| 此版本的新增功能 | 首次提交留空，不粘贴内部测试记录 |
| 产品功能 | 十项分别填写，每项不超过 200 个字符，页面自动添加项目符号 |
| 短标题 | `xdzip` |
| 语音标题 | 留空，本次为桌面应用，不使用 Xbox 语音标题 |
| 简短描述 | 使用本地化文案，控制在建议的 270 个字符以内 |
| 关键字 | 七项分别添加，每项不超过 40 个字符，空格分词总数不超过 21 |
| 版权和商标信息 | 使用不超过 200 个字符的本地化版权说明 |
| 其他许可条款 | 留空，不额外制定商店专用补充条款 |
| 开发者 | `xiaodaozhi` |
| 网站、支持和隐私政策 | 使用 [listing.md](listing.md) 中的专用链接，不把 URL 写进说明正文 |

补充条款留空不会改变应用已有的 Apache-2.0 许可及第三方通知。本材料包不决定商店定价，也不代替法律条款的接受操作。

## 应用截图与融合宣传图

需要重新截图时，使用[原生 PNG 自动截图脚本](capture.md)。每种语言运行一次，即可自动准备并保存四个场景，无需手动切换界面或逐张确认。

每种语言均提供四张真实应用界面，使用自有演示文件，程序构建对应上方标明的候选。**1027 × 624 PNG** 保留完整的默认窗口，包括标题栏和状态栏。光标清理使用同一静止界面的两次捕获，导入器核对中部界面不变，并验证导出像素均来自实际捕获。

桌面截图栏建议使用 **1366 × 768 商店截图**，默认窗口按 1:1 居中放置，周围添加中性留白。融合版把同一窗口放入已有的蓝色品牌版式，配以标题和说明。1080p 版保持窗口的捕获尺寸，4K 版将整体版式放大两倍，不代表原生 4K 截图。微软截图指引建议截图栏不要叠加营销文案。

| 场景 | 默认窗口 | 商店截图 | 1920 × 1080 宣传图 | 3840 × 2160 宣传图 |
| --- | --- | --- | --- | --- |
| 浏览目录与选择文件 | [PNG](images/screenshots/01-browse.png) | [PNG](images/store-screenshots/01-browse.png) | [PNG](images/combined/01-browse-1920x1080.png) | [PNG](images/combined/01-browse-3840x2160.png) |
| 打开与创建入口 | [PNG](images/screenshots/02-create.png) | [PNG](images/store-screenshots/02-create.png) | [PNG](images/combined/02-create-1920x1080.png) | [PNG](images/combined/02-create-3840x2160.png) |
| 按需图像预览 | [PNG](images/screenshots/03-preview.png) | [PNG](images/store-screenshots/03-preview.png) | [PNG](images/combined/03-preview-1920x1080.png) | [PNG](images/combined/03-preview-3840x2160.png) |
| 多标签与网格浏览 | [PNG](images/screenshots/04-workspace.png) | [PNG](images/store-screenshots/04-workspace.png) | [PNG](images/combined/04-workspace-1920x1080.png) | [PNG](images/combined/04-workspace-3840x2160.png) |

[listing.md](listing.md) 提供四张界面的说明文字。起始页展示创建入口，不表示完成了一次创建任务。本套截图采用浅色主题，文案中的外观选项仍以候选功能为准。

## 功能宣传图

原有矢量插画继续用于宣传，明确标注为**功能示意图**。商店截图栏使用上方的应用界面图片。

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

[设计画廊](../../../../design/hifi/sources/windows-store-artwork.html)是高保真入口。可编辑的矢量构图位于 [build-windows-store-assets.cjs](../../../../scripts/build-windows-store-assets.cjs)，本地化文案、配色与输出画布参数位于 [windows-store.json](../../../../specs/marketing/windows-store.json)，[素材清单](../../../../specs/marketing/windows-store-assets.json)保存源图和 PNG 的摘要，[截图素材清单](../../../../specs/marketing/windows-store-captures.json)保存候选身份、两份来源摘要、完整窗口尺寸、光标清理范围、图片说明和解码像素摘要。固定输出画布只用于商店图片，不改变 Qt 窗口逻辑尺寸，也不把屏幕缩放写入产品。

替换截图前，先审阅两种语言中使用自有演示文件捕获的画面。对每个静止界面，分别在指针位于状态栏附近、标题栏附近时捕获，前者按 `<locale>-<scene>.jpg` 命名，例如 `zh-CN-01-browse.jpg`，后者按 `<locale>-<scene>-bottom.jpg` 命名。导入器取第二张中的干净底部补齐，所有界面像素保留实际捕获内容。原始捕获放在 `build/`，将目录传给 [import-windows-store-captures.cjs](../../../../scripts/import-windows-store-captures.cjs)，导入器核对全部八对输入后写入 PNG。公开使用前仍需审阅完整图片。

环境中具备 Node.js 和 `sharp` 后执行：

```sh
node scripts/build-windows-store-assets.cjs
node scripts/build-windows-store-assets.cjs --check
python scripts/check-docs.py
```

替换为已审阅的截图时，先执行 `node scripts/import-windows-store-captures.cjs build/approved-store-captures`，再执行上方命令。输入尺寸须与 `screenshot_artwork.default_capture_size` 一致，清理范围由 `cursor_cleanup_height` 指定。截图变更后，同时重新生成两种语言、商店留白版与两种宣传分辨率。

检查覆盖字段长度、双语条目对应、PNG 尺寸、不透明度、文件大小、完整素材清单及源图和输出摘要，同时核对商店留白版按 1:1 保留全部窗口像素、文案是否过期、横幅是否含文字。修改目标候选或文案后，请同时重新生成两种语言。

## 官方参考

[商店一览字段说明](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info)规定说明、功能和补充字段，[MSIX 图片说明](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images)规定截图与品牌图的用途。当前页面可显示比文档更多的上传位置；本套每种语言提供四张界面截图及四组功能示意图，不因出现某类设备字段就推定产品支持该设备。
