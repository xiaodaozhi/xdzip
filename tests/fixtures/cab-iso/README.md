# CAB 与光盘镜像测试输入

本目录只保存小型固定测试输入，不含用户归档。生成报告、解压镜像和测试输出存放于 build/，不提交。

来源及保存字节的 SHA-256 位于 [sources.json](sources.json)。`mszip_lzx_qtm.cab`、`normal_2files_1folder.cab` 及五个 `cabd_multi_basic_ptN.cab` 来自 libmspack 固定提交 `55d501976171397ccd5d5a7a1ca7da065b1d9a06`。分卷文件仅加 `cabd_` 前缀以匹配内部卷名，内容不变。上游使用 LGPL-2.1-or-later，保留 [COPYING.LIB](COPYING.LIB)，不将测试数据标为项目原创代码。

上游 `cabd_test.c` 定义 MSZIP、LZX 和 Quantum 的预期 MD5，分别是 `940cba86658fbceb582faecd2b5975d1`、`703474293b614e7110b3eb8ac2762b53`、`98fcfa4962a0f169a3c7fdbcb445cf17`。本项目同时核对完整已知明文，不将 MD5 用作来源身份或安全认证。五卷 Store 数据依次为 `This is the data from cabinet part N.\n`，三个文件按 76、38、76 字节切分。

`owned-part1.cab` 至 `owned-part4.cab` 是项目自有的独立 MakeCAB MSZIP 输入，单个文件跨四个 64 KiB 卷，第三和第四卷的逻辑前卷指向首卷。内容采用 `scripts/author-cab-spanning-fixture.py` 的固定 32 位生成配方，196608 字节的 SHA-256 及作者工具摘要记录在 `sources.json`。可用 Microsoft Cabinet FDI 完整解码后与配方逐字节比较，实际验证结果单独记录。重新生成的 CAB 时间字段可不同，须审阅并更新每卷摘要。

`named.cab` 和 gzip 保存的镜像是本项目自有输入，使用 Apache-2.0。通过[生成器](../../../scripts/generate-cab-iso-fixtures.py)及固定测试工具 pycdlib 1.14.0 创建，产品和 CI 验证不依赖该 Python 包。ISO9660、Joliet、Rock Ridge、UDF 2.60 和混合镜像分别覆盖大小写、Unicode、空目录、空文件与链接。UDF 命名空间独有 `udf-only.txt`，用于验证混合镜像优先级；独立 UDF 输入去掉 ISO 描述符但保留 UDF VRS 和锚点。

普通 `hello.txt` 明文为 `xdzip read-only archive fixture\n`，嵌套和中文文件为 UTF-8 `Unicode payload: 中文\n`。原生 worker 解码必须逐字节一致，不能仅以打开或列目录成功作为验收。`udf-link` 和 Rock Ridge 的 `link` 仅用于拒绝特殊对象输出。

运行 `python3 scripts/prepare-cab-iso-fixtures.py --output build/cab-iso-fixtures` 可在验证存储与展开摘要后生成读取输入，不需要安装 pycdlib。重建镜像时文件时间可能改变，须审阅并同步来源清单；不得把新的摘要自动当作既有验收结果。
