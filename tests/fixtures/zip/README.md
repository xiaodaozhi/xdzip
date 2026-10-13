# 自建 ZIP 测试样本

`visual-main.zip` 由 [generate-visual-fixture.py](../../../scripts/generate-visual-fixture.py) 生成，用于 Qt 与设计稿的布局对照：三个空目录、两张仓库示例缩略 PNG、一份有效空白 PDF、TXT、GPX、Markdown，共九条物理记录。日期固定、Store 不依赖 zlib 输出；UI 数据始终来自真实运行时，字节数与设计示例不同。

由仓库原创脚本 [generate-zip-fixtures.py](../../../scripts/generate-zip-fixtures.py) 生成，采用项目 Apache-2.0，无外部或个人数据。固定样本时间，压缩结果可能随 Python/zlib 版本变化；CI 读取提交的样本，不自动重生成。

normal.zip 含中文目录、Deflate、空文件及两个同名 notes.txt；empty.zip 是有效空归档；pages.zip 含 270 个记录；unsafe-names.zip 包含绝对/上级路径和双向控制符，仅用于元数据读取；truncated.zip 与 wrong-count.zip 验证失败关闭。不得将危险名称样本用于未经保护的解压测试。


目录导航回归样本（同一生成脚本）：

| 样本 | 预期 | SHA-256 |
| --- | --- | --- |
| navigation.zip | 5 个物理条目、4 个目录节点；隐式双层目录、重复名称、空目录及需检查后代 | `00916004ff6610121ec62c0a86e07e412caa6cf28be91e003600aba462e01c3c` |
| navigation-depth.zip | 1 个物理条目、65 层路径；目录索引完整降级，列表仍可用 | `a51d2f1c1b576a92942a946eaade300fbcd0fc25af73a19fa7bb966bf56b0530` |
