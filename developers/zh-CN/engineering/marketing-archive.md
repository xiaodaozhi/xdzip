# 宣传素材打包归档

[English](../../engineering/marketing-archive.md) | 简体中文

Windows、macOS 和 Linux 各自生成完成后，分别打包。导出内容可以仅有底图，也可以包含实际采集语言的截图与融合图。生成和采集方法见 [Windows 流程](../market/windows/capture.md)及 [macOS、Linux 流程](platform-marketing.md)。

```sh
python3 scripts/package-marketing-assets.py --platform windows --input build/windows-marketing
python3 scripts/package-marketing-assets.py --platform macos --input build/macos-marketing
python3 scripts/package-marketing-assets.py --platform linux --input build/linux-marketing
```

`--input` 应改为各平台实际生成的目录，也支持 `package-windows-store-materials.py` 导出的 Windows 截图包。压缩包默认保存在 `build/marketing-packages/`，可用 `--output-dir` 指定 `build/`、`dist/` 或仓库外的其他目录。包内保留原语言目录、截图、底图、插图、文案、截图来源、清单和校验文件。

文件名格式为 `xdzip-<平台>-marketing-<backgrounds|captures>-<清单摘要>.zip`。`backgrounds` 表示没有记录的截图，`captures` 表示至少一种语言有截图，不代表十种语言都已采集。摘要用于标识本次导出的清单；脚本不会覆盖已有同名压缩包。打包前检查平台、文件清单和各文件 SHA-256，完成后检查 ZIP 文件列表、摘要和 CRC，并输出压缩包 SHA-256，供归档或传输时核对。

将所得 ZIP 归档到指定 Google Drive 账号的 `xdzip/assets` 文件夹。连接了多个账号时，使用标记为主账号的连接。打包命令只在本地生成压缩包，不会自动上传；可让 Codex 使用已连接的 Google Drive 应用上传，也可通过 Drive 上传。上传后核对账号、文件夹、文件名和大小，不覆盖不同内容的已有文件。上传接口提供摘要时，可与本地输出的压缩包摘要比较。不要将仅含底图的压缩包描述为真实截图，也不要把生成结果提交到 Git。

运行 `python3 scripts/test-package-marketing-assets.py` 可定向检查打包流程。
