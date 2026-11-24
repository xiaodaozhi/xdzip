# Microsoft Store MSIX 构建与提交

[English](../../../market/windows/build.md) | 简体中文

在 GitHub 仓库的 **Settings → Secrets and variables → Actions → Variables** 设置 `XDZIP_STORE_MSIX_NAME`、`XDZIP_STORE_MSIX_PUBLISHER` 和 `XDZIP_STORE_MSIX_PUBLISHER_DISPLAY_NAME`，值取自合作伙伴中心的**产品标识**。任意值缺失时，工作流在构建前停止。

准备商店更新时，先递增 `VERSION.json` 中的 `build`，同步版本文件并提交。在 GitHub 打开 [Windows Store MSIX](../../../../.github/workflows/windows-store-msix.yml)，在 `main` 上选择 **Run workflow**，输入已合入 `main` 的完整提交 SHA。下载 `windows-store-msix-<包版本>-<运行次数>` 工件，其中包含未签名的 `xdzip-windows-x64.msix`，以及构建、测试、运行时校验和程序包检查的回执。

上传前，在 `package-msix.json` 和 `windows-msix.json` 中核对源提交、包版本和 SHA-256，并确认包版本高于商店已发布版本。将 MSIX 上传到合作伙伴中心，等待程序包验证完成，然后更新 Store 一览。

商店字段与素材见[商店材料清单](README.md)。

通过合作伙伴中心操作提交、填写一览和上传材料时，优先使用浏览器扩展插件；扩展不支持所需操作时再使用 computer use 插件。

## 一览验收与认证提交

1. 根据[语言规则和材料清单](README.md)列出本次需要更新的语言地区及其文案、图片来源。确认材料对应本次版本；已有图片仍准确时可以沿用。桌面截图栏使用各语言的四张 1920 × 1080 融合宣传图。
2. 对每个目标地区检查本次要求的说明、新增功能、产品功能、简短描述、关键字及其他字段，并核对四张图片的数量、语言、顺序和对应图片标题。按语言族复用时，确认简体、繁体及其他语言映射正确。
3. 批量导入使用包含全部字段的 CSV。首次更改导入方式时，先选少量地区试验，核对目标变更和已有字段，再扩大范围。此前只包含图片标题行的 CSV 未写入标题，并清空了产品功能和关键字。发现数据丢失时，先恢复并复核，再提交认证。
4. 分批保存并记录已完成的地区、失败项和待处理项。确认保存成功后再转到下一批；操作超时或中断时先重新读取实际状态，再修正并续跑，避免标题和图片错位。
5. 所有目标地区的材料核对完成后，检查程序包版本、验证状态及认证后的发布方式，再提交认证。平台显示一览“完成”不代表本次要求的图片、标题和其他材料均已核对。提交后确认页面进入认证流程；认证通过并开始发布后，才能按实际状态报告发布进度。

采用手动发布时，认证通过后选择 **立即发布**。微软签名并分发该包，随后可以在 Windows 上验证实际安装的商店版。

## 批量更新商店一览

[地区映射表](../../../../specs/marketing/windows-store-locales.json)按素材语言记录已观察到的 337 个地区一览。草稿导出和映射不一致时，先对照语言选择器，再更新映射。生成器同时核对 MSIX 清单中声明的语言和营销规格中的素材语言。

将整个草稿导出为 UTF-8 CSV 并保留，便于恢复。使用已验证的 MSIX 运行[一览生成器](../../../../scripts/prepare-windows-store-listings.py)。生成器保留所有字段行、默认值和它不负责更新的属性。确认营销规格含本次新增功能文案，也可用 `--release-notes` 提供以十种素材语言为键的 JSON。`--materials` 可选，用来对照素材导出的清单校验宣传图。生成计划在 `build/` 下记录变更与输入摘要。

```sh
python scripts/prepare-windows-store-listings.py prepare --baseline build/store-before.csv --package build/xdzip-windows-x64.msix --output build/store-listing-update
python scripts/prepare-windows-store-listings.py verify --baseline build/store-before.csv --intended build/store-listing-update/listing.csv --actual build/store-after.csv
```

将生成的 `listing.csv` 导入草稿。默认更新文案和图片标题，并保留导出中的截图 URL。只更新图片标题时可加 `--images-only`，生成的 CSV 仍包含全部字段行。需要先在少量地区试验时，可用 `--only` 指定地区代码。更换图片时，先为受影响的一览上传已审核的四张宣传图并核对内容和顺序，再重新导出，生成文案和图片标题更新。每次操作使用最新导出作为基线。地区清单不符或缺少图片 URL 时，先核对再生成。生成的 CSV 和 JSON 保存在本地工作目录。

[三个地区的草稿试验](https://github.com/xiaodaozhi/xdzip/issues/440)已验证，全字段 CSV 保留了原有一览字段，修改后的图片标题也成功保存。使用 `--reuse-source-urls` 后，合作伙伴中心为 `fr-ca` 保存了四个不同的新 URL，四张图与 `fr-fr` 相同。`zh-hant-hk` 和 `zh-hans-sg` 导出的 URL 保持原样。将复用 URL 的选项限于草稿试验，并核对实际图片及其顺序。仅凭导出的 URL 无法判断页面最终显示了哪些图片。

导入后重新导出。核对命令会比较整个草稿，包括未上传的地区。发现图片 URL、标题、原有字段或地区有差异时，先在合作伙伴中心查看报告指出的字段，必要时用保存的导出恢复数据，再决定扩大更新范围和提交认证。[微软 MSIX 导入导出说明](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/import-and-export-store-listings)介绍了素材复用及空白单元格的处理方式。

## 评估提交 API

Windows 应用商店提交 API 需要[关联到合作伙伴中心的 Microsoft Entra 应用](https://learn.microsoft.com/en-us/windows/uwp/monetize/create-and-manage-submissions-using-windows-store-services#how-to-associate-an-azure-ad-application-with-your-partner-center-account)，并为应用授予管理员（Manager）角色。若个人开发者帐户尚未关联租户，先向 [Microsoft Store 开发者支持](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account#need-help-contact-us)确认帐户是否支持这项授权，再配置租户或应用。评估 API 期间仍可使用 CSV 流程。

获得访问权限后，从受保护的存储加载 `PARTNER_CENTER_TENANT_ID`、`PARTNER_CENTER_CLIENT_ID` 和 `PARTNER_CENTER_CLIENT_SECRET`，读取已有提交，并使用同一提交的 CSV 导出做对照。若已用官方 CLI 保存响应，可用 `--submission-json` 代替 `--product-id`。

```sh
python scripts/prepare-windows-store-listings.py api-report --export build/store-published.csv --product-id STORE_PRODUCT_ID --submission-id SUBMISSION_ID
```

报告按顺序核对截图描述，同时检查地区、文案及 API 是否返回 `runFullTrust`。启用 API 发布前，由 API 创建并编辑独立草稿，通过写入和回读验证地区清单、图片内容与标题、受限功能用途说明。复用已验证的 MSIX。合作伙伴中心已有待处理草稿时，先执行只读验证，待草稿处理完毕后再做写入试验。顶层 `msstore publish` 命令可能替换待处理草稿。[微软提交 API 指南](https://learn.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions)和 [CLI 命令](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/commands)列出了可用操作。
