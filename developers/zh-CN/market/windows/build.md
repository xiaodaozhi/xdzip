# Microsoft Store MSIX 构建与提交

[English](../../../market/windows/build.md) | 简体中文

在 GitHub 仓库的 **Settings → Secrets and variables → Actions → Variables** 设置 `XDZIP_STORE_MSIX_NAME`、`XDZIP_STORE_MSIX_PUBLISHER` 和 `XDZIP_STORE_MSIX_PUBLISHER_DISPLAY_NAME`，值取自合作伙伴中心的**产品标识**。任意值缺失时，工作流在构建前停止。

准备商店更新时，先递增 `VERSION.json` 中的 `build`，同步版本文件并提交。在 GitHub 打开 [Windows Store MSIX](../../../../.github/workflows/windows-store-msix.yml)，在 `main` 上选择 **Run workflow**，输入已合入 `main` 的完整提交 SHA。下载 `windows-store-msix-<包版本>-<运行次数>` 工件，其中包含未签名的 `xdzip-windows-x64.msix`，以及构建、测试、运行时校验和程序包检查的回执。

上传前，在 `package-msix.json` 和 `windows-msix.json` 中核对源提交、包版本和 SHA-256，并确认包版本高于商店已发布版本。将 MSIX 上传到合作伙伴中心，更新 Store 一览后提交认证。采用手动发布时，认证通过后选择 **立即发布**。微软签名并分发该包，随后可以在 Windows 上验证实际安装的商店版。

商店字段与素材见[商店材料清单](README.md)。

通过合作伙伴中心操作提交、填写一览和上传材料时，优先使用浏览器扩展插件；扩展不支持所需操作时再使用 computer use 插件。
