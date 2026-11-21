# Qt 界面字体

[English](README.en.md) | 简体中文

Xdzip Design Sans SC 是官方 Noto Sans SC 2.004-H2 的可变字体子集，用于固定界面文案，字重范围为 400–750。Qt 使用由同一 WOFF2 无损导出的 TTF，保留相同字形、字重和文字度量。文件名、目录名、路径等用户数据仍优先使用启动时保存的系统字体，并由 Qt 和平台补充缺失字形。

## 自动生成

仓库只保留[固定来源与生成配置](../../../specs/dependencies/ui-font.json)、生成工具依赖和 OFL 许可。完整上游字体缓存在 `build/toolchain/fonts/<SHA-256>/source.ttf`；TTF、WOFF2 和结果清单只生成到构建目录，不提交到 Git。

`python scripts/desktop.py prepare` 准备共享 Python 工具环境、固定版本的 FontTools/Brotli 和经过 SHA-256 校验的上游字体。随后 `python scripts/desktop.py build` 使用该环境离线生成子集。直接使用 CMake 时，以 `Python3_EXECUTABLE` 指定同一工具环境；每次 `cmake --build` 都会检查当前文案，包括新增和删除的源文件。Qt 字体结果位于 `<构建目录>/generated/fonts/`，资源地址继续为 `:/xdzip/fonts/xdzip-design-sans-sc.ttf`。

生成器收集 `resources/i18n/` 的各语言文案、界面 QML/C++ 固定文字、ASCII 和配置中的保留字符。普通文案只修改原文，不维护另一份字符表。源字体、生成算法、配置、工具版本和排序去重后的字符集合共同确定生成身份；字符集合不变时复用结果，不改写字体或触发额外资源编译。

字体缓存缺失时先执行准备；缓存哈希不符时移除损坏的 `source.ttf` 再准备。上游不支持所需字符时构建失败并列出码位，不静默回退。准备阶段允许下载，构建与打包阶段不下载字体；打包工具检查该次构建的字体结果清单，不重新裁剪。

## 手动准备与检查

手动准备与检查使用 `build/toolchain/desktop-tools` 共享工具环境。尚未创建该环境时先执行 `python3 -m venv build/toolchain/desktop-tools`，然后在仓库根目录执行：

```sh
build/toolchain/desktop-tools/bin/python -m pip install -r design/assets/fonts/requirements.txt
python3 scripts/ui-fonts.py prepare
build/toolchain/desktop-tools/bin/python scripts/ui-fonts.py check
```

已有完整上游字体时，可用 `python scripts/ui-fonts.py prepare --source /path/to/NotoSansSC.ttf` 导入，仍须通过固定哈希检查；准备后可用 `prepare --offline` 检查源缓存。Windows 将 `bin/python` 换成 `Scripts/python.exe`，将 `python3` 换成 `python`。

## 来源与许可

固定来源为 [Google Fonts](https://github.com/google/fonts/tree/a85815a42757630ce188fdad368c2dfc444d4773/ofl/notosanssc) 提交 `a85815a42757630ce188fdad368c2dfc444d4773`，上游 `NotoSansSC[wght].ttf` 的 SHA-256 为 `a3041811a78c361b1de50f953c805e0244951c21c5bd412f7232ef0d899af0da`。原始来源、版本和工具版本见固定配置，生成结果的哈希和实际字符清单见构建目录中的 `manifest.json`。

版权为 Adobe 2014–2021，保留字体名为 `Source`，派生字体采用独立家族名。字体及其派生字体遵循 [OFL-1.1](OFL.txt)，不适用项目 Apache-2.0。产品许可材料保留完整版权及许可。字体统一不代表与历史图片的未知原始字体完全一致，平台抗锯齿和用户数据的实际字形仍有差异。
