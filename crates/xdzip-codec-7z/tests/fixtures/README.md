# 独立作者 7z 向量

这些小归档仅包含项目自有的三个固定输入，由固定官方 7-Zip 26.05 工具生成。
`sources.json` 保留工具身份、生成参数、方法清单及每个归档和输入的摘要，不包含执行报告或用户归档。
明文定义见 `scripts/check-sevenzip-rust-interop.py` 的 `known_payloads`，加密密码是公开的测试值。

普通 Rust 测试完整解码全部24组，覆盖 Copy、LZMA、LZMA2、PPMd、BZip2、Deflate、Deflate64、11种过滤器、BCJ2、固实、压缩头及三种 AES 组合。
Foundation 的同名检查另经真实工作进程及宿主服务逐字节比对输入，记录工件到 build。
较大过滤器分支、PPMd重启、错误输入和资源边界另有测试，不用这些小向量代表算法完备或原生桌面验收。

`password/copy-aes.7z` 另用于运行时默认测试。它保留独立作者生成的三个自有输入及密码来源清单；被选文件的原始字节由公开公式重新生成。Copy/AES 让错误密码下的暂存写入确实发生，再由 CRC 拒绝，验证重试、十次失败、取消与原目标保护，不能用提前拒绝的无输出路径代替。

English: These small archives contain only project-owned deterministic input, authored by pinned official 7-Zip 26.05. The manifest records provenance and hashes. Ordinary Rust tests decode all 24 profiles; Foundation separately compares every output byte through the real worker and host service. Synthetic passwords are public test values. Algorithm edge cases and manual desktop acceptance remain separate.
