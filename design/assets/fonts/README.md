# HTML 与 Qt 共用界面字体

`xdzip-design-sans-sc.woff2` 是官方 **Noto Sans SC** 的小型可变字体子集，家族名为 **Xdzip Design Sans SC**。它用于固定 HTML 设计稿的字形、字重和文字宽度，覆盖当前界面、示例数据、ASCII 及常用界面标点，支持 400–750 的真实可变字重。文件由 `build-subset.py` 生成，不能手工编辑。

Qt 产品使用由同一 WOFF2 无损导出的 TTF，约 297 KiB。`export-qt.py` 保持字形、可变字重和度量不变，`UiFont.h` 在应用与测试加载相同家族；子集以外的归档名称仍由系统回退，不宣称覆盖所有语言。高保真原稿是生成图片，无法确认其中的原始字体；统一字体不能据此宣称字形与原稿完全相同，平台抗锯齿仍有差异。

## 文件

| 文件 | 用途 |
| --- | --- |
| `fonts.css` | `@font-face` 声明；界面样式另行设置家族名 |
| `xdzip-design-sans-sc.woff2` | 实际用于 HTML 的离线字体 |
| `xdzip-design-sans-sc.ttf` | Qt 原生运行字体，与 WOFF2 同源，SHA-256 `2cc30875231bb4f611863ecb87fc33fa73e5537badb3e58790651dbb15f3d173` |
| `export-qt.py` | 导出 TTF；`--check` 检查是否与当前 WOFF2 完全对应 |
| `OFL.txt` | 上游原文版权及 SIL Open Font License 1.1 |
| `manifest.json` | 固定来源、原始及子集 SHA-256、覆盖字符、字重和工具版本 |
| `validation.json` | 当前子集在 Chromium 中加载的实际字体及重复生成证据 |
| `build-subset.py` | 从固定的上游 TTF 生成子集，或检查当前界面字符覆盖 |
| `requirements.txt` | 生成和检查工具的固定版本；不属于产品运行依赖 |

## 来源与许可

字体来源是 [Google Fonts `ofl/notosanssc`](https://github.com/google/fonts/tree/a85815a42757630ce188fdad368c2dfc444d4773/ofl/notosanssc)，固定提交 `a85815a42757630ce188fdad368c2dfc444d4773`。上游文件 `NotoSansSC[wght].ttf` 的 SHA-256 为 `a3041811a78c361b1de50f953c805e0244951c21c5bd412f7232ef0d899af0da`，内部版本为 `2.004-H2`。Google Fonts 元数据记录的原始来源为 `notofonts/noto-cjk` 提交 `523d033d6cb47f4a80c58a35753646f5c3608a78`。

版权为 Adobe 2014–2021；保留字体名是 `Source`。子集采用单独家族名，保留上游版权和 OFL 元数据。**此字体及其派生字体遵循 OFL-1.1，不能改为项目的 Apache-2.0 许可证。** 分发字体时携带完整 `OFL.txt` 和版权说明；离线 HTML 嵌入字体时也必须包含完整许可文本。

## 生成与检查

下载固定来源的原始字体到临时路径；17.8 MB 的上游 TTF 不放入仓库。示例下载地址：

```text
https://raw.githubusercontent.com/google/fonts/a85815a42757630ce188fdad368c2dfc444d4773/ofl/notosanssc/NotoSansSC%5Bwght%5D.ttf
```

安装工具后，在仓库根目录执行：

```sh
python -m pip install -r design/assets/fonts/requirements.txt
python design/assets/fonts/build-subset.py /tmp/NotoSansSC.ttf
python design/assets/fonts/build-subset.py --check
python design/assets/fonts/export-qt.py
python design/assets/fonts/export-qt.py --check
```

生成过程校验原始文件哈希，收集 `design/html/` 中 HTML、CSS、JavaScript 的文字（排除生成的图标登记文件 `assets.js`），保留常用标点和 ASCII，将可变字重范围限制为 400–750，然后通过 FontTools 与 Brotli 导出 WOFF2。检查过程验证字体、许可哈希、字重范围和当前界面的字形覆盖。增加界面文字时如检查发现缺字，应重新生成子集及单文件 HTML。

CSS 从 HTML 引用时使用 `../assets/fonts/fonts.css`，实际文字家族设为 `"Xdzip Design Sans SC"`。截图和几何测量前等待 `document.fonts.ready`，避免读取回退字体的宽度。单文件构建需要将 `fonts.css` 中的字体 URL 替换为 WOFF2 data URL，并嵌入完整许可。
