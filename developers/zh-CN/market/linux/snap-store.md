# Snap Store 发布与商店素材

[English](../../../market/linux/snap-store.md) | 简体中文

本流程用于 XDZip 的 `xdzip` 商店页面和 Ubuntu 24.04 amd64 strict Snap。其他 Linux 展示素材沿用[宣传素材导出流程](README.md)。在发行议题记录版本、源码提交、包的 SHA-256、商店修订号、实际验收和渠道变更。

## 准备并发布修订

从 `main` 按[正式发行流程](../../engineering/release-runbook.md#触发并核验发行)运行 [Release](https://github.com/xiaodaozhi/xdzip/actions/workflows/release.yml)，选择稳定版标签。Linux 任务复用同一次原生构建的程序和 Qt SDK 生成 amd64 strict Snap，检查 `core24`、锁定依赖、成品加载和离屏启动。正式发布任务沿用 `release-signing` 的审批，在发布 GitHub Release 后直接上传成品到 `latest/stable`。商店审核通过后按下载哈希核对修订号，运行摘要记录源码提交和包的 SHA-256。候选或预发行运行跳过商店发布。

把 `SNAPCRAFT_STORE_CREDENTIALS` 保存在 GitHub 的 `release-signing` 环境中。凭据限定为 `--snaps=xdzip --channels=latest/stable --acls=package_access,package_push,package_release`，并设置到期时间。可用 `gh secret set SNAPCRAFT_STORE_CREDENTIALS --env release-signing --repo xiaodaozhi/xdzip` 从标准输入写入凭据文件的内容。 [凭据导出参数](https://ubuntu.com/docs/snapcraft/latest/reference/commands/export-login/)

如果当前终端已导出旧凭据，请用 `env -u SNAPCRAFT_STORE_CREDENTIALS snapcraft export-login` 开始导出命令，避免旧变量阻止重新认证。

### 手动发布

1. 冻结已批准的源码提交，按 [Snap 验证流程](../../qa/README.md#snap-候选包的运行库检查)使用现有 Linux 部署和 Snap 打包脚本生成候选。针对该包核对安装入口、基础包依赖、清单、许可证通知及对应源码入口。 [发行许可复核](../../qa/release-review.md)
2. 用 `snapcraft upload <安装包路径> --release=latest/stable` 上传已核验的 `.snap`。如果 Canonical 要求人工审核，在[渠道后台](https://snapcraft.io/xdzip/releases)查看结果。审核通过后可重试失败的发布任务，流程会识别已经公开的同一成品。 [Canonical 上传命令](https://ubuntu.com/docs/snapcraft/9/reference/commands/upload/)
3. 如果已上传但尚未发布，用 `snapcraft revisions xdzip` 核对修订，再用 `snapcraft release xdzip <修订号> latest/stable` 发布。用 `snap info xdzip` 核对公开渠道。需要撤回时，先核对渠道回退关系和对已安装用户的影响。 [Canonical 发布命令](https://ubuntu.com/docs/snapcraft/9/reference/commands/release/)

## 准备文案和图片

公开发布后台每个 Snap 只有一套标题、摘要、描述和图片。XDZip 的共享商店页面使用英语（`en-US`），通过[多语言产品文档](../../../../docs/zh-CN/README.md)提供其他语言入口。翻译和本地化宣传素材沿用[十语言 Linux 营销配置](../../../../specs/marketing/linux-store.json)及其导出内容。商店开放语言选项后，先在实际表单和官方 API 核对，再建立逐语言提交。 [Canonical 商店元数据 API](https://dashboard.snapcraft.io/docs/reference/v1/snap.html#managing-snap-metadata)

使用已审阅的当前版文案。摘要控制在 79 个字符以内，描述以实际发行的 Snap 所支持的格式和功能为准。按需更新网站、源码、支持与隐私链接，并核对用户可以访问对应源码说明。发布新修订时保留经审阅的后台文案。

| 后台字段 | XDZip 的提交素材 |
| --- | --- |
| Snap icon | 使用项目的 256 × 256 PNG 图标。后台接受边长 40 至 512 像素的正方形图标，单个文件不超过 256,000 字节。 |
| Images | 最多提交五张使用演示数据的 Linux 实拍图。首张展示起始界面，后续展示主要功能。后台接受 480 × 480 至 4320 × 2160 像素，宽高比介于 1:2 和 2:1，单张不超过 2,000,000 字节。 |
| Featured banner | 上传 `snap-store/featured-banner.png` 中的 1440 × 480 PNG。后台接受 3:1 的 PNG 或 JPEG，尺寸从 720 × 240 至 4320 × 1440，单张不超过 2,000,000 字节。是否在商店推荐位展示由 Canonical 决定。 |

上述尺寸和大小来自当前发布后台。每次准备提交时核对表单中的图片限制。 [Canonical 素材表单源码](https://github.com/canonical/snapcraft.io/tree/main/static/js/publisher/pages/Listing/ListingDetails)

```sh
node scripts/build-linux-store-assets.cjs --output build/linux-store-assets
```

[截图与导出流程](../../engineering/platform-marketing.md#导出和检查)说明依赖和如何加入经核验的 Linux 实拍图。导出目录包含逐语言的 `text/`、品牌素材、一张共享宣传横幅和哈希清单。商店截图使用实际应用图像，品牌插画放在 Featured banner 栏。

## 保存并核验商店页面

首个修订可用后，从发布后台打开 [XDZip Listing](https://snapcraft.io/xdzip/listing)。录入文案和链接，上传图标及图片，调整顺序并预览。保存后刷新页面，核对实际文案和图片。公开渠道可用后，核对[公开页面](https://snapcraft.io/xdzip)及目标渠道。商店资料与渠道变更在不同页面保存。

可以在发布后台维护素材，也可以用 Canonical 的[文字和图片元数据 API](https://dashboard.snapcraft.io/docs/reference/v1/snap.html#managing-snap-metadata)。`snapcraft upload-metadata` 可以从包内同步它支持的字段。在有人工编辑的 Listing 上使用前，先核对会覆盖哪些内容。素材没有变化时沿用已审阅的图片。
