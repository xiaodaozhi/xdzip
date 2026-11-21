# macOS 和 Linux 宣传素材

[English](../../engineering/platform-marketing.md) | 简体中文

此流程生成可用于官网或其他展示场景的截图、底图和宣传图，不会发布素材，也不声称符合特定商店的上架要求。两个平台沿用 [Windows 素材](../market/windows/README.md)的视觉构图，分别维护 [macOS 文案](../../../specs/marketing/macos-store.json)和 [Linux 文案](../../../specs/marketing/linux-store.json)。当前与 Windows 一致，覆盖[产品语言清单](../../../resources/ui-languages.json)中的十种语言和浏览、创建、预览、工作区四个场景。

## 准备图片生成环境

需要 Node.js 及与 Windows 生成器相同的固定版本 `sharp`。请为目标语言安装适合的系统字体，中文、日文和韩文需要相应地区的字形字体，并在使用前检查渲染结果。在共享本地工具链中安装一次：

```sh
npm install --prefix build/toolchain/windows-store-tools --no-save --no-package-lock sharp@0.34.5
```

在仓库根目录执行以下任一或两条命令，即可在没有截图时生成底图和插图。输出目录必须不存在。

```sh
node scripts/build-macos-store-assets.cjs --output build/macos-marketing
node scripts/build-linux-store-assets.cjs --output build/linux-marketing
```

省略 `--output` 时，结果保存到 `build/<平台>-store-assets/` 下的时间戳目录。增加 `--4k` 可额外生成 3840 × 2160 底图、插图和融合图。素材包含文案、带截图框的底图、功能插图、图标、头像、海报、方图和横幅。不提供截图时，不生成截图和融合图。

## 截取真实应用窗口

在目标操作系统上使用实际的 xdzip 构建。记录器默认采集十种语言，可用 `--languages ja-JP fr-FR` 仅选择所需语言。逐个准备对应语言和场景，并使用不含敏感信息的演示数据。检查窗口内容正确且没有遮挡。macOS 的交互截图使用 `screencapture` 并提示选择窗口。Linux 使用 `gnome-screenshot`，缺少时先安装，在倒计时内激活目标窗口。场景准备和选窗由操作人员完成，此流程不会在其他平台运行 Windows 界面自动化。

| 场景 | 应用中需准备的内容 |
| --- | --- |
| `01-browse` | 浏览压缩包目录树，选择文件并显示属性。 |
| `02-create` | 为选中文件打开创建压缩包窗口，显示 ZIP 设置。 |
| `03-preview` | 在文件列表和属性旁显示图像预览。 |
| `04-workspace` | 在标签页中打开两个压缩包，以网格视图显示选中文件。 |

先计算程序的 SHA-256，macOS 使用 `shasum -a 256 /path/to/xdzip`，Linux 使用 `sha256sum /path/to/xdzip`。替换下列平台、路径和摘要占位符：

```sh
python3 scripts/capture-platform-store-assets.py --platform macos --executable /path/to/xdzip --expected-sha256 LOWERCASE_SHA256 --interactive --output build/macos-captures
python3 scripts/capture-platform-store-assets.py --platform linux --executable /path/to/xdzip --expected-sha256 LOWERCASE_SHA256 --interactive --output build/linux-captures
```

macOS 请指定 `.app/Contents/MacOS/` 中的可执行文件。记录器读取其 `--version-json` 身份，截图前后检查摘要，并记录操作系统、架构、场景、语言、PNG 尺寸及摘要。可使用 `--source-revision` 记录与程序短提交号匹配的完整提交号。记录的身份属于所选程序，不能证明截图来自哪个窗口；操作人员必须核对截图主体和场景。

也可先在目标平台截图，分别放入所选语言目录，例如 `ja-JP/`、`fr-FR/`，文件名为 `01-browse.png`、`02-create.png`、`03-preview.png`、`04-workspace.png`，然后将 `--interactive` 换成 `--from-directory /path/to/images`。仍需在对应系统使用实际构建运行记录器。回执将这些图片标记为操作人员提供，不能证明其拍摄时间、地点或对应程序。

## 导出和检查

```sh
node scripts/build-macos-store-assets.cjs --captures build/macos-captures --output build/macos-marketing-with-screenshots
node scripts/build-linux-store-assets.cjs --captures build/linux-captures --output build/linux-marketing-with-screenshots
```

导出器检查平台、每种已采集语言的完整四场景、截图摘要和尺寸，并拒绝覆盖已有输出。各语言目录中，`screenshots/` 是 PNG 原图，`backgrounds/` 是可复用底图，`promotional/` 是已采集语言的融合图，`feature-illustrations/`、`branding/` 和 `text/` 分别包含其他图形与文案。`capture-source/` 保留原始回执与 PNG，供再次导出使用。`manifest.json` 记录输出用途和来源身份，`SHA256SUMS.txt` 覆盖包括清单在内的所有导出文件。截图不会被放大，使用前应检查图片和来源。不要提交截图或导出结果。

执行 `python3 scripts/test-platform-store-captures.py` 和 `node scripts/test-platform-store-assets.cjs` 可定向检查截图记录和渲染。渲染测试使用合成素材，不是真实应用截图。
