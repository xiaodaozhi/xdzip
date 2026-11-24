# Microsoft Store MSIX 构建与提交

[English](../../../market/windows/build.md) | 简体中文

在 GitHub 仓库的 **Settings → Secrets and variables → Actions → Variables** 设置 `XDZIP_STORE_MSIX_NAME`、`XDZIP_STORE_MSIX_PUBLISHER` 和 `XDZIP_STORE_MSIX_PUBLISHER_DISPLAY_NAME`，值取自合作伙伴中心的**产品标识**。任意值缺失时，工作流在构建前停止。

准备商店更新时，先递增 `VERSION.json` 中的 `build`，同步版本文件并提交。在 GitHub 打开 [Windows Store MSIX](../../../../.github/workflows/windows-store-msix.yml)，在 `main` 上选择 **Run workflow**，输入已合入 `main` 的完整提交 SHA。下载 `windows-store-msix-<包版本>-<运行次数>` 工件，其中包含未签名的 `xdzip-windows-x64.msix`，以及构建、测试、运行时校验和程序包检查的回执。

上传前，在 `package-msix.json` 和 `windows-msix.json` 中核对源提交、包版本和 SHA-256，并确认包版本高于商店已发布版本。将 MSIX 上传到合作伙伴中心，等待程序包验证完成，然后更新 Store 一览。

商店字段与素材见[商店材料清单](README.md)。

通过合作伙伴中心操作提交、填写一览和上传材料时，优先使用浏览器扩展插件；扩展不支持所需操作时再使用 computer use 插件。

## 一览验收与认证提交

1. 根据[语言规则和材料清单](README.md)列出本次需要更新的语言地区及其文案、图片来源。确认材料对应本次版本；已有图片仍准确时可以沿用。桌面截图栏使用各语言的四张 1920 × 1080 融合宣传图。
2. 对每个目标地区检查本次要求的说明、新增功能、产品功能、简短描述、关键字及其他字段。核对素材语言，并将四张桌面截图固定排为 **1. 起始页 → 2. 创建压缩包 → 3. 图片预览 → 4. 多归档工作区**，标题与图片逐张对应。按语言族复用时，确认简体、繁体及其他语言映射正确。
3. 批量导入使用包含全部字段的 CSV。首次更改导入方式时，先选少量地区试验，核对目标变更和已有字段，再扩大范围。此前只包含图片标题行的 CSV 未写入标题，并清空了产品功能和关键字。发现数据丢失时，先恢复并复核，再提交认证。
4. 分批保存并记录已完成的地区、失败项和待处理项。确认保存成功后再转到下一批；操作超时或中断时先重新读取实际状态，再修正并续跑，避免标题和图片错位。
5. 最后一次保存或导入后，按下文对本次草稿执行回读校验。覆盖全部地区且命令返回 `0` 后才能提交认证。缺少地区、读取失败、图片数量或顺序不符、图片或标题不匹配时，停止提交。校验通过后检查程序包版本、验证状态和认证后的发布方式，再提交认证。提交后确认页面进入认证流程，后续按实际状态报告进度。

采用手动发布时，认证通过后选择 **立即发布**。微软签名并分发该包，随后可以在 Windows 上验证实际安装的商店版。

## 批量更新商店一览

[地区映射表](../../../../specs/marketing/windows-store-locales.json)按素材语言记录已观察到的 337 个地区一览。草稿导出和映射不一致时，先对照语言选择器，再更新映射。生成器同时核对 MSIX 清单中声明的语言和营销规格中的素材语言。

将整个草稿导出为 UTF-8 CSV 并保留，便于恢复。使用已验证的 MSIX 运行[一览生成器](../../../../scripts/prepare-windows-store-listings.py)。生成器保留所有字段行、默认值和它不负责更新的属性。确认营销规格含本次新增功能文案，也可用 `--release-notes` 提供以十种素材语言为键的 JSON。`--materials` 可选，用来对照素材导出的清单校验宣传图。生成计划在 `build/` 下记录变更与输入摘要。

```sh
python scripts/prepare-windows-store-listings.py prepare --baseline build/store-before.csv --package build/xdzip-windows-x64.msix --output build/store-listing-update
python scripts/prepare-windows-store-listings.py verify --baseline build/store-before.csv --intended build/store-listing-update/listing.csv --actual build/store-after.csv --product-id STORE_PRODUCT_ID --submission-id DRAFT_SUBMISSION_ID
```

将生成的 `listing.csv` 导入草稿。默认更新文案和图片标题，并保留导出中的截图 URL。只更新图片标题时可加 `--images-only`，生成的 CSV 仍包含全部字段行。需要先在少量地区试验时，可用 `--only` 指定地区代码。更换图片时，先为受影响的一览上传已审核的四张宣传图并核对内容和顺序，再重新导出，生成文案和图片标题更新。每次操作使用最新导出作为基线。地区清单不符或缺少图片 URL 时，先核对再生成。生成的 CSV 和 JSON 保存在本地工作目录。

[三个地区的草稿试验](https://github.com/xiaodaozhi/xdzip/issues/440)已验证，全字段 CSV 保留了原有一览字段，修改后的图片标题也成功保存。使用 `--reuse-source-urls` 后，合作伙伴中心为 `fr-ca` 保存了四个不同的新 URL，四张图与 `fr-fr` 相同。`zh-hant-hk` 和 `zh-hans-sg` 导出的 URL 保持原样。将复用 URL 的选项限于草稿试验，并核对实际图片及其顺序。仅凭导出的 URL 无法判断页面最终显示了哪些图片。

导入时，将四张图片引用及对应标题分别填入 `DesktopScreenshot1` 至 `4`、`DesktopScreenshotCaption1` 至 `4` 的同一编号。导入后重新导出，用 `verify` 比较全部 CSV 字段，并通过 MSIX API 回读同一草稿，覆盖本次未上传的地区。CSV 中的槽位可能保持原样，而后台预览顺序已经改变。对在合作伙伴中心编辑的草稿，如果发现乱序，在对应地区拖放排序，保存后重新打开核对，再用最新导出重跑校验。[微软 MSIX 导入导出说明](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/import-and-export-store-listings)和[截图排序说明](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images)介绍了这些操作。

## 评估提交 API

Windows 应用商店提交 API 需要[关联到合作伙伴中心的 Microsoft Entra 应用](https://learn.microsoft.com/en-us/windows/uwp/monetize/create-and-manage-submissions-using-windows-store-services#how-to-associate-an-azure-ad-application-with-your-partner-center-account)，并为应用授予管理员（Manager）角色。若个人开发者帐户尚未关联租户，先向 [Microsoft Store 开发者支持](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account#need-help-contact-us)确认帐户是否支持这项授权，再配置租户或应用。评估 API 期间仍可使用 CSV 流程。

为 `verify` 和 `api-report` 从受保护的存储加载 `PARTNER_CENTER_TENANT_ID`、`PARTNER_CENTER_CLIENT_ID` 和 `PARTNER_CENTER_CLIENT_SECRET`。通过 API 更新时，以写入前准备的全量目标 CSV 为参照，在各地区的 `baseListing.images` 数组中按目标顺序写入四条桌面 `Screenshot` 记录。提交认证前回读同一草稿。

```sh
python scripts/prepare-windows-store-listings.py api-report --export build/store-listing-update/listing.csv --product-id STORE_PRODUCT_ID --submission-id DRAFT_SUBMISSION_ID
```

两个命令都按 API 实际返回的数组顺序核对数量、上传文件名和对应标题。素材文件名统一使用 `01-welcome-1920x1080.png`、`02-create-1920x1080.png`、`03-preview-1920x1080.png` 和 `04-workspace-1920x1080.png`。API 提供图片元数据。更换素材时，还须对照已审核的素材核对 PNG 内容和语言。诊断时可用 `--submission-json` 代替 `--product-id` 回放原始 MSIX GET 响应。发布验收以最后一次编辑后的实时回读为准。

报告还检查地区、文案及 API 是否返回 `runFullTrust`。启用 API 发布前，在由 API 创建并编辑的独立草稿内复用已验证的 MSIX，通过写入和回读验证图片内容、标题、顺序和受限功能用途说明。合作伙伴中心已有待处理草稿时，先执行只读验证，待草稿处理完毕后再做写入试验。顶层 `msstore publish` 命令可能替换待处理草稿。[微软提交 API 指南](https://learn.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions)和 [CLI 命令](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/commands)列出了可用操作。
