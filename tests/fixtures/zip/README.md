# 自建 ZIP 测试样本

由仓库原创脚本 [generate-zip-fixtures.py](../../../scripts/generate-zip-fixtures.py) 生成，采用项目 Apache-2.0，无外部或个人数据。固定样本时间，压缩结果可能随 Python/zlib 版本变化；CI 读取提交的样本，不自动重生成。

normal.zip 含中文目录、Deflate、空文件及两个同名 notes.txt；empty.zip 是有效空归档；pages.zip 含 270 个记录；unsafe-names.zip 包含绝对/上级路径和双向控制符，仅用于元数据读取；truncated.zip 与 wrong-count.zip 验证失败关闭。不得将危险名称样本用于未经保护的解压测试。
