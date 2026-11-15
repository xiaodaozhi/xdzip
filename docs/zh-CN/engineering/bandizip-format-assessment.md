# Bandizip 格式清单与 xdzip 可行性评估

本评估记录本次格式扩展使用的 Bandizip 格式清单，以及 xdzip 能用源码、样本和自动化验证负责的能力边界。清单来自 Bandizip 产品页及格式帮助页，核对日期为 2026-11-15：

- [Bandizip 产品页](https://www.bandisoft.com/bandizip/) 列出：`7Z`、`ACE`、`AES`、`ALZ`、`ARJ`、`BH`、`BIN`、`BR`、`BZ`、`BZ2`、`CAB`、`Compound(MSI)`、`DAA(1.0)`、`DEB`、`EGG`、`GZ`、`IMG`、`ISO`、`ISZ`、`LHA`、`LZ`、`LZH`、`LZMA`、`PMA`、`RAR`、`RAR5`、`SFX(EXE)`、`TAR`、`TBZ/TBZ2`、`TGZ`、`TLZ`、`TXZ`、`UDF`、`WIM`、`XPI`、`XZ`、`Z`、`ZIP`、`ZIPX`、`ZPAQ`、`PEA`、`UU`、`UUE`、`XXE`、`ASAR`、`ZSTD`、`NSIS` 和 `LZ4`。
- 同一页面说明 Bandizip 可创建 `ZIP`、`7Z`、`ZIPX`、`EXE（SFX）`、`TAR`、`TGZ`、`LZH`、`ISO`、`GZ` 和 `XZ`，较长的清单主要是解压清单。
- [Bandizip ALZ 和 EGG 帮助](https://www.bandisoft.com/bandizip/help/how-to-open-alz-and-egg-files/) 说明 ALZ 的 Deflate、混淆 Deflate、BZip2 与 ZipCrypto，以及 EGG 的 Deflate、BZip2、LZMA、AZO、AES/LEA 加密、固实和分卷。
- [Bandizip ZPAQ 帮助](https://www.bandisoft.com/bandizip/help/zpaq/) 将 ZPAQ 描述为独立的固实、日志式格式，不能当作 ZIP 方法别名。
- M10 候选门槛文档记录了 DAA、ISZ、ZPAQ、PEA、BOND 和 J2J 的公开资料、许可、样本和净室检查。BOND 与 J2J 尚未确认是当前官方 Bandizip 归档条目，只保留为历史线索。

## xdzip 当前结论

表格中的含义如下。“已准入”表示当前登记、runtime、CLI 和桌面路径已有自动化证据，且只承诺表中配置。“别名/配置”表示通过现有容器入口准入，仍以内容验证为准。“候选”表示后续入口，当前不声明解码。“阻塞”表示在公开、可独立测试的线格式规范和兼容实现可用前，不能负责任地准入。“明确不支持”表示项目已经明确决定不支持该格式，除非重新作出产品决定，不登记入口或探测器。

| Bandizip 名称 | xdzip 状态 | 当前可负责的范围或原因 |
| --- | --- | --- |
| 7Z | 已准入 | 自研 Rust 7z 后端，覆盖已声明的密码、固实和分卷配置。 |
| ACE | 阻塞 | 专有旧格式，旧版 `unacev2` 需要许可与隔离决策，产品没有解码器。 |
| AES | 已准入 | AES Crypt 0–3 版、密码和 HMAC 配置；这是 `.aes` 封装，不是任意加密归档。 |
| ALZ | 已准入 | 只读普通单卷 Store 和原始 Deflate，验证准确尺寸与 CRC-32；改造 BZip2、ZipCrypto 和分卷仍不在准入范围。 |
| ARJ | 已准入 | 只读 Store 和方法 1–4，完整头部、尺寸和成员 CRC-32。 |
| BH | 已准入 | 只读 `BH 05 07` 普通成员、方法 8 raw Deflate、准确尺寸和 CRC-32，不含 Store、加密、分卷及其他方法。 |
| BIN | 候选 | 扩展名可能表示磁盘、可执行文件或普通数据，只能在验证内部格式后处理，不能注册通用 BIN 归档解码。 |
| BR | 已准入 | 有界 Brotli 流和 TAR.BR。 |
| BZ / BZ2 | 已准入 | BZip2 单文件流及登记的 TAR.BZ2 别名。 |
| CAB | 已准入 | 自研 Rust CAB 后端，覆盖 Store、MSZip、Quantum、LZX 和已授权卷集合。 |
| Compound(MSI) | 已准入 | 对经内容验证的 `.msi` 提供只读 CFB 结构、存储/流名称和有界原始流复制。MSI 表、内嵌 CAB 解释、变换、自定义动作和安装仍不在能力范围。 |
| DAA(1.0) | 阻塞 | 专有磁盘/归档格式，当前依赖策略中没有维护的公开实现。 |
| DEB | 已准入 | 普通 `ar` 封装、control/data 视图及已准入压缩成员。 |
| EGG | 已准入 | 只读无加密、非固实普通单卷 Store 和原始 Deflate，验证准确块尺寸与 CRC-32；AZO、BZip2、LZMA、加密、分卷和其他配置仍不在范围。 |
| GZ | 已准入 | GZIP 成员，包括串联流和 TAR.GZ。 |
| IMG | 候选 | 磁盘镜像扩展名含义不唯一，ISO/UDF 按内容准入不代表通用 IMG 支持。 |
| ISO | 已准入 | ISO9660、Joliet 和 Rock Ridge 只读镜像。 |
| ISZ | 明确不支持 | 产品决定放弃 ISZ，不增加探测器、登记项或密钥处理。 |
| LHA / LZH | 已准入 | LHA 0–3 级头部、Store、LH1–LH7、LZS/LZ5，并支持 LH5 创建。 |
| LZ | 已准入 | Lzip v1 和 TAR.LZ。 |
| LZMA | 已准入 | LZMA Alone 和 TAR.LZMA。 |
| PMA | 已准入 | PM0/PM1/PM2 只读配置、CP/M 结构和 CRC 检查。 |
| RAR / RAR5 | 已准入 | 自研 Rust RAR 后端，覆盖声明范围内的旧版、RAR5、RAR7 扩展、密码、固实和分卷。 |
| SFX(EXE) | 别名/配置 | ZIP SFX 前缀由 ZIP 读取器验证，绝不授予执行权限；SFX 创建仍不在当前写入器集合。 |
| TAR | 已准入 | TAR 及登记的 TAR.GZ、TAR.BZ2、TAR.LZMA、TAR.LZ、TAR.XZ、TAR.Z、TAR.BR、TAR.LZ4、TAR.ZST 配置。 |
| TBZ / TBZ2 | 别名/配置 | BZip2 过滤的 TAR 别名。 |
| TGZ | 别名/配置 | GZIP 过滤的 TAR 别名。 |
| TLZ | 别名/配置 | LZMA 过滤的 TAR 别名。 |
| TXZ | 别名/配置 | XZ 过滤的 TAR 别名。 |
| UDF | 已准入 | UDF 只读镜像，包括混合镜像命名空间规则。 |
| WIM | 明确不支持 | 产品决定不投入 WIM 所需的镜像资源模型、读取器和完整语料库。 |
| XPI | 别名/配置 | XPI 后缀下的 ZIP 内容，仍以内容验证为准。 |
| XZ | 已准入 | XZ 和 TAR.XZ，覆盖 LZMA2、声明校验及已准入过滤器。 |
| Z | 已准入 | Unix compress LZW 和 TAR.Z/TAZ 别名。 |
| ZIP | 已准入 | ZIP/ZIP64、AES-256、支持的分卷、方法及修改配置。 |
| ZIPX | 别名/配置 | ZIPX 通过已验证的 ZIP 方法（如 XZ）准入，不是独立容器；当前不提供 ZIPX 创建。 |
| ZPAQ | 明确不支持 | 产品决定不实现该固实、日志式格式所需的 ZPAQL 虚拟机。 |
| PEA | 已准入 | PEA 1.x 单卷、UTF-8、无加密 PCOMPRESS0 存储文件与目录只读配置，校验、加密、压缩、分卷、创建和修改仍不在范围。 |
| UU / UUE | 已准入 | 有界 UU/UUE 封装、物理成员和行结构验证。 |
| XXE | 已准入 | 有界 XXE 封装。 |
| ASAR | 已准入 | 有序元数据、存储范围和声明 SHA-256 检查。 |
| ZSTD | 已准入 | 标准 Zstandard 帧和 TAR.ZST/TZST。 |
| NSIS | 已准入 | 只读 `firstheader` 静态元数据。按内容验证有界 PE overlay，`.exe` 不参与拖放或系统关联，绝不执行 PE、脚本、插件或载荷。 |
| LZ4 | 已准入 | 标准 LZ4 帧和 TAR.LZ4。 |

因此不能仅因为扩展名加入通用 `BIN`、`IMG` 或 `SFX(EXE)` 处理器。这些名称可能指向可执行文件、磁盘镜像或安装器，扩展名不足以建立归档能力。NSIS 仅增加按内容验证的静态元数据读取器，不授予执行或载荷权限，登记的 `.exe` 也不参与拖放和系统关联。Compound(MSI) 已具备按内容验证的 CFB 读取器，但 `.msi` 后缀仍不授予执行权限。

M10 还检查了 DAA、ISZ、ZPAQ、PEA、BOND 和 J2J。ISZ 和 ZPAQ 按产品决定明确不支持，BOND 与 J2J 也因尚未确认 Bandizip 归档身份而明确不支持。DAA 仍因来源、许可和完整语料库门槛保持阻塞，详见 [M10 候选格式资料和许可门槛](m10-candidate-source-gates.md)。PEA 仅按该文档中的保守只读配置准入。

## 详细推进方案

1. **清单与登记门槛。** 每个已准入配置在 `specs/engineering/archive-formats.json` 中保留一个机器可读条目，生成 Rust、C++、JavaScript 和关联入口消费者，然后运行两项格式登记检查。每个新格式同时增加格式契约和验收条目。
2. **先定线格式契约。** 固定公开源码 revision 和 SHA-256，记录头部字段、名称编码、物理结束、方法、校验和、重复身份和资源限额。参考工具只有在产出完整自有样本时才作为证据，崩溃或单一 Store 样本只能记录为限制。
3. **先做原始算法自动测试。** Qt 之外实现有界读取器，覆盖空和边界载荷、每个准入方法、准确输入/输出/工作/内存限额、取消、类型化 I/O、来源变化、截断、尾随字节、CRC 和失败终止。未知方法保留可见性，不获得载荷权限。
4. **再做容器和产品准入。** 增加 codec crate、runtime 强/弱探测、CLI list/test/extract、桌面标签和关联 fixture。验证冻结元数据与物理重复选择，再执行必要的自动 Qt 和布局检查。本次研究与格式切片无需人工视觉验收。
5. **候选优先级。** ALZ 的保守 Store/raw Deflate 切片、EGG 的无加密且非固实 Store/raw Deflate 切片、CFB/Compound(MSI) 元数据切片、PEA 保守存储配置和 NSIS 静态 `firstheader` 元数据切片已经准入。WIM、ISZ、ZPAQ、BOND 与 J2J 按产品决定明确不支持。ACE 与 DAA 在许可、规范和完整语料库门槛清除前保持阻塞。ZIPX 创建和 SFX 创建是独立写入器项目，不会因读取别名自动获得。
6. **发行边界。** 候选格式在 codec、runtime、CLI 和桌面证据完成前不进入公开能力表。未知方法和参考工具限制记录在 issue 中，不能静默回退到 Store 或其他容器。

当前 BH 切片遵循这套方案：只读、无密码和分卷路径，使用独立有界的 Deflate 读取器，强制 CRC-32 和准确物理结束。Compound(MSI) 切片遵循同一边界：CFB FAT/DIFAT、目录树、mini-stream 和环检测独立于 Qt，只有在不可变元数据快照匹配后才复制原始流。NSIS 切片停在固定的 firstheader 元数据记录，并对载荷访问返回不支持，绝不执行 MSI 或 NSIS 安装行为。
