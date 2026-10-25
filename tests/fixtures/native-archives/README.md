# RAR 互操作夹具

这些小型公开测试归档取自 libarchive v3.8.2 官方测试集，来源、原始 UU 编码和解码后字节的 SHA-256 见 [sources.json](sources.json)。许可及对应测试作者的版权声明保留于 [COPYING.txt](COPYING.txt)，不含用户归档。

`test_read_format_rar_encryption.c` 定义加密夹具的四个文件 a.txt、b.txt、c.txt、d.txt，内容分别为 `This is from <filename>`，不带换行。固实和文件名加密夹具统一使用测试密码 `password`；非固实且未加密文件名的包中 a/c 明文，b 使用 `password`，d 使用 `password2`。这些是公开固定测试值，不用于产品秘密。

运行 [原生互操作检查](../../../scripts/check-native-codecs.py)时使用项目构建的工作进程；独立官方 7-Zip 工具仅用于生成和复核测试数据，不进入产品。运行结果及生成的 7z 文件只保存到 build/。

`rar7-store-small.rar` 和 `rar7-dictionary-limit.rar` 是本项目依据官方格式说明，用[确定性生成器](../../../scripts/generate-rar7-fixtures.py)构造的最小输入，分别检查 RAR7 格式版本 1 的 Store 内容，以及超预算字典的分配前拒绝。前者经过独立官方 UnRAR 7.23 完整测试。它们不包含 RAR 压缩算法实现，也不能代替 RAR7 LZ 压缩流的全面互操作证据；本项目生成部分使用 Apache-2.0。
