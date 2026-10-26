# RAR26 大文件自有作者样本

本档由固定官方 RAR 2.90 从 2 GiB 加 1 字节的自有稀疏数学输入原生生成。真实头为 UnpVer26、method1、LARGE，未修改版本字段。归档约 1 MiB，未提交作者程序或大明文。来源、命令、完整输入及归档摘要保留在 [sources.json](sources.json)，可选 [Windows 稀疏生成脚本](../../generate-independent-legacy26-fixture.py) 只接受固定 SHA-256 的作者程序，并将所有输出限制在仓库 build/ 中。

普通测试默认忽略这项大规模回归。Foundation 通过 [显式检查脚本](../../../../../scripts/check-rar-large-fixtures.py) 在 release 模式解码全部字节，增量比较原文 SHA-256，并要求格式完整 CRC 校验成功。sink 不保存明文，限制每块最多 8 KiB、总输出精确到文件尺寸，独立检查实际 WORK、AUX 和一次性输出额度，最后额度必须耗尽。生成的二进制输入从不执行。

这是组件级单文件、单卷、method1 的真实版本 26 证据。Native worker、运行时高膨胀比授权、分卷、其他大文件组合及桌面验收分别跟踪，不能由该测试代替。自有数学内容及归档遵循仓库许可；官方作者及对照工具仍保留各自许可，没有纳入仓库。

English: The pinned official RAR2.90 writer produced this unchanged archive from project-owned sparse mathematical input of2GiB+1bytes. Its real header declares version26,method1,LARGE. Only the approximately1MiB archive and provenance are vendored. The explicit Foundation release test incrementally hashes every output byte and requires complete size/CRC validation through a bounded sink. It saves no plaintext, limits chunks to8KiB, separates decoded work from auxiliary work and consumes all prepaid credit. Ordinary debug tests ignore it. The optional Windows recipe requires an explicit hash-checked writer and writes only below build/. No generated input is executed. Actual worker/runtime high-ratio policy, other large-file combinations and desktop acceptance remain separate.
