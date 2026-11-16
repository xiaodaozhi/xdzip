# 通过真实工作进程验证 runtime

[runtime 集成测试清单](../../../specs/engineering/runtime-worker-tests.json) 声明全部依赖环境的 runtime 库测试及其前提。普通 `cargo test` 将它们保留为显式运行项，因为它们需要另行构建的隔离工作进程、固定外部数据或较多临时存储。ignore 标记不代表验证证据。

复用项目已构建的 `xdzip-native-codec`。以下入口准备已授权的输入，只构建 runtime 测试二进制，并按清单串行执行，不构建 Qt，不调用作者工具，不安装产品，也不授予发行许可。

```powershell
python scripts/check-runtime-worker-tests.py --worker build/desktop/tools/native-codec/xdzip-native-codec.exe --output build/runtime-worker-tests
```

Linux/macOS 去掉 `.exe`。默认选择完整清单。`--profile worker` 选择 24 项普通工作进程用例，`--profile large` 选择四项大用例：超过 2 GiB 的版本 26 RAR、513 MiB 7z 创建、真实 4.25 GiB RAR7 流和百万条目索引。这些用例保留完整尺寸与断言，不以小样本替代。须为大 7z 输出和索引预留足够临时空间，大 RAR 检查不会将解码后的数 GiB 明文写入磁盘。

`--include-regular` 启用集成测试并运行整个 runtime 库。编译后二进制列出的测试和每项实际结果必须一致，ignored 数量必须为零。子进程成功退出但运行零项测试、选择不完整或出现 ignored，都判为验证失败。可重复使用 `--case <完整测试名>`，仅重试指定用例，不重复成功且无关的测试；它不能与 profile 或 `--include-regular` 混用。

入口复用[固定 RAR 输入准备](../engineering/native-codecs.md)、固定身份的 RAR7 数据与通知检查器，以及项目自有 ZIP64 生成器。公共输入继续遵守尺寸、哈希、响应及来源限制，不下载第三方解码器源码或作者可执行文件。`--rar-cache` 接受已有公共数据准备目录或原始缓存布局，`--rar7-cache` 接受 RAR7 准备目录中的 `inputs`。`--offline` 要求已核验的输入齐全，缺少时失败，不静默跳过。数学构造的未知尺寸头部只验证拒绝行为，不代表 RAR 算法兼容性。

每次尝试在指定的 `build/` 输出下有独立目录，TEMP/TMP/TMPDIR 指向该次私有目录，Windows 同样限制权限。报告保留工作进程与测试二进制哈希、来源身份、精确用例结果和日志。超时终止本次测试及其工作进程子树，失败和未执行与通过分别记录，不修改父进程环境。保留必要回执后，使用[仓库清理脚本](../engineering/repository-hygiene.md#研发周期工件收尾)清理临时尝试。

桌面测试入口与 Release 契约任务在普通 Rust 检查后调用同一份完整清单。自动化结果仅覆盖实际工作进程与任务路径，与界面操作、其他平台及正式发行验收分别记录。
