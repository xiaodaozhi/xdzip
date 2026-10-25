<!-- tools/native-codec/README.md -->

# 隔离格式工作进程

此程序链接官方 7-Zip 的 7z、Cab 明确白名单，以及官方 UnRAR。
CMake 拒绝其他容器路径，每次 7-Zip 任务检查实际静态注册表恰好为这两个容器。
ISO/UDF 通过 `xdzip-optical-bridge` 调用自研 Rust `xdzip-codec-optical`，
构建中不含官方 Iso/Udf 处理器。Cab、ISO、UDF 只读，只有 7z 保留写入。
已有 ZIP、TAR、TAR.GZ 后端不经过此程序。

先运行 `python scripts/prepare-native-codecs.py` 准备锁定的官方来源，再通过项目
统一构建入口构建 `xdzip-native-codec` 目标。独立构建可设置
`XDZIP_7ZIP_SOURCE` 和 `XDZIP_UNRAR_SOURCE` 指向已验证的来源目录。

Rust `xdzip-codec-native` 通过 stdin/stdout 的长度前缀二进制帧通信，帧最大
1 MiB，实际数据块最大 64 KiB。源读取使用宿主流的偏移和长度，编码输入使用
冻结的物理索引，输出使用宿主 sink 或受控 `write_at`。命令行不携带密码或
用户文件路径，不解析控制台文本，不调用归档通用目录解包。

CAB 的 Start 帧在原有卷长度表之后附加同序的原始卷名表，宿主在扫描前冻结全部
授权卷。每个原生输入流限制在单卷范围，换卷回调只能匹配这份名字表，缺卷立即失败。
ROW 保留 37 字节头，flags 的 0x10 表示原始名字字节，0x20 表示 CAB 显式 UTF-8；
0x04 链接与 0x08 特殊对象语义不变。CAB 与非 Joliet ISO 的原始名字由宿主解码，
仅 CAB 将反斜杠解释为目录分隔符。Joliet、UDF 传 Unicode 路径，树层级固定为 `/`。

构建副本中的精确变换保留 CAB 原始名字和卷名字，扫描检查官方后端错误标记，
损坏结构或缺卷不能形成部分成功索引。Rust ISO 使用 Rock Ridge 符号链接与
POSIX 类型属性分类，UDF 使用真实 ICB 类型及链接数；链接及特殊类型不会进入输出流。
ISO/UDF 的 CXX 回调同步借用切片，不保留指针；C++ 异常和 Rust unwind 均在边界内
转换为稳定错误，OOM/abort 导致隔离进程退出，宿主拒绝不完整暂存结果。
内容读取不宣称格式未提供的文件校验和。规格及扩展边界见
[自研契约](../../docs/engineering/independent-rust-codecs.md#iso与udf只读规格)。

UnRAR 使用此目录的 `host_file.cpp` 替换上游 File 实现，只能打开宿主提供的
synthetic 卷，不能创建、删除或重命名输出。构建仅在生成目录改变上游
`File::IsOpened` 声明，并检查原始声明精确匹配；对应源码材料须保留原始来源、
本目录和构建清单。此隔离保护宿主文件提交边界，不是操作系统文件系统沙箱。

读取器为升序条目保留同一解码进程和固实状态，跳过的实际输出单独计量。
倒序或密码变化会重开进程，输入读取预算不会重置。原始读取无目标权限，
编码修改后保留旧物理条目的时间与属性，宿主负责最终重新扫描、完整校验和提交。

默认 native 预算为 1 GiB。Windows 用 Job Object 限制提交内存，Linux 用
RLIMIT_AS，macOS 用启动时虚拟地址基线加预算设置 RLIMIT_AS，排除 dyld 预留。
Unix 限制不是 RSS 计量。7z 创建按偏移流式写入，修改暂以最多 512 MiB 原内容
缓存重建，同时将 native 子进程预算降为 512 MiB。超限明确失败，不发布新包。

执行真实后端回归前，设置 `XDZIP_NATIVE_CODEC_WORKER` 为当前构建的程序，运行
`cargo test -p xdzip-codec-native --locked`。缺少 worker 会失败，不静默跳过。
可设置 `XDZIP_NATIVE_INTEROP_OUTPUT` 为仓库 build 下的专用目录，导出测试创建
的普通、固实、内容加密和目录加密 7z，以及 expected 文件供独立工具逐字节核验。
RAR fixture 来源与许可证位于 `tests/fixtures/native-archives`。三平台构建和
桌面验收分别记录，不用本地 Windows 通过结果替代其他平台证据。
