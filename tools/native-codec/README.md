<!-- tools/native-codec/README.md -->

# 官方格式工作进程

此程序仅链接官方 7-Zip 的 Format7z 容器及算法清单，以及官方 UnRAR。CMake
拒绝其他 7-Zip 容器路径，7z 每次任务还检查实际静态注册表只有一个名为 `7z`
的容器。已有 ZIP、TAR、TAR.GZ 后端不经过此程序。

先运行 `python scripts/prepare-native-codecs.py` 准备锁定的官方来源，再通过项目
统一构建入口构建 `xdzip-native-codec` 目标。独立构建可设置
`XDZIP_7ZIP_SOURCE` 和 `XDZIP_UNRAR_SOURCE` 指向已验证的来源目录。

Rust `xdzip-codec-native` 通过 stdin/stdout 的长度前缀二进制帧通信，帧最大
1 MiB，实际数据块最大 64 KiB。源读取使用宿主流的偏移和长度，编码输入使用
冻结的物理索引，输出使用宿主 sink 或受控 `write_at`。命令行不携带密码或
用户文件路径，不解析控制台文本，不调用归档通用目录解包。

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
