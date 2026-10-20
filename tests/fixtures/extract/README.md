# 安全解压输入

本目录为仓库自行构造的可再分发回归输入，不包含用户压缩包。基础 ZIP、链接及权限输入由 [generate-extract-fixtures.py](../../../scripts/generate-extract-fixtures.py)生成，预期边界见[夹具规则](../../../docs/qa/fixtures.md)。

`password-retry.zip` 是固定的 186 字节 WinZip AE-2 AES-256 Deflate 输入，唯一条目 `owned.txt` 为 18 字节，内容为 `owned AES payload` 加换行。SHA-256 为 `cdb1a34b259be86e1670c58d26fb328e3e2dfe6b71defaa6f333393bb4156d41`。

生成源码为 [generate-password-fixture.rs](../../../crates/xdzip-codec-zip/examples/generate-password-fixture.rs)，通过锁定 zip-rs 9.0.3 写入器创建，不实现密码学原语。日期、权限、级别及 16 字节测试盐固定，公开的正确／错误测试字符串位于源码常量中，仅用于这个合成输入，不能作为真实密码或生产盐策略。产品创建仍使用随机盐。

从仓库根目录执行 `cargo run --locked -p xdzip-codec-zip --example generate-password-fixture` 可重新生成。生成器对内存和写盘重读分别核对无密码、固定错误密码和正确密码的真实 codec 结果。固定错误字符串必须返回 `WrongPassword`，以避免随机盐的两字节验证值偶然匹配而使 Qt 重试断言不稳定。随机创建、认证破坏、不同密码、空文件和独立工具互操作另有运行时与格式回归，不依赖此固定输入证明随机性或密码学安全。
